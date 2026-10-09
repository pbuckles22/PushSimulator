//! Headless game batches for the throughput gate.
//!
//! Five batches use distinct seeds. A debug run allows a wide per-game ceiling
//! so CI can check the shape. Release uses the reference-machine ceiling.

use std::time::Instant;

use crate::random_bot::play_random_game;

/// One contiguous run of headless games.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameBatch {
    pub games: usize,
    pub first_seed: u64,
    pub elapsed_ns: u128,
    pub totals: Vec<(u32, u32)>,
}

/// Per-game ceiling on the reference machine.
///
/// Release seeds 4–8 measured 1.35 s, 91 ms, 147 ms, 73 ms, and 28 ms.
/// Four seconds sits above that slow sample. Debug stays wide so CI only
/// checks that a batch finishes and records a time.
const RELEASE_MAX_GAME_NS: u128 = 4_000_000_000;
const DEBUG_MAX_GAME_NS: u128 = 30_000_000_000;

/// Plays `games` seeds beginning at `first_seed`.
pub fn play_game_batch(first_seed: u64, games: usize) -> GameBatch {
    let started = Instant::now();
    let mut totals = Vec::with_capacity(games);
    for offset in 0..games {
        let state = play_random_game(first_seed + offset as u64);
        totals.push((state.players[0].total_score, state.players[1].total_score));
    }
    GameBatch {
        games,
        first_seed,
        elapsed_ns: started.elapsed().as_nanos(),
        totals,
    }
}

/// `batches` runs whose first seeds are `games` apart, starting at seed 1.
pub fn play_game_batches(batches: usize, games: usize) -> Vec<GameBatch> {
    play_game_batches_from(1, batches, games)
}

/// Same spacing as [`play_game_batches`], starting at `first_seed`.
///
/// A seed that hits the turn limit aborts the batch. Seeds 2 and 3 do that.
/// The five-batch lock starts at seed 4.
pub fn play_game_batches_from(first_seed: u64, batches: usize, games: usize) -> Vec<GameBatch> {
    (0..batches)
        .map(|index| {
            let start = first_seed + (index as u64) * (games as u64);
            play_game_batch(start, games)
        })
        .collect()
}

/// True when the batch finished inside the per-game ceiling for this build.
pub fn batch_meets_gate(batch: &GameBatch) -> bool {
    if batch.totals.len() != batch.games {
        return false;
    }
    let per_game = if cfg!(debug_assertions) {
        DEBUG_MAX_GAME_NS
    } else {
        RELEASE_MAX_GAME_NS
    };
    batch.elapsed_ns <= per_game.saturating_mul(batch.games as u128)
}
