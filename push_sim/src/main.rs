//! Monte Carlo simulator CLI — Phase 2.
//!
//! One headless game: two random seats, five rounds.

fn main() {
    let state = push_core::random_bot::play_random_game(1);
    println!(
        "push_sim random game seed=1 rounds=5 next_round={} scores={},{}",
        state.round_number,
        state.players[0].total_score,
        state.players[1].total_score
    );
}
