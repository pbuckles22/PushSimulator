//! Seat win rates, average turns, and score variance for a batch of games.
//!
//! The lowest total score wins. A shared lowest score is a tie, and a tie is
//! not a win. Win rate is wins divided by games. Score variance is the
//! population variance of that seat's totals.

use push_core::metrics::{batch_metrics, GameSample};
use push_core::random_bot::{finish_random_game, play_random_game};

fn sample(turns: u32, scores: &[u32]) -> GameSample {
    GameSample {
        turns,
        scores: scores.to_vec(),
    }
}

/// Two seats, four games: seat 0 wins two, seat 1 wins one, one tie.
///
/// Turns 10, 20, 30, and 40 average 25. Seat 0 scores 2, 4, 4, 6 (mean 4,
/// variance 2). Seat 1 scores 8, 8, 4, 4 (mean 6, variance 4).
#[test]
fn test_batch_metrics_seat_win_rates_average_turns_and_score_variance() {
    let metrics = batch_metrics(&[
        sample(10, &[2, 8]),
        sample(20, &[4, 8]),
        sample(30, &[4, 4]),
        sample(40, &[6, 4]),
    ]);
    assert_eq!(metrics.games, 4);
    assert_eq!(metrics.ties, 1);
    assert_eq!(metrics.average_turns, 25.0);
    assert_eq!(metrics.seats.len(), 2);

    let seat0 = &metrics.seats[0];
    assert_eq!(seat0.seat, 0);
    assert_eq!(seat0.wins, 2);
    assert_eq!(seat0.win_rate, 0.5);
    assert_eq!(seat0.mean_score, 4.0);
    assert_eq!(seat0.score_variance, 2.0);

    let seat1 = &metrics.seats[1];
    assert_eq!(seat1.seat, 1);
    assert_eq!(seat1.wins, 1);
    assert_eq!(seat1.win_rate, 0.25);
    assert_eq!(seat1.mean_score, 6.0);
    assert_eq!(seat1.score_variance, 4.0);

    let accounted = seat0.wins + seat1.wins + metrics.ties;
    assert_eq!(accounted, metrics.games);
}

/// Equal totals are a tie. Both win rates stay 0. A lower score on the other
/// games still wins, including a score of 0 against a positive score.
#[test]
fn test_batch_metrics_tie_is_not_a_win() {
    let metrics = batch_metrics(&[sample(8, &[0, 0]), sample(12, &[10, 4]), sample(4, &[3, 3])]);
    assert_eq!(metrics.games, 3);
    assert_eq!(metrics.ties, 2);
    assert_eq!(metrics.average_turns, 8.0);
    assert_eq!(metrics.seats[0].wins, 0);
    assert_eq!(metrics.seats[0].win_rate, 0.0);
    assert_eq!(metrics.seats[1].wins, 1);
    assert_eq!(metrics.seats[1].win_rate, 1.0 / 3.0);
    assert_eq!(
        metrics.seats[0].wins + metrics.seats[1].wins + metrics.ties,
        3
    );
}

/// One game has win rate 1 or 0, variance 0, and average turns equal to that game.
#[test]
fn test_batch_metrics_one_game_has_zero_variance() {
    let metrics = batch_metrics(&[sample(17, &[5, 85])]);
    assert_eq!(metrics.games, 1);
    assert_eq!(metrics.ties, 0);
    assert_eq!(metrics.average_turns, 17.0);
    assert_eq!(metrics.seats[0].wins, 1);
    assert_eq!(metrics.seats[0].win_rate, 1.0);
    assert_eq!(metrics.seats[0].mean_score, 5.0);
    assert_eq!(metrics.seats[0].score_variance, 0.0);
    assert_eq!(metrics.seats[1].wins, 0);
    assert_eq!(metrics.seats[1].win_rate, 0.0);
    assert_eq!(metrics.seats[1].mean_score, 85.0);
    assert_eq!(metrics.seats[1].score_variance, 0.0);
}

/// Identical totals across games have variance 0. The lower seat still wins each one.
#[test]
fn test_batch_metrics_identical_scores_have_zero_variance() {
    let metrics = batch_metrics(&[sample(3, &[9, 1]), sample(5, &[9, 1]), sample(7, &[9, 1])]);
    assert_eq!(metrics.games, 3);
    assert_eq!(metrics.ties, 0);
    assert_eq!(metrics.average_turns, 5.0);
    assert_eq!(metrics.seats[0].wins, 0);
    assert_eq!(metrics.seats[0].mean_score, 9.0);
    assert_eq!(metrics.seats[0].score_variance, 0.0);
    assert_eq!(metrics.seats[1].wins, 3);
    assert_eq!(metrics.seats[1].win_rate, 1.0);
    assert_eq!(metrics.seats[1].mean_score, 1.0);
    assert_eq!(metrics.seats[1].score_variance, 0.0);
}

/// Every game tied: both win rates stay 0, and the score spread is still recorded.
#[test]
fn test_batch_metrics_every_game_ties() {
    let metrics = batch_metrics(&[sample(1, &[4, 4]), sample(3, &[0, 0])]);
    assert_eq!(metrics.games, 2);
    assert_eq!(metrics.ties, 2);
    assert_eq!(metrics.average_turns, 2.0);
    assert_eq!(metrics.seats[0].wins, 0);
    assert_eq!(metrics.seats[0].win_rate, 0.0);
    assert_eq!(metrics.seats[1].wins, 0);
    assert_eq!(metrics.seats[1].win_rate, 0.0);
    assert_eq!(metrics.seats[0].mean_score, 2.0);
    assert_eq!(metrics.seats[0].score_variance, 4.0);
    assert_eq!(metrics.seats[1].score_variance, 4.0);
}

/// One seat is the unique lowest score, so that seat wins. A tie needs two seats sharing it.
#[test]
fn test_batch_metrics_one_seat_wins_every_game() {
    let metrics = batch_metrics(&[sample(4, &[7]), sample(6, &[9])]);
    assert_eq!(metrics.games, 2);
    assert_eq!(metrics.ties, 0);
    assert_eq!(metrics.average_turns, 5.0);
    assert_eq!(metrics.seats.len(), 1);
    assert_eq!(metrics.seats[0].seat, 0);
    assert_eq!(metrics.seats[0].wins, 2);
    assert_eq!(metrics.seats[0].win_rate, 1.0);
    assert_eq!(metrics.seats[0].mean_score, 8.0);
    assert_eq!(metrics.seats[0].score_variance, 1.0);
}

/// An empty batch records no games and does not divide by zero.
#[test]
fn test_batch_metrics_empty_batch() {
    let metrics = batch_metrics(&[]);
    assert_eq!(metrics.games, 0);
    assert_eq!(metrics.ties, 0);
    assert_eq!(metrics.average_turns, 0.0);
    assert!(metrics.seats.is_empty());
}

/// A sample with no seats, or a later sample with a different seat count, is refused.
#[test]
#[should_panic(expected = "same seats")]
fn test_batch_metrics_rejects_a_different_seat_count() {
    let _ = batch_metrics(&[sample(1, &[4, 5]), sample(1, &[4])]);
}

#[test]
#[should_panic(expected = "at least one seat")]
fn test_batch_metrics_rejects_a_game_with_no_scores() {
    let _ = batch_metrics(&[sample(1, &[])]);
}

/// Three seats: only the unique lowest total wins. A three-way tie is one tie.
///
/// Seat 0 scores 5, 5, 5, 5 (mean 5, variance 0) and never wins. Seat 1 scores
/// 1, 5, 5, 9 (mean 5, variance 8) and wins once. Seat 2 scores 5, 0, 5, 1
/// (mean 2.75, variance 5.1875) and wins twice.
#[test]
fn test_batch_metrics_three_seats_lowest_score_wins() {
    let metrics = batch_metrics(&[
        sample(6, &[5, 1, 5]),
        sample(10, &[5, 5, 0]),
        sample(2, &[5, 5, 5]),
        sample(6, &[5, 9, 1]),
    ]);
    assert_eq!(metrics.games, 4);
    assert_eq!(metrics.ties, 1);
    assert_eq!(metrics.average_turns, 6.0);
    assert_eq!(metrics.seats.len(), 3);
    assert_eq!(metrics.seats[0].wins, 0);
    assert_eq!(metrics.seats[0].win_rate, 0.0);
    assert_eq!(metrics.seats[0].mean_score, 5.0);
    assert_eq!(metrics.seats[0].score_variance, 0.0);
    assert_eq!(metrics.seats[1].wins, 1);
    assert_eq!(metrics.seats[1].win_rate, 0.25);
    assert_eq!(metrics.seats[1].mean_score, 5.0);
    assert_eq!(metrics.seats[1].score_variance, 8.0);
    assert_eq!(metrics.seats[2].wins, 2);
    assert_eq!(metrics.seats[2].win_rate, 0.5);
    assert_eq!(metrics.seats[2].mean_score, 2.75);
    assert_eq!(metrics.seats[2].score_variance, 5.1875);
    assert_eq!(
        metrics.seats[0].wins + metrics.seats[1].wins + metrics.seats[2].wins + metrics.ties,
        4
    );
}

/// Chain: play_random_game → finish_random_game → batch_metrics.
///
/// Seed 1 scores 5 and 85, so seat 0 wins that game. The same seed through
/// `play_random_game` is that same table. A second seed is scored by the same
/// lowest-total rule, and the two-game averages match those two results.
#[test]
fn test_play_random_game_finish_batch_metrics_seed_one_wins_on_five() {
    let finished = finish_random_game(1);
    let played = play_random_game(1);
    assert_eq!(finished.seed, 1);
    assert_eq!(finished.state, played);
    assert_eq!(finished.state.round_number, 6);
    assert_eq!(finished.state.players[0].total_score, 5);
    assert_eq!(finished.state.players[1].total_score, 85);
    assert_eq!(finished.state.players[0].points, 0);
    assert_eq!(finished.state.players[1].points, 0);
    assert!(finished.turns > 0);

    let alone = batch_metrics(&[finished.sample()]);
    assert_eq!(alone.games, 1);
    assert_eq!(alone.ties, 0);
    assert_eq!(alone.average_turns, f64::from(finished.turns));
    assert_eq!(alone.seats[0].wins, 1);
    assert_eq!(alone.seats[0].win_rate, 1.0);
    assert_eq!(alone.seats[0].mean_score, 5.0);
    assert_eq!(alone.seats[0].score_variance, 0.0);
    assert_eq!(alone.seats[1].wins, 0);
    assert_eq!(alone.seats[1].win_rate, 0.0);
    assert_eq!(alone.seats[1].mean_score, 85.0);
    assert_eq!(alone.seats[1].score_variance, 0.0);

    let second = finish_random_game(2);
    assert_eq!(second.seed, 2);
    assert_eq!(second.state.round_number, 6);
    assert!(second.turns > 0);
    let metrics = batch_metrics(&[finished.sample(), second.sample()]);
    assert_eq!(metrics.games, 2);
    assert_eq!(metrics.seats.len(), 2);
    assert_eq!(
        metrics.average_turns,
        (f64::from(finished.turns) + f64::from(second.turns)) / 2.0
    );

    let mut wins = [0u64, 0u64];
    let mut ties = 0u64;
    for game in [&finished, &second] {
        let left = game.state.players[0].total_score;
        let right = game.state.players[1].total_score;
        if left < right {
            wins[0] += 1;
        } else if right < left {
            wins[1] += 1;
        } else {
            ties += 1;
        }
    }
    assert_eq!(metrics.ties, ties);
    assert_eq!(metrics.seats[0].wins, wins[0]);
    assert_eq!(metrics.seats[1].wins, wins[1]);
    assert_eq!(metrics.seats[0].win_rate, wins[0] as f64 / 2.0);
    assert_eq!(metrics.seats[1].win_rate, wins[1] as f64 / 2.0);
    assert_eq!(wins[0] + wins[1] + ties, 2);
    assert_eq!(
        metrics.seats[0].mean_score,
        (5.0 + second.state.players[0].total_score as f64) / 2.0
    );
    assert_eq!(
        metrics.seats[1].mean_score,
        (85.0 + second.state.players[1].total_score as f64) / 2.0
    );
}
