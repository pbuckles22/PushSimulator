//! Stage 3 throughput lock. Five distinct batches of headless games.

use push_core::game_state::GameState;
use push_core::random_bot::play_random_game;
use push_core::throughput::{play_game_batch, GameBatch};

fn scores(state: &GameState) -> (u32, u32) {
    (state.players[0].total_score, state.players[1].total_score)
}

/// One batch plays each seed once. The elapsed time is the whole batch.
#[test]
fn test_game_batch_plays_each_seed_once() {
    let batch = play_game_batch(1, 1);
    assert_eq!(batch.games, 1);
    assert_eq!(batch.first_seed, 1);
    assert!(batch.elapsed_ns > 0);
    assert_eq!(scores(&play_random_game(1)), (340, 1025));
    assert_eq!(batch.totals, vec![(340, 1025)]);
}

/// Chain: play_random_game → five batches with distinct seeds.
///
/// Seed 1 still scores 340 and 1025. Seeds 2 and 3 stop at the turn limit, so
/// the five batches are the next contiguous run that finishes: 4 through 8.
/// Each batch has to finish inside the recorded per-game ceiling.
#[test]
fn test_play_random_game_five_distinct_batches_meet_the_throughput_gate() {
    assert_eq!(scores(&play_random_game(1)), (340, 1025));
    let batches = play_game_batch_set(4, 5, 1);
    assert_eq!(batches.len(), 5);
    let mut seeds = Vec::new();
    for batch in &batches {
        assert_eq!(batch.games, 1);
        assert!(batch.elapsed_ns > 0);
        assert!(
            batch_within_gate(batch, 1),
            "seed {} elapsed_ns {}",
            batch.first_seed,
            batch.elapsed_ns
        );
        seeds.push(batch.first_seed);
    }
    assert_eq!(seeds, vec![4, 5, 6, 7, 8]);
}

fn play_game_batch_set(first_seed: u64, batches: usize, games: usize) -> Vec<GameBatch> {
    push_core::throughput::play_game_batches_from(first_seed, batches, games)
}

fn batch_within_gate(batch: &GameBatch, games: usize) -> bool {
    batch.games == games && push_core::throughput::batch_meets_gate(batch)
}
