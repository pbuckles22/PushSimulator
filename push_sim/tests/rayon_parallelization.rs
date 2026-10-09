//! Chain: finish_random_game → rayon par_iter → batch_metrics → CSV.
//!
//! `test_rayon_parallelization` plays 1,000 games through that parallel loop.
//! The shorter tests lock the same loop, the lowest-score win rule, and the CSV
//! columns on a few games.

use std::path::{Path, PathBuf};

use push_core::metrics::{batch_metrics, BatchMetrics, GameSample};
use push_core::random_bot::{finish_random_game, game_finished_five_rounds};
use push_sim::{parse_sim_args, run_parallel_batch, write_metrics_csv};

fn sample(turns: u32, scores: &[u32]) -> GameSample {
    GameSample {
        turns,
        scores: scores.to_vec(),
    }
}

fn csv_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("push_sim_{name}_{}.csv", std::process::id()))
}

fn read_rows(path: &Path) -> (Vec<String>, Vec<Vec<String>>) {
    let mut reader = csv::Reader::from_path(path).expect("read metrics csv");
    let headers = reader
        .headers()
        .expect("headers")
        .iter()
        .map(str::to_string)
        .collect();
    let rows = reader
        .records()
        .map(|row| {
            row.expect("row")
                .iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect();
    (headers, rows)
}

fn assert_csv_matches(path: &Path, metrics: &BatchMetrics) {
    let (headers, rows) = read_rows(path);
    assert_eq!(
        headers,
        vec![
            "seat",
            "wins",
            "games",
            "win_rate",
            "mean_score",
            "score_variance",
            "average_turns",
            "ties",
        ]
    );
    assert_eq!(rows.len(), metrics.seats.len());
    for (row, seat) in rows.iter().zip(&metrics.seats) {
        assert_eq!(row[0], seat.seat.to_string());
        assert_eq!(row[1], seat.wins.to_string());
        assert_eq!(row[2], metrics.games.to_string());
        assert_eq!(row[3].parse::<f64>().expect("win_rate"), seat.win_rate);
        assert_eq!(row[4].parse::<f64>().expect("mean_score"), seat.mean_score);
        assert_eq!(
            row[5].parse::<f64>().expect("score_variance"),
            seat.score_variance
        );
        assert_eq!(
            row[6].parse::<f64>().expect("average_turns"),
            metrics.average_turns
        );
        assert_eq!(row[7], metrics.ties.to_string());
    }
}

fn expect_workers(batch: &push_sim::ParallelBatch, games: u64) {
    let cores = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    if games == 0 {
        assert_eq!(batch.worker_threads, 0);
        assert_eq!(batch.pool_threads, 0);
        return;
    }
    assert!(batch.worker_threads >= 1, "a game ran on a worker");
    if cores > 1 && games > 1 {
        assert!(
            batch.pool_threads > 1,
            "rayon par_iter pool was {} on {cores} cores",
            batch.pool_threads
        );
        assert!(
            batch.worker_threads > 1,
            "rayon par_iter used {} thread on {cores} cores for {games} games",
            batch.worker_threads
        );
    }
}

/// The four-game sample: seat 0 wins twice (rate 0.5, variance 2), seat 1 wins
/// once (rate 0.25, variance 4), one tie, average turns 25.
#[test]
fn test_metrics_csv_records_seat_win_rates_average_turns_and_score_variance() {
    let metrics = batch_metrics(&[
        sample(10, &[2, 8]),
        sample(20, &[4, 8]),
        sample(30, &[4, 4]),
        sample(40, &[6, 4]),
    ]);
    let path = csv_path("four_games");
    write_metrics_csv(&path, &metrics).expect("write csv");
    assert_csv_matches(&path, &metrics);
    let _ = std::fs::remove_file(&path);
}

/// A second write replaces the previous rows.
#[test]
fn test_metrics_csv_overwrites_an_existing_file() {
    let path = csv_path("overwrite");
    let first = batch_metrics(&[sample(3, &[1, 9])]);
    write_metrics_csv(&path, &first).expect("first csv");
    let second = batch_metrics(&[sample(8, &[9, 1]), sample(12, &[0, 0])]);
    write_metrics_csv(&path, &second).expect("second csv");
    assert_csv_matches(&path, &second);
    let (_headers, rows) = read_rows(&path);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][1], "0");
    assert_eq!(rows[1][1], "1");
    assert_eq!(rows[0][7], "1");
    assert_eq!(rows[1][7], "1");
    let _ = std::fs::remove_file(&path);
}

/// No games writes the header and no seat rows.
#[test]
fn test_parallel_batch_of_zero_games_writes_a_header_only_csv() {
    let path = csv_path("zero");
    let (batch, metrics) = run_parallel_batch(1, 0, &path).expect("empty csv");
    assert!(batch.games.is_empty());
    expect_workers(&batch, 0);
    assert_eq!(metrics.games, 0);
    assert_eq!(metrics.ties, 0);
    assert_eq!(metrics.average_turns, 0.0);
    assert!(metrics.seats.is_empty());
    let (headers, rows) = read_rows(&path);
    assert_eq!(headers[0], "seat");
    assert!(rows.is_empty());
    let _ = std::fs::remove_file(&path);
}

/// One game at an offset seed matches the sequential game and logs that one row pair.
#[test]
fn test_play_random_game_par_iter_one_game_csv_matches_finish_random_game() {
    let path = csv_path("one");
    let (batch, metrics) = run_parallel_batch(10, 1, &path).expect("one-game csv");
    let sequential = finish_random_game(10);
    assert_eq!(batch.games.len(), 1);
    assert_eq!(batch.games[0], sequential);
    assert!(game_finished_five_rounds(&batch.games[0].state));
    expect_workers(&batch, 1);
    assert_eq!(metrics, batch_metrics(&[sequential.sample()]));
    assert_csv_matches(&path, &metrics);
    let _ = std::fs::remove_file(&path);
}

/// Chain: two real games through par_iter, then the CSV of win rate, turns, and variance.
///
/// Seed 1 still scores 5 and 85. Seed 2 is the same table the sequential game
/// plays. Both games finish five rounds.
#[test]
fn test_play_random_game_par_iter_batch_metrics_csv_win_rate_turns_variance() {
    let path = csv_path("chain");
    let (batch, metrics) = run_parallel_batch(1, 2, &path).expect("chain csv");
    let sequential: Vec<_> = (1..=2).map(finish_random_game).collect();
    assert_eq!(batch.games, sequential);
    expect_workers(&batch, 2);
    assert!(batch
        .games
        .iter()
        .all(|game| game_finished_five_rounds(&game.state)));
    assert_eq!(batch.games[0].state.players[0].total_score, 5);
    assert_eq!(batch.games[0].state.players[1].total_score, 85);
    assert_eq!(batch.games[0].state.players[0].points, 0);
    assert_eq!(batch.games[0].state.players[1].points, 0);

    let samples: Vec<_> = sequential.iter().map(|game| game.sample()).collect();
    assert_eq!(metrics, batch_metrics(&samples));
    assert_eq!(metrics.games, 2);
    assert_eq!(metrics.seats.len(), 2);
    assert_eq!(
        metrics.seats[0].wins + metrics.seats[1].wins + metrics.ties,
        2
    );
    assert_eq!(
        metrics.average_turns,
        (f64::from(sequential[0].turns) + f64::from(sequential[1].turns)) / 2.0
    );
    assert_csv_matches(&path, &metrics);
    let _ = std::fs::remove_file(&path);
}

/// 1,000 headless games, seeds 1..=1000, through `rayon::par_iter`.
///
/// Every game finishes five rounds. Seed 1 scores 5 and 85. The last seed
/// matches a sequential play of that same seed. The CSV records 1,000 games.
#[test]
fn test_rayon_parallelization() {
    let path = csv_path("thousand");
    let (batch, metrics) = run_parallel_batch(1, 1_000, &path).expect("thousand csv");
    assert_eq!(batch.games.len(), 1_000);
    assert_eq!(batch.games.first().map(|game| game.seed), Some(1));
    assert_eq!(batch.games.last().map(|game| game.seed), Some(1_000));
    assert!(batch
        .games
        .iter()
        .enumerate()
        .all(|(index, game)| game.seed == 1 + index as u64));
    assert!(batch
        .games
        .iter()
        .all(|game| game_finished_five_rounds(&game.state)));
    expect_workers(&batch, 1_000);
    assert_eq!(batch.games[0].state.players[0].total_score, 5);
    assert_eq!(batch.games[0].state.players[1].total_score, 85);
    assert_eq!(batch.games[999], finish_random_game(1_000));

    assert_eq!(metrics.games, 1_000);
    assert_eq!(metrics.seats.len(), 2);
    assert_eq!(
        metrics.seats[0].wins + metrics.seats[1].wins + metrics.ties,
        1_000
    );
    assert!(metrics.average_turns > 0.0);
    for seat in &metrics.seats {
        assert!((0.0..=1.0).contains(&seat.win_rate));
        assert!(seat.score_variance >= 0.0);
    }
    assert_csv_matches(&path, &metrics);
    let (_headers, rows) = read_rows(&path);
    assert_eq!(rows[0][2], "1000");
    assert_eq!(rows[1][2], "1000");
    let _ = std::fs::remove_file(&path);
}

/// No arguments keeps the one-game print. `--games` selects the parallel CSV batch.
#[test]
fn test_parse_sim_args_defaults_and_selects_a_csv_batch() {
    let quiet = parse_sim_args(std::iter::empty::<&str>());
    assert!(quiet.batch().is_none());

    let batch = parse_sim_args(["--games", "1000", "--csv", "out.csv"]);
    assert_eq!(
        batch
            .batch()
            .map(|(games, path)| (games, path.display().to_string())),
        Some((1_000, "out.csv".to_string()))
    );

    let default_csv = parse_sim_args(["--games", "4"]);
    assert_eq!(
        default_csv
            .batch()
            .map(|(games, path)| (games, path.display().to_string())),
        Some((4, "metrics.csv".to_string()))
    );

    let swapped = parse_sim_args(["--csv", "out.csv", "--games", "2"]);
    assert_eq!(
        swapped
            .batch()
            .map(|(games, path)| (games, path.display().to_string())),
        Some((2, "out.csv".to_string()))
    );
}

#[test]
#[should_panic(expected = "unknown argument")]
fn test_parse_sim_args_rejects_an_unknown_flag() {
    let _ = parse_sim_args(["--threads"]);
}

#[test]
#[should_panic(expected = "--games needs a count")]
fn test_parse_sim_args_rejects_games_without_a_count() {
    let _ = parse_sim_args(["--games"]);
}

#[test]
#[should_panic(expected = "--games is a number")]
fn test_parse_sim_args_rejects_a_games_count_that_is_not_a_number() {
    let _ = parse_sim_args(["--games", "many"]);
}

#[test]
#[should_panic(expected = "--csv needs a path")]
fn test_parse_sim_args_rejects_csv_without_a_path() {
    let _ = parse_sim_args(["--games", "1", "--csv"]);
}

#[test]
#[should_panic(expected = "--csv needs --games")]
fn test_parse_sim_args_rejects_csv_without_games() {
    let _ = parse_sim_args(["--csv", "out.csv"]);
}
