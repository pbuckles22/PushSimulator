//! Chain: finish_profile_game → rayon par_iter → batch_metrics → CSV.
//!
//! Seat 0 is point-averse. Seat 1 is the hoarder. The CSV columns stay
//! `seat`, `wins`, `games`, `win_rate`, `mean_score`, `score_variance`,
//! `average_turns`, and `ties`.

use std::path::{Path, PathBuf};

use push_core::metrics::batch_metrics;
use push_core::profiles::BotProfile;
use push_core::random_bot::game_finished_five_rounds;
use push_sim::{parse_sim_args, run_profile_batch};

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

/// Two profile games through the parallel loop write the same eight columns.
#[test]
fn test_finish_profile_game_par_iter_batch_metrics_csv_point_averse_vs_hoarder() {
    let seats = [BotProfile::PointAverse, BotProfile::Hoarder];
    let path = csv_path("profiles");
    let (batch, metrics) = run_profile_batch(1, 2, seats, &path).expect("profile csv");
    assert_eq!(batch.games.len(), 2);
    assert!(batch
        .games
        .iter()
        .all(|game| game_finished_five_rounds(&game.state)));
    let samples: Vec<_> = batch.games.iter().map(|game| game.sample()).collect();
    assert_eq!(metrics, batch_metrics(&samples));
    let (headers, rows) = read_rows(&path);
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
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], "0");
    assert_eq!(rows[1][0], "1");
    assert_eq!(rows[0][2], "2");
    assert_eq!(rows[1][2], "2");
    let _ = std::fs::remove_file(&path);
}

/// `--seat0` and `--seat1` select the profiles. Omitting them leaves both random.
#[test]
fn test_parse_sim_args_selects_point_averse_and_hoarder() {
    let quiet = parse_sim_args(std::iter::empty::<&str>());
    assert_eq!(quiet.seats(), [BotProfile::Random, BotProfile::Random]);

    let matched = parse_sim_args([
        "--games",
        "100000",
        "--seat0",
        "point-averse",
        "--seat1",
        "hoarder",
        "--csv",
        "profiles.csv",
    ]);
    assert_eq!(
        matched.seats(),
        [BotProfile::PointAverse, BotProfile::Hoarder]
    );

    let reserve = parse_sim_args([
        "--games",
        "1",
        "--seat0",
        "keep-2",
        "--seat1",
        "point-averse",
    ]);
    assert_eq!(
        reserve.seats(),
        [BotProfile::Keep(2), BotProfile::PointAverse]
    );
    assert_eq!(
        matched
            .batch()
            .map(|(games, path)| (games, path.display().to_string())),
        Some((100_000, "profiles.csv".to_string()))
    );
}

#[test]
#[should_panic(expected = "unknown profile")]
fn test_parse_sim_args_rejects_an_unknown_profile() {
    let _ = parse_sim_args(["--games", "1", "--seat0", "greedy"]);
}
