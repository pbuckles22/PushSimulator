//! Frozen copy of the current apply path and the deep move generators.
//!
//! Production `apply_with_rng` stays the live path. This module is the comparison
//! oracle for later validation and generation changes. It is test-only.

use std::collections::HashSet;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::actions::{Action, MeldHit};
use crate::card::{Card, Rank, Suit};
use crate::deck::{Deck, TurnDraw};
use crate::game_state::{GameState, TurnPhase};
use crate::legal_moves::generate_legal_moves;
use crate::player::{deal_initial_hands, Player};
use crate::validation::{card_can_be_played, check_round_requirements, validate_run, validate_set};

/// Runs `action` with `rng` using the apply behavior frozen at this isolation.
pub(crate) fn legacy_apply_with_rng(
    state: &mut GameState,
    action: Action,
    actor_index: usize,
    rng: &mut impl Rng,
) -> bool {
    if state.round_over {
        return false;
    }
    match action {
        Action::TakeDiscard => {
            let card = state
                .deck
                .discard
                .pop()
                .expect("take discard needs a top card");
            state.players[actor_index].hand.push(card);
            state.drawn_card_id = state.players[actor_index].hand.last().map(|card| card.id);
            true
        }
        Action::PushDiscard => {
            push_discard(&mut state.players, actor_index, &mut state.deck, rng);
            state.drawn_card_id = state.players[actor_index].hand.last().map(|card| card.id);
            true
        }
        Action::PlayMeld(melds) => play_meld(state, actor_index, &melds),
        Action::HitMeld(hits) => hit_meld(state, actor_index, &hits),
        Action::StealWild(steal) => steal_wild(state, actor_index, &steal),
        Action::DiscardCard(card) => discard_card(state, actor_index, card),
        Action::DrawFromDeck => draw_from_deck(state, actor_index, rng),
    }
}

pub(crate) fn accepts(state: &GameState, actor_index: usize, action: Action) -> bool {
    let mut trial = state.clone();
    legacy_apply_with_rng(&mut trial, action, actor_index, &mut rand::thread_rng())
}

pub(crate) fn assign_plays(
    round: u8,
    cards: &[Card],
    index: usize,
    groups: &mut [Vec<Card>],
    found: &mut Vec<Action>,
) {
    if index == cards.len() {
        if groups.iter().all(|group| !group.is_empty()) && check_round_requirements(round, groups) {
            found.push(Action::PlayMeld(groups.to_vec()));
        }
        return;
    }
    assign_plays(round, cards, index + 1, groups, found);
    for slot in 0..groups.len() {
        groups[slot].push(cards[index]);
        assign_plays(round, cards, index + 1, groups, found);
        groups[slot].pop();
    }
}

pub(crate) fn hit_moves(state: &GameState, actor_index: usize) -> Vec<Action> {
    if !state.players[actor_index].is_on_board || state.board.is_empty() {
        return Vec::new();
    }
    let hand = playable_hand(state, actor_index);
    if hand.is_empty() {
        return Vec::new();
    }
    let additions: Vec<Vec<Vec<Card>>> = state
        .board
        .iter()
        .map(|meld| additions(meld, &hand))
        .collect();
    let mut found = Vec::new();
    let mut hits = Vec::new();
    let mut used = HashSet::new();
    combine_hits(0, &additions, &mut used, &mut hits, &mut found);
    found
        .into_iter()
        .filter(|action| accepts(state, actor_index, action.clone()))
        .collect()
}

pub(crate) fn additions(meld: &[Card], hand: &[Card]) -> Vec<Vec<Card>> {
    let mut found = Vec::new();
    let mut extra = Vec::new();
    collect_additions(meld, hand, 0, &mut extra, &mut found);
    found
}

fn discard_card(state: &mut GameState, actor_index: usize, card: Card) -> bool {
    if !state.players[actor_index].hand.contains(&card) {
        return false;
    }
    let quick = state.drawn_card_id == Some(card.id);
    if !state.players[actor_index].is_on_board
        && !quick
        && card_fits_board(&state.board, &card, state.turn_counter)
    {
        if state.penalty_seat.is_none() || state.penalty_seat == Some(actor_index) {
            state.turn_phase = TurnPhase::PenaltyDrawing;
            state.penalty_seat = Some(actor_index);
        }
        return false;
    }
    let Some(hand) = hand_without(&state.players[actor_index].hand, &[vec![card]]) else {
        return false;
    };
    state.players[actor_index].hand = hand;
    state.deck.discard.push(card);
    if quick {
        state.drawn_card_id = None;
    }
    if state.penalty_seat.is_none() || state.penalty_seat == Some(actor_index) {
        state.turn_phase = TurnPhase::Playing;
        state.penalty_seat = None;
    }
    end_round_if_hand_empty(state, actor_index);
    true
}

fn end_round_if_hand_empty(state: &mut GameState, actor_index: usize) {
    if state.players[actor_index].hand.is_empty() {
        state.round_over = true;
    }
}

fn draw_from_deck(state: &mut GameState, actor_index: usize, rng: &mut impl Rng) -> bool {
    if state.turn_phase != TurnPhase::PenaltyDrawing || state.penalty_seat != Some(actor_index) {
        return false;
    }
    let mut drew = false;
    loop {
        let card = match state.deck.draw_with(rng) {
            TurnDraw::One(card) | TurnDraw::LastCard(card) => card,
            TurnDraw::LastTwo { current, next } => {
                let next_index = (actor_index + 1) % state.players.len();
                state.players[next_index].hand.push(next);
                current
            }
            TurnDraw::Empty => break,
        };
        drew = true;
        if card_fits_board(&state.board, &card, state.turn_counter) {
            state.players[actor_index].hand.push(card);
            continue;
        }
        state.deck.discard.push(card);
        state.turn_phase = TurnPhase::Playing;
        state.penalty_seat = None;
        return true;
    }
    drew
}

fn card_fits_board(board: &[Vec<Card>], card: &Card, turn_counter: u32) -> bool {
    if !card_can_be_played(card, turn_counter) {
        return false;
    }
    board.iter().any(|meld| {
        let mut with = meld.clone();
        with.push(*card);
        validate_set(&with) || validate_run(&with)
    })
}

fn hit_meld(state: &mut GameState, actor_index: usize, hits: &[MeldHit]) -> bool {
    if hits.is_empty() || !state.players[actor_index].is_on_board {
        return false;
    }
    if hits
        .iter()
        .flat_map(|hit| &hit.cards)
        .any(|card| !card_can_be_played(card, state.turn_counter))
    {
        return false;
    }
    let mut board = state.board.clone();
    let mut added = vec![Vec::new(); board.len()];
    let mut removing = Vec::new();
    for hit in hits {
        if hit.cards.is_empty() {
            return false;
        }
        let Some(extra) = added.get_mut(hit.meld_index) else {
            return false;
        };
        extra.extend(hit.cards.iter().copied());
        removing.extend(hit.cards.iter().copied());
    }
    for (meld, extra) in board.iter_mut().zip(&added) {
        if extra.is_empty() {
            continue;
        }
        meld.extend(extra.iter().copied());
        if !validate_set(meld) && !validate_run(meld) {
            return false;
        }
    }
    let Some(hand) = hand_without(&state.players[actor_index].hand, &[removing]) else {
        return false;
    };
    state.board = board;
    state.players[actor_index].hand = hand;
    end_round_if_hand_empty(state, actor_index);
    true
}

fn steal_wild(
    state: &mut GameState,
    actor_index: usize,
    steal: &crate::actions::WildSteal,
) -> bool {
    if !state.players[actor_index].is_on_board || steal.natural.is_wild() || !steal.wild.is_wild() {
        return false;
    }
    if !card_can_be_played(&steal.natural, state.turn_counter) {
        return false;
    }
    let Some(meld) = state.board.get(steal.meld_index) else {
        return false;
    };
    let Some(wild_at) = meld.iter().position(|card| card == &steal.wild) else {
        return false;
    };
    if meld.iter().any(|card| card.id == steal.natural.id)
        || !natural_replaces_wild(meld, &steal.natural)
    {
        return false;
    }
    let Some(mut hand) = hand_without(&state.players[actor_index].hand, &[vec![steal.natural]])
    else {
        return false;
    };
    let mut board = state.board.clone();
    board[steal.meld_index][wild_at] = steal.natural;
    let replaced = &board[steal.meld_index];
    if !validate_set(replaced) && !validate_run(replaced) {
        return false;
    }
    let mut stolen = steal.wild;
    stolen.locked_until_turn = state.turn_counter + 1;
    hand.push(stolen);
    state.board = board;
    state.players[actor_index].hand = hand;
    true
}

fn natural_replaces_wild(meld: &[Card], natural: &Card) -> bool {
    if let Some(rank) = set_natural_rank(meld) {
        return natural.rank == rank;
    }
    if !validate_run(meld) || natural.suit != run_suit(meld) {
        return false;
    }
    run_wild_ranks(meld).contains(&natural.rank)
}

fn set_natural_rank(cards: &[Card]) -> Option<Rank> {
    if !validate_set(cards) {
        return None;
    }
    cards
        .iter()
        .find(|card| !card.is_wild())
        .map(|card| card.rank)
}

fn run_suit(cards: &[Card]) -> Suit {
    cards
        .iter()
        .find(|card| !card.is_wild())
        .map(|card| card.suit)
        .unwrap_or(Suit::None)
}

fn run_wild_ranks(cards: &[Card]) -> Vec<Rank> {
    let naturals: Vec<Rank> = cards
        .iter()
        .filter(|card| !card.is_wild())
        .map(|card| card.rank)
        .collect();
    let mut found = Vec::new();
    add_wild_ranks(&naturals, cards.len(), false, &mut found);
    add_wild_ranks(&naturals, cards.len(), true, &mut found);
    found
}

fn add_wild_ranks(naturals: &[Rank], total: usize, ace_high: bool, found: &mut Vec<Rank>) {
    if naturals.is_empty() {
        return;
    }
    let (floor, ceiling) = if ace_high { (2, 14) } else { (1, 13) };
    let mut min_rank = ceiling;
    let mut max_rank = floor;
    let mut natural_values = Vec::new();
    for rank in naturals {
        let value = rank_value(*rank, ace_high);
        if value < floor || value > ceiling {
            return;
        }
        min_rank = min_rank.min(value);
        max_rank = max_rank.max(value);
        natural_values.push(value);
    }
    let span = usize::from(max_rank - min_rank + 1);
    if span > total {
        return;
    }
    let extra = total - span;
    let before = usize::from(min_rank - floor);
    let after = usize::from(ceiling - max_rank);
    if extra > before + after {
        return;
    }
    for value in min_rank..=max_rank {
        if !natural_values.contains(&value) {
            push_rank(value, found);
        }
    }
    for left in 0..=extra {
        let right = extra - left;
        if left > before || right > after {
            continue;
        }
        for step in 1..=left {
            push_rank(min_rank - step as u8, found);
        }
        for step in 1..=right {
            push_rank(max_rank + step as u8, found);
        }
    }
}

fn push_rank(value: u8, found: &mut Vec<Rank>) {
    let Some(rank) = rank_from_value(value) else {
        return;
    };
    if !found.contains(&rank) {
        found.push(rank);
    }
}

fn rank_from_value(value: u8) -> Option<Rank> {
    match value {
        1 | 14 => Some(Rank::Ace),
        3 => Some(Rank::Three),
        4 => Some(Rank::Four),
        5 => Some(Rank::Five),
        6 => Some(Rank::Six),
        7 => Some(Rank::Seven),
        8 => Some(Rank::Eight),
        9 => Some(Rank::Nine),
        10 => Some(Rank::Ten),
        11 => Some(Rank::Jack),
        12 => Some(Rank::Queen),
        13 => Some(Rank::King),
        _ => None,
    }
}

fn rank_value(rank: Rank, ace_high: bool) -> u8 {
    match rank {
        Rank::Ace => {
            if ace_high {
                14
            } else {
                1
            }
        }
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
        Rank::Two | Rank::Joker => 0,
    }
}

fn play_meld(state: &mut GameState, actor_index: usize, melds: &[Vec<Card>]) -> bool {
    if state.players[actor_index].is_on_board {
        return false;
    }
    if melds
        .iter()
        .flatten()
        .any(|card| !card_can_be_played(card, state.turn_counter))
    {
        return false;
    }
    if !check_round_requirements(state.round_number, melds) {
        return false;
    }
    let Some(hand) = hand_without(&state.players[actor_index].hand, melds) else {
        return false;
    };
    state.board.extend(melds.iter().cloned());
    let player = &mut state.players[actor_index];
    player.hand = hand;
    player.is_on_board = true;
    end_round_if_hand_empty(state, actor_index);
    true
}

fn hand_without(hand: &[Card], melds: &[Vec<Card>]) -> Option<Vec<Card>> {
    let mut next = hand.to_vec();
    for meld in melds {
        for card in meld {
            let index = next.iter().position(|held| held == card)?;
            next.remove(index);
        }
    }
    Some(next)
}

fn push_discard(players: &mut [Player], actor_index: usize, deck: &mut Deck, rng: &mut impl Rng) {
    assert!(players.len() >= 2, "Push is played with 2 or more players");
    let next_index = (actor_index + 1) % players.len();
    let discarded = deck.discard.pop().expect("push discard needs a top card");
    let penalty = draw_one(deck, rng, "push penalty");
    let start = draw_one(deck, rng, "push turn");
    players[next_index].hand.push(discarded);
    players[next_index].hand.push(penalty);
    players[actor_index].hand.push(start);
}

fn draw_one(deck: &mut Deck, rng: &mut impl Rng, why: &str) -> Card {
    match deck.draw_with(rng) {
        TurnDraw::One(card) | TurnDraw::LastCard(card) => card,
        other => panic!("{why} expected one card, got {other:?}"),
    }
}

fn playable_hand(state: &GameState, actor_index: usize) -> Vec<Card> {
    state.players[actor_index]
        .hand
        .iter()
        .copied()
        .filter(|card| card_can_be_played(card, state.turn_counter))
        .collect()
}

fn collect_additions(
    meld: &[Card],
    hand: &[Card],
    index: usize,
    extra: &mut Vec<Card>,
    found: &mut Vec<Vec<Card>>,
) {
    if index == hand.len() {
        if extra.is_empty() {
            return;
        }
        let mut with = meld.to_vec();
        with.extend(extra.iter().copied());
        if validate_set(&with) || validate_run(&with) {
            found.push(extra.clone());
        }
        return;
    }
    collect_additions(meld, hand, index + 1, extra, found);
    extra.push(hand[index]);
    collect_additions(meld, hand, index + 1, extra, found);
    extra.pop();
}

fn combine_hits(
    meld_index: usize,
    additions: &[Vec<Vec<Card>>],
    used: &mut HashSet<u32>,
    hits: &mut Vec<MeldHit>,
    found: &mut Vec<Action>,
) {
    if meld_index == additions.len() {
        if !hits.is_empty() {
            found.push(Action::HitMeld(hits.clone()));
        }
        return;
    }
    combine_hits(meld_index + 1, additions, used, hits, found);
    for extra in &additions[meld_index] {
        if extra.iter().any(|card| used.contains(&card.id)) {
            continue;
        }
        for card in extra {
            used.insert(card.id);
        }
        hits.push(MeldHit {
            meld_index,
            cards: extra.clone(),
        });
        combine_hits(meld_index + 1, additions, used, hits, found);
        hits.pop();
        for card in extra {
            used.remove(&card.id);
        }
    }
}

/// A reshuffle push and a penalty draw match `apply_with_rng` for the same seed.
/// A fitting discard that is refused enters the same penalty on both paths.
#[test]
fn test_legacy_apply_with_rng_matches_the_live_seeded_path() {
    let push = push_table();
    let (seed_a, seed_b) = seeds_whose_draws_differ(&push.deck, 2);
    assert_same_push(&push, seed_a);
    assert_same_push(&push, seed_b);

    let penalty = penalty_table();
    let (seed_a, seed_b) = seeds_whose_draws_differ(&penalty.deck, 3);
    assert_same_penalty(&penalty, seed_a);
    assert_same_penalty(&penalty, seed_b);

    let mut live = fitting_discard_table();
    let mut oracle = live.clone();
    let seven = live.players[0].hand[0];
    assert!(!live.apply_with_rng(Action::DiscardCard(seven), 0, &mut StdRng::seed_from_u64(1),));
    assert!(!legacy_apply_with_rng(
        &mut oracle,
        Action::DiscardCard(seven),
        0,
        &mut StdRng::seed_from_u64(1),
    ));
    assert_eq!(live.players, oracle.players);
    assert_eq!(live.deck, oracle.deck);
    assert_eq!(live.board, oracle.board);
    assert_eq!(live.turn_phase, TurnPhase::PenaltyDrawing);
    assert_eq!(oracle.turn_phase, TurnPhase::PenaltyDrawing);
    assert_eq!(live.penalty_seat, oracle.penalty_seat);

    let mut live = steal_table();
    let mut oracle = live.clone();
    let steal = crate::actions::WildSteal {
        meld_index: 0,
        wild: live.board[0][2],
        natural: live.players[0].hand[0],
    };
    assert!(live.apply_with_rng(Action::StealWild(steal), 0, &mut StdRng::seed_from_u64(1),));
    assert!(legacy_apply_with_rng(
        &mut oracle,
        Action::StealWild(steal),
        0,
        &mut StdRng::seed_from_u64(1),
    ));
    assert_eq!(live.players, oracle.players);
    assert_eq!(live.board, oracle.board);
    assert_eq!(
        live.players[0]
            .hand
            .last()
            .map(|card| card.locked_until_turn),
        Some(1)
    );
}

/// Plays, hits, and additions from the oracle match the live generator on a bounded hand.
#[test]
fn test_legacy_assign_plays_hit_moves_and_additions_match_the_live_generator() {
    let plays_state = bounded_play_hand();
    let live_plays: Vec<Action> = generate_legal_moves(&plays_state, 0)
        .into_iter()
        .filter(|action| matches!(action, Action::PlayMeld(_)))
        .collect();
    let mut found = Vec::new();
    let cards = plays_state.players[0].hand.clone();
    let mut groups = vec![Vec::new(); 2];
    assign_plays(plays_state.round_number, &cards, 0, &mut groups, &mut found);
    let oracle_plays: Vec<Action> = found
        .into_iter()
        .filter(|action| accepts(&plays_state, 0, action.clone()))
        .collect();
    assert!(!oracle_plays.is_empty());
    assert_eq!(sorted(oracle_plays), sorted(live_plays));

    let hits_state = bounded_hit_hand();
    let live_hits: Vec<Action> = generate_legal_moves(&hits_state, 0)
        .into_iter()
        .filter(|action| matches!(action, Action::HitMeld(_)))
        .collect();
    let oracle_hits = sorted(hit_moves(&hits_state, 0));
    assert_eq!(oracle_hits, sorted(live_hits));
    assert!(!oracle_hits.is_empty());

    let eight = hits_state.players[0].hand[0];
    let extras = additions(
        &hits_state.board[0],
        &[eight, hits_state.players[0].hand[1]],
    );
    assert!(extras.iter().any(|extra| extra == &vec![eight]));
    assert!(extras
        .iter()
        .all(|extra| extra.iter().any(|card| card.rank == Rank::Eight)));
}

/// Chain: Deck::new → is_wild → seeded shuffle → deal → push and penalty draw.
/// The oracle and `apply_with_rng` leave the same table. Every card id stays in play.
#[test]
fn test_suit_rank_card_deck_new_is_wild_seeded_shuffle_deal_legacy_oracle_matches_apply() {
    let mut deck = Deck::new();
    let original: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();
    assert_eq!(deck.cards.iter().filter(|card| card.is_wild()).count(), 12);
    deck.shuffle_with(&mut StdRng::seed_from_u64(1));
    let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
    deal_initial_hands(&mut players, &mut deck);
    let mut state = GameState::new(players, deck);
    let kept = state.players[0]
        .hand
        .pop()
        .expect("the dealt hand has a card");
    state.deck.discard.append(&mut state.players[0].hand);
    state.players[0].hand = vec![kept];
    state.deck.discard.append(&mut state.deck.cards);
    assert!(state.deck.discard.len() >= 4);

    let mut live = state.clone();
    let mut oracle = state.clone();
    let seed = 5u64;
    assert!(live.apply_with_rng(Action::PushDiscard, 0, &mut StdRng::seed_from_u64(seed),));
    assert!(legacy_apply_with_rng(
        &mut oracle,
        Action::PushDiscard,
        0,
        &mut StdRng::seed_from_u64(seed),
    ));
    assert_eq!(live.players, oracle.players);
    assert_eq!(live.deck, oracle.deck);
    assert_eq!(live.board, oracle.board);
    assert_eq!(live.drawn_card_id, oracle.drawn_card_id);
    assert_eq!(live.turn_phase, oracle.turn_phase);
    assert_eq!(ids_on_table(&live), sorted_ids(&original));

    let mut shoe = cards_on_table(&state);
    let five = take_suited(&mut shoe, Suit::Hearts, Rank::Five);
    let six = take_suited(&mut shoe, Suit::Hearts, Rank::Six);
    let joker = take_suited(&mut shoe, Suit::None, Rank::Joker);
    let held = take_suited(&mut shoe, Suit::Hearts, Rank::Seven);
    let four = take_suited(&mut shoe, Suit::Hearts, Rank::Four);
    let eight = take_suited(&mut shoe, Suit::Hearts, Rank::Eight);
    let another = take_suited(&mut shoe, Suit::Hearts, Rank::Seven);
    let king = take_suited(&mut shoe, Suit::Spades, Rank::King);
    state.players[0].hand = vec![held];
    state.players[1].hand = shoe;
    state.board = vec![vec![five, six, joker]];
    state.deck.cards.clear();
    state.deck.discard = vec![four, eight, another, king];
    state.turn_phase = TurnPhase::PenaltyDrawing;
    state.penalty_seat = Some(0);
    let mut live = state.clone();
    let mut oracle = state;
    let seed = 6u64;
    assert!(live.apply_with_rng(Action::DrawFromDeck, 0, &mut StdRng::seed_from_u64(seed),));
    assert!(legacy_apply_with_rng(
        &mut oracle,
        Action::DrawFromDeck,
        0,
        &mut StdRng::seed_from_u64(seed),
    ));
    assert_eq!(live.players, oracle.players);
    assert_eq!(live.deck, oracle.deck);
    assert_eq!(live.board, oracle.board);
    assert_eq!(live.turn_phase, oracle.turn_phase);
    assert_eq!(live.penalty_seat, oracle.penalty_seat);
    assert_eq!(ids_on_table(&live), sorted_ids(&original));
}

fn assert_same_push(before: &GameState, seed: u64) {
    let mut live = before.clone();
    let mut oracle = before.clone();
    assert!(live.apply_with_rng(Action::PushDiscard, 0, &mut StdRng::seed_from_u64(seed),));
    assert!(legacy_apply_with_rng(
        &mut oracle,
        Action::PushDiscard,
        0,
        &mut StdRng::seed_from_u64(seed),
    ));
    assert_eq!(live.players, oracle.players);
    assert_eq!(live.deck, oracle.deck);
    assert_eq!(live.drawn_card_id, oracle.drawn_card_id);
    assert_eq!(live.board, oracle.board);
    assert_eq!(live.turn_phase, oracle.turn_phase);
    assert!(!live.round_over);
}

fn assert_same_penalty(before: &GameState, seed: u64) {
    let mut live = before.clone();
    let mut oracle = before.clone();
    assert!(live.apply_with_rng(Action::DrawFromDeck, 0, &mut StdRng::seed_from_u64(seed),));
    assert!(legacy_apply_with_rng(
        &mut oracle,
        Action::DrawFromDeck,
        0,
        &mut StdRng::seed_from_u64(seed),
    ));
    assert_eq!(live.players, oracle.players);
    assert_eq!(live.deck, oracle.deck);
    assert_eq!(live.turn_phase, oracle.turn_phase);
    assert_eq!(live.penalty_seat, oracle.penalty_seat);
    assert_eq!(live.board, oracle.board);
}

fn push_table() -> GameState {
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![card(20, Suit::Spades, Rank::King)];
    players[1].hand = vec![card(21, Suit::Diamonds, Rank::Nine)];
    GameState::new(
        players,
        Deck {
            cards: Vec::new(),
            discard: vec![
                card(1, Suit::Hearts, Rank::Three),
                card(2, Suit::Clubs, Rank::Four),
                card(3, Suit::Diamonds, Rank::Five),
                card(4, Suit::Spades, Rank::Six),
            ],
        },
    )
}

fn penalty_table() -> GameState {
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![card(4, Suit::Hearts, Rank::Seven)];
    players[1].hand = vec![card(10, Suit::Diamonds, Rank::Nine)];
    let mut state = GameState::new(
        players,
        Deck {
            cards: Vec::new(),
            discard: vec![
                card(6, Suit::Hearts, Rank::Four),
                card(7, Suit::Hearts, Rank::Eight),
                card(8, Suit::Hearts, Rank::Seven),
                card(9, Suit::Clubs, Rank::King),
            ],
        },
    );
    state.board = vec![vec![
        card(1, Suit::Hearts, Rank::Five),
        card(2, Suit::Hearts, Rank::Six),
        card(3, Suit::None, Rank::Joker),
    ]];
    state.turn_phase = TurnPhase::PenaltyDrawing;
    state.penalty_seat = Some(0);
    state
}

fn steal_table() -> GameState {
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].is_on_board = true;
    players[0].hand = vec![card(11, Suit::Diamonds, Rank::Five)];
    let mut state = GameState::new(players, Deck::new());
    state.board = vec![vec![
        card(1, Suit::Hearts, Rank::Five),
        card(2, Suit::Spades, Rank::Five),
        card(3, Suit::None, Rank::Joker),
    ]];
    state.deck.cards.clear();
    state.deck.discard = vec![card(4, Suit::Clubs, Rank::Queen)];
    state
}

fn fitting_discard_table() -> GameState {
    let mut state = penalty_table();
    state.turn_phase = TurnPhase::Playing;
    state.penalty_seat = None;
    state.deck.cards = vec![card(30, Suit::Clubs, Rank::King)];
    state.deck.discard = vec![card(31, Suit::Diamonds, Rank::Queen)];
    state
}

fn bounded_play_hand() -> GameState {
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![
        card(12, Suit::Hearts, Rank::Four),
        card(13, Suit::Spades, Rank::Four),
        card(14, Suit::Clubs, Rank::Four),
        card(15, Suit::Hearts, Rank::Five),
        card(16, Suit::Spades, Rank::Five),
        card(17, Suit::Clubs, Rank::Five),
        card(18, Suit::Spades, Rank::King),
    ];
    let mut state = GameState::new(players, Deck::new());
    state.deck.cards.clear();
    state.deck.discard = vec![card(19, Suit::Diamonds, Rank::Queen)];
    state
}

fn bounded_hit_hand() -> GameState {
    let mut state = bounded_play_hand();
    state.players[0].is_on_board = true;
    state.players[0]
        .hand
        .insert(0, card(11, Suit::Hearts, Rank::Eight));
    state.board = vec![vec![
        card(1, Suit::Spades, Rank::Eight),
        card(2, Suit::Clubs, Rank::Eight),
        card(3, Suit::Diamonds, Rank::Eight),
    ]];
    state
}

fn card(id: u32, suit: Suit, rank: Rank) -> Card {
    Card {
        id,
        suit,
        rank,
        locked_until_turn: 0,
    }
}

fn sorted(mut actions: Vec<Action>) -> Vec<Action> {
    actions.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));
    actions
}

fn seeds_whose_draws_differ(deck: &Deck, count: usize) -> (u64, u64) {
    let first = draw_ids(deck, 0, count);
    for seed in 1..64 {
        if draw_ids(deck, seed, count) != first {
            return (0, seed);
        }
    }
    panic!("expected two seeds to draw in a different order");
}

fn draw_ids(deck: &Deck, seed: u64, count: usize) -> Vec<u32> {
    let mut deck = deck.clone();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut ids = Vec::new();
    for _ in 0..count {
        let card = match deck.draw_with(&mut rng) {
            crate::deck::TurnDraw::One(card) | crate::deck::TurnDraw::LastCard(card) => card,
            other => panic!("expected one card, got {other:?}"),
        };
        ids.push(card.id);
    }
    ids
}

fn take_suited(cards: &mut Vec<Card>, suit: Suit, rank: Rank) -> Card {
    let index = cards
        .iter()
        .position(|card| card.suit == suit && card.rank == rank)
        .expect("the shoe contains this suit and rank");
    cards.remove(index)
}

fn cards_on_table(state: &GameState) -> Vec<Card> {
    let mut cards = Vec::new();
    for player in &state.players {
        cards.extend(player.hand.iter().copied());
    }
    for meld in &state.board {
        cards.extend(meld.iter().copied());
    }
    cards.extend(state.deck.cards.iter().copied());
    cards.extend(state.deck.discard.iter().copied());
    cards
}

fn ids_on_table(state: &GameState) -> Vec<u32> {
    sorted_ids(
        &cards_on_table(state)
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>(),
    )
}

fn sorted_ids(ids: &[u32]) -> Vec<u32> {
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    ids
}
