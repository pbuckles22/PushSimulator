//! Stage 3 throughput gate: five disjoint batches of 1,000 headless games.
//!
//! The short tests stay on the default run. The five full batches, and the
//! 14-card release ceilings, stay off that run.

use push_core::game_state::GameState;
#[cfg(not(debug_assertions))]
use push_core::latency::{profile_fourteen_card, HIT_GATE, PLAY_GATE, VALIDATE_GATE};
#[cfg(not(debug_assertions))]
use push_core::random_bot::THROUGHPUT_BATCH_GATE;
use push_core::random_bot::{
    new_two_seat_table, play_game_batch, play_random_game, play_random_turn,
    throughput_batch_first_seed, THROUGHPUT_BATCH_COUNT, THROUGHPUT_BATCH_GAMES,
};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn cards_on_the_table(state: &GameState) -> usize {
    let mut count = state.deck.cards.len() + state.deck.discard.len();
    for player in &state.players {
        count += player.hand.len();
    }
    for meld in &state.board {
        count += meld.len();
    }
    count
}

/// The five batches are seeds 1..=1000, 1001..=2000, 2001..=3000, 3001..=4000, and 4001..=5000.
#[test]
fn test_throughput_batch_windows_are_five_disjoint_thousands() {
    assert_eq!(THROUGHPUT_BATCH_COUNT, 5);
    assert_eq!(THROUGHPUT_BATCH_GAMES, 1_000);
    let mut seeds = Vec::new();
    for index in 0..THROUGHPUT_BATCH_COUNT {
        let first = throughput_batch_first_seed(index);
        assert_eq!(first, 1 + index as u64 * THROUGHPUT_BATCH_GAMES);
        seeds.extend(first..first + THROUGHPUT_BATCH_GAMES);
    }
    seeds.sort_unstable();
    seeds.dedup();
    assert_eq!(seeds.len(), 5_000);
    assert_eq!(seeds.first().copied(), Some(1));
    assert_eq!(seeds.last().copied(), Some(5_000));
}

/// Chain: new_two_seat_table → play_random_turn → play_random_game → play_game_batch.
///
/// One turn keeps every card. Seed 1 scores 5 and 85. A one-game batch of that
/// seed finishes, and so do the seeds that used to stop at 8,000 turns.
#[test]
fn test_new_two_seat_table_play_random_turn_play_random_game_batch_finishes() {
    let mut rng = StdRng::seed_from_u64(1);
    let mut table = new_two_seat_table(&mut rng);
    assert_eq!(cards_on_the_table(&table), 108);
    play_random_turn(&mut table, 0, &mut rng);
    assert_eq!(cards_on_the_table(&table), 108);
    assert_eq!(table.players[0].points, 0);
    assert_eq!(table.players[1].points, 0);

    let state = play_random_game(1);
    assert_eq!(state.round_number, 6);
    assert_eq!(state.players[0].total_score, 5);
    assert_eq!(state.players[1].total_score, 85);
    assert_eq!(cards_on_the_table(&state), 108);

    let one = play_game_batch(1, 1);
    assert_eq!(one.first_seed, 1);
    assert_eq!(one.games, 1);
    assert_eq!(one.finished, 1);
    assert!(one.elapsed > std::time::Duration::ZERO);

    for seed in [16_u64, 55, 84] {
        let batch = play_game_batch(seed, 1);
        assert_eq!(batch.finished, 1, "seed {seed}");
        assert_eq!(batch.first_seed, seed);
        assert_eq!(batch.games, 1);
    }
}

/// One batch of 1,000 games finishes inside the release ceiling.
/// This is the check for removing the one-card large-hand path.
#[test]
#[ignore = "one batch of 1000 headless games"]
fn test_one_thousand_game_batch_stays_inside_the_throughput_gate() {
    let first = throughput_batch_first_seed(0);
    let batch = play_game_batch(first, THROUGHPUT_BATCH_GAMES);
    assert_eq!(batch.first_seed, first);
    assert_eq!(batch.games, THROUGHPUT_BATCH_GAMES);
    assert_eq!(batch.finished, batch.games);
    eprintln!(
        "throughput_batch index=0 first_seed={first} games={} finished={} elapsed_ms={}",
        batch.games,
        batch.finished,
        batch.elapsed.as_millis()
    );
    #[cfg(not(debug_assertions))]
    assert!(
        batch.elapsed <= THROUGHPUT_BATCH_GATE,
        "batch 0 took {:?}, gate {:?}",
        batch.elapsed,
        THROUGHPUT_BATCH_GATE
    );
}

/// Five batches of 1,000 games finish inside the release ceiling.
/// The same run checks the 14-card validate, play, and hit ceilings.
#[test]
#[ignore = "five batches of 1000 headless games"]
fn test_headless_random_games_meet_the_throughput_gate() {
    for index in 0..THROUGHPUT_BATCH_COUNT {
        let first = throughput_batch_first_seed(index);
        let batch = play_game_batch(first, THROUGHPUT_BATCH_GAMES);
        assert_eq!(batch.first_seed, first, "batch {index}");
        assert_eq!(batch.games, THROUGHPUT_BATCH_GAMES, "batch {index}");
        assert_eq!(
            batch.finished, batch.games,
            "batch {index} finished {} of {} from seed {first}",
            batch.finished, batch.games
        );
        eprintln!(
            "throughput_batch index={index} first_seed={first} games={} finished={} elapsed_ms={}",
            batch.games,
            batch.finished,
            batch.elapsed.as_millis()
        );
        #[cfg(not(debug_assertions))]
        assert!(
            batch.elapsed <= THROUGHPUT_BATCH_GATE,
            "batch {index} took {:?}, gate {:?}",
            batch.elapsed,
            THROUGHPUT_BATCH_GATE
        );
    }

    #[cfg(not(debug_assertions))]
    {
        let profile = profile_fourteen_card(100);
        assert!(
            profile.validate.latency.within(VALIDATE_GATE),
            "validate median {} p95 {} p99 {}",
            profile.validate.latency.median_ns,
            profile.validate.latency.p95_ns,
            profile.validate.latency.p99_ns
        );
        assert!(
            profile.play.latency.within(PLAY_GATE),
            "play median {} p95 {} p99 {}",
            profile.play.latency.median_ns,
            profile.play.latency.p95_ns,
            profile.play.latency.p99_ns
        );
        assert!(
            profile.hit.latency.within(HIT_GATE),
            "hit median {} p95 {} p99 {}",
            profile.hit.latency.median_ns,
            profile.hit.latency.p95_ns,
            profile.hit.latency.p99_ns
        );
        eprintln!(
            "fourteen_card_gates validate_median_ns={} play_median_ns={} hit_median_ns={}",
            profile.validate.latency.median_ns,
            profile.play.latency.median_ns,
            profile.hit.latency.median_ns
        );
    }
}
