//! Latency of one 14-card validation, play walk, and hit walk.
//!
//! Percentiles use the nearest rank. A release run compares them with the
//! recorded ceilings. Debug runs keep the shape check. Those ceilings are the
//! reference-machine record, not a GitHub latency gate.

use std::time::Instant;

use crate::actions::{validate_action, Action};
use crate::card::{Card, Rank, Suit};
use crate::deck::Deck;
use crate::game_state::GameState;
use crate::legal_moves::{visit_legal_kind, LegalKind};
use crate::player::Player;
use crate::resolution::ActionResolution;

/// Host recorded with a latency profile. GitHub runners may differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceMachine {
    pub arch: &'static str,
    pub os: &'static str,
    pub family: &'static str,
}

/// Nearest-rank summary of one component, in nanoseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencySummary {
    pub samples: usize,
    pub median_ns: u128,
    pub p95_ns: u128,
    pub p99_ns: u128,
}

/// Ceiling for one component. A sample at the ceiling still passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencyGate {
    pub median_ns: u128,
    pub p95_ns: u128,
    pub p99_ns: u128,
}

/// How many actions one component emitted, and how long the walks took.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimedComponent {
    pub actions: usize,
    pub latency: LatencySummary,
}

/// Validation, play, and hit on the 14-card set fixture, timed apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FourteenCardProfile {
    pub machine: ReferenceMachine,
    pub validate: TimedComponent,
    pub play: TimedComponent,
    pub hit: TimedComponent,
}

impl LatencySummary {
    /// True when every recorded percentile is at or under `gate`.
    pub fn within(&self, gate: LatencyGate) -> bool {
        self.median_ns <= gate.median_ns && self.p95_ns <= gate.p95_ns && self.p99_ns <= gate.p99_ns
    }
}

/// Nearest rank. Rank 1 is the smallest sample. An empty slice has no summary.
pub fn summarize_latencies(samples_ns: &[u128]) -> Option<LatencySummary> {
    if samples_ns.is_empty() {
        return None;
    }
    let mut ordered = samples_ns.to_vec();
    ordered.sort_unstable();
    Some(LatencySummary {
        samples: ordered.len(),
        median_ns: percentile(&ordered, 50),
        p95_ns: percentile(&ordered, 95),
        p99_ns: percentile(&ordered, 99),
    })
}

/// Reference ceilings from release runs of 100 samples on x86_64 Windows,
/// Intel64 Family 6 Model 141, rustc 1.96.0.
///
/// Validation stayed at the clock quantum (median 0 ns, p99 100 ns).
/// Play medians were 13.5 ms to 16.3 ms, and the slower p99 was 30.1 ms.
/// Hit medians were 13.9 µs to 20.4 µs, and the slower p99 was 66 µs.
pub const VALIDATE_GATE: LatencyGate = LatencyGate {
    median_ns: 5_000,
    p95_ns: 5_000,
    p99_ns: 5_000,
};
pub const PLAY_GATE: LatencyGate = LatencyGate {
    median_ns: 40_000_000,
    p95_ns: 60_000_000,
    p99_ns: 80_000_000,
};
pub const HIT_GATE: LatencyGate = LatencyGate {
    median_ns: 80_000,
    p95_ns: 100_000,
    p99_ns: 150_000,
};

fn percentile(sorted: &[u128], pct: usize) -> u128 {
    let rank = (pct * sorted.len()).div_ceil(100).max(1);
    sorted[rank - 1]
}

/// Times `samples` validations, play walks, and hit walks on the 14-card fixture.
///
/// The hand is fourteen set cards in round 4. Validation is one take. Play is
/// the lay-down walk. Hit is that seat on a board of three other fours.
pub fn profile_fourteen_card(samples: usize) -> FourteenCardProfile {
    let state = set_table();
    let validate = time_validate(&state, samples);
    let play = time_kind(&state, LegalKind::Play, samples);
    let hitting = hit_table(state);
    let hit = time_kind(&hitting, LegalKind::Hit, samples);
    FourteenCardProfile {
        machine: ReferenceMachine {
            arch: std::env::consts::ARCH,
            os: std::env::consts::OS,
            family: std::env::consts::FAMILY,
        },
        validate,
        play,
        hit,
    }
}

fn time_validate(state: &GameState, samples: usize) -> TimedComponent {
    warmup(|| {
        std::hint::black_box(validate_action(state, 0, &Action::TakeDiscard));
    });
    let mut times = Vec::with_capacity(samples);
    let mut actions = 0usize;
    for _ in 0..samples {
        let started = Instant::now();
        let resolution = validate_action(state, 0, &Action::TakeDiscard);
        let elapsed = started.elapsed().as_nanos();
        std::hint::black_box(&resolution);
        if matches!(resolution, ActionResolution::Accepted(_)) {
            actions = 1;
        }
        times.push(elapsed);
    }
    TimedComponent {
        actions,
        latency: summarize_latencies(&times).expect("validation samples"),
    }
}

fn time_kind(state: &GameState, kind: LegalKind, samples: usize) -> TimedComponent {
    warmup(|| {
        let mut seen = 0usize;
        visit_legal_kind(state, 0, kind, &mut |_| seen += 1);
        std::hint::black_box(seen);
    });
    let mut times = Vec::with_capacity(samples);
    let mut actions = 0usize;
    for _ in 0..samples {
        let started = Instant::now();
        let mut seen = 0usize;
        visit_legal_kind(state, 0, kind, &mut |_| seen += 1);
        let elapsed = started.elapsed().as_nanos();
        std::hint::black_box(seen);
        actions = seen;
        times.push(elapsed);
    }
    TimedComponent {
        actions,
        latency: summarize_latencies(&times).expect("walk samples"),
    }
}

fn warmup(run: impl FnOnce()) {
    run();
}

fn take(deck: &mut Deck, suit: Suit, rank: Rank) -> Card {
    let index = deck
        .cards
        .iter()
        .position(|card| card.suit == suit && card.rank == rank)
        .expect("shoe has the card");
    deck.cards.remove(index)
}

fn set_table() -> GameState {
    let hand = [
        (Suit::Hearts, Rank::Four),
        (Suit::Spades, Rank::Four),
        (Suit::Diamonds, Rank::Four),
        (Suit::Clubs, Rank::Four),
        (Suit::Hearts, Rank::Five),
        (Suit::Spades, Rank::Five),
        (Suit::Diamonds, Rank::Five),
        (Suit::Clubs, Rank::Five),
        (Suit::Hearts, Rank::Six),
        (Suit::Spades, Rank::Six),
        (Suit::Diamonds, Rank::Six),
        (Suit::Clubs, Rank::Six),
        (Suit::Hearts, Rank::Seven),
        (Suit::Spades, Rank::Seven),
    ];
    let mut deck = Deck::new();
    let mut cards = Vec::new();
    for (suit, rank) in hand {
        cards.push(take(&mut deck, suit, rank));
    }
    let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
    players[0].hand = cards;
    players[1].hand = vec![take(&mut deck, Suit::Clubs, Rank::Three)];
    let queen = take(&mut deck, Suit::Diamonds, Rank::Queen);
    deck.discard.push(queen);
    let mut state = GameState::new(players, deck);
    state.round_number = 4;
    state
}

fn hit_table(mut state: GameState) -> GameState {
    let four_hearts = take(&mut state.deck, Suit::Hearts, Rank::Four);
    let four_spades = take(&mut state.deck, Suit::Spades, Rank::Four);
    let four_diamonds = take(&mut state.deck, Suit::Diamonds, Rank::Four);
    state.board = vec![vec![four_hearts, four_spades, four_diamonds]];
    state.players[0].is_on_board = true;
    state
}
