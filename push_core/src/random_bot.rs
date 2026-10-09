//! A seat that plays by choosing among the actions the engine would accept.

use std::collections::BTreeMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::actions::{validate_action, Action, MeldHit};
use crate::card::{Card, Rank, Suit};
use crate::deck::Deck;
use crate::game_state::{GameState, TurnPhase};
use crate::legal_moves::{push_is_legal, visit_legal_kind, LegalKind};
use crate::player::{deal_initial_hands, Player};
use crate::resolution::ActionResolution;
use crate::validation::{card_can_be_played, check_round_requirements, validate_run, validate_set};

/// Stop after this many turns so a stuck table fails instead of running on.
const TURN_LIMIT: u32 = 8_000;

/// Two seats play five rounds.
///
/// `seed` shuffles the shoe, chooses each action, and shuffles a push or a penalty
/// draw that recycles the discard. The table is dealt, one
/// discard is flipped, and the seats take turns. A turn takes or pushes, lays
/// down when the round allows it, may hit, may steal, then discards. A fitting
/// card that cannot be discarded draws until a safe card leaves. When a round
/// ends, the hands are scored and the next round is dealt. After the fifth
/// round, `round_number` is 6 and that round has not been played.
pub fn play_random_game(seed: u64) -> GameState {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut state = new_two_seat_table(&mut rng);
    let mut seat = 0usize;
    let mut finished = 0u8;
    let mut turns = 0u32;
    while finished < 5 {
        turns += 1;
        assert!(
            turns <= TURN_LIMIT,
            "seed {seed} did not finish five rounds in {TURN_LIMIT} turns"
        );
        play_random_turn(&mut state, seat, &mut rng);
        if state.round_over {
            assert!(state.advance_to_next_round_with(&mut rng));
            finished += 1;
            seat = 0;
            continue;
        }
        if state.turn_phase == TurnPhase::PenaltyDrawing {
            continue;
        }
        state.advance_turn();
        seat = (seat + 1) % state.players.len();
    }
    state
}

/// Shuffles a fresh shoe, deals two seats, and flips one discard.
pub fn new_two_seat_table(rng: &mut impl Rng) -> GameState {
    let mut deck = Deck::new();
    deck.shuffle_with(rng);
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    deal_initial_hands(&mut players, &mut deck);
    let starter = deck
        .cards
        .pop()
        .expect("the first round starts a discard pile");
    deck.discard.push(starter);
    GameState::new(players, deck)
}

/// Plays one turn for `actor`.
///
/// The seat pushes when the deck can give the next seat a card and this seat a
/// card. Otherwise the seat takes the discard. On the board, a hand of one card
/// that can be discarded is discarded first, which ends the round. If nothing
/// on the board can be laid down or hit, one card is discarded without taking.
/// Off the board, one safe card is still pushed or taken. It lays down when the round allows
/// it, then lays additional valid sets or runs while on the board, hits until
/// nothing else fits, and sometimes steals. The turn ends with a discard, or with
/// a draw until a safe card. That draw, and a push that recycles the discard, use
/// `rng`. A hand above 11 cards is played one card at a time.
pub fn play_random_turn(state: &mut GameState, actor: usize, rng: &mut impl Rng) {
    if state.round_over || actor >= state.players.len() {
        return;
    }
    if state.turn_phase == TurnPhase::PenaltyDrawing && state.penalty_seat == Some(actor) {
        let _ = state.apply_with_rng(Action::DrawFromDeck, actor, rng);
        return;
    }
    if discard_to_go_out(state, actor, rng) {
        return;
    }
    if state.players[actor].hand.len() > 11 {
        play_one_card_at_a_time(state, actor, rng);
        return;
    }
    if state.players[actor].is_on_board && !can_play_or_hit(state, actor) {
        if apply_one_discard(state, actor, rng) {
            return;
        }
    }
    if state.drawn_card_id.is_none() {
        if let Some(action) = choose_opening(state) {
            apply_listed(state, actor, action, rng);
        }
    }
    if state.round_over {
        return;
    }
    lay_down(state, actor, rng);
    if state.round_over {
        return;
    }
    if !state.players[actor].is_on_board {
        apply_kind(state, actor, rng, LegalKind::Play);
    }
    if state.round_over {
        return;
    }
    if state.players[actor].is_on_board {
        let mut guard = state.players[actor].hand.len();
        while guard > 0 {
            guard -= 1;
            if state.round_over {
                return;
            }
            let before = state.players[actor].hand.len();
            lay_down(state, actor, rng);
            if state.players[actor].hand.len() >= before {
                break;
            }
        }
        if state.round_over {
            return;
        }
        for _ in 0..12 {
            if state.round_over || !apply_widest_hit(state, actor, rng) {
                break;
            }
        }
        if state.round_over {
            return;
        }
        if rng.gen_bool(0.25) {
            apply_kind(state, actor, rng, LegalKind::Steal);
        }
    }
    if state.round_over || apply_one_discard(state, actor, rng) {
        return;
    }
    draw_until_a_safe_card(state, actor, rng);
}

/// Take, lay down, hit one fitting card at a time, and discard.
///
/// Listing every meld of a large hand is too slow. A lay-down is checked with
/// [`check_round_requirements`]. Each hit is one card the engine accepts.
fn play_one_card_at_a_time(state: &mut GameState, actor: usize, rng: &mut impl Rng) {
    if state.drawn_card_id.is_none() && !state.deck.discard.is_empty() {
        apply_listed(state, actor, Action::TakeDiscard, rng);
    }
    if state.round_over {
        return;
    }
    lay_down(state, actor, rng);
    let mut guard = state.players[actor].hand.len();
    while guard > 0 {
        guard -= 1;
        if state.round_over {
            return;
        }
        let Some(action) = one_card_hit(state, actor, rng) else {
            break;
        };
        apply_listed(state, actor, action, rng);
    }
    if state.round_over || apply_one_discard(state, actor, rng) {
        return;
    }
    draw_until_a_safe_card(state, actor, rng);
}

fn one_card_hit(state: &GameState, actor: usize, rng: &mut impl Rng) -> Option<Action> {
    if !state.players[actor].is_on_board {
        return None;
    }
    let mut hits = Vec::new();
    for card in &state.players[actor].hand {
        for meld_index in 0..state.board.len() {
            let action = Action::HitMeld(vec![MeldHit {
                meld_index,
                cards: vec![*card],
            }]);
            let mut trial = state.clone();
            if trial.apply(action.clone(), actor) {
                hits.push(action);
            }
        }
    }
    if hits.is_empty() {
        return None;
    }
    Some(hits[rng.gen_range(0..hits.len())].clone())
}

fn apply_one_discard(state: &mut GameState, actor: usize, rng: &mut impl Rng) -> bool {
    let mut discards = Vec::new();
    visit_legal_kind(state, actor, LegalKind::Discard, &mut |action| {
        discards.push(action);
    });
    if discards.is_empty() {
        return false;
    }
    let action = discards[rng.gen_range(0..discards.len())].clone();
    apply_listed(state, actor, action, rng);
    true
}

/// On the board, with nothing to lay down or hit, a discard sheds one card.
/// Taking first would put that card back.
fn can_play_or_hit(state: &GameState, actor: usize) -> bool {
    let mut found = false;
    visit_legal_kind(state, actor, LegalKind::Play, &mut |_| found = true);
    if found {
        return true;
    }
    visit_legal_kind(state, actor, LegalKind::Hit, &mut |_| found = true);
    found
}

/// One card, once that seat is on the board, is a discard that ends the round.
/// Taking or pushing first would put a card back into the hand. Off the board,
/// a single safe card is still pushed or taken.
fn discard_to_go_out(state: &mut GameState, actor: usize, rng: &mut impl Rng) -> bool {
    if !state.players[actor].is_on_board {
        return false;
    }
    let card = match state.players[actor].hand.as_slice() {
        [card] => *card,
        _ => return false,
    };
    let action = Action::DiscardCard(card);
    if !matches!(
        validate_action(state, actor, &action),
        ActionResolution::Accepted(_)
    ) {
        return false;
    }
    apply_listed(state, actor, action, rng);
    true
}

fn choose_opening(state: &GameState) -> Option<Action> {
    if state.deck.discard.is_empty() {
        return None;
    }
    if push_is_legal(&state.deck) {
        Some(Action::PushDiscard)
    } else {
        Some(Action::TakeDiscard)
    }
}

fn apply_widest_hit(state: &mut GameState, actor: usize, rng: &mut impl Rng) -> bool {
    let mut hits = Vec::new();
    visit_legal_kind(state, actor, LegalKind::Hit, &mut |action| {
        hits.push(action)
    });
    if hits.is_empty() {
        return false;
    }
    let widest = hits.iter().map(hit_width).max().expect("a hit has a width");
    let choices: Vec<Action> = hits
        .into_iter()
        .filter(|action| hit_width(action) == widest)
        .collect();
    let action = choices[rng.gen_range(0..choices.len())].clone();
    apply_listed(state, actor, action, rng);
    true
}

fn hit_width(action: &Action) -> usize {
    match action {
        Action::HitMeld(hits) => hits.iter().map(|hit| hit.cards.len()).sum(),
        _ => 0,
    }
}

fn apply_kind(state: &mut GameState, actor: usize, rng: &mut impl Rng, kind: LegalKind) -> bool {
    let Some(action) = choose_kind(state, actor, rng, kind) else {
        return false;
    };
    apply_listed(state, actor, action, rng);
    true
}

fn choose_kind(
    state: &GameState,
    actor: usize,
    rng: &mut impl Rng,
    kind: LegalKind,
) -> Option<Action> {
    let mut choices = Vec::new();
    visit_legal_kind(state, actor, kind, &mut |action| choices.push(action));
    if choices.is_empty() {
        return None;
    }
    Some(choices[rng.gen_range(0..choices.len())].clone())
}

fn apply_listed(state: &mut GameState, actor: usize, action: Action, rng: &mut impl Rng) {
    let refused = action.clone();
    assert!(
        state.apply_with_rng(action, actor, rng),
        "a listed action was refused: {refused:?}"
    );
}

/// Discards a card that fits, which opens the penalty, then draws until a safe card.
fn draw_until_a_safe_card(state: &mut GameState, actor: usize, rng: &mut impl Rng) {
    let hand = state.players[actor].hand.clone();
    if hand.is_empty() {
        return;
    }
    let card = hand[rng.gen_range(0..hand.len())];
    if state.apply_with_rng(Action::DiscardCard(card), actor, rng) {
        return;
    }
    if state.turn_phase == TurnPhase::PenaltyDrawing && state.penalty_seat == Some(actor) {
        let _ = state.apply_with_rng(Action::DrawFromDeck, actor, rng);
    }
}

fn lay_down(state: &mut GameState, actor: usize, rng: &mut impl Rng) {
    if state.round_over {
        return;
    }
    if state.players[actor].is_on_board {
        lay_free_meld(state, actor, rng);
        return;
    }
    let cards: Vec<Card> = state.players[actor]
        .hand
        .iter()
        .copied()
        .filter(|card| card_can_be_played(card, state.turn_counter))
        .collect();
    let Some(melds) = melds_for_round(state.round_number, &cards) else {
        return;
    };
    let action = Action::PlayMeld(melds);
    let mut trial = state.clone();
    if trial.apply(action.clone(), actor) {
        apply_listed(state, actor, action, rng);
    }
}

fn lay_free_meld(state: &mut GameState, actor: usize, rng: &mut impl Rng) {
    if state.players[actor].hand.len() > 11 {
        lay_free_meld_large(state, actor, rng);
        return;
    }
    apply_kind(state, actor, rng, LegalKind::Play);
}

fn lay_free_meld_large(state: &mut GameState, actor: usize, rng: &mut impl Rng) {
    let cards: Vec<Card> = state.players[actor]
        .hand
        .iter()
        .copied()
        .filter(|card| card_can_be_played(card, state.turn_counter))
        .collect();
    let mut candidates = set_groups(&cards);
    candidates.extend(run_groups(&cards, 4));
    if candidates.is_empty() {
        return;
    }
    let start = rng.gen_range(0..candidates.len());
    for offset in 0..candidates.len() {
        let meld = &candidates[(start + offset) % candidates.len()];
        let action = Action::PlayMeld(vec![meld.clone()]);
        let mut trial = state.clone();
        if trial.apply(action.clone(), actor) {
            apply_listed(state, actor, action, rng);
            return;
        }
    }
}

fn melds_for_round(round: u8, cards: &[Card]) -> Option<Vec<Vec<Card>>> {
    let sets = set_groups(cards);
    let runs_of_four = run_groups(cards, 4);
    let runs_of_seven = run_groups(cards, 7);
    let melds = match round {
        1 => disjoint_groups(&[&sets, &sets]),
        2 => disjoint_groups(&[&sets, &runs_of_four]),
        3 => disjoint_groups(&[&runs_of_four, &runs_of_four]),
        4 => disjoint_groups(&[&sets, &sets, &sets]),
        5 => disjoint_groups(&[&sets, &runs_of_seven]),
        _ => None,
    }?;
    check_round_requirements(round, &melds).then_some(melds)
}

fn disjoint_groups(pools: &[&[Vec<Card>]]) -> Option<Vec<Vec<Card>>> {
    let mut chosen = Vec::new();
    pick_disjoint(pools, 0, &mut chosen)
}

fn pick_disjoint(
    pools: &[&[Vec<Card>]],
    index: usize,
    chosen: &mut Vec<Vec<Card>>,
) -> Option<Vec<Vec<Card>>> {
    if index == pools.len() {
        return Some(chosen.clone());
    }
    for group in pools[index] {
        if chosen.iter().any(|held| shares_a_card(held, group)) {
            continue;
        }
        chosen.push(group.clone());
        if let Some(found) = pick_disjoint(pools, index + 1, chosen) {
            return Some(found);
        }
        chosen.pop();
    }
    None
}

fn shares_a_card(left: &[Card], right: &[Card]) -> bool {
    left.iter()
        .any(|card| right.iter().any(|other| other.id == card.id))
}

fn set_groups(cards: &[Card]) -> Vec<Vec<Card>> {
    let wilds: Vec<Card> = cards
        .iter()
        .copied()
        .filter(|card| card.is_wild())
        .collect();
    let mut by_rank: Vec<(Rank, Vec<Card>)> = Vec::new();
    for card in cards {
        if card.is_wild() {
            continue;
        }
        if let Some((_, group)) = by_rank.iter_mut().find(|(rank, _)| *rank == card.rank) {
            group.push(*card);
        } else {
            by_rank.push((card.rank, vec![*card]));
        }
    }
    let mut found = Vec::new();
    for (_, group) in &by_rank {
        let mut start = 0;
        while start + 3 <= group.len() {
            let meld = group[start..start + 3].to_vec();
            if validate_set(&meld) {
                found.push(meld);
            }
            start += 3;
        }
        let leftover = &group[start..];
        if leftover.is_empty() || leftover.len() >= 3 {
            continue;
        }
        let need = 3 - leftover.len();
        for wilds in wilds.windows(need) {
            let mut meld = leftover.to_vec();
            meld.extend(wilds.iter().copied());
            if validate_set(&meld) {
                found.push(meld);
            }
        }
    }
    if wilds.len() >= 3 {
        for wilds in wilds.windows(3) {
            let meld = wilds.to_vec();
            if validate_set(&meld) {
                found.push(meld);
            }
        }
    }
    found
}

fn run_groups(cards: &[Card], min_len: usize) -> Vec<Vec<Card>> {
    let wilds: Vec<Card> = cards
        .iter()
        .copied()
        .filter(|card| card.is_wild())
        .collect();
    let mut found = Vec::new();
    for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
        for ace_high in [false, true] {
            let mut by_rank: BTreeMap<u8, Card> = BTreeMap::new();
            for card in cards {
                if card.suit != suit || card.is_wild() {
                    continue;
                }
                let Some(value) = rank_value(card.rank, ace_high) else {
                    continue;
                };
                by_rank.entry(value).or_insert(*card);
            }
            let values: Vec<u8> = by_rank.keys().copied().collect();
            for start in 0..values.len() {
                for end in start..values.len() {
                    let window = &values[start..=end];
                    let span = usize::from(window[window.len() - 1] - window[0] + 1);
                    let holes = span - window.len();
                    if holes > wilds.len() || span > 13 {
                        continue;
                    }
                    let wild_count = if span >= min_len {
                        holes
                    } else if holes + (min_len - span) <= wilds.len() {
                        holes + (min_len - span)
                    } else {
                        continue;
                    };
                    if wild_count == 0 {
                        let meld: Vec<Card> = window.iter().map(|value| by_rank[value]).collect();
                        if validate_run(&meld) {
                            found.push(meld);
                        }
                        continue;
                    }
                    for chosen in wilds.windows(wild_count) {
                        let mut meld: Vec<Card> =
                            window.iter().map(|value| by_rank[value]).collect();
                        meld.extend(chosen.iter().copied());
                        if meld.len() <= 13 && validate_run(&meld) {
                            found.push(meld);
                        }
                    }
                }
            }
        }
    }
    found
}

fn rank_value(rank: Rank, ace_high: bool) -> Option<u8> {
    Some(match rank {
        Rank::Ace => {
            if ace_high {
                14
            } else {
                1
            }
        }
        Rank::Two => 2,
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
        Rank::Joker => return None,
    })
}
