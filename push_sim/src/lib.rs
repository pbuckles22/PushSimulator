//! Parallel headless games and the CSV log of their results.
//!
//! [`play_games_parallel`] runs each seed on a Rayon worker. The lowest total
//! score wins a game. [`write_metrics_csv`] records seat win rates, average
//! turns, and score variance.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread::ThreadId;

use push_core::metrics::{batch_metrics, BatchMetrics};
use push_core::profiles::{finish_profile_game, BotProfile};
use push_core::random_bot::{finish_random_game, FinishedGame};
use rayon::prelude::*;

/// Games played on the Rayon pool, in seed order.
///
/// `worker_threads` is how many distinct threads played a game.
/// `pool_threads` is the Rayon pool size observed while a game was playing.
/// Both stay 0 when `games` is 0.
#[derive(Clone, Debug)]
pub struct ParallelBatch {
    pub games: Vec<FinishedGame>,
    pub worker_threads: usize,
    pub pool_threads: usize,
}

/// What `push_sim` was asked to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimArgs {
    games: Option<u64>,
    csv: Option<PathBuf>,
    seats: Vec<BotProfile>,
}

impl SimArgs {
    /// The parallel batch, if `--games` was set. The CSV path defaults to `metrics.csv`.
    pub fn batch(&self) -> Option<(u64, PathBuf)> {
        self.games.map(|games| {
            let path = self
                .csv
                .clone()
                .unwrap_or_else(|| PathBuf::from("metrics.csv"));
            (games, path)
        })
    }

    /// Seat 0, then each later seat. Both are [`BotProfile::Random`] when the flags are omitted.
    ///
    /// `--players` above 2 copies seat 1 into every later seat.
    pub fn seats(&self) -> &[BotProfile] {
        &self.seats
    }
}

/// Reads `--games <count>`, `--csv <path>`, `--seat0`, `--seat1`, and `--players`.
///
/// An empty list leaves the one-game print in place. `--csv` without `--games`
/// is refused. An unknown flag is refused. A seat is `random`, `point-averse`,
/// `hoarder`, or `keep-<count>`. Omitting a seat leaves it random. `--players`
/// deals that many seats: seat 0 keeps `--seat0`, and every other seat is `--seat1`.
pub fn parse_sim_args<I, S>(args: I) -> SimArgs
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut games = None;
    let mut csv = None;
    let mut players = None;
    let mut seats = vec![BotProfile::Random, BotProfile::Random];
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--games" => {
                let value = iter
                    .next()
                    .map(|item| item.as_ref().to_string())
                    .expect("--games needs a count");
                games = Some(
                    value
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("--games is a number")),
                );
            }
            "--csv" => {
                let value = iter
                    .next()
                    .map(|item| item.as_ref().to_string())
                    .expect("--csv needs a path");
                csv = Some(PathBuf::from(value));
            }
            "--seat0" => seats[0] = parse_profile("--seat0", &mut iter),
            "--seat1" => seats[1] = parse_profile("--seat1", &mut iter),
            "--players" => {
                let value = iter
                    .next()
                    .map(|item| item.as_ref().to_string())
                    .expect("--players needs a count");
                players = Some(
                    value
                        .parse::<usize>()
                        .unwrap_or_else(|_| panic!("--players is a number")),
                );
            }
            other => panic!("unknown argument {other}"),
        }
    }
    if csv.is_some() && games.is_none() {
        panic!("--csv needs --games");
    }
    if let Some(count) = players {
        assert!(count >= 2, "--players is at least 2");
        assert!(count <= 10, "--players fits the shoe");
        let seat0 = seats[0];
        let rest = seats[1];
        seats = Vec::with_capacity(count);
        seats.push(seat0);
        seats.extend(std::iter::repeat(rest).take(count - 1));
    }
    SimArgs { games, csv, seats }
}

fn parse_profile<I, S>(flag: &str, iter: &mut I) -> BotProfile
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let value = iter
        .next()
        .map(|item| item.as_ref().to_string())
        .unwrap_or_else(|| panic!("{flag} needs a profile"));
    BotProfile::parse(&value).unwrap_or_else(|| panic!("unknown profile {value}"))
}

/// Plays `games` five-round matches, seeds `first_seed .. first_seed + games`.
///
/// The iterator is `rayon::iter::ParallelIterator`. Result order matches seed order.
pub fn play_games_parallel(first_seed: u64, games: u64) -> ParallelBatch {
    let seen = Mutex::new(HashSet::<ThreadId>::new());
    let pool_threads = AtomicUsize::new(0);
    let played = (0..games)
        .into_par_iter()
        .map(|offset| {
            seen.lock()
                .expect("worker set")
                .insert(std::thread::current().id());
            pool_threads.fetch_max(rayon::current_num_threads(), Ordering::Relaxed);
            finish_random_game(first_seed + offset)
        })
        .collect();
    let worker_threads = seen.lock().expect("worker set").len();
    ParallelBatch {
        games: played,
        worker_threads,
        pool_threads: pool_threads.load(Ordering::Relaxed),
    }
}

/// Writes one row per seat.
///
/// Columns are `seat`, `wins`, `games`, `win_rate`, `mean_score`,
/// `score_variance`, `average_turns`, and `ties`. An empty batch writes the
/// header only. `average_turns` and `ties` repeat on every seat row.
pub fn write_metrics_csv(path: &Path, metrics: &BatchMetrics) -> Result<(), csv::Error> {
    let mut writer = csv::Writer::from_path(path)?;
    writer.write_record([
        "seat",
        "wins",
        "games",
        "win_rate",
        "mean_score",
        "score_variance",
        "average_turns",
        "ties",
    ])?;
    for seat in &metrics.seats {
        writer.write_record([
            seat.seat.to_string(),
            seat.wins.to_string(),
            metrics.games.to_string(),
            seat.win_rate.to_string(),
            seat.mean_score.to_string(),
            seat.score_variance.to_string(),
            metrics.average_turns.to_string(),
            metrics.ties.to_string(),
        ])?;
    }
    writer.flush()?;
    Ok(())
}

/// Plays `games` five-round matches on the Rayon pool.
///
/// Seat 0 uses `seats[0]`. Each later seat uses its own profile. Result order
/// matches seed order.
pub fn play_profile_games_parallel(
    first_seed: u64,
    games: u64,
    seats: &[BotProfile],
) -> ParallelBatch {
    let seen = Mutex::new(HashSet::<ThreadId>::new());
    let pool_threads = AtomicUsize::new(0);
    let played = (0..games)
        .into_par_iter()
        .map(|offset| {
            seen.lock()
                .expect("worker set")
                .insert(std::thread::current().id());
            pool_threads.fetch_max(rayon::current_num_threads(), Ordering::Relaxed);
            finish_profile_game(first_seed + offset, seats)
        })
        .collect();
    let worker_threads = seen.lock().expect("worker set").len();
    ParallelBatch {
        games: played,
        worker_threads,
        pool_threads: pool_threads.load(Ordering::Relaxed),
    }
}

/// Plays the batch and writes its metrics to `csv_path`.
pub fn run_parallel_batch(
    first_seed: u64,
    games: u64,
    csv_path: &Path,
) -> Result<(ParallelBatch, BatchMetrics), csv::Error> {
    let batch = play_games_parallel(first_seed, games);
    let samples: Vec<_> = batch.games.iter().map(FinishedGame::sample).collect();
    let metrics = batch_metrics(&samples);
    write_metrics_csv(csv_path, &metrics)?;
    Ok((batch, metrics))
}

/// Plays a profile matchup and writes its metrics to `csv_path`.
///
/// The columns match [`write_metrics_csv`].
pub fn run_profile_batch(
    first_seed: u64,
    games: u64,
    seats: &[BotProfile],
    csv_path: &Path,
) -> Result<(ParallelBatch, BatchMetrics), csv::Error> {
    let batch = play_profile_games_parallel(first_seed, games, seats);
    let samples: Vec<_> = batch.games.iter().map(FinishedGame::sample).collect();
    let metrics = batch_metrics(&samples);
    write_metrics_csv(csv_path, &metrics)?;
    Ok((batch, metrics))
}
