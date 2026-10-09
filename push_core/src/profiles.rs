//! Seat profiles for a headless match.
//!
//! [`BotProfile::PointAverse`] pushes a discard worth 10 or more and discards
//! the highest card it is allowed to play. [`BotProfile::Hoarder`] keeps wilds
//! through round 3. From round 4 it plays them, hits with them, discards them,
//! and pushes them. [`BotProfile::Random`] is the seat from [`crate::random_bot`].

use std::cmp::Reverse;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::actions::{validate_action, Action};
use crate::card::Card;
use crate::game_state::{GameState, TurnPhase};
use crate::legal_moves::{push_is_legal, visit_legal_kind_within, LegalKind};
use crate::random_bot::{
    finish_random_game, game_finished_five_rounds, melds_for_round, new_two_seat_table,
    FinishedGame, ACTION_SAMPLE_CAP, LARGE_HAND_WALK_NODES, TURN_LIMIT,
};
use crate::resolution::ActionResolution;
use crate::validation::card_can_be_played;

/// Rounds 4 and 5. A hoarder stops keeping wilds when the round reaches this.
const HOARDER_LATE_ROUND: u8 = 4;

/// A 10 through king, an ace, or a wild. Ranks 3–9 stay at 5 and are low.
const HIGH_CARD_POINTS: u32 = 10;

/// Turns already played in this round before an off-board seat pushes a low card.
///
/// Taking every low discard leaves the draw pile untouched. Two runs of four
/// never show up, and the round does not end.
const STUCK_ROUND_TURNS: u32 = 24;

/// How one seat chooses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BotProfile {
    /// [`crate::random_bot::play_random_turn`].
    Random,
    /// Push and discard high cards.
    PointAverse,
    /// Keep wilds until round 4.
    Hoarder,
    /// Hold at most this many wilds. Extra wilds are shed like a point-averse seat.
    Keep(u8),
}

impl BotProfile {
    /// `random`, `point-averse`, `hoarder`, or `keep-<count>`.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "random" => Some(Self::Random),
            "point-averse" => Some(Self::PointAverse),
            "hoarder" => Some(Self::Hoarder),
            other => {
                let count = other.strip_prefix("keep-")?.parse::<u8>().ok()?;
                Some(Self::Keep(count))
            }
        }
    }

    /// The name [`Self::parse`] accepts.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::PointAverse => "point-averse",
            Self::Hoarder => "hoarder",
            Self::Keep(_) => "keep",
        }
    }

    /// `keep-2` includes the count. The other names match [`Self::as_str`].
    pub fn label(self) -> String {
        match self {
            Self::Keep(count) => format!("keep-{count}"),
            other => other.as_str().to_string(),
        }
    }
}

/// Two seats play five rounds. Each seat uses its own profile.
///
/// Two [`BotProfile::Random`] seats are [`finish_random_game`]. Seed 1 still
/// scores 5 and 85 on that path. A strategic seat pushes, takes, lays down,
/// hits, and discards. It does not steal. A hand above 11 still stops after
/// 10,000 search steps, and a play or hit walk still keeps 100 actions.
pub fn finish_profile_game(seed: u64, seats: [BotProfile; 2]) -> FinishedGame {
    if seats == [BotProfile::Random, BotProfile::Random] {
        return finish_random_game(seed);
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let mut state = new_two_seat_table(&mut rng);
    let mut seat = 0usize;
    let mut finished = 0u8;
    let mut turns = 0u32;
    while finished < 5 {
        turns += 1;
        assert!(
            turns <= TURN_LIMIT,
            "seed {seed} did not finish five rounds in {TURN_LIMIT} turns at round {} hands {}/{} draw {} board {} on_board {:?}/{:?}",
            state.round_number,
            state.players[0].hand.len(),
            state.players[1].hand.len(),
            state.deck.cards.len(),
            state.board.len(),
            state.players[0].is_on_board,
            state.players[1].is_on_board
        );
        play_profile_turn(&mut state, seat, &mut rng, seats[seat]);
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
    debug_assert!(game_finished_five_rounds(&state));
    FinishedGame { seed, turns, state }
}

/// Plays one turn for `actor` using `profile`.
///
/// [`BotProfile::Random`] calls [`crate::random_bot::play_random_turn`].
pub fn play_profile_turn(
    state: &mut GameState,
    actor: usize,
    rng: &mut impl Rng,
    profile: BotProfile,
) {
    if profile == BotProfile::Random {
        crate::random_bot::play_random_turn(state, actor, rng);
        return;
    }
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
    if state.players[actor].is_on_board && !can_shed_by_playing(state, actor, profile) {
        if shed_one(state, actor, rng, profile) {
            return;
        }
    }
    if state.drawn_card_id.is_none() && !state.players[actor].is_on_board {
        if let Some(action) = choose_opening(state, actor, profile) {
            apply(state, actor, action, rng);
        }
    }
    if state.round_over {
        return;
    }
    if state.players[actor].is_on_board {
        lay_and_hit(state, actor, rng, profile);
    } else {
        lay_down_off_board(state, actor, rng, profile);
    }
    if state.round_over || shed_one(state, actor, rng, profile) {
        return;
    }
    draw_until_a_safe_card(state, actor, rng, profile);
}

/// Off the board, the round requirement is a targeted set and run search.
///
/// The play visitor stops after 10,000 steps on a hand above 11. A push grows
/// that hand, so the visitor never reaches three sets. This search is the one
/// the random seat already uses to lay down.
fn lay_down_off_board(
    state: &mut GameState,
    actor: usize,
    rng: &mut impl Rng,
    profile: BotProfile,
) {
    if state.round_over {
        return;
    }
    let round = state.round_number;
    let cards: Vec<Card> = state.players[actor]
        .hand
        .iter()
        .copied()
        .filter(|card| card_can_be_played(card, state.turn_counter))
        .filter(|card| !protects_wilds(state, actor, profile) || !card.is_wild())
        .collect();
    let Some(melds) = melds_for_round(round, &cards).or_else(|| {
        if !matches!(profile, BotProfile::Keep(_)) || !protects_wilds(state, actor, profile) {
            return None;
        }
        let with_wilds: Vec<Card> = state.players[actor]
            .hand
            .iter()
            .copied()
            .filter(|card| card_can_be_played(card, state.turn_counter))
            .collect();
        melds_for_round(round, &with_wilds)
    }) else {
        return;
    };
    let action = Action::PlayMeld(melds);
    let mut trial = state.clone();
    if trial.apply(action.clone(), actor) {
        apply(state, actor, action, rng);
    }
}

fn lay_and_hit(state: &mut GameState, actor: usize, rng: &mut impl Rng, profile: BotProfile) {
    let mut guard = state.players[actor].hand.len();
    while guard > 0 {
        guard -= 1;
        if state.round_over {
            return;
        }
        let before = state.players[actor].hand.len();
        if let Some(action) = best_action(state, actor, profile, LegalKind::Play) {
            apply(state, actor, action, rng);
        }
        if state.players[actor].hand.len() >= before {
            break;
        }
    }
    for _ in 0..12 {
        if state.round_over {
            return;
        }
        let Some(action) = best_action(state, actor, profile, LegalKind::Hit) else {
            break;
        };
        apply(state, actor, action, rng);
    }
}

fn choose_opening(state: &GameState, actor: usize, profile: BotProfile) -> Option<Action> {
    let top = *state.deck.discard.last()?;
    if state.players.iter().any(|player| player.is_on_board) {
        return Some(Action::TakeDiscard);
    }
    let stuck = state.turn_counter >= STUCK_ROUND_TURNS;
    let hold_wild = top.is_wild()
        && !stuck
        && match profile {
            BotProfile::Hoarder => state.round_number < HOARDER_LATE_ROUND,
            BotProfile::Keep(limit) => wilds_in_hand(state, actor) < usize::from(limit),
            BotProfile::Random | BotProfile::PointAverse => false,
        };
    if hold_wild || (!stuck && top.get_penalty_value() < HIGH_CARD_POINTS) {
        return Some(Action::TakeDiscard);
    }
    if push_is_legal(&state.deck) {
        Some(Action::PushDiscard)
    } else {
        Some(Action::TakeDiscard)
    }
}

fn can_shed_by_playing(state: &GameState, actor: usize, profile: BotProfile) -> bool {
    for kind in [LegalKind::Play, LegalKind::Hit] {
        let mut found = false;
        let _ = visit_legal_kind_within(state, actor, kind, LARGE_HAND_WALK_NODES, &mut |action| {
            if accepts(state, actor, profile, &action) {
                found = true;
                return false;
            }
            true
        });
        if found {
            return true;
        }
    }
    false
}

fn best_action(
    state: &GameState,
    actor: usize,
    profile: BotProfile,
    kind: LegalKind,
) -> Option<Action> {
    let mut choices = Vec::new();
    let _ = visit_legal_kind_within(state, actor, kind, LARGE_HAND_WALK_NODES, &mut |action| {
        if accepts(state, actor, profile, &action) {
            choices.push(action);
        }
        choices.len() < ACTION_SAMPLE_CAP
    });
    choices
        .into_iter()
        .enumerate()
        .max_by_key(|(index, action)| (shed_points(action), Reverse(*index)))
        .map(|(_, action)| action)
}

fn shed_one(state: &mut GameState, actor: usize, rng: &mut impl Rng, profile: BotProfile) -> bool {
    let mut choices = Vec::new();
    let _ = visit_legal_kind_within(
        state,
        actor,
        LegalKind::Discard,
        LARGE_HAND_WALK_NODES,
        &mut |action| {
            choices.push(action);
            choices.len() < ACTION_SAMPLE_CAP
        },
    );
    let Some(action) = best_discard(choices, state, actor, profile) else {
        return false;
    };
    apply(state, actor, action, rng);
    true
}

fn best_discard(
    choices: Vec<Action>,
    state: &GameState,
    actor: usize,
    profile: BotProfile,
) -> Option<Action> {
    choices
        .into_iter()
        .enumerate()
        .max_by_key(|(index, action)| {
            let score = match action {
                Action::DiscardCard(card) => hold_rank(state, actor, profile, *card),
                _ => 0,
            };
            (score, Reverse(*index))
        })
        .map(|(_, action)| action)
}

fn wilds_in_hand(state: &GameState, actor: usize) -> usize {
    state.players[actor]
        .hand
        .iter()
        .filter(|card| card.is_wild())
        .count()
}

/// A hoarder protects every wild through round 3. A keep seat protects wilds
/// while the hand is at or under its cap. Extra wilds are shed.
fn protects_wilds(state: &GameState, actor: usize, profile: BotProfile) -> bool {
    match profile {
        BotProfile::Hoarder => state.round_number < HOARDER_LATE_ROUND,
        BotProfile::Keep(limit) => wilds_in_hand(state, actor) <= usize::from(limit),
        BotProfile::Random | BotProfile::PointAverse => false,
    }
}

fn hold_rank(state: &GameState, actor: usize, profile: BotProfile, card: Card) -> u32 {
    if protects_wilds(state, actor, profile) && card.is_wild() {
        0
    } else {
        card.get_penalty_value()
    }
}

fn accepts(state: &GameState, actor: usize, profile: BotProfile, action: &Action) -> bool {
    let used = wilds_in_action(action);
    if used == 0 {
        return true;
    }
    match profile {
        BotProfile::Hoarder if state.round_number < HOARDER_LATE_ROUND => false,
        BotProfile::Keep(limit) => {
            wilds_in_hand(state, actor).saturating_sub(used) >= usize::from(limit)
        }
        BotProfile::Random | BotProfile::PointAverse | BotProfile::Hoarder => true,
    }
}

fn wilds_in_action(action: &Action) -> usize {
    match action {
        Action::PlayMeld(melds) => melds.iter().flatten().filter(|card| card.is_wild()).count(),
        Action::HitMeld(hits) => hits
            .iter()
            .flat_map(|hit| &hit.cards)
            .filter(|card| card.is_wild())
            .count(),
        Action::DiscardCard(card) if card.is_wild() => 1,
        Action::StealWild(_) => 1,
        Action::DiscardCard(_)
        | Action::TakeDiscard
        | Action::PushDiscard
        | Action::DrawFromDeck => 0,
    }
}

fn shed_points(action: &Action) -> u32 {
    match action {
        Action::PlayMeld(melds) => melds.iter().flatten().map(Card::get_penalty_value).sum(),
        Action::HitMeld(hits) => hits
            .iter()
            .flat_map(|hit| &hit.cards)
            .map(Card::get_penalty_value)
            .sum(),
        _ => 0,
    }
}

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
    apply(state, actor, action, rng);
    true
}

fn draw_until_a_safe_card(
    state: &mut GameState,
    actor: usize,
    rng: &mut impl Rng,
    profile: BotProfile,
) {
    let hand = state.players[actor].hand.clone();
    let Some(card) = hand
        .into_iter()
        .enumerate()
        .max_by_key(|(index, card)| (hold_rank(state, actor, profile, *card), Reverse(*index)))
        .map(|(_, card)| card)
    else {
        return;
    };
    if state.apply_with_rng(Action::DiscardCard(card), actor, rng) {
        return;
    }
    if state.turn_phase == TurnPhase::PenaltyDrawing && state.penalty_seat == Some(actor) {
        let _ = state.apply_with_rng(Action::DrawFromDeck, actor, rng);
    }
}

fn apply(state: &mut GameState, actor: usize, action: Action, rng: &mut impl Rng) {
    let refused = action.clone();
    assert!(
        state.apply_with_rng(action, actor, rng),
        "a listed action was refused: {refused:?}"
    );
}
