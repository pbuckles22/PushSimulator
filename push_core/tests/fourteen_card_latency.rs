//! Stage 2 latency lock for the 14-card fixture.
//!
//! Percentiles are exact. Live ceilings are not GitHub latency gates: debug
//! `cargo test` checks the shape, and release checks the recorded ceilings.

use push_core::actions::{validate_action, Action};
use push_core::card::{Card, Rank, Suit};
use push_core::deck::Deck;
use push_core::game_state::GameState;
use push_core::latency::{
    profile_fourteen_card, summarize_latencies, LatencyGate, HIT_GATE, PLAY_GATE, VALIDATE_GATE,
};
use push_core::legal_moves::{visit_legal_kind, LegalKind};
use push_core::player::Player;
use push_core::resolution::{ActionPlan, ActionResolution};

fn take(deck: &mut Deck, suit: Suit, rank: Rank) -> Card {
    let index = deck
        .cards
        .iter()
        .position(|card| card.suit == suit && card.rank == rank)
        .expect("shoe has the card");
    deck.cards.remove(index)
}

fn set_hand() -> Vec<(Suit, Rank)> {
    vec![
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
    ]
}

fn table_from(hand: Vec<(Suit, Rank)>, round: u8) -> GameState {
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
    state.round_number = round;
    state
}

fn same_table(left: &GameState, right: &GameState) -> bool {
    left.players == right.players
        && left.deck == right.deck
        && left.board == right.board
        && left.round_number == right.round_number
        && left.turn_counter == right.turn_counter
        && left.drawn_card_id == right.drawn_card_id
        && left.turn_phase == right.turn_phase
        && left.penalty_seat == right.penalty_seat
        && left.round_over == right.round_over
}

fn counted(state: &GameState, kind: LegalKind) -> usize {
    let mut actions = 0usize;
    let _ = visit_legal_kind(state, 0, kind, &mut |_| actions += 1);
    actions
}

/// One hundred ordered samples land on the 50th, 95th, and 99th values.
/// A single sample is every percentile. An empty list has no summary.
#[test]
fn test_latency_summary_median_p95_and_p99() {
    let samples: Vec<u128> = (1..=100).collect();
    let summary = summarize_latencies(&samples).expect("one hundred samples");
    assert_eq!(summary.samples, 100);
    assert_eq!(summary.median_ns, 50);
    assert_eq!(summary.p95_ns, 95);
    assert_eq!(summary.p99_ns, 99);

    let shuffled = vec![40u128, 10, 30, 20];
    let summary = summarize_latencies(&shuffled).expect("four samples");
    assert_eq!(summary.median_ns, 20);
    assert_eq!(summary.p95_ns, 40);
    assert_eq!(summary.p99_ns, 40);

    let only = summarize_latencies(&[7]).expect("one sample");
    assert_eq!(only.median_ns, 7);
    assert_eq!(only.p95_ns, 7);
    assert_eq!(only.p99_ns, 7);
    assert!(summarize_latencies(&[]).is_none());

    let inside = summarize_latencies(&[10, 20, 30]).expect("three samples");
    assert!(inside.within(LatencyGate {
        median_ns: 20,
        p95_ns: 30,
        p99_ns: 30,
    }));
    assert!(!inside.within(LatencyGate {
        median_ns: 19,
        p95_ns: 30,
        p99_ns: 30,
    }));
}

/// Chain: Deck::new → has_draw_capacity → validate_action → visit play → visit hit
/// → profile those three on the same 14-card fixture.
///
/// The take is accepted and the table stays. Play emits 750. Hit emits 15.
/// Each profile's median is at most its p95, and that is at most its p99.
#[test]
fn test_suit_rank_card_deck_new_has_draw_capacity_validate_action_visit_play_hit_fourteen_card_latency(
) {
    let state = table_from(set_hand(), 4);
    assert_eq!(state.players[0].hand.len(), 14);
    assert!(state.deck.has_draw_capacity(2));
    let before = state.clone();
    assert_eq!(
        validate_action(&state, 0, &Action::TakeDiscard),
        ActionResolution::Accepted(ActionPlan::TakeDiscard)
    );
    assert!(same_table(&state, &before));
    assert_eq!(counted(&state, LegalKind::Play), 750);

    let mut hitting = state.clone();
    let four_hearts = take(&mut hitting.deck, Suit::Hearts, Rank::Four);
    let four_spades = take(&mut hitting.deck, Suit::Spades, Rank::Four);
    let four_diamonds = take(&mut hitting.deck, Suit::Diamonds, Rank::Four);
    hitting.board = vec![vec![four_hearts, four_spades, four_diamonds]];
    hitting.players[0].is_on_board = true;
    assert_eq!(counted(&hitting, LegalKind::Hit), 15);

    let profile = profile_fourteen_card(20);
    assert_eq!(profile.machine.arch, std::env::consts::ARCH);
    assert_eq!(profile.machine.os, std::env::consts::OS);
    assert!(!profile.machine.family.is_empty());
    assert_eq!(profile.validate.actions, 1);
    assert_eq!(profile.play.actions, 750);
    assert_eq!(profile.hit.actions, 15);
    for component in [&profile.validate, &profile.play, &profile.hit] {
        assert!(component.latency.samples >= 20);
        assert!(component.latency.median_ns <= component.latency.p95_ns);
        assert!(component.latency.p95_ns <= component.latency.p99_ns);
    }
    assert!(VALIDATE_GATE.median_ns <= VALIDATE_GATE.p95_ns);
    assert!(VALIDATE_GATE.p95_ns <= VALIDATE_GATE.p99_ns);
    assert!(PLAY_GATE.median_ns <= PLAY_GATE.p95_ns);
    assert!(PLAY_GATE.p95_ns <= PLAY_GATE.p99_ns);
    assert!(HIT_GATE.median_ns <= HIT_GATE.p95_ns);
    assert!(HIT_GATE.p95_ns <= HIT_GATE.p99_ns);
    #[cfg(not(debug_assertions))]
    {
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
    }
}
