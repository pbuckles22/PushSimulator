//! Allocation lock for a 14-card hand.
//!
//! Ceilings count heap allocations during one family walk. They are not latency
//! gates, so a GitHub runner may enforce them. A large hand stops after a sample.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use rand::rngs::StdRng;
use rand::SeedableRng;

use push_core::actions::{validate_action, Action};
use push_core::card::{Card, Rank, Suit};
use push_core::deck::Deck;
use push_core::game_state::GameState;
use push_core::legal_moves::{visit_legal_kind, LegalKind};
use push_core::player::Player;
use push_core::random_bot::play_random_turn;
use push_core::resolution::{ActionPlan, ActionResolution};

struct Counting;

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static MEASURE: Mutex<()> = Mutex::new(());

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn take(deck: &mut Deck, suit: Suit, rank: Rank) -> Card {
    let index = deck
        .cards
        .iter()
        .position(|card| card.suit == suit && card.rank == rank)
        .expect("shoe has the card");
    deck.cards.remove(index)
}

fn mixed_hand() -> Vec<(Suit, Rank)> {
    vec![
        (Suit::Hearts, Rank::Four),
        (Suit::Spades, Rank::Four),
        (Suit::Hearts, Rank::Five),
        (Suit::Clubs, Rank::Five),
        (Suit::Diamonds, Rank::Six),
        (Suit::Hearts, Rank::Seven),
        (Suit::Spades, Rank::Eight),
        (Suit::Clubs, Rank::Nine),
        (Suit::Diamonds, Rank::Ten),
        (Suit::Hearts, Rank::Jack),
        (Suit::Spades, Rank::Queen),
        (Suit::Clubs, Rank::King),
        (Suit::Diamonds, Rank::Ace),
        (Suit::Hearts, Rank::Three),
    ]
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

fn shoe_ids(state: &GameState) -> Vec<u32> {
    let mut ids = Vec::new();
    for player in &state.players {
        ids.extend(player.hand.iter().map(|card| card.id));
    }
    for meld in &state.board {
        ids.extend(meld.iter().map(|card| card.id));
    }
    ids.extend(state.deck.cards.iter().map(|card| card.id));
    ids.extend(state.deck.discard.iter().map(|card| card.id));
    ids.sort_unstable();
    ids
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

fn lock_allocations() -> std::sync::MutexGuard<'static, ()> {
    MEASURE.lock().expect("allocation lock")
}

fn counted_visit(state: &GameState, kind: LegalKind) -> (usize, u64) {
    ALLOCS.store(0, Ordering::Relaxed);
    let mut actions = 0usize;
    let _ = visit_legal_kind(state, 0, kind, &mut |_| actions += 1);
    (actions, ALLOCS.load(Ordering::Relaxed))
}

fn assert_walk_budget(actions: usize, allocs: u64) {
    let cap = (actions as u64).saturating_mul(16).saturating_add(64);
    assert!(
        allocs <= cap,
        "walk emitted {actions} actions and allocated {allocs} times; cap is {cap}"
    );
}

fn heart_run_hand() -> Vec<(Suit, Rank)> {
    vec![
        (Suit::Hearts, Rank::Three),
        (Suit::Hearts, Rank::Four),
        (Suit::Hearts, Rank::Five),
        (Suit::Hearts, Rank::Six),
        (Suit::Hearts, Rank::Seven),
        (Suit::Hearts, Rank::Eight),
        (Suit::Hearts, Rank::Nine),
        (Suit::Hearts, Rank::Ten),
        (Suit::Hearts, Rank::Jack),
        (Suit::Hearts, Rank::Queen),
        (Suit::Hearts, Rank::King),
        (Suit::Hearts, Rank::Ace),
        (Suit::Diamonds, Rank::Three),
        (Suit::Spades, Rank::Four),
    ]
}

/// Fourteen hearts-and-neighbors against one heart run. Same-suit cards are not
/// a dead group, so the hit walk still has to stay inside the allocation budget.
#[test]
fn test_fourteen_card_hit_walk_stays_within_the_allocation_budget() {
    let _guard = lock_allocations();
    let mut state = table_from(heart_run_hand(), 1);
    let six = take(&mut state.deck, Suit::Hearts, Rank::Six);
    let seven = take(&mut state.deck, Suit::Hearts, Rank::Seven);
    let eight = take(&mut state.deck, Suit::Hearts, Rank::Eight);
    state.board = vec![vec![six, seven, eight]];
    state.players[0].is_on_board = true;
    assert_eq!(state.players[0].hand.len(), 14);

    let (actions, allocs) = counted_visit(&state, LegalKind::Hit);
    assert_eq!(actions, 27);
    assert_walk_budget(actions, allocs);
}

/// A 14-card hand with no legal lay-down still walks every partial group.
/// That walk may allocate its hand buffer. It may not allocate once per group.
#[test]
fn test_fourteen_card_mixed_play_walk_stays_within_the_allocation_budget() {
    let _guard = lock_allocations();
    let state = table_from(mixed_hand(), 1);
    assert_eq!(state.players[0].hand.len(), 14);
    assert!(!state.players[0].is_on_board);

    let (actions, allocs) = counted_visit(&state, LegalKind::Play);
    assert_eq!(actions, 0);
    assert_walk_budget(actions, allocs);
}

/// Chain: Deck::new → has_draw_capacity → validate_action → visit play, hit, and
/// discard on a 14-card hand → play_random_turn.
///
/// Validation accepts a take and leaves the table. The play walk keeps its action
/// count inside the allocation budget. The bot still plays that large hand.
#[test]
fn test_suit_rank_card_deck_new_has_draw_capacity_validate_action_fourteen_card_visit_random_turn_allocation(
) {
    let _guard = lock_allocations();
    let state = table_from(set_hand(), 4);
    assert_eq!(state.players[0].hand.len(), 14);
    assert!(state.deck.has_draw_capacity(2));

    let before = state.clone();
    ALLOCS.store(0, Ordering::Relaxed);
    let resolution = validate_action(&state, 0, &Action::TakeDiscard);
    let allocs = ALLOCS.load(Ordering::Relaxed);
    assert_eq!(allocs, 0, "validate_action allocated {allocs} times");
    assert_eq!(
        resolution,
        ActionResolution::Accepted(ActionPlan::TakeDiscard)
    );
    assert!(same_table(&state, &before));

    let (plays, play_allocs) = counted_visit(&state, LegalKind::Play);
    assert_eq!(plays, 750);
    assert_walk_budget(plays, play_allocs);

    let (discards, discard_allocs) = counted_visit(&state, LegalKind::Discard);
    assert_eq!(discards, 14);
    assert_eq!(discard_allocs, 0);

    let (takes, take_allocs) = counted_visit(&state, LegalKind::Take);
    assert_eq!(takes, 1);
    assert_eq!(take_allocs, 0);

    let mut hitting = state.clone();
    let four_hearts = take(&mut hitting.deck, Suit::Hearts, Rank::Four);
    let four_spades = take(&mut hitting.deck, Suit::Spades, Rank::Four);
    let four_diamonds = take(&mut hitting.deck, Suit::Diamonds, Rank::Four);
    hitting.board = vec![vec![four_hearts, four_spades, four_diamonds]];
    hitting.players[0].is_on_board = true;
    let (hits, hit_allocs) = counted_visit(&hitting, LegalKind::Hit);
    assert_eq!(hits, 15);
    assert_walk_budget(hits, hit_allocs);

    let ids = shoe_ids(&state);
    let mut playing = state;
    assert!(playing.players[0].hand.len() > 11);
    play_random_turn(&mut playing, 0, &mut StdRng::seed_from_u64(1));
    assert_eq!(shoe_ids(&playing), ids);
    assert_eq!(playing.players[0].points, 0);
    assert_eq!(playing.players[1].points, 0);
}
