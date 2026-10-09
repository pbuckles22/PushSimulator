//! Monte Carlo simulator CLI — Phase 2.
//!
//! With no arguments, one headless game is printed.
//! `--games <count> --csv <path>` plays that many games through Rayon and
//! writes seat win rates, average turns, and score variance.

use std::path::Path;

fn main() {
    let args = push_sim::parse_sim_args(std::env::args().skip(1));
    if let Some((games, csv)) = args.batch() {
        let seats = args.seats();
        let (_batch, metrics) = if seats
            == [
                push_core::profiles::BotProfile::Random,
                push_core::profiles::BotProfile::Random,
            ] {
            push_sim::run_parallel_batch(1, games, Path::new(&csv)).expect("write metrics csv")
        } else {
            push_sim::run_profile_batch(1, games, seats, Path::new(&csv))
                .expect("write metrics csv")
        };
        println!(
            "push_sim parallel games={games} seats={},{} csv={} ties={} average_turns={}",
            seats[0].label(),
            seats[1].label(),
            csv.display(),
            metrics.ties,
            metrics.average_turns
        );
        for seat in &metrics.seats {
            println!(
                "seat={} wins={} win_rate={} mean_score={} score_variance={}",
                seat.seat, seat.wins, seat.win_rate, seat.mean_score, seat.score_variance
            );
        }
        return;
    }

    let state = push_core::random_bot::play_random_game(1);
    println!(
        "push_sim random game seed=1 rounds=5 next_round={} scores={},{}",
        state.round_number, state.players[0].total_score, state.players[1].total_score
    );
}
