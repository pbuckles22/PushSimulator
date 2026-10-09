//! Action handlers: take, push, meld, hit, steal, discard (Epics 1.3–1.7).

use rand::Rng;

use crate::card::{Card, Rank, Suit};
use crate::deck::{Deck, TurnDraw};
use crate::game_state::{GameState, TurnPhase};
use crate::player::Player;
use crate::resolution::{invalid_resolution, ActionPlan, ActionResolution, RejectionPlan};
use crate::validation::{card_can_be_played, check_round_requirements, validate_run, validate_set};

/// A player choice during a turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Move the discard pile's top card into the player's hand.
    TakeDiscard,
    /// Move the discard top to the next player, who also draws a penalty. The actor then draws.
    PushDiscard,
    /// Lay these melds down. Off the board, they must meet the round minimum.
    /// On the board, each group must be a valid set or run.
    PlayMeld(Vec<Vec<Card>>),
    /// Add cards from the actor's hand onto melds already on the board.
    ///
    /// Each entry names one meld. One action can name several melds, and each of
    /// those melds receives only the cards listed for it.
    HitMeld(Vec<MeldHit>),
    /// Swap one natural card from the hand for a wild on a meld. The wild moves into the hand.
    StealWild(WildSteal),
    /// Place one card from the hand onto the discard pile.
    ///
    /// A player who is off the board cannot discard a card that could be added to
    /// a meld already on the table. The card this turn's take or push just drew
    /// is the exception. A player on the board can discard a card that fits.
    /// Refusing that discard for an off-board player enters penalty drawing.
    DiscardCard(Card),
    /// While the turn is in penalty drawing, draw until a card that fits nothing is discarded.
    /// If nothing is left to draw, the turn ends and the hand keeps what it drew.
    DrawFromDeck,
}

/// Cards from the hand added onto one meld already on the board.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeldHit {
    pub meld_index: usize,
    pub cards: Vec<Card>,
}

/// One natural card swapped for one wild already on a meld.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WildSteal {
    pub meld_index: usize,
    pub wild: Card,
    pub natural: Card,
}

impl Action {
    /// Runs this action for the player at `actor_index`.
    ///
    /// [`Action::TakeDiscard`] moves the discard top onto the end of that player's hand.
    /// Cards under that top stay. The draw pile stays.
    ///
    /// [`Action::PushDiscard`] moves the discard top onto the end of the next player's hand.
    /// That player then draws one card. The actor draws the next card to start the turn.
    /// Cards under the pushed top stay.
    ///
    /// [`Action::PlayMeld`] applies through [`GameState::apply`]. The round on that table
    /// decides which melds count, and the cards have to be in the actor's hand.
    pub fn apply(self, players: &mut [Player], actor_index: usize, deck: &mut Deck) {
        match self {
            Self::TakeDiscard => {
                let card = deck.discard.pop().expect("take discard needs a top card");
                players[actor_index].hand.push(card);
            }
            Self::PushDiscard => push_discard(players, actor_index, deck, &mut rand::thread_rng()),
            Self::PlayMeld(_) => panic!("PlayMeld applies on GameState"),
            Self::HitMeld(_) => panic!("HitMeld applies on GameState"),
            Self::StealWild(_) => panic!("StealWild applies on GameState"),
            Self::DiscardCard(_) => panic!("DiscardCard applies on GameState"),
            Self::DrawFromDeck => panic!("DrawFromDeck applies on GameState"),
        }
    }
}

impl GameState {
    /// Runs `action` for the player at `actor_index`.
    ///
    /// Take and push behave as [`Action::apply`]. A meld play returns false, and leaves
    /// the table as it was, when the round rejects an opening play, a free meld is not a
    /// set or a run, or a card is not in that hand. A successful play appends the melds to
    /// `board`, removes those cards from the hand, and sets `is_on_board`.
    /// A steal returns false, and leaves the table as it was, when that player is off
    /// the board, the natural is still locked, or the natural is not the card the wild
    /// is standing in for. A successful
    /// steal puts the natural where the wild sat and moves the wild onto the end of the
    /// hand, locked until `turn_counter` plus one.
    /// A play or a hit returns false when a card in that action is still locked:
    /// `locked_until_turn` is ahead of `turn_counter`. The table stays as it was.
    ///
    /// A discard returns false, and leaves the table as it was, when the card is
    /// not in that hand, or when that player is off the board and the card could
    /// be added to a meld. The card just taken or drawn on a push may still be
    /// discarded. A successful discard of that card clears `drawn_card_id`.
    /// Once `round_over` is set, every action is refused and the table stays.
    /// A push and a penalty draw shuffle with the thread rng.
    /// [`Self::apply_with_rng`] uses the caller's rng for those draws.
    /// An invalid action returns false and leaves the table as it was.
    pub fn apply(&mut self, action: Action, actor_index: usize) -> bool {
        self.apply_with_rng(action, actor_index, &mut rand::thread_rng())
    }

    /// Runs `action` for the player at `actor_index`, using `rng` when a pile is shuffled.
    ///
    /// [`validate_action`] decides first, without cloning the table. An accepted plan
    /// commits and returns true. A rejected plan commits only its refusal effect and
    /// returns false. An invalid plan does not change the table and returns false.
    /// Take, meld, hit, steal, and discard do not draw. A push draws the penalty card
    /// and the turn card through `rng`. A penalty draw does too. The same seed repeats those draws.
    pub fn apply_with_rng(
        &mut self,
        action: Action,
        actor_index: usize,
        rng: &mut impl Rng,
    ) -> bool {
        match validate_action(self, actor_index, &action) {
            ActionResolution::Invalid(_) => false,
            ActionResolution::Rejected(RejectionPlan::Unchanged) => false,
            ActionResolution::Rejected(RejectionPlan::StartPenaltyDraw { seat }) => {
                self.turn_phase = TurnPhase::PenaltyDrawing;
                self.penalty_seat = Some(seat);
                false
            }
            ActionResolution::Accepted(plan) => {
                commit_plan(self, actor_index, plan, rng);
                true
            }
        }
    }
}

/// Classifies `action` for `actor` without changing `state` and without cloning it.
///
/// A closed round is [`RejectionPlan::Unchanged`]. A move [`GameState::apply`] would
/// panic on is [`ActionResolution::Invalid`]. A fitting off-board discard that opens
/// penalty drawing is [`RejectionPlan::StartPenaltyDraw`]. Anything else `apply` would
/// refuse without changing the table is [`RejectionPlan::Unchanged`].
pub fn validate_action(state: &GameState, actor: usize, action: &Action) -> ActionResolution {
    if state.round_over {
        return unchanged();
    }
    if let Some(resolution) = invalid_resolution(state, actor, action) {
        return resolution;
    }
    match action {
        Action::TakeDiscard => ActionResolution::Accepted(ActionPlan::TakeDiscard),
        Action::PushDiscard => ActionResolution::Accepted(ActionPlan::PushDiscard),
        Action::PlayMeld(melds) => validate_play(state, actor, melds),
        Action::HitMeld(hits) => validate_hit(state, actor, hits),
        Action::StealWild(steal) => validate_steal(state, actor, steal),
        Action::DiscardCard(card) => validate_discard(state, actor, *card),
        Action::DrawFromDeck => validate_draw(state, actor),
    }
}

fn unchanged() -> ActionResolution {
    ActionResolution::Rejected(RejectionPlan::Unchanged)
}

fn validate_play(state: &GameState, actor: usize, melds: &[Vec<Card>]) -> ActionResolution {
    if melds.is_empty() {
        return unchanged();
    }
    if melds
        .iter()
        .flatten()
        .any(|card| !card_can_be_played(card, state.turn_counter))
    {
        return unchanged();
    }
    if state.players[actor].is_on_board {
        if !melds
            .iter()
            .all(|meld| validate_set(meld) || validate_run(meld))
        {
            return unchanged();
        }
    } else if !check_round_requirements(state.round_number, melds) {
        return unchanged();
    }
    if hand_without(&state.players[actor].hand, melds).is_none() {
        return unchanged();
    }
    ActionResolution::Accepted(ActionPlan::PlayMeld(melds.to_vec()))
}

fn validate_hit(state: &GameState, actor: usize, hits: &[MeldHit]) -> ActionResolution {
    if hits.is_empty() || !state.players[actor].is_on_board {
        return unchanged();
    }
    if hits
        .iter()
        .flat_map(|hit| &hit.cards)
        .any(|card| !card_can_be_played(card, state.turn_counter))
    {
        return unchanged();
    }
    let mut extras = vec![Vec::new(); state.board.len()];
    let mut removing = Vec::new();
    for hit in hits {
        if hit.cards.is_empty() {
            return unchanged();
        }
        let Some(extra) = extras.get_mut(hit.meld_index) else {
            return unchanged();
        };
        extra.extend(hit.cards.iter().copied());
        removing.extend(hit.cards.iter().copied());
    }
    for (meld, extra) in state.board.iter().zip(&extras) {
        if extra.is_empty() {
            continue;
        }
        let mut with = Vec::with_capacity(meld.len() + extra.len());
        with.extend(meld.iter().copied());
        with.extend(extra.iter().copied());
        if !validate_set(&with) && !validate_run(&with) {
            return unchanged();
        }
    }
    if hand_without(&state.players[actor].hand, &[removing]).is_none() {
        return unchanged();
    }
    ActionResolution::Accepted(ActionPlan::HitMeld(hits.to_vec()))
}

fn validate_steal(state: &GameState, actor: usize, steal: &WildSteal) -> ActionResolution {
    if !state.players[actor].is_on_board || steal.natural.is_wild() || !steal.wild.is_wild() {
        return unchanged();
    }
    if !card_can_be_played(&steal.natural, state.turn_counter) {
        return unchanged();
    }
    let Some(meld) = state.board.get(steal.meld_index) else {
        return unchanged();
    };
    let Some(wild_at) = meld.iter().position(|card| card == &steal.wild) else {
        return unchanged();
    };
    if meld.iter().any(|card| card.id == steal.natural.id)
        || !natural_replaces_wild(meld, &steal.natural)
    {
        return unchanged();
    }
    if hand_without(&state.players[actor].hand, &[vec![steal.natural]]).is_none() {
        return unchanged();
    }
    let mut replaced = meld.to_vec();
    replaced[wild_at] = steal.natural;
    if !validate_set(&replaced) && !validate_run(&replaced) {
        return unchanged();
    }
    ActionResolution::Accepted(ActionPlan::StealWild(*steal))
}

fn validate_discard(state: &GameState, actor: usize, card: Card) -> ActionResolution {
    if !state.players[actor].hand.contains(&card) {
        return unchanged();
    }
    let quick = state.drawn_card_id == Some(card.id);
    if !state.players[actor].is_on_board
        && !quick
        && card_fits_board(&state.board, &card, state.turn_counter)
    {
        if state.penalty_seat.is_none() || state.penalty_seat == Some(actor) {
            return ActionResolution::Rejected(RejectionPlan::StartPenaltyDraw { seat: actor });
        }
        return unchanged();
    }
    ActionResolution::Accepted(ActionPlan::DiscardCard(card))
}

fn validate_draw(state: &GameState, actor: usize) -> ActionResolution {
    if state.turn_phase != TurnPhase::PenaltyDrawing || state.penalty_seat != Some(actor) {
        return unchanged();
    }
    ActionResolution::Accepted(ActionPlan::DrawFromDeck)
}

fn commit_plan(state: &mut GameState, actor: usize, plan: ActionPlan, rng: &mut impl Rng) {
    let committed = match plan {
        ActionPlan::TakeDiscard => {
            Action::TakeDiscard.apply(&mut state.players, actor, &mut state.deck);
            state.drawn_card_id = state.players[actor].hand.last().map(|card| card.id);
            true
        }
        ActionPlan::PushDiscard => {
            push_discard(&mut state.players, actor, &mut state.deck, rng);
            state.drawn_card_id = state.players[actor].hand.last().map(|card| card.id);
            true
        }
        ActionPlan::PlayMeld(melds) => play_meld(state, actor, &melds),
        ActionPlan::HitMeld(hits) => hit_meld(state, actor, &hits),
        ActionPlan::StealWild(steal) => steal_wild(state, actor, &steal),
        ActionPlan::DiscardCard(card) => discard_card(state, actor, card),
        ActionPlan::DrawFromDeck => draw_from_deck(state, actor, rng),
    };
    assert!(committed, "an accepted plan commits");
}

/// Places one card from the actor's hand onto the discard pile.
///
/// The card has to be in that hand. Off the board, a card that could join a meld
/// stays in the hand, unless it is the card `drawn_card_id` names. That refusal
/// belongs to this seat. Another seat's discard does not close it, and that other
/// seat cannot draw. A locked card cannot be played, so it is safe to discard.
/// An empty hand ends the round.
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

/// Draws until a card that fits nothing can be discarded.
///
/// `rng` shuffles when the draw pile is empty and the discard is recycled or split.
/// Only the seat already in [`TurnPhase::PenaltyDrawing`] may draw. Playable cards
/// stay in the hand. The first safe card goes onto the discard pile and the
/// phase returns to playing. When nothing is left to draw, the turn ends the
/// same way: the phase returns to playing and the hand keeps every card it drew.
fn draw_from_deck(state: &mut GameState, actor_index: usize, rng: &mut impl Rng) -> bool {
    if state.turn_phase != TurnPhase::PenaltyDrawing || state.penalty_seat != Some(actor_index) {
        return false;
    }
    loop {
        let card = match state.deck.draw_with(rng) {
            TurnDraw::One(card) | TurnDraw::LastCard(card) => card,
            TurnDraw::LastTwo { current, next } => {
                let next_index = (actor_index + 1) % state.players.len();
                state.players[next_index].hand.push(next);
                current
            }
            TurnDraw::Empty => {
                state.turn_phase = TurnPhase::Playing;
                state.penalty_seat = None;
                return true;
            }
        };
        if card_fits_board(&state.board, &card, state.turn_counter) {
            state.players[actor_index].hand.push(card);
            continue;
        }
        state.deck.discard.push(card);
        state.turn_phase = TurnPhase::Playing;
        state.penalty_seat = None;
        return true;
    }
}

/// A card fits the board when it can be played and adding it to some meld is a set or a run.
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

/// Adds cards from the actor's hand onto one or more board melds.
///
/// The actor has to already be on the board. Every card has to be in that hand,
/// and a card can be used once. Each meld, after every card aimed at it is added,
/// has to be a set or a run. One card that does not fit refuses the whole action.
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

/// Swaps a natural card for a wild that is standing in for it.
///
/// The actor has to be on the board. The natural has to be in that hand, and it
/// has to be the card the wild is standing in for. A natural that is still locked
/// stays in the hand. The wild moves onto the end of the hand and locks until
/// `turn_counter` plus one. The counter stays put.
/// A refusal leaves the table as it was.
fn steal_wild(state: &mut GameState, actor_index: usize, steal: &WildSteal) -> bool {
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

/// A natural takes a wild's place when that wild is standing in for it.
///
/// In a set, the wild stands in for the natural rank. In a run, it stands in for
/// a missing rank of that suit, including a rank just past either end. An all-wild
/// meld has no single card to replace.
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

/// Moves verified melds from the actor's hand onto the board.
fn play_meld(state: &mut GameState, actor_index: usize, melds: &[Vec<Card>]) -> bool {
    if melds.is_empty() {
        return false;
    }
    if melds
        .iter()
        .flatten()
        .any(|card| !card_can_be_played(card, state.turn_counter))
    {
        return false;
    }
    if state.players[actor_index].is_on_board {
        if !melds
            .iter()
            .all(|meld| validate_set(meld) || validate_run(meld))
        {
            return false;
        }
    } else if !check_round_requirements(state.round_number, melds) {
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

/// Drops each played card out of the hand. The same card twice, or a card that is not held, fails.
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

/// The next seat takes the discard and one draw-pile card. The actor draws after that.
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

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::{RngCore, SeedableRng};

    use crate::actions::{Action, MeldHit, WildSteal};
    use crate::card::{Card, Rank, Suit};
    use crate::deck::{Deck, TurnDraw};
    use crate::player::Player;

    #[test]
    fn test_take_discard() {
        let mut deck = Deck::new();
        let draw_before = deck.cards.clone();
        let kept = Card {
            id: 200,
            suit: Suit::Hearts,
            rank: Rank::Four,
            locked_until_turn: 2,
        };
        let under = Card {
            id: 201,
            suit: Suit::Clubs,
            rank: Rank::Five,
            locked_until_turn: 0,
        };
        let top = Card {
            id: 202,
            suit: Suit::Spades,
            rank: Rank::Ace,
            locked_until_turn: 4,
        };
        let mut players = vec![Player::new(1, 0)];
        players[0].hand = vec![kept];
        deck.discard = vec![under, top];

        Action::TakeDiscard.apply(&mut players, 0, &mut deck);

        assert_eq!(players[0].hand, vec![kept, top]);
        assert_eq!(deck.discard, vec![under]);
        assert_eq!(deck.cards, draw_before);
        assert_eq!(players[0].points, 0);
        assert_eq!(players[0].total_score, 0);
        assert!(!players[0].is_on_board);
        assert_eq!(players[0].id, 1);
        assert_eq!(players[0].seat_index, 0);
    }

    fn card(id: u32, suit: Suit, rank: Rank) -> Card {
        Card {
            id,
            suit,
            rank,
            locked_until_turn: 0,
        }
    }

    /// Player 1 at seat 0, Player 2 at seat 1. The draw pile's last card is the penalty.
    fn push_setup() -> (Vec<Player>, Deck, Card, Card, Card, Card) {
        let kept = card(200, Suit::Hearts, Rank::Four);
        let other = card(201, Suit::Clubs, Rank::Five);
        let under = card(202, Suit::Diamonds, Rank::Six);
        let top = card(203, Suit::Spades, Rank::Ace);
        let actor_draw = card(204, Suit::Hearts, Rank::Jack);
        let penalty = card(205, Suit::Clubs, Rank::Three);
        let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
        players[0].hand = vec![kept];
        players[1].hand = vec![other];
        let mut deck = Deck::new();
        deck.cards = vec![actor_draw, penalty];
        deck.discard = vec![under, top];
        (players, deck, under, top, penalty, actor_draw)
    }

    #[test]
    fn test_push_discard_moves_card() {
        let (mut players, mut deck, under, top, _, _) = push_setup();

        Action::PushDiscard.apply(&mut players, 0, &mut deck);

        assert_eq!(players[1].hand[0].id, 201);
        assert_eq!(players[1].hand[1], top);
        assert_eq!(deck.discard, vec![under]);
        assert!(players[0].hand.iter().all(|card| card.id != top.id));
        assert_eq!(players[0].id, 1);
        assert_eq!(players[1].id, 2);
        assert_eq!(players[0].points, 0);
        assert_eq!(players[1].points, 0);
        assert_eq!(players[0].total_score, 0);
        assert_eq!(players[1].total_score, 0);
    }

    #[test]
    fn test_push_discard_penalty_draw() {
        let (mut players, mut deck, _, top, penalty, _) = push_setup();

        Action::PushDiscard.apply(&mut players, 0, &mut deck);

        assert_eq!(
            players[1].hand,
            vec![card(201, Suit::Clubs, Rank::Five), top, penalty]
        );
        assert_eq!(penalty.get_penalty_value(), 5);
        assert_eq!(
            players[1].calculate_hand_penalty(),
            5 + top.get_penalty_value() + penalty.get_penalty_value()
        );
        assert_eq!(players[1].points, 0);
        assert_eq!(players[1].total_score, 0);
    }

    #[test]
    fn test_push_turn_advancement() {
        let (mut players, mut deck, _, _, penalty, actor_draw) = push_setup();
        let draw_before = deck.cards.len();

        Action::PushDiscard.apply(&mut players, 0, &mut deck);

        assert_eq!(
            players[0].hand,
            vec![card(200, Suit::Hearts, Rank::Four), actor_draw]
        );
        assert_eq!(actor_draw.get_penalty_value(), 10);
        assert_eq!(deck.cards.len(), draw_before - 2);
        assert!(deck
            .cards
            .iter()
            .all(|card| card.id != penalty.id && card.id != actor_draw.id));
        assert_eq!(players[0].points, 0);
        assert_eq!(players[0].total_score, 0);
        assert!(!players[0].is_on_board);
        assert!(!players[1].is_on_board);
    }

    use crate::game_state::{GameState, TurnPhase};

    fn locked(id: u32, suit: Suit, rank: Rank, locked_until_turn: u32) -> Card {
        Card {
            id,
            suit,
            rank,
            locked_until_turn,
        }
    }

    fn table(hand: Vec<Card>, other: Vec<Card>, draw: Vec<Card>) -> GameState {
        let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
        players[0].points = 4;
        players[1].points = 4;
        players[0].total_score = 9;
        players[1].total_score = 9;
        players[0].hand = hand;
        players[1].hand = other;
        let mut deck = Deck::new();
        deck.cards = draw;
        deck.discard = vec![card(900, Suit::Diamonds, Rank::Queen)];
        let state = GameState::new(players, deck);
        assert_eq!(state.round_number, 1);
        assert!(state.board.is_empty());
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.drawn_card_id, None);
        state
    }

    fn assert_still(before: &GameState, after: &GameState) {
        assert_eq!(after.players, before.players);
        assert_eq!(after.board, before.board);
        assert_eq!(after.round_number, before.round_number);
        assert_eq!(after.deck, before.deck);
        assert_eq!(after.turn_counter, before.turn_counter);
        assert_eq!(after.drawn_card_id, before.drawn_card_id);
        assert_eq!(after.round_over, before.round_over);
    }

    fn refuse(state: &GameState, actor: usize, melds: Vec<Vec<Card>>) {
        let mut next = state.clone();
        assert!(!next.apply(Action::PlayMeld(melds), actor));
        assert_still(state, &next);
    }

    /// Two sets of three, a second four of hearts that stays, and a lookalike four in the draw pile.
    /// The four that is played is locked until 3. The counter has caught up, so that lock is not ahead.
    fn round_1_lay() -> (GameState, Vec<Vec<Card>>, Vec<Card>) {
        let keep_four = locked(1, Suit::Hearts, Rank::Four, 2);
        let play_four = locked(2, Suit::Hearts, Rank::Four, 3);
        let four_spades = card(3, Suit::Spades, Rank::Four);
        let four_clubs = card(4, Suit::Clubs, Rank::Four);
        let keep_king = card(5, Suit::Diamonds, Rank::King);
        let five_hearts = card(6, Suit::Hearts, Rank::Five);
        let five_spades = card(7, Suit::Spades, Rank::Five);
        let five_clubs = card(8, Suit::Clubs, Rank::Five);
        let keep_nine = card(9, Suit::Clubs, Rank::Nine);
        let lookalike = card(50, Suit::Hearts, Rank::Four);
        let marker = card(51, Suit::Clubs, Rank::Jack);
        let other_four = card(60, Suit::Diamonds, Rank::Four);
        let other_ace = card(61, Suit::Spades, Rank::Ace);
        let fours = vec![play_four, four_spades, four_clubs];
        let fives = vec![five_hearts, five_spades, five_clubs];
        let mut state = table(
            vec![
                keep_four,
                play_four,
                four_spades,
                four_clubs,
                keep_king,
                five_hearts,
                five_spades,
                five_clubs,
                keep_nine,
            ],
            vec![other_four, other_ace],
            vec![lookalike, marker],
        );
        state.turn_counter = 3;
        (
            state,
            vec![fours, fives],
            vec![keep_four, keep_king, keep_nine],
        )
    }

    #[test]
    fn test_play_meld_success() {
        let (mut state, melds, hand_after) = round_1_lay();
        let before_penalty = state.players[0].calculate_hand_penalty();
        let played_penalty: u32 = melds
            .iter()
            .flatten()
            .map(|card| card.get_penalty_value())
            .sum();
        let deck = state.deck.clone();
        let other = state.players[1].clone();

        assert!(state.apply(Action::PlayMeld(melds.clone()), 0));

        assert_eq!(state.players[0].hand, hand_after);
        assert_eq!(state.board, melds);
        assert_eq!(
            state.players[0].calculate_hand_penalty(),
            before_penalty - played_penalty
        );
        assert_eq!(hand_after[0].locked_until_turn, 2);
        assert_eq!(state.board[0][0].locked_until_turn, 3);
        assert_eq!(state.players[0].id, 1);
        assert_eq!(state.players[0].seat_index, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1], other);
        assert_eq!(state.deck, deck);
        assert_eq!(state.round_number, 1);
        assert!(state.deck.cards.iter().any(|card| card.id == 50));
        assert!(state.board.iter().flatten().all(|card| card.id != 50));
        assert!(state.board.iter().flatten().all(|card| card.id != 1));
    }

    #[test]
    fn test_play_meld_flags_player() {
        let (mut state, melds, _) = round_1_lay();

        assert!(state.apply(Action::PlayMeld(melds.clone()), 0));

        assert!(state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].total_score, 9);
        refuse(&state, 0, melds);
    }

    /// On the board in round 5, a single set of 3 may be laid. The round minimum is ignored.
    #[test]
    fn test_play_meld_after_on_board_ignores_round_requirements() {
        let fours = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
        ];
        let king = card(4, Suit::Diamonds, Rank::King);
        let mut state = table(
            vec![fours[0], fours[1], fours[2], king],
            vec![card(5, Suit::Hearts, Rank::Three)],
            vec![card(6, Suit::Clubs, Rank::Jack)],
        );
        state.round_number = 5;
        state.players[0].is_on_board = true;
        state.board = vec![vec![
            card(10, Suit::Hearts, Rank::Eight),
            card(11, Suit::Spades, Rank::Eight),
            card(12, Suit::Clubs, Rank::Eight),
        ]];
        let board_before = state.board.clone();

        assert!(state.apply(Action::PlayMeld(vec![fours.clone()]), 0));

        assert_eq!(state.players[0].hand, vec![king]);
        assert_eq!(state.board.len(), board_before.len() + 1);
        assert_eq!(state.board.last(), Some(&fours));
        assert!(state.players[0].is_on_board);
        assert!(!state.round_over);
    }

    /// On the board, two cards that are not a set or a run leave the table unchanged.
    #[test]
    fn test_play_meld_after_on_board_rejects_invalid_melds() {
        let king = card(1, Suit::Diamonds, Rank::King);
        let three = card(2, Suit::Hearts, Rank::Three);
        let mut state = table(
            vec![king, three],
            vec![card(3, Suit::Clubs, Rank::Ace)],
            vec![card(4, Suit::Spades, Rank::Jack)],
        );
        state.players[0].is_on_board = true;
        state.board = vec![vec![
            card(10, Suit::Hearts, Rank::Eight),
            card(11, Suit::Spades, Rank::Eight),
            card(12, Suit::Clubs, Rank::Eight),
        ]];
        let before = state.clone();

        assert!(!state.apply(Action::PlayMeld(vec![vec![king, three]]), 0));
        assert_still(&before, &state);
    }

    #[test]
    fn test_play_meld_refuses_and_leaves_the_table() {
        let keep_four = locked(1, Suit::Hearts, Rank::Four, 2);
        let play_four = locked(2, Suit::Hearts, Rank::Four, 3);
        let four_spades = card(3, Suit::Spades, Rank::Four);
        let four_clubs = card(4, Suit::Clubs, Rank::Four);
        let five_hearts = card(6, Suit::Hearts, Rank::Five);
        let five_spades = card(7, Suit::Spades, Rank::Five);
        let five_clubs = card(8, Suit::Clubs, Rank::Five);
        let six_hearts = card(16, Suit::Hearts, Rank::Six);
        let six_spades = card(17, Suit::Spades, Rank::Six);
        let six_clubs = card(18, Suit::Clubs, Rank::Six);
        let run = vec![
            card(19, Suit::Hearts, Rank::Seven),
            card(20, Suit::Hearts, Rank::Eight),
            card(21, Suit::Hearts, Rank::Nine),
            card(22, Suit::Hearts, Rank::Ten),
        ];
        let pair = vec![
            card(23, Suit::Hearts, Rank::King),
            card(24, Suit::Spades, Rank::King),
        ];
        let wilds = vec![
            card(28, Suit::None, Rank::Joker),
            card(29, Suit::Clubs, Rank::Two),
            card(30, Suit::None, Rank::Joker),
            card(31, Suit::Spades, Rank::Two),
        ];
        let six_fours = vec![
            card(40, Suit::Hearts, Rank::Four),
            card(41, Suit::Spades, Rank::Four),
            card(42, Suit::Clubs, Rank::Four),
            card(43, Suit::Diamonds, Rank::Four),
            card(44, Suit::Hearts, Rank::Four),
            card(45, Suit::Spades, Rank::Four),
        ];
        let lookalike = card(50, Suit::Hearts, Rank::Four);
        let draw_only = card(51, Suit::Clubs, Rank::Jack);
        let discard_only = card(52, Suit::Diamonds, Rank::Three);
        let other_four = card(60, Suit::Diamonds, Rank::Four);
        let fours = vec![play_four, four_spades, four_clubs];
        let fives = vec![five_hearts, five_spades, five_clubs];
        let sixes = vec![six_hearts, six_spades, six_clubs];
        let mut hand = vec![keep_four, play_four, four_spades, four_clubs];
        hand.extend(fives.iter().copied());
        hand.extend(sixes.iter().copied());
        hand.extend(run.iter().copied());
        hand.extend(pair.iter().copied());
        hand.extend(wilds.iter().copied());
        hand.extend(six_fours.iter().copied());
        let mut state = table(hand, vec![other_four], vec![lookalike, draw_only]);
        state.deck.discard.push(discard_only);

        refuse(&state, 0, vec![fours.clone()]);
        refuse(&state, 0, vec![fours.clone(), fives.clone(), sixes.clone()]);
        refuse(&state, 0, vec![fours.clone(), pair.clone()]);
        refuse(&state, 0, vec![fours.clone(), vec![]]);
        refuse(
            &state,
            0,
            vec![vec![play_four, play_four, four_spades], fives.clone()],
        );
        refuse(
            &state,
            0,
            vec![fours.clone(), vec![play_four, five_hearts, five_spades]],
        );
        refuse(
            &state,
            0,
            vec![vec![lookalike, four_spades, four_clubs], fives.clone()],
        );
        refuse(
            &state,
            0,
            vec![vec![draw_only, four_spades, four_clubs], fives.clone()],
        );
        refuse(
            &state,
            0,
            vec![vec![discard_only, four_spades, four_clubs], fives.clone()],
        );
        refuse(
            &state,
            0,
            vec![vec![other_four, four_spades, four_clubs], fives.clone()],
        );
        let forged = locked(2, Suit::Hearts, Rank::Four, 0);
        refuse(
            &state,
            0,
            vec![vec![forged, four_spades, four_clubs], fives.clone()],
        );
        refuse(&state, 0, vec![fours.clone(), run.clone()]);
        refuse(&state, 0, vec![run.clone(), wilds.clone()]);
        refuse(&state, 0, vec![wilds.clone()]);
        refuse(&state, 0, vec![six_fours.clone()]);
        refuse(&state, 0, vec![]);

        state.round_number = 0;
        refuse(&state, 0, vec![fours.clone(), fives.clone()]);
        state.round_number = 6;
        refuse(&state, 0, vec![fours.clone(), fives.clone()]);
        state.round_number = 1;
        state.turn_counter = 3;
        state.players[0].is_on_board = true;
        assert!(state.apply(Action::PlayMeld(vec![fours.clone(), fives.clone()]), 0));
        assert!(state.players[0].is_on_board);
        assert_eq!(state.board, vec![fours, fives]);
    }

    fn run_of(id: u32, suit: Suit, ranks: &[Rank]) -> Vec<Card> {
        ranks
            .iter()
            .enumerate()
            .map(|(offset, rank)| card(id + offset as u32, suit, *rank))
            .collect()
    }

    #[test]
    fn test_play_meld_rounds_2_through_5() {
        let fours = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
        ];
        let fives = vec![
            card(4, Suit::Hearts, Rank::Five),
            card(5, Suit::Spades, Rank::Five),
            card(6, Suit::Clubs, Rank::Five),
        ];
        let sixes = vec![
            card(7, Suit::Hearts, Rank::Six),
            card(8, Suit::Spades, Rank::Six),
            card(9, Suit::Clubs, Rank::Six),
        ];
        let hearts = run_of(
            10,
            Suit::Hearts,
            &[Rank::Four, Rank::Five, Rank::Six, Rank::Seven],
        );
        let mut unsorted = hearts.clone();
        unsorted.reverse();
        let spades = run_of(
            20,
            Suit::Spades,
            &[Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack],
        );
        let ace_high = run_of(
            30,
            Suit::Diamonds,
            &[Rank::Jack, Rank::Queen, Rank::King, Rank::Ace],
        );
        let ace_low = vec![
            card(40, Suit::Clubs, Rank::Ace),
            card(41, Suit::Hearts, Rank::Two),
            card(42, Suit::Clubs, Rank::Three),
            card(43, Suit::Clubs, Rank::Four),
        ];
        let long = run_of(
            50,
            Suit::Hearts,
            &[
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
            ],
        );
        let wilds = vec![
            card(70, Suit::None, Rank::Joker),
            card(71, Suit::Clubs, Rank::Two),
            card(72, Suit::None, Rank::Joker),
            card(73, Suit::Spades, Rank::Two),
        ];
        let seven_wilds: Vec<Card> = (0..7)
            .map(|offset| card(80 + offset, Suit::None, Rank::Joker))
            .collect();
        let seven_wilds_b: Vec<Card> = (0..7)
            .map(|offset| card(90 + offset, Suit::Clubs, Rank::Two))
            .collect();

        let mut round_2 = table(
            fours
                .iter()
                .chain(hearts.iter())
                .chain(ace_high.iter())
                .chain(ace_low.iter())
                .chain(wilds.iter())
                .copied()
                .collect(),
            vec![],
            vec![],
        );
        round_2.round_number = 2;
        refuse(&round_2, 0, vec![fours.clone(), fives.clone()]);
        assert!(round_2.apply(Action::PlayMeld(vec![unsorted.clone(), fours.clone()]), 0));
        assert_eq!(round_2.board, vec![unsorted, fours.clone()]);
        assert_eq!(
            round_2.players[0].hand,
            ace_high
                .iter()
                .chain(ace_low.iter())
                .chain(wilds.iter())
                .copied()
                .collect::<Vec<_>>()
        );
        assert!(round_2.players[0].is_on_board);

        let mut wild_round = table(
            wilds.iter().chain(ace_low.iter()).copied().collect(),
            vec![],
            vec![],
        );
        wild_round.round_number = 2;
        assert!(wild_round.apply(Action::PlayMeld(vec![wilds.clone(), ace_low.clone()]), 0));
        assert_eq!(wild_round.board[0], wilds);
        assert_eq!(wild_round.board[1], ace_low);

        let eight = run_of(
            100,
            Suit::Clubs,
            &[
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
            ],
        );
        let mut round_3 = table(
            hearts
                .iter()
                .chain(spades.iter())
                .chain(eight.iter())
                .chain(fours.iter())
                .copied()
                .collect(),
            vec![],
            vec![],
        );
        round_3.round_number = 3;
        refuse(&round_3, 0, vec![eight.clone()]);
        refuse(&round_3, 0, vec![hearts.clone(), fours.clone()]);
        assert!(round_3.apply(Action::PlayMeld(vec![spades.clone(), hearts.clone()]), 0));
        assert_eq!(round_3.board, vec![spades, hearts.clone()]);
        assert_eq!(
            round_3.players[0].hand,
            eight
                .iter()
                .chain(fours.iter())
                .copied()
                .collect::<Vec<_>>()
        );

        let mut round_4 = table(
            fours
                .iter()
                .chain(fives.iter())
                .chain(sixes.iter())
                .chain(hearts.iter())
                .copied()
                .collect(),
            vec![],
            vec![],
        );
        round_4.round_number = 4;
        refuse(
            &round_4,
            0,
            vec![fours.clone(), fives.clone(), hearts.clone()],
        );
        refuse(&round_4, 0, vec![fours.clone(), fives.clone()]);
        assert!(round_4.apply(
            Action::PlayMeld(vec![sixes.clone(), fours.clone(), fives.clone()]),
            0
        ));
        assert_eq!(round_4.board, vec![sixes, fours.clone(), fives.clone()]);

        let mut round_5 = table(
            fours
                .iter()
                .chain(long.iter())
                .chain(hearts.iter())
                .copied()
                .collect(),
            vec![],
            vec![],
        );
        round_5.round_number = 5;
        refuse(&round_5, 0, vec![fours.clone(), hearts.clone()]);
        refuse(&round_5, 0, vec![long.clone()]);
        assert!(round_5.apply(Action::PlayMeld(vec![long.clone(), fours.clone()]), 0));
        assert_eq!(round_5.board, vec![long.clone(), fours.clone()]);
        assert_eq!(round_5.players[0].hand, hearts);

        let mut both = table(
            seven_wilds
                .iter()
                .chain(seven_wilds_b.iter())
                .chain(fours.iter())
                .copied()
                .collect(),
            vec![],
            vec![],
        );
        both.round_number = 5;
        refuse(&both, 0, vec![seven_wilds.clone()]);
        refuse(
            &both,
            0,
            vec![seven_wilds.clone(), seven_wilds_b.clone(), fours.clone()],
        );
        assert!(both.apply(
            Action::PlayMeld(vec![seven_wilds.clone(), seven_wilds_b.clone()]),
            0
        ));
        assert_eq!(both.board, vec![seven_wilds, seven_wilds_b]);
        assert_eq!(both.players[0].hand, fours);
        assert_eq!(both.players[0].points, 4);
        assert_eq!(both.players[0].total_score, 9);
    }

    #[test]
    fn test_play_meld_six_fours_need_two_melds() {
        let six = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
            card(4, Suit::Diamonds, Rank::Four),
            card(5, Suit::Hearts, Rank::Four),
            card(6, Suit::Spades, Rank::Four),
        ];
        let left = six[..3].to_vec();
        let right = six[3..].to_vec();
        let mut state = table(six.clone(), vec![card(7, Suit::Clubs, Rank::Ace)], vec![]);

        refuse(&state, 0, vec![six]);
        assert!(state.apply(Action::PlayMeld(vec![right.clone(), left.clone()]), 0));
        assert_eq!(state.board, vec![right, left]);
        assert!(state.players[0].hand.is_empty());
        assert!(state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[1].hand, vec![card(7, Suit::Clubs, Rank::Ace)]);
    }

    #[test]
    fn test_play_meld_second_player_appends() {
        let (mut state, first, _) = round_1_lay();
        let sixes = vec![
            card(30, Suit::Hearts, Rank::Six),
            card(31, Suit::Spades, Rank::Six),
            card(32, Suit::Clubs, Rank::Six),
        ];
        let sevens = vec![
            card(33, Suit::Hearts, Rank::Seven),
            card(34, Suit::Spades, Rank::Seven),
            card(35, Suit::Clubs, Rank::Seven),
        ];
        let keeper = card(36, Suit::Diamonds, Rank::Ace);
        state.players[1].hand = vec![
            keeper, sixes[0], sixes[1], sixes[2], sevens[0], sevens[1], sevens[2],
        ];

        refuse(&state, 1, first.clone());
        assert!(state.apply(Action::PlayMeld(first.clone()), 0));
        let after_first = state.players[0].clone();
        let deck = state.deck.clone();
        refuse(&state, 1, vec![sixes.clone()]);
        assert!(state.apply(Action::PlayMeld(vec![sevens.clone(), sixes.clone()]), 1));

        assert_eq!(state.players[0], after_first);
        assert!(state.players[0].is_on_board);
        assert!(state.players[1].is_on_board);
        assert_eq!(state.players[1].hand, vec![keeper]);
        assert_eq!(
            state.board,
            vec![first[0].clone(), first[1].clone(), sevens, sixes]
        );
        assert_eq!(state.deck, deck);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        refuse(&state, 0, first);
    }

    #[test]
    fn test_play_meld_can_empty_the_hand() {
        let fours = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
        ];
        let fives = vec![
            card(4, Suit::Hearts, Rank::Five),
            card(5, Suit::Spades, Rank::Five),
            card(6, Suit::Clubs, Rank::Five),
        ];
        let mut state = table(
            fours.iter().chain(fives.iter()).copied().collect(),
            vec![card(7, Suit::Clubs, Rank::Ace)],
            vec![card(8, Suit::Diamonds, Rank::King)],
        );

        assert!(state.apply(Action::PlayMeld(vec![fours.clone(), fives.clone()]), 0));

        assert!(state.players[0].hand.is_empty());
        assert!(state.players[0].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.board, vec![fours, fives]);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.deck.cards, vec![card(8, Suit::Diamonds, Rank::King)]);
    }

    #[test]
    fn test_play_meld_wild_sets_leave_the_hand() {
        let wild_set = vec![
            locked(1, Suit::Hearts, Rank::Four, 1),
            locked(2, Suit::None, Rank::Joker, 4),
            card(3, Suit::Spades, Rank::Two),
        ];
        let jokers = vec![
            card(4, Suit::None, Rank::Joker),
            card(5, Suit::None, Rank::Joker),
            card(6, Suit::None, Rank::Joker),
        ];
        let keeper = card(7, Suit::Diamonds, Rank::Nine);
        let lookalike = card(8, Suit::None, Rank::Joker);
        let other = card(9, Suit::Clubs, Rank::Ace);
        let mut state = table(
            vec![
                keeper,
                wild_set[0],
                wild_set[1],
                wild_set[2],
                jokers[0],
                jokers[1],
                jokers[2],
            ],
            vec![other],
            vec![lookalike],
        );
        let before_penalty = state.players[0].calculate_hand_penalty();
        let played_penalty: u32 = wild_set
            .iter()
            .chain(jokers.iter())
            .map(|card| card.get_penalty_value())
            .sum();
        let deck = state.deck.clone();
        state.turn_counter = 4;

        refuse(&state, 0, vec![wild_set.clone()]);
        assert!(state.apply(Action::PlayMeld(vec![wild_set.clone(), jokers.clone()]), 0));

        assert_eq!(state.players[0].hand, vec![keeper]);
        assert_eq!(state.board, vec![wild_set.clone(), jokers.clone()]);
        assert_eq!(state.board[0][1].locked_until_turn, 4);
        assert_eq!(
            state.players[0].calculate_hand_penalty(),
            before_penalty - played_penalty
        );
        assert!(state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[1].hand, vec![other]);
        assert_eq!(state.deck, deck);
        assert!(state.deck.cards.iter().any(|card| card.id == lookalike.id));
        assert!(state
            .board
            .iter()
            .flatten()
            .all(|card| card.id != lookalike.id));
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.round_number, 1);
        refuse(&state, 0, vec![wild_set, jokers]);
    }

    #[test]
    fn test_play_meld_oversized_sets_move_every_card() {
        let nines = vec![
            card(1, Suit::Hearts, Rank::Nine),
            card(2, Suit::Spades, Rank::Nine),
            card(3, Suit::Clubs, Rank::Nine),
            card(4, Suit::Diamonds, Rank::Nine),
            card(5, Suit::Hearts, Rank::Nine),
        ];
        let fives = vec![
            card(6, Suit::Hearts, Rank::Five),
            card(7, Suit::Spades, Rank::Five),
            card(8, Suit::Clubs, Rank::Five),
            card(9, Suit::Diamonds, Rank::Five),
        ];
        let keeper = card(10, Suit::Diamonds, Rank::King);
        let mut hand = vec![nines[0], keeper];
        hand.extend(nines[1..].iter().copied());
        hand.extend(fives.iter().copied());
        let mut state = table(hand, vec![card(11, Suit::Clubs, Rank::Ace)], vec![]);
        let deck = state.deck.clone();

        refuse(&state, 0, vec![nines.clone()]);
        assert!(state.apply(Action::PlayMeld(vec![nines.clone(), fives.clone()]), 0));

        assert_eq!(state.players[0].hand, vec![keeper]);
        assert_eq!(state.board, vec![nines, fives]);
        assert_eq!(state.board.iter().flatten().count(), 9);
        assert!(state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.deck, deck);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(keeper.get_penalty_value(), 10);
        assert_eq!(state.players[0].calculate_hand_penalty(), 10);
    }

    #[test]
    fn test_play_meld_second_seat_opens_the_board() {
        let (mut state, melds, hand_after) = round_1_lay();
        let opener = state.players[0].hand.clone();
        let waiting = card(70, Suit::Clubs, Rank::Ace);
        state.players[0].hand = vec![waiting];
        state.players[1].hand = opener;
        let seat_0 = state.players[0].clone();
        let deck = state.deck.clone();

        refuse(&state, 1, vec![melds[0].clone()]);
        assert!(state.apply(Action::PlayMeld(melds.clone()), 1));

        assert_eq!(state.players[0], seat_0);
        assert!(!state.players[0].is_on_board);
        assert!(state.players[1].is_on_board);
        assert_eq!(state.players[1].hand, hand_after);
        assert_eq!(state.board, melds);
        assert_eq!(state.deck, deck);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    #[test]
    fn test_play_meld_third_seat_only_the_actor_changes() {
        let fours = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
        ];
        let fives = vec![
            card(4, Suit::Hearts, Rank::Five),
            card(5, Suit::Spades, Rank::Five),
            card(6, Suit::Clubs, Rank::Five),
        ];
        let keeper = card(7, Suit::Diamonds, Rank::Nine);
        let mut players = vec![Player::new(1, 0), Player::new(2, 1), Player::new(3, 2)];
        for player in &mut players {
            player.points = 4;
            player.total_score = 9;
        }
        players[0].hand = vec![card(10, Suit::Hearts, Rank::King)];
        players[1].hand = vec![card(11, Suit::Spades, Rank::Ace)];
        players[2].hand = fours
            .iter()
            .chain(fives.iter())
            .copied()
            .chain(std::iter::once(keeper))
            .collect();
        let mut deck = Deck::new();
        deck.cards = vec![card(12, Suit::Clubs, Rank::Jack)];
        deck.discard = vec![card(13, Suit::Diamonds, Rank::Queen)];
        let mut state = GameState::new(players, deck);
        let seat_0 = state.players[0].clone();
        let seat_1 = state.players[1].clone();
        let piles = state.deck.clone();

        refuse(&state, 2, vec![fours.clone()]);
        assert!(state.apply(Action::PlayMeld(vec![fours.clone(), fives.clone()]), 2));

        assert_eq!(state.players[0], seat_0);
        assert_eq!(state.players[1], seat_1);
        assert!(!state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert!(state.players[2].is_on_board);
        assert_eq!(state.players[2].hand, vec![keeper]);
        assert_eq!(state.players[2].id, 3);
        assert_eq!(state.players[2].seat_index, 2);
        assert_eq!(state.board, vec![fours, fives]);
        assert_eq!(state.deck, piles);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[2].points, 4);
        assert_eq!(state.players[2].total_score, 9);
    }

    #[test]
    fn test_play_meld_four_wilds_fill_the_run_beside_a_set() {
        let fours = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
        ];
        let wilds = vec![
            card(4, Suit::None, Rank::Joker),
            card(5, Suit::Clubs, Rank::Two),
            card(6, Suit::None, Rank::Joker),
            card(7, Suit::Spades, Rank::Two),
        ];
        let keeper = card(8, Suit::Diamonds, Rank::King);
        let mut state = table(
            fours
                .iter()
                .chain(wilds.iter())
                .copied()
                .chain(std::iter::once(keeper))
                .collect(),
            vec![card(9, Suit::Clubs, Rank::Ace)],
            vec![card(10, Suit::Hearts, Rank::Jack)],
        );
        state.round_number = 2;
        let other = state.players[1].clone();
        let deck = state.deck.clone();

        refuse(&state, 0, vec![wilds.clone()]);
        refuse(&state, 0, vec![fours.clone()]);
        refuse(&state, 0, vec![fours.clone(), fours.clone()]);
        assert!(state.apply(Action::PlayMeld(vec![wilds.clone(), fours.clone()]), 0));

        assert_eq!(state.board, vec![wilds, fours]);
        assert_eq!(state.players[0].hand, vec![keeper]);
        assert!(state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[1], other);
        assert_eq!(state.deck, deck);
        assert_eq!(state.round_number, 2);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(keeper.get_penalty_value(), 10);
        assert_eq!(state.players[0].calculate_hand_penalty(), 10);
    }

    fn board_with(hand: Vec<Card>, meld: Vec<Card>, on_board: bool) -> GameState {
        let mut state = table(
            hand,
            vec![card(60, Suit::Diamonds, Rank::Four)],
            vec![card(51, Suit::Clubs, Rank::Jack)],
        );
        state.board = vec![meld];
        state.players[0].is_on_board = on_board;
        state
    }

    #[test]
    fn test_hit_rejection_if_not_on_board() {
        let eight_hearts = card(4, Suit::Hearts, Rank::Eight);
        let keeper = card(5, Suit::Diamonds, Rank::King);
        let meld = vec![
            card(1, Suit::Spades, Rank::Eight),
            card(2, Suit::Clubs, Rank::Eight),
            card(3, Suit::None, Rank::Joker),
        ];
        let state = board_with(vec![eight_hearts, keeper], meld, false);

        let mut next = state.clone();
        assert!(!next.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![eight_hearts]
            }]),
            0,
        ));
        assert_still(&state, &next);
        assert!(!next.players[0].is_on_board);
        assert_eq!(next.players[0].points, 4);
        assert_eq!(next.players[0].total_score, 9);
    }

    #[test]
    fn test_valid_hit_set() {
        let eight_spades = card(1, Suit::Spades, Rank::Eight);
        let eight_clubs = card(2, Suit::Clubs, Rank::Eight);
        let joker = card(3, Suit::None, Rank::Joker);
        let eight_hearts = card(4, Suit::Hearts, Rank::Eight);
        let nine = card(6, Suit::Hearts, Rank::Nine);
        let keeper = card(5, Suit::Diamonds, Rank::King);
        let lookalike = card(50, Suit::Hearts, Rank::Eight);
        let meld = vec![eight_spades, eight_clubs, joker];
        let mut state = board_with(vec![eight_hearts, nine, keeper], meld.clone(), true);
        state.deck.cards.push(lookalike);
        let before_penalty = state.players[0].calculate_hand_penalty();

        let mut missed = state.clone();
        assert!(!missed.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![nine]
            }]),
            0,
        ));
        assert_still(&state, &missed);
        let mut absent = state.clone();
        assert!(!absent.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![lookalike]
            }]),
            0,
        ));
        assert_still(&state, &absent);
        let mut empty = state.clone();
        assert!(!empty.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![]
            }]),
            0,
        ));
        assert_still(&state, &empty);
        let mut missing = state.clone();
        assert!(!missing.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 1,
                cards: vec![eight_hearts]
            }]),
            0,
        ));
        assert_still(&state, &missing);

        let deck = state.deck.clone();
        let other = state.players[1].clone();
        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![eight_hearts]
            }]),
            0,
        ));

        assert_eq!(
            state.board,
            vec![vec![eight_spades, eight_clubs, joker, eight_hearts]]
        );
        assert_eq!(state.players[0].hand, vec![nine, keeper]);
        assert!(state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(state.players[1], other);
        assert_eq!(state.deck, deck);
        assert_eq!(state.round_number, 1);
        assert_eq!(
            state.players[0].calculate_hand_penalty(),
            before_penalty - eight_hearts.get_penalty_value()
        );
        assert!(state.deck.cards.iter().any(|card| card.id == lookalike.id));
        assert!(state
            .board
            .iter()
            .flatten()
            .all(|card| card.id != lookalike.id));
    }

    #[test]
    fn test_valid_hit_run() {
        let five = card(1, Suit::Hearts, Rank::Five);
        let six = card(2, Suit::Hearts, Rank::Six);
        let seven = card(3, Suit::Hearts, Rank::Seven);
        let eight = card(4, Suit::Hearts, Rank::Eight);
        let eight_spades = card(8, Suit::Spades, Rank::Eight);
        let keeper = card(5, Suit::Diamonds, Rank::King);
        let meld = vec![five, six, seven];
        let state = board_with(vec![eight, eight_spades, keeper], meld, true);

        let mut wrong_suit = state.clone();
        assert!(!wrong_suit.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![eight_spades]
            }]),
            0,
        ));
        assert_still(&state, &wrong_suit);

        let mut next = state.clone();
        let deck = next.deck.clone();
        let other = next.players[1].clone();
        assert!(next.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![eight]
            }]),
            0,
        ));

        assert_eq!(next.board, vec![vec![five, six, seven, eight]]);
        assert_eq!(next.players[0].hand, vec![eight_spades, keeper]);
        assert!(next.players[0].is_on_board);
        assert_eq!(next.players[1], other);
        assert_eq!(next.deck, deck);
        assert_eq!(next.round_number, 1);
        assert_eq!(next.players[0].points, 4);
        assert_eq!(next.players[0].total_score, 9);
        assert_eq!(eight.get_penalty_value(), 5);
    }

    /// A 3 joins a set of threes and a 4 of spades joins a spade run in one action.
    /// The 6 stays in the hand. This action does not discard it.
    #[test]
    fn test_hit_several_melds_in_one_action() {
        let three_hearts = card(1, Suit::Hearts, Rank::Three);
        let three_diamonds = card(2, Suit::Diamonds, Rank::Three);
        let three_clubs = card(3, Suit::Clubs, Rank::Three);
        let five = card(11, Suit::Spades, Rank::Five);
        let six_spades = card(12, Suit::Spades, Rank::Six);
        let seven = card(13, Suit::Spades, Rank::Seven);
        let eight = card(14, Suit::Spades, Rank::Eight);
        let three_spades = card(21, Suit::Spades, Rank::Three);
        let four_spades = card(22, Suit::Spades, Rank::Four);
        let six_hearts = card(23, Suit::Hearts, Rank::Six);
        let mut state = table(
            vec![three_spades, four_spades, six_hearts],
            vec![card(60, Suit::Diamonds, Rank::King)],
            vec![card(51, Suit::Clubs, Rank::Jack)],
        );
        state.board = vec![
            vec![three_hearts, three_diamonds, three_clubs],
            vec![five, six_spades, seven, eight],
        ];
        state.players[0].is_on_board = true;

        let mut gap = state.clone();
        gap.board[1] = vec![
            card(31, Suit::Spades, Rank::Six),
            card(32, Suit::Spades, Rank::Seven),
            card(33, Suit::Spades, Rank::Eight),
            card(34, Suit::Spades, Rank::Nine),
        ];
        let gap_before = gap.clone();
        assert!(!gap.apply(
            Action::HitMeld(vec![
                MeldHit {
                    meld_index: 0,
                    cards: vec![three_spades],
                },
                MeldHit {
                    meld_index: 1,
                    cards: vec![four_spades],
                },
            ]),
            0,
        ));
        assert_still(&gap_before, &gap);

        let deck = state.deck.clone();
        let other = state.players[1].clone();
        assert!(state.apply(
            Action::HitMeld(vec![
                MeldHit {
                    meld_index: 0,
                    cards: vec![three_spades],
                },
                MeldHit {
                    meld_index: 1,
                    cards: vec![four_spades],
                },
            ]),
            0,
        ));

        assert_eq!(
            state.board[0],
            vec![three_hearts, three_diamonds, three_clubs, three_spades]
        );
        assert_eq!(
            state.board[1],
            vec![five, six_spades, seven, eight, four_spades]
        );
        assert_eq!(state.players[0].hand, vec![six_hearts]);
        assert!(state.players[0].is_on_board);
        assert_eq!(state.players[1], other);
        assert_eq!(state.deck, deck);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(six_hearts.get_penalty_value(), 5);
    }

    /// Both ends of one run can be added together. A 3 alone does not fill the gap below a 5.
    #[test]
    fn test_hit_both_ends_of_one_run() {
        let five = card(11, Suit::Spades, Rank::Five);
        let six = card(12, Suit::Spades, Rank::Six);
        let seven = card(13, Suit::Spades, Rank::Seven);
        let eight = card(14, Suit::Spades, Rank::Eight);
        let three = card(21, Suit::Spades, Rank::Three);
        let four = card(22, Suit::Spades, Rank::Four);
        let nine = card(23, Suit::Spades, Rank::Nine);
        let state = board_with(vec![three, four, nine], vec![five, six, seven, eight], true);

        let mut only_three = state.clone();
        assert!(!only_three.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![three],
            }]),
            0,
        ));
        assert_still(&state, &only_three);

        let mut next = state.clone();
        assert!(next.apply(
            Action::HitMeld(vec![
                MeldHit {
                    meld_index: 0,
                    cards: vec![four, nine],
                },
                MeldHit {
                    meld_index: 0,
                    cards: vec![three],
                },
            ]),
            0,
        ));
        assert_eq!(
            next.board[0],
            vec![five, six, seven, eight, four, nine, three]
        );
        assert!(next.players[0].hand.is_empty());
        assert!(next.players[0].is_on_board);
        assert_eq!(next.players[0].points, 4);
        assert_eq!(next.players[0].total_score, 9);
    }

    /// A 5 of diamonds replaces a joker in a set of fives. The joker moves into the hand.
    #[test]
    fn test_wild_steal_success() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_diamonds = card(4, Suit::Diamonds, Rank::Five);
        let keeper = card(5, Suit::Diamonds, Rank::King);
        let meld = vec![five_hearts, five_spades, joker];
        let state = board_with(vec![five_diamonds, keeper], meld, true);

        let mut next = state.clone();
        let deck = next.deck.clone();
        let other = next.players[1].clone();
        assert!(next.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five_diamonds,
            }),
            0,
        ));

        assert_eq!(
            next.board,
            vec![vec![five_hearts, five_spades, five_diamonds]]
        );
        assert_eq!(next.players[0].hand.len(), 2);
        assert_eq!(next.players[0].hand[0], keeper);
        assert_eq!(next.players[0].hand[1].id, joker.id);
        assert_eq!(next.players[0].hand[1].suit, Suit::None);
        assert_eq!(next.players[0].hand[1].rank, Rank::Joker);
        assert!(next.board[0].iter().all(|card| card.id != joker.id));
        assert!(next.players[0].is_on_board);
        assert!(!next.players[1].is_on_board);
        assert_eq!(next.players[1], other);
        assert_eq!(next.deck, deck);
        assert_eq!(next.round_number, 1);
        assert_eq!(next.players[0].points, 4);
        assert_eq!(next.players[0].total_score, 9);
        assert_eq!(next.players[1].points, 4);
        assert_eq!(next.players[1].total_score, 9);
        assert_eq!(joker.get_penalty_value(), 20);
        assert_eq!(
            next.players[0].calculate_hand_penalty(),
            keeper.get_penalty_value() + 20
        );
        assert_eq!(next.turn_counter, 0);
    }

    /// The stolen joker locks until the turn counter plus one. The counter itself stays put.
    #[test]
    fn test_wild_steal_applies_lock() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let joker = locked(3, Suit::None, Rank::Joker, 3);
        let five_diamonds = card(4, Suit::Diamonds, Rank::Five);
        let keeper = locked(5, Suit::Diamonds, Rank::King, 2);
        let meld = vec![five_hearts, five_spades, joker];
        let mut state = board_with(vec![five_diamonds, keeper], meld, true);
        state.turn_counter = 6;

        let mut next = state.clone();
        assert!(next.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five_diamonds,
            }),
            0,
        ));

        assert_eq!(next.players[0].hand[0], keeper);
        assert_eq!(next.players[0].hand[1].id, joker.id);
        assert_eq!(next.players[0].hand[1].locked_until_turn, 7);
        assert_eq!(next.board[0][2], five_diamonds);
        assert_eq!(next.board[0][2].locked_until_turn, 0);
        assert_eq!(next.turn_counter, 6);
        assert_eq!(next.players[0].points, 4);
        assert_eq!(next.players[0].total_score, 9);
        assert_eq!(next.round_number, 1);
    }

    fn refuse_steal(state: &GameState, actor: usize, meld_index: usize, wild: Card, natural: Card) {
        let mut next = state.clone();
        assert!(!next.apply(
            Action::StealWild(WildSteal {
                meld_index,
                wild,
                natural,
            }),
            actor,
        ));
        assert_still(state, &next);
    }

    /// A player who is off the board cannot steal. A nine is not a five. A card that is
    /// not wild cannot be stolen, and a wild cannot be the card that replaces one.
    /// An all-wild meld has no natural card to put back. The other meld stays put.
    #[test]
    fn test_wild_steal_refuses_and_leaves_the_table() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_diamonds = card(4, Suit::Diamonds, Rank::Five);
        let nine = card(6, Suit::Diamonds, Rank::Nine);
        let two = card(7, Suit::Clubs, Rank::Two);
        let lookalike = card(50, Suit::Diamonds, Rank::Five);
        let other_joker = card(8, Suit::None, Rank::Joker);
        let threes = vec![
            card(11, Suit::Hearts, Rank::Three),
            card(12, Suit::Spades, Rank::Three),
            card(13, Suit::Clubs, Rank::Three),
        ];
        let mut state = table(
            vec![five_diamonds, nine, two],
            vec![card(60, Suit::Hearts, Rank::King)],
            vec![lookalike],
        );
        state.board = vec![vec![five_hearts, five_spades, joker], threes.clone()];
        state.players[0].is_on_board = false;
        state.turn_counter = 4;
        refuse_steal(&state, 0, 0, joker, five_diamonds);

        state.players[0].is_on_board = true;
        refuse_steal(&state, 0, 0, joker, nine);
        refuse_steal(&state, 0, 0, five_hearts, five_diamonds);
        refuse_steal(&state, 0, 0, joker, two);
        refuse_steal(&state, 0, 0, other_joker, five_diamonds);
        refuse_steal(&state, 0, 0, joker, lookalike);
        refuse_steal(&state, 0, 2, joker, five_diamonds);
        refuse_steal(&state, 1, 0, joker, five_diamonds);

        let wilds = vec![
            card(21, Suit::None, Rank::Joker),
            card(22, Suit::None, Rank::Joker),
            card(23, Suit::Hearts, Rank::Two),
        ];
        let mut only_wilds = state.clone();
        only_wilds.board = vec![wilds.clone()];
        refuse_steal(&only_wilds, 0, 0, wilds[0], five_diamonds);
        let four_wilds = vec![
            card(31, Suit::None, Rank::Joker),
            card(32, Suit::None, Rank::Joker),
            card(33, Suit::Clubs, Rank::Two),
            card(34, Suit::Spades, Rank::Two),
        ];
        only_wilds.board = vec![four_wilds.clone()];
        refuse_steal(&only_wilds, 0, 0, four_wilds[0], five_diamonds);

        let wrap = vec![
            card(41, Suit::Hearts, Rank::King),
            card(42, Suit::Hearts, Rank::Ace),
            card(43, Suit::Clubs, Rank::Two),
            card(44, Suit::Hearts, Rank::Three),
        ];
        only_wilds.board = vec![wrap.clone()];
        refuse_steal(
            &only_wilds,
            0,
            0,
            wrap[2],
            card(45, Suit::Hearts, Rank::Three),
        );

        assert_eq!(state.board[1], threes);
        assert_eq!(state.turn_counter, 4);
        assert!(state.deck.cards.iter().any(|card| card.id == lookalike.id));
    }

    /// The joker in 4♦, joker, 6♦, 7♦ is the 5♦. An 8♦ still makes a run, and it is not that card.
    /// A joker on the end of 4-5-6 is the 3 or the 7. Jack, queen, king, and a joker is the 10 or the ace.
    /// A joker between ace and 3 is the deuce, and no natural card is that deuce.
    #[test]
    fn test_wild_steal_run_takes_the_card_the_wild_stands_for() {
        let four = card(1, Suit::Diamonds, Rank::Four);
        let joker = card(2, Suit::None, Rank::Joker);
        let six = card(3, Suit::Diamonds, Rank::Six);
        let seven = card(4, Suit::Diamonds, Rank::Seven);
        let five_diamonds = card(5, Suit::Diamonds, Rank::Five);
        let five_hearts = card(6, Suit::Hearts, Rank::Five);
        let eight = card(7, Suit::Diamonds, Rank::Eight);
        let mut state = board_with(
            vec![five_diamonds, five_hearts, eight],
            vec![four, joker, six, seven],
            true,
        );
        state.turn_counter = 2;
        refuse_steal(&state, 0, 0, joker, five_hearts);
        refuse_steal(&state, 0, 0, joker, eight);

        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five_diamonds,
            }),
            0,
        ));
        assert_eq!(state.board[0], vec![four, five_diamonds, six, seven]);
        assert_eq!(state.players[0].hand[0], five_hearts);
        assert_eq!(state.players[0].hand[1], eight);
        assert_eq!(state.players[0].hand[2].id, joker.id);
        assert_eq!(state.players[0].hand[2].locked_until_turn, 3);
        assert!(state.players[0].is_on_board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 2);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);

        let heart = |id, rank| card(id, Suit::Hearts, rank);
        let end_joker = card(20, Suit::None, Rank::Joker);
        let end = board_with(
            vec![
                heart(21, Rank::Three),
                heart(22, Rank::Seven),
                heart(23, Rank::Eight),
                card(24, Suit::Spades, Rank::Two),
            ],
            vec![
                heart(11, Rank::Four),
                heart(12, Rank::Five),
                heart(13, Rank::Six),
                end_joker,
            ],
            true,
        );
        refuse_steal(&end, 0, 0, end_joker, heart(23, Rank::Eight));
        refuse_steal(&end, 0, 0, end_joker, card(24, Suit::Spades, Rank::Two));
        let mut low = end.clone();
        assert!(low.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: end_joker,
                natural: heart(21, Rank::Three),
            }),
            0,
        ));
        assert_eq!(low.board[0][3], heart(21, Rank::Three));
        assert_eq!(low.players[0].hand.last().unwrap().locked_until_turn, 1);
        let mut high = end.clone();
        assert!(high.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: end_joker,
                natural: heart(22, Rank::Seven),
            }),
            0,
        ));
        assert_eq!(high.board[0][3], heart(22, Rank::Seven));

        let face_joker = card(30, Suit::None, Rank::Joker);
        let faces = board_with(
            vec![
                heart(31, Rank::Ten),
                heart(32, Rank::Ace),
                heart(33, Rank::Nine),
            ],
            vec![
                heart(34, Rank::Jack),
                heart(35, Rank::Queen),
                heart(36, Rank::King),
                face_joker,
            ],
            true,
        );
        refuse_steal(&faces, 0, 0, face_joker, heart(33, Rank::Nine));
        let mut ten = faces.clone();
        assert!(ten.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: face_joker,
                natural: heart(31, Rank::Ten),
            }),
            0,
        ));
        assert_eq!(ten.board[0][3], heart(31, Rank::Ten));
        let mut ace = faces.clone();
        assert!(ace.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: face_joker,
                natural: heart(32, Rank::Ace),
            }),
            0,
        ));
        assert_eq!(ace.board[0][3], heart(32, Rank::Ace));

        let deuce_joker = card(40, Suit::None, Rank::Joker);
        let deuce_place = board_with(
            vec![heart(41, Rank::Five), card(42, Suit::Hearts, Rank::Two)],
            vec![
                heart(43, Rank::Ace),
                deuce_joker,
                heart(44, Rank::Three),
                heart(45, Rank::Four),
            ],
            true,
        );
        refuse_steal(&deuce_place, 0, 0, deuce_joker, heart(41, Rank::Five));
        refuse_steal(
            &deuce_place,
            0,
            0,
            deuce_joker,
            card(42, Suit::Hearts, Rank::Two),
        );
    }

    /// A two in a set of fours moves into the hand. One of two jokers in 4-7 can be the 5.
    /// The other joker stays, still locked as it was. The other seat can take that wild.
    #[test]
    fn test_wild_steal_two_and_one_of_two_wilds() {
        let four_hearts = card(1, Suit::Hearts, Rank::Four);
        let four_spades = card(2, Suit::Spades, Rank::Four);
        let two = locked(3, Suit::Clubs, Rank::Two, 9);
        let four_diamonds = card(4, Suit::Diamonds, Rank::Four);
        let mut state = board_with(
            vec![four_diamonds],
            vec![four_hearts, four_spades, two],
            true,
        );
        state.turn_counter = 1;
        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: two,
                natural: four_diamonds,
            }),
            0,
        ));
        assert_eq!(
            state.board[0],
            vec![four_hearts, four_spades, four_diamonds]
        );
        assert_eq!(state.players[0].hand.len(), 1);
        assert_eq!(state.players[0].hand[0].id, two.id);
        assert_eq!(state.players[0].hand[0].rank, Rank::Two);
        assert_eq!(state.players[0].hand[0].locked_until_turn, 2);
        assert!(state.players[0].is_on_board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);

        let four = card(11, Suit::Diamonds, Rank::Four);
        let joker_a = locked(12, Suit::None, Rank::Joker, 1);
        let joker_b = locked(13, Suit::None, Rank::Joker, 4);
        let seven = card(14, Suit::Diamonds, Rank::Seven);
        let five = card(15, Suit::Diamonds, Rank::Five);
        let eight = card(16, Suit::Diamonds, Rank::Eight);
        let mut holes = table(
            vec![card(70, Suit::Hearts, Rank::King)],
            vec![five, eight],
            vec![card(61, Suit::Clubs, Rank::Jack)],
        );
        holes.board = vec![
            vec![
                card(17, Suit::Hearts, Rank::Nine),
                card(18, Suit::Spades, Rank::Nine),
                card(19, Suit::Clubs, Rank::Nine),
            ],
            vec![four, joker_a, joker_b, seven],
        ];
        holes.players[1].is_on_board = true;
        holes.turn_counter = 9;
        refuse_steal(&holes, 1, 1, joker_a, eight);
        assert!(holes.apply(
            Action::StealWild(WildSteal {
                meld_index: 1,
                wild: joker_a,
                natural: five,
            }),
            1,
        ));
        assert_eq!(holes.board[0].len(), 3);
        assert_eq!(holes.board[1], vec![four, five, joker_b, seven]);
        assert_eq!(joker_b.locked_until_turn, 4);
        assert_eq!(holes.board[1][2].locked_until_turn, 4);
        assert_eq!(
            holes.players[1].hand,
            vec![eight, {
                let mut stolen = joker_a;
                stolen.locked_until_turn = 10;
                stolen
            }]
        );
        assert_eq!(
            holes.players[0].hand,
            vec![card(70, Suit::Hearts, Rank::King)]
        );
        assert!(!holes.players[0].is_on_board);
        assert!(holes.players[1].is_on_board);
        assert_eq!(holes.turn_counter, 9);
        assert_eq!(holes.round_number, 1);
        assert_eq!(holes.players[1].points, 4);
        assert_eq!(holes.players[1].total_score, 9);
    }

    /// A stolen wild cannot be played while its lock is still ahead of the turn counter.
    /// Hitting that joker back onto the set is refused. Laying a locked wild down to get
    /// on the board is refused. The hand, the board, the piles, and both scores stay put.
    #[test]
    fn test_stolen_wild_play_rejection() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_diamonds = card(4, Suit::Diamonds, Rank::Five);
        let extra_five = card(5, Suit::Clubs, Rank::Five);
        let mut state = board_with(
            vec![five_diamonds, extra_five],
            vec![five_hearts, five_spades, joker],
            true,
        );
        state.turn_counter = 4;
        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five_diamonds,
            }),
            0,
        ));
        let stolen = state.players[0].hand[1];
        assert_eq!(stolen.id, joker.id);
        assert_eq!(stolen.rank, Rank::Joker);
        assert_eq!(stolen.locked_until_turn, 5);
        assert_eq!(state.turn_counter, 4);

        let before_hit = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_still(&before_hit, &state);
        assert_eq!(state.players[0].hand[1].locked_until_turn, 5);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.round_number, 1);
        assert!(state.players[0].is_on_board);

        let locked_joker = locked(10, Suit::None, Rank::Joker, 1);
        let fours = vec![
            card(11, Suit::Hearts, Rank::Four),
            card(12, Suit::Spades, Rank::Four),
            locked_joker,
        ];
        let sixes = vec![
            card(13, Suit::Hearts, Rank::Six),
            card(14, Suit::Spades, Rank::Six),
            card(15, Suit::Clubs, Rank::Six),
        ];
        let laying = table(
            fours.iter().chain(sixes.iter()).copied().collect(),
            vec![card(16, Suit::Clubs, Rank::Ace)],
            vec![card(17, Suit::Diamonds, Rank::King)],
        );
        refuse(&laying, 0, vec![fours, sixes]);
        assert!(!laying.players[0].is_on_board);
        assert!(laying.board.is_empty());
        assert_eq!(laying.turn_counter, 0);
        assert_eq!(laying.players[0].points, 4);
        assert_eq!(laying.players[0].total_score, 9);
    }

    /// After the turn counter advances, the stolen wild can be played. The lock on the
    /// card stays. The round does not end. One advance is not enough when the lock is
    /// still two turns ahead.
    #[test]
    fn test_stolen_wild_play_acceptance() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_diamonds = card(4, Suit::Diamonds, Rank::Five);
        let mut state = board_with(
            vec![five_diamonds],
            vec![five_hearts, five_spades, joker],
            true,
        );
        state.turn_counter = 4;
        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five_diamonds,
            }),
            0,
        ));
        let stolen = state.players[0].hand[0];
        assert_eq!(stolen.locked_until_turn, 5);
        let before_advance = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_still(&before_advance, &state);

        state.advance_turn();
        assert_eq!(state.turn_counter, 5);
        assert_eq!(state.players, before_advance.players);
        assert_eq!(state.board, before_advance.board);
        assert_eq!(state.deck, before_advance.deck);
        assert_eq!(state.round_number, before_advance.round_number);
        assert_eq!(state.players[0].hand[0].locked_until_turn, 5);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_eq!(
            state.board[0],
            vec![five_hearts, five_spades, five_diamonds, stolen]
        );
        assert_eq!(state.board[0][3].locked_until_turn, 5);
        assert!(state.players[0].hand.is_empty());
        assert!(state.players[0].is_on_board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 5);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert!(!state.players[1].is_on_board);

        let locked_joker = locked(10, Suit::None, Rank::Joker, 1);
        let fours = vec![
            card(11, Suit::Hearts, Rank::Four),
            card(12, Suit::Spades, Rank::Four),
            locked_joker,
        ];
        let sixes = vec![
            card(13, Suit::Hearts, Rank::Six),
            card(14, Suit::Spades, Rank::Six),
            card(15, Suit::Clubs, Rank::Six),
        ];
        let mut laying = table(
            fours.iter().chain(sixes.iter()).copied().collect(),
            vec![card(16, Suit::Clubs, Rank::Ace)],
            vec![card(17, Suit::Diamonds, Rank::King)],
        );
        refuse(&laying, 0, vec![fours.clone(), sixes.clone()]);
        laying.advance_turn();
        assert_eq!(laying.turn_counter, 1);
        assert!(laying.apply(Action::PlayMeld(vec![fours.clone(), sixes.clone()]), 0));
        assert_eq!(laying.board, vec![fours, sixes]);
        assert_eq!(laying.board[0][2].locked_until_turn, 1);
        assert!(laying.players[0].hand.is_empty());
        assert!(laying.players[0].is_on_board);
        assert!(!laying.players[1].is_on_board);
        assert_eq!(laying.round_number, 1);
        assert_eq!(laying.players[0].points, 4);
        assert_eq!(laying.players[0].total_score, 9);

        let early = locked(20, Suit::None, Rank::Joker, 6);
        let eights = vec![
            card(21, Suit::Hearts, Rank::Eight),
            card(22, Suit::Spades, Rank::Eight),
            card(23, Suit::Clubs, Rank::Eight),
        ];
        let nines = vec![
            card(24, Suit::Hearts, Rank::Nine),
            card(25, Suit::Spades, Rank::Nine),
            early,
        ];
        let mut still = table(
            eights.iter().chain(nines.iter()).copied().collect(),
            vec![card(26, Suit::Clubs, Rank::Ace)],
            vec![card(27, Suit::Diamonds, Rank::King)],
        );
        still.turn_counter = 4;
        refuse(&still, 0, vec![eights.clone(), nines.clone()]);
        still.advance_turn();
        assert_eq!(still.turn_counter, 5);
        refuse(&still, 0, vec![eights.clone(), nines.clone()]);
        still.advance_turn();
        assert_eq!(still.turn_counter, 6);
        assert!(still.apply(Action::PlayMeld(vec![eights.clone(), nines.clone()]), 0));
        assert_eq!(still.board[1][2].locked_until_turn, 6);
        assert!(still.players[0].is_on_board);
        assert_eq!(still.round_number, 1);
    }

    /// Steal a joker, wait until the lock catches up, then go out with that joker.
    /// Hitting it while the lock is ahead leaves the round open. Playing it while a
    /// king is still in the hand does too. The king can leave first. The joker, once
    /// it is the last card and the counter has caught up, ends the round.
    #[test]
    fn test_steal_hold_win() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_diamonds = card(4, Suit::Diamonds, Rank::Five);
        let king = card(5, Suit::Spades, Rank::King);
        let mut state = board_with(
            vec![five_diamonds, king],
            vec![five_hearts, five_spades, joker],
            true,
        );
        let other = state.players[1].clone();
        let draw = state.deck.cards.clone();
        let queen = state.deck.discard[0];
        assert!(!state.round_over);

        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five_diamonds,
            }),
            0,
        ));
        let stolen = state.players[0].hand[1];
        assert_eq!(stolen.id, joker.id);
        assert_eq!(stolen.locked_until_turn, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].hand, vec![king, stolen]);
        assert_eq!(
            state.board[0],
            vec![five_hearts, five_spades, five_diamonds]
        );
        assert!(!state.round_over);

        let before_locked = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_still(&before_locked, &state);

        let mut king_stays = state.clone();
        king_stays.advance_turn();
        assert_eq!(king_stays.turn_counter, 1);
        assert!(king_stays.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_eq!(king_stays.players[0].hand, vec![king]);
        assert!(!king_stays.round_over);
        assert_eq!(king_stays.round_number, 1);
        assert_eq!(king_stays.players[0].points, 4);
        assert_eq!(king_stays.players[0].total_score, 9);

        assert!(state.apply(Action::DiscardCard(king), 0));
        assert_eq!(state.players[0].hand, vec![stolen]);
        assert_eq!(state.players[0].hand[0].locked_until_turn, 1);
        assert!(!state.round_over);
        assert_eq!(state.deck.discard, vec![queen, king]);

        let before_last = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_still(&before_last, &state);

        state.advance_turn();
        assert_eq!(state.turn_counter, 1);
        assert_eq!(state.players[0].hand, vec![stolen]);
        assert!(!state.round_over);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1], other);

        let mut discard_win = state.clone();
        assert!(discard_win.apply(Action::DiscardCard(stolen), 0));
        assert!(discard_win.players[0].hand.is_empty());
        assert!(discard_win.round_over);
        assert_eq!(discard_win.deck.discard, vec![queen, king, stolen]);
        assert_eq!(discard_win.board[0], before_last.board[0]);
        assert_eq!(discard_win.round_number, 1);
        assert_eq!(discard_win.turn_counter, 1);
        assert_eq!(discard_win.players[0].points, 4);
        assert_eq!(discard_win.players[0].total_score, 9);
        assert_eq!(discard_win.players[1], other);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert!(state.players[0].hand.is_empty());
        assert!(state.round_over);
        assert_eq!(
            state.board[0],
            vec![five_hearts, five_spades, five_diamonds, stolen]
        );
        assert_eq!(state.board[0][3].locked_until_turn, 1);
        assert_eq!(state.deck.cards, draw);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert_eq!(state.players[1], other);
        assert!(state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 1);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(joker.get_penalty_value(), 20);

        let board = state.board.clone();
        let hands = state.players.clone();
        state.advance_turn();
        assert!(state.round_over);
        assert!(state.players[0].hand.is_empty());
        assert_eq!(state.board, board);
        assert_eq!(state.players, hands);
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_eq!(state.board, board);
        assert_eq!(state.players, hands);
        assert!(state.round_over);
    }

    /// The joker in 4♦, joker, 6♦, 7♦ is the 5♦. After the wait, that joker is the last
    /// card and extends the run. The round ends. Hitting it before the wait does not.
    #[test]
    fn test_steal_hold_win_from_a_run() {
        let four = card(1, Suit::Diamonds, Rank::Four);
        let joker = card(2, Suit::None, Rank::Joker);
        let six = card(3, Suit::Diamonds, Rank::Six);
        let seven = card(4, Suit::Diamonds, Rank::Seven);
        let five = card(5, Suit::Diamonds, Rank::Five);
        let mut state = board_with(vec![five], vec![four, joker, six, seven], true);
        let other = state.players[1].clone();
        assert!(!state.round_over);

        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five,
            }),
            0,
        ));
        let stolen = state.players[0].hand[0];
        assert_eq!(stolen.id, joker.id);
        assert_eq!(stolen.locked_until_turn, 1);
        assert_eq!(state.board[0], vec![four, five, six, seven]);
        assert!(!state.round_over);

        let before = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_still(&before, &state);

        state.advance_turn();
        assert!(!state.round_over);
        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert!(state.players[0].hand.is_empty());
        assert!(state.round_over);
        assert_eq!(state.board[0].len(), 5);
        assert_eq!(state.board[0][4].id, joker.id);
        assert_eq!(state.board[0][4].locked_until_turn, 1);
        assert_eq!(state.players[1], other);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[0].calculate_hand_penalty(), 0);
    }

    /// The other seat goes out while the stolen joker is still locked. That joker stays
    /// in the hand, so the 20 is still there to count. The total does not change yet.
    #[test]
    fn test_steal_hold_other_seat_wins() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_diamonds = card(4, Suit::Diamonds, Rank::Five);
        let eight_hearts = card(11, Suit::Hearts, Rank::Eight);
        let eight_spades = card(12, Suit::Spades, Rank::Eight);
        let eight_clubs = card(13, Suit::Clubs, Rank::Eight);
        let eight_diamonds = card(14, Suit::Diamonds, Rank::Eight);
        let mut state = table(
            vec![five_diamonds],
            vec![eight_diamonds],
            vec![card(51, Suit::Clubs, Rank::Jack)],
        );
        state.board = vec![
            vec![five_hearts, five_spades, joker],
            vec![eight_hearts, eight_spades, eight_clubs],
        ];
        state.players[0].is_on_board = true;
        state.players[1].is_on_board = true;
        let draw = state.deck.cards.clone();
        assert!(!state.round_over);

        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five_diamonds,
            }),
            0,
        ));
        let stolen = state.players[0].hand[0];
        assert_eq!(stolen.locked_until_turn, 1);
        assert_eq!(state.players[0].calculate_hand_penalty(), 20);
        assert!(!state.round_over);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 1,
                cards: vec![eight_diamonds],
            }]),
            1,
        ));
        assert!(state.players[1].hand.is_empty());
        assert!(state.round_over);
        assert_eq!(state.players[0].hand, vec![stolen]);
        assert_eq!(state.players[0].hand[0].locked_until_turn, 1);
        assert_eq!(state.players[0].calculate_hand_penalty(), 20);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.deck.cards, draw);
        assert!(state.players[0].is_on_board);
        assert!(state.players[1].is_on_board);

        state.advance_turn();
        assert_eq!(state.turn_counter, 1);
        assert_eq!(state.players[0].hand[0].locked_until_turn, 1);
        let after = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_eq!(state.players, after.players);
        assert_eq!(state.board, after.board);
        assert_eq!(state.deck, after.deck);
        assert!(state.round_over);
        assert_eq!(state.players[0].calculate_hand_penalty(), 20);
    }

    /// A locked wild sitting in the hand does not stop an unlocked card. Putting both
    /// in one hit refuses the whole action, and the unlocked card stays in the hand.
    /// A locked natural is refused the same way. The other meld stays put.
    #[test]
    fn test_locked_wild_does_not_block_an_unlocked_card() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let five_clubs = card(3, Suit::Clubs, Rank::Five);
        let five_diamonds = locked(4, Suit::Diamonds, Rank::Five, 5);
        let joker = locked(5, Suit::None, Rank::Joker, 5);
        let eight_diamonds = card(6, Suit::Diamonds, Rank::Eight);
        let meld = vec![five_hearts, five_spades, five_clubs];
        let other = vec![
            card(7, Suit::Hearts, Rank::Eight),
            card(8, Suit::Spades, Rank::Eight),
            card(9, Suit::Clubs, Rank::Eight),
        ];
        let mut state = table(
            vec![five_diamonds, joker, eight_diamonds],
            vec![card(60, Suit::Diamonds, Rank::Four)],
            vec![card(51, Suit::Clubs, Rank::Jack)],
        );
        state.board = vec![meld.clone(), other.clone()];
        state.players[0].is_on_board = true;
        state.turn_counter = 4;

        let before_both = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![five_diamonds, joker],
            }]),
            0,
        ));
        assert_still(&before_both, &state);

        let before_natural = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![five_diamonds],
            }]),
            0,
        ));
        assert_still(&before_natural, &state);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 1,
                cards: vec![eight_diamonds],
            }]),
            0,
        ));
        assert_eq!(state.board[0], meld);
        assert_eq!(
            state.board[1],
            vec![other[0], other[1], other[2], eight_diamonds]
        );
        assert_eq!(state.players[0].hand, vec![five_diamonds, joker]);
        assert_eq!(state.players[0].hand[0].locked_until_turn, 5);
        assert_eq!(state.players[0].hand[1].locked_until_turn, 5);
        assert_eq!(state.turn_counter, 4);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert!(state.players[0].is_on_board);

        state.advance_turn();
        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![five_diamonds, joker],
            }]),
            0,
        ));
        assert_eq!(
            state.board[0],
            vec![five_hearts, five_spades, five_clubs, five_diamonds, joker]
        );
        assert_eq!(state.board[0][3].locked_until_turn, 5);
        assert_eq!(state.board[0][4].locked_until_turn, 5);
        assert!(state.players[0].hand.is_empty());
        assert_eq!(state.board[1].len(), 4);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 5);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    /// A locked natural cannot take a wild's place while that lock is ahead of the counter.
    /// After the counter catches up, the swap works and the stolen wild gets a new lock.
    #[test]
    fn test_locked_natural_cannot_steal_until_the_counter_catches_up() {
        let five_hearts = card(1, Suit::Hearts, Rank::Five);
        let five_spades = card(2, Suit::Spades, Rank::Five);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_diamonds = locked(4, Suit::Diamonds, Rank::Five, 3);
        let mut state = board_with(
            vec![five_diamonds],
            vec![five_hearts, five_spades, joker],
            true,
        );
        state.turn_counter = 2;
        refuse_steal(&state, 0, 0, joker, five_diamonds);
        state.advance_turn();
        assert_eq!(state.turn_counter, 3);
        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five_diamonds,
            }),
            0,
        ));
        assert_eq!(state.board[0][2].id, five_diamonds.id);
        assert_eq!(state.board[0][2].locked_until_turn, 3);
        assert_eq!(state.players[0].hand[0].id, joker.id);
        assert_eq!(state.players[0].hand[0].locked_until_turn, 4);
        assert_eq!(state.turn_counter, 3);
        assert_eq!(state.round_number, 1);
        assert!(state.players[0].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    fn refuse_discard(state: &GameState, actor: usize, card: Card) {
        let mut next = state.clone();
        assert!(!next.apply(Action::DiscardCard(card), actor));
        assert_still(state, &next);
    }

    /// 5♥ 6♥ and a joker. The joker can stand in for 8♥, so 7♥ fits between them.
    fn heart_gap(hand: Vec<Card>, on_board: bool) -> GameState {
        board_with(
            hand,
            vec![
                card(1, Suit::Hearts, Rank::Five),
                card(2, Suit::Hearts, Rank::Six),
                card(3, Suit::None, Rank::Joker),
            ],
            on_board,
        )
    }

    #[test]
    fn test_pre_board_discard_rejection() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let state = heart_gap(vec![seven, king], false);
        let under = state.deck.discard.clone();

        refuse_discard(&state, 0, seven);

        assert_eq!(state.players[0].hand, vec![seven, king]);
        assert_eq!(state.deck.discard, under);
        assert!(!state.players[0].is_on_board);
        assert_eq!(state.board.len(), 1);
        assert_eq!(state.board[0].len(), 3);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(seven.get_penalty_value(), 5);
    }

    #[test]
    fn test_pre_board_set_discard_rejection() {
        let eight = card(4, Suit::Hearts, Rank::Eight);
        let king = card(5, Suit::Spades, Rank::King);
        let meld = vec![
            card(1, Suit::Spades, Rank::Eight),
            card(2, Suit::Clubs, Rank::Eight),
            card(3, Suit::Diamonds, Rank::Eight),
        ];
        let state = board_with(vec![eight, king], meld, false);

        refuse_discard(&state, 0, eight);
        assert_eq!(state.players[0].hand, vec![eight, king]);
        assert!(!state.players[0].is_on_board);
    }

    #[test]
    fn test_pre_board_safe_discard() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![seven, king], false);
        let queen = state.deck.discard[0];
        let other = state.players[1].clone();
        let board = state.board.clone();

        assert!(state.apply(Action::DiscardCard(king), 0));

        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[0].hand, vec![seven]);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert_eq!(state.board, board);
        assert_eq!(state.players[1], other);
        assert_eq!(state.drawn_card_id, None);
        assert!(!state.players[0].is_on_board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(king.get_penalty_value(), 10);
    }

    #[test]
    fn test_quick_discard() {
        let five = card(1, Suit::Hearts, Rank::Five);
        let six = card(2, Suit::Hearts, Rank::Six);
        let wild = card(3, Suit::None, Rank::Joker);
        let other_seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let drawn = card(6, Suit::Hearts, Rank::Seven);
        let penalty = card(7, Suit::Clubs, Rank::Three);
        let buried = card(8, Suit::Diamonds, Rank::Queen);
        let top = card(9, Suit::Spades, Rank::Ace);
        let next_kept = card(10, Suit::Clubs, Rank::Nine);
        let mut state = heart_gap(vec![other_seven, king], false);
        state.players[1].hand = vec![next_kept];
        state.deck.cards = vec![drawn, penalty];
        state.deck.discard = vec![buried, top];

        refuse_discard(&state, 0, other_seven);
        assert!(state.apply(Action::PushDiscard, 0));

        assert_eq!(state.players[0].hand, vec![other_seven, king, drawn]);
        assert_eq!(state.drawn_card_id, Some(drawn.id));
        assert_eq!(state.players[1].hand, vec![next_kept, top, penalty]);
        assert_eq!(state.deck.discard, vec![buried]);
        assert!(state.deck.cards.is_empty());
        refuse_discard(&state, 0, other_seven);

        let mut safe_first = state.clone();
        assert!(safe_first.apply(Action::DiscardCard(king), 0));
        assert_eq!(safe_first.drawn_card_id, Some(drawn.id));
        assert_eq!(safe_first.players[0].hand, vec![other_seven, drawn]);
        assert!(safe_first.apply(Action::DiscardCard(drawn), 0));
        assert_eq!(safe_first.drawn_card_id, None);
        assert_eq!(safe_first.players[0].hand, vec![other_seven]);
        assert_eq!(safe_first.deck.discard, vec![buried, king, drawn]);
        assert_eq!(safe_first.board, vec![vec![five, six, wild]]);

        assert!(state.apply(Action::DiscardCard(drawn), 0));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.players[0].hand, vec![other_seven, king]);
        assert_eq!(state.deck.discard, vec![buried, drawn]);
        assert_eq!(state.board, vec![vec![five, six, wild]]);
        refuse_discard(&state, 0, other_seven);
        assert!(state.apply(Action::DiscardCard(king), 0));
        assert_eq!(state.players[0].hand, vec![other_seven]);
        assert_eq!(state.deck.discard, vec![buried, drawn, king]);
        assert!(!state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(drawn.get_penalty_value(), 5);
    }

    #[test]
    fn test_quick_discard_of_the_card_just_taken() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let other_seven = card(6, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![other_seven, king], false);
        let queen = state.deck.discard[0];
        state.deck.discard.push(seven);

        refuse_discard(&state, 0, other_seven);
        assert!(state.apply(Action::TakeDiscard, 0));
        assert_eq!(state.players[0].hand, vec![other_seven, king, seven]);
        assert_eq!(state.drawn_card_id, Some(seven.id));
        assert_eq!(state.deck.discard, vec![queen]);
        refuse_discard(&state, 0, other_seven);

        assert!(state.apply(Action::DiscardCard(seven), 0));
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.players[0].hand, vec![other_seven, king]);
        assert_eq!(state.deck.discard, vec![queen, seven]);
        assert!(!state.players[0].is_on_board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        refuse_discard(&state, 0, other_seven);
    }

    #[test]
    fn test_advance_turn_ends_the_quick_discard() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![king], false);
        state.deck.discard.push(seven);
        assert!(state.apply(Action::TakeDiscard, 0));
        assert_eq!(state.drawn_card_id, Some(seven.id));

        let mut this_turn = state.clone();
        assert!(this_turn.apply(Action::DiscardCard(seven), 0));
        assert_eq!(this_turn.drawn_card_id, None);
        assert_eq!(this_turn.players[0].hand, vec![king]);

        let before = state.clone();
        state.advance_turn();
        assert_eq!(state.turn_counter, 1);
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.players, before.players);
        assert_eq!(state.board, before.board);
        assert_eq!(state.deck, before.deck);
        assert_eq!(state.round_number, 1);
        refuse_discard(&state, 0, seven);
        assert_eq!(state.players[0].hand, vec![king, seven]);
    }

    #[test]
    fn test_on_board_discard_allows_a_playable_card() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![seven, king], true);
        let queen = state.deck.discard[0];
        let board = state.board.clone();
        let other = state.players[1].clone();

        assert!(state.apply(Action::DiscardCard(seven), 0));

        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[0].hand, vec![king]);
        assert_eq!(state.deck.discard, vec![queen, seven]);
        assert_eq!(state.board, board);
        assert_eq!(state.players[1], other);
        assert!(state.players[0].is_on_board);
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);

        state.players[0].hand = vec![seven];
        state.deck.discard = vec![queen];
        assert!(state.apply(Action::DiscardCard(seven), 0));
        assert!(state.players[0].hand.is_empty());
        assert_eq!(state.deck.discard, vec![queen, seven]);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.board, board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    #[test]
    fn test_discard_refuses_a_card_that_is_not_held() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let lookalike = card(50, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![seven, king], false);
        let other = state.players[1].hand[0];
        let on_the_board = state.board[0][0];
        state.deck.cards = vec![lookalike];

        refuse_discard(&state, 0, lookalike);
        refuse_discard(&state, 0, other);
        refuse_discard(&state, 0, on_the_board);
        state.players[0].hand.clear();
        refuse_discard(&state, 0, seven);
        assert!(state.players[0].hand.is_empty());
        assert_eq!(state.deck.cards, vec![lookalike]);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_locked_card_is_safe_to_discard_until_it_can_be_played() {
        let seven = locked(4, Suit::Hearts, Rank::Seven, 3);
        let king = card(5, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![seven, king], false);
        let queen = state.deck.discard[0];
        let board = state.board.clone();

        let mut allowed = state.clone();
        assert!(allowed.apply(Action::DiscardCard(seven), 0));
        assert_eq!(allowed.turn_phase, TurnPhase::Playing);
        assert_eq!(allowed.players[0].hand, vec![king]);
        assert_eq!(allowed.deck.discard, vec![queen, seven]);
        assert_eq!(allowed.board, board);
        assert_eq!(allowed.deck.discard.last().unwrap().locked_until_turn, 3);
        assert_eq!(allowed.round_number, 1);
        assert_eq!(allowed.turn_counter, 0);

        state.advance_turn();
        state.advance_turn();
        state.advance_turn();
        assert_eq!(state.turn_counter, 3);
        refuse_discard(&state, 0, seven);
        assert_eq!(state.players[0].hand, vec![seven, king]);
        assert_eq!(state.board, board);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_off_board_cannot_discard_a_wild_that_fits_a_set() {
        let joker = card(4, Suit::None, Rank::Joker);
        let king = card(5, Suit::Spades, Rank::King);
        let meld = vec![
            card(1, Suit::Spades, Rank::Eight),
            card(2, Suit::Clubs, Rank::Eight),
            card(3, Suit::Diamonds, Rank::Eight),
        ];
        let mut state = board_with(vec![joker, king], meld.clone(), false);
        let queen = state.deck.discard[0];

        refuse_discard(&state, 0, joker);
        assert!(state.apply(Action::DiscardCard(king), 0));
        assert_eq!(state.players[0].hand, vec![joker]);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert_eq!(state.board, vec![meld]);
        assert!(!state.players[0].is_on_board);
        assert_eq!(joker.get_penalty_value(), 20);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
    }

    #[test]
    fn test_empty_board_discard() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![seven], false);
        state.board.clear();
        let queen = state.deck.discard[0];

        assert!(state.apply(Action::DiscardCard(seven), 0));
        assert!(state.players[0].hand.is_empty());
        assert!(state.round_over);
        assert_eq!(state.deck.discard, vec![queen, seven]);
        assert!(state.board.is_empty());
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    #[test]
    fn test_discard_that_fits_only_the_second_meld() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![seven], false);
        let fours = vec![
            card(11, Suit::Spades, Rank::Four),
            card(12, Suit::Clubs, Rank::Four),
            card(13, Suit::Diamonds, Rank::Four),
        ];
        state.board.insert(0, fours);

        refuse_discard(&state, 0, seven);
        assert_eq!(state.players[0].hand, vec![seven]);
        assert_eq!(state.board.len(), 2);
    }

    #[test]
    fn test_penalty_draw_initiation() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![seven, king], false);
        let queen = state.deck.discard[0];
        let board = state.board.clone();
        let other = state.players[1].clone();
        assert_eq!(state.turn_phase, TurnPhase::Playing);

        assert!(!state.apply(Action::DiscardCard(seven), 0));

        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.players[0].hand, vec![seven, king]);
        assert_eq!(state.board, board);
        assert_eq!(state.deck.discard, vec![queen]);
        assert_eq!(state.players[1], other);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert!(!state.players[0].is_on_board);

        assert!(state.apply(Action::DiscardCard(king), 0));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[0].hand, vec![seven]);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert_eq!(state.board, board);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_discard_of_an_absent_card_stays_in_playing() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let lookalike = card(50, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![seven], false);
        state.deck.cards = vec![lookalike];

        refuse_discard(&state, 0, lookalike);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
    }

    #[test]
    fn test_draw_from_deck_refuses_outside_penalty() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![seven], false);
        let before = state.clone();

        assert!(!state.apply(Action::DrawFromDeck, 0));
        assert_eq!(state.players, before.players);
        assert_eq!(state.board, before.board);
        assert_eq!(state.deck, before.deck);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_penalty_draw_execution() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let first = card(6, Suit::Hearts, Rank::Seven);
        let second = card(7, Suit::Hearts, Rank::Eight);
        let third = card(8, Suit::Hearts, Rank::Four);
        let safe = card(9, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![held], false);
        let queen = state.deck.discard[0];
        let other = state.players[1].clone();
        let board = state.board.clone();
        state.deck.cards = vec![safe, third, second, first];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.players[0].hand, vec![held]);

        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held, first, second, third]);
        assert_eq!(state.deck.discard, vec![queen, safe]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.board, board);
        assert_eq!(state.players[1], other);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(safe.get_penalty_value(), 10);
        assert!(!state.players[0].is_on_board);

        let finished = state.clone();
        assert!(!state.apply(Action::DrawFromDeck, 0));
        assert_eq!(state.players, finished.players);
        assert_eq!(state.deck, finished.deck);
        assert_eq!(state.board, finished.board);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.turn_counter, 0);
    }

    #[test]
    fn test_penalty_draw_keeps_playable_cards_when_the_pile_runs_out() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let first = card(6, Suit::Hearts, Rank::Eight);
        let second = card(7, Suit::Hearts, Rank::Four);
        let mut state = heart_gap(vec![held], false);
        state.deck.discard.clear();
        state.deck.cards = vec![second, first];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held, first, second]);
        assert!(state.deck.cards.is_empty());
        assert!(state.deck.discard.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_penalty_last_two_playable_cards_stay_in_the_hands() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let low = card(6, Suit::Hearts, Rank::Four);
        let high = card(7, Suit::Hearts, Rank::Eight);
        let mut state = heart_gap(vec![held], false);
        let other_before = state.players[1].hand.clone();
        state.deck.cards.clear();
        state.deck.discard = vec![low, high];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand.len(), 2);
        assert_eq!(state.players[0].hand[0], held);
        assert_eq!(state.players[1].hand.len(), other_before.len() + 1);
        assert!(state.deck.cards.is_empty());
        assert!(state.deck.discard.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert!(!state.round_over);
        let mut moved = vec![
            state.players[0].hand[1],
            state.players[1].hand[other_before.len()],
        ];
        moved.sort_by_key(|card| card.id);
        let mut expect = vec![low, high];
        expect.sort_by_key(|card| card.id);
        assert_eq!(moved, expect);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_advance_turn_leaves_the_penalty() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![seven], false);
        assert!(!state.apply(Action::DiscardCard(seven), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        let before = state.clone();

        state.advance_turn();

        assert_eq!(state.turn_counter, 1);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players, before.players);
        assert_eq!(state.board, before.board);
        assert_eq!(state.deck, before.deck);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_penalty_draw_keeps_a_wild_that_fits() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let joker = card(6, Suit::None, Rank::Joker);
        let two = card(7, Suit::Clubs, Rank::Two);
        let safe = card(9, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![held], false);
        let queen = state.deck.discard[0];
        let other = state.players[1].clone();
        state.deck.cards = vec![safe, two, joker];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held, joker, two]);
        assert_eq!(state.deck.discard, vec![queen, safe]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[1], other);
        assert_eq!(joker.get_penalty_value(), 20);
        assert_eq!(two.get_penalty_value(), 20);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    /// Draw a joker, then a king. The king is discarded. The joker stays in the hand.
    /// That joker fits the heart run, so it cannot be discarded or hit. A later king
    /// is discarded too. The joker's 20 stays in the hand. The round does not end.
    #[test]
    fn test_trapped_by_a_draw() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let joker = card(6, Suit::None, Rank::Joker);
        let king = card(9, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![held], false);
        let queen = state.deck.discard[0];
        let board = state.board.clone();
        let other = state.players[1].clone();
        state.deck.cards = vec![king, joker];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(0));

        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held, joker]);
        assert_eq!(state.players[0].hand[1].locked_until_turn, 0);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.board, board);
        assert_eq!(state.players[1], other);
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(joker.get_penalty_value(), 20);
        assert_eq!(state.players[0].calculate_hand_penalty(), 25);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert!(!state.round_over);
        assert!(!state.players[0].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);

        let mut hit = state.clone();
        assert!(!hit.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![joker],
            }]),
            0,
        ));
        assert_eq!(hit.turn_phase, TurnPhase::Playing);
        assert_eq!(hit.players[0].hand, vec![held, joker]);
        assert_eq!(hit.board, board);

        let mut shed_seven = state.clone();
        assert!(!shed_seven.apply(Action::DiscardCard(held), 0));
        assert_eq!(shed_seven.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(shed_seven.players[0].hand, vec![held, joker]);
        assert_eq!(shed_seven.deck.discard, vec![queen, king]);

        assert!(!state.apply(Action::DiscardCard(joker), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(0));
        assert_eq!(state.players[0].hand, vec![held, joker]);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert_eq!(state.board, board);
        assert_eq!(state.players[1], other);
        assert!(!state.round_over);
        assert!(!state.apply(Action::DrawFromDeck, 1));

        let later = card(19, Suit::Clubs, Rank::King);
        state.deck.cards = vec![later];
        assert!(state.apply(Action::DrawFromDeck, 0));
        assert_eq!(state.players[0].hand, vec![held, joker]);
        assert_eq!(state.deck.discard, vec![queen, king, later]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.players[0].calculate_hand_penalty(), 25);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert!(!state.round_over);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert!(!state.apply(Action::DrawFromDeck, 0));
    }

    /// The card just taken stays the quick discard. The joker drawn in the penalty
    /// does not take that exemption, so discarding the joker is still refused.
    #[test]
    fn test_trapped_by_a_draw_leaves_the_quick_discard() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let joker = card(6, Suit::None, Rank::Joker);
        let king = card(9, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![seven], false);
        let queen = state.deck.discard[0];
        state.deck.cards = vec![king, joker];

        assert!(state.apply(Action::TakeDiscard, 0));
        assert_eq!(state.drawn_card_id, Some(queen.id));
        assert_eq!(state.players[0].hand, vec![seven, queen]);
        assert!(!state.apply(Action::DiscardCard(seven), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.drawn_card_id, Some(queen.id));

        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![seven, queen, joker]);
        assert_eq!(state.deck.discard, vec![king]);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.drawn_card_id, Some(queen.id));
        assert_eq!(state.players[0].calculate_hand_penalty(), 35);
        assert!(!state.apply(Action::DiscardCard(joker), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.players[0].hand, vec![seven, queen, joker]);
        assert_eq!(state.drawn_card_id, Some(queen.id));

        assert!(state.apply(Action::DiscardCard(queen), 0));
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[0].hand, vec![seven, joker]);
        assert_eq!(state.deck.discard, vec![king, queen]);
        assert_eq!(state.players[0].calculate_hand_penalty(), 25);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    /// The same trap on a set. The drawn joker fits the eights. The king does not.
    /// Discarding the joker is refused. Its 20 stays in the hand.
    #[test]
    fn test_trapped_by_a_draw_on_a_set() {
        let held = card(4, Suit::Hearts, Rank::Eight);
        let joker = card(6, Suit::None, Rank::Joker);
        let king = card(9, Suit::Spades, Rank::King);
        let meld = vec![
            card(1, Suit::Spades, Rank::Eight),
            card(2, Suit::Clubs, Rank::Eight),
            card(3, Suit::Diamonds, Rank::Eight),
        ];
        let mut state = board_with(vec![held], meld.clone(), false);
        let queen = state.deck.discard[0];
        let other = state.players[1].clone();
        state.deck.cards = vec![king, joker];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held, joker]);
        assert_eq!(state.players[0].hand[1].locked_until_turn, 0);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.board, vec![meld]);
        assert_eq!(state.players[1], other);
        assert_eq!(state.players[0].calculate_hand_penalty(), 25);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);

        assert!(!state.apply(Action::DiscardCard(joker), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(0));
        assert_eq!(state.players[0].hand, vec![held, joker]);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert!(!state.players[0].is_on_board);
        assert!(!state.round_over);
    }

    #[test]
    fn test_penalty_draw_keeps_a_card_that_fits_only_the_second_meld() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let four = card(14, Suit::Clubs, Rank::Four);
        let safe = card(9, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![held], false);
        let queen = state.deck.discard[0];
        let fours = vec![
            card(11, Suit::Spades, Rank::Four),
            card(12, Suit::Hearts, Rank::Four),
            card(13, Suit::Diamonds, Rank::Four),
        ];
        state.board.push(fours);
        state.deck.cards = vec![safe, four];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held, four]);
        assert_eq!(state.deck.discard, vec![queen, safe]);
        assert_eq!(state.board.len(), 2);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_penalty_draw_discards_a_locked_card() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let locked_seven = locked(6, Suit::Hearts, Rank::Seven, 5);
        let mut state = heart_gap(vec![held], false);
        let queen = state.deck.discard[0];
        let board = state.board.clone();
        state.deck.cards = vec![locked_seven];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held]);
        assert_eq!(state.deck.discard, vec![queen, locked_seven]);
        assert_eq!(state.deck.discard.last().unwrap().locked_until_turn, 5);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.board, board);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert_eq!(locked_seven.get_penalty_value(), 5);
    }

    /// Both piles are empty while this seat is drawing a penalty. The draw is
    /// accepted, the turn returns to playing, and the playable card stays.
    #[test]
    fn test_penalty_draw_ends_turn_when_deck_exhausted() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![held], false);
        let other = state.players[1].clone();
        let board = state.board.clone();
        assert!(!state.apply(Action::DiscardCard(held), 0));
        state.deck.cards.clear();
        state.deck.discard.clear();
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(0));

        let mut rng = StdRng::seed_from_u64(16);
        assert!(state.apply_with_rng(Action::DrawFromDeck, 0, &mut rng));

        assert_eq!(state.players[0].hand, vec![held]);
        assert_eq!(state.players[1], other);
        assert_eq!(state.board, board);
        assert!(state.deck.cards.is_empty());
        assert!(state.deck.discard.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert!(!state.round_over);
        assert!(!state.players[0].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);

        assert!(!state.apply_with_rng(Action::DrawFromDeck, 0, &mut rng));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.players[0].hand, vec![held]);
        assert!(state.deck.cards.is_empty());
        assert!(state.deck.discard.is_empty());
    }

    /// The other seat cannot draw, and the penalty stays, when nothing is left.
    #[test]
    fn test_penalty_draw_empty_piles_refuse_the_other_seat() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![held], false);
        assert!(!state.apply(Action::DiscardCard(held), 0));
        state.deck.cards.clear();
        state.deck.discard.clear();
        let before = state.clone();

        assert!(!state.apply(Action::DrawFromDeck, 1));

        assert_eq!(state.players, before.players);
        assert_eq!(state.board, before.board);
        assert_eq!(state.deck, before.deck);
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(0));
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.turn_counter, 0);
        assert!(!state.round_over);
    }

    /// An empty shoe outside penalty drawing is still refused. The phase stays playing.
    #[test]
    fn test_draw_from_deck_refuses_outside_penalty_when_the_piles_are_empty() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![seven], false);
        state.deck.cards.clear();
        state.deck.discard.clear();
        let before = state.clone();

        assert!(!state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players, before.players);
        assert_eq!(state.board, before.board);
        assert_eq!(state.deck, before.deck);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.turn_counter, 0);
        assert!(!state.round_over);
    }

    /// A closed round does not accept the draw, even when the piles are empty.
    #[test]
    fn test_penalty_draw_closed_round_stays_when_the_piles_are_empty() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![held], false);
        state.turn_phase = TurnPhase::PenaltyDrawing;
        state.penalty_seat = Some(0);
        state.deck.cards.clear();
        state.deck.discard.clear();
        state.round_over = true;
        let before = state.clone();

        assert!(!state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players, before.players);
        assert_eq!(state.deck, before.deck);
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(0));
        assert!(state.round_over);
    }

    /// Every recycled card fits. The hand keeps them and the turn ends.
    #[test]
    fn test_penalty_draw_ends_turn_when_every_recycled_card_fits() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let four = card(6, Suit::Hearts, Rank::Four);
        let eight = card(7, Suit::Hearts, Rank::Eight);
        let another = card(8, Suit::Hearts, Rank::Seven);
        let mut state = heart_gap(vec![held], false);
        let other = state.players[1].clone();
        let board = state.board.clone();
        state.deck.cards.clear();
        state.deck.discard = vec![four, eight, another];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        let mut rng = StdRng::seed_from_u64(1);
        assert!(state.apply_with_rng(Action::DrawFromDeck, 0, &mut rng));

        let mut hand = state.players[0].hand.clone();
        hand.sort_by_key(|card| card.id);
        let mut expect = vec![held, four, eight, another];
        expect.sort_by_key(|card| card.id);
        assert_eq!(hand, expect);
        assert!(state.deck.cards.is_empty());
        assert!(state.deck.discard.is_empty());
        assert_eq!(state.players[1], other);
        assert_eq!(state.board, board);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    #[test]
    fn test_penalty_draw_puts_the_last_safe_discard_card_back() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let king = card(9, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![held], false);
        let other = state.players[1].clone();
        state.deck.cards.clear();
        state.deck.discard = vec![king];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held]);
        assert_eq!(state.players[1], other);
        assert_eq!(state.deck.discard, vec![king]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.turn_counter, 0);
    }

    #[test]
    fn test_penalty_draw_keeps_the_last_discard_card_when_it_fits() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let eight = card(8, Suit::Hearts, Rank::Eight);
        let mut state = heart_gap(vec![held], false);
        let other = state.players[1].clone();
        state.deck.cards.clear();
        state.deck.discard = vec![eight];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held, eight]);
        assert_eq!(state.players[1], other);
        assert!(state.deck.discard.is_empty());
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
    }

    #[test]
    fn test_penalty_draw_splits_the_last_two_safe_cards() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let king = card(9, Suit::Spades, Rank::King);
        let queen = card(10, Suit::Clubs, Rank::Queen);
        let bystander = card(70, Suit::Diamonds, Rank::Nine);
        let mut state = heart_gap(vec![held], false);
        state.players.push(Player::new(3, 2));
        state.players[2].hand = vec![bystander];
        let other_before = state.players[1].hand.clone();
        state.deck.cards.clear();
        state.deck.discard = vec![king, queen];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.players[0].hand, vec![held]);
        assert_eq!(state.players[2].hand, vec![bystander]);
        assert_eq!(state.players[1].hand.len(), other_before.len() + 1);
        assert_eq!(state.deck.discard.len(), 1);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        let mut moved = vec![
            state.deck.discard[0],
            state.players[1].hand[other_before.len()],
        ];
        moved.sort_by_key(|card| card.id);
        let mut expect = vec![king, queen];
        expect.sort_by_key(|card| card.id);
        assert_eq!(moved, expect);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert!(!state.players[0].is_on_board);
    }

    #[test]
    fn test_penalty_draw_reshuffle_keeps_playable_cards_then_discards_the_top() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let four = card(6, Suit::Hearts, Rank::Four);
        let eight = card(7, Suit::Hearts, Rank::Eight);
        let another = card(8, Suit::Hearts, Rank::Seven);
        let king = card(9, Suit::Clubs, Rank::King);
        let mut state = heart_gap(vec![held], false);
        let other = state.players[1].clone();
        let board = state.board.clone();
        state.deck.cards.clear();
        state.deck.discard = vec![four, eight, another, king];

        assert!(!state.apply(Action::DiscardCard(held), 0));
        assert!(state.apply(Action::DrawFromDeck, 0));

        let mut hand = state.players[0].hand.clone();
        hand.sort_by_key(|card| card.id);
        let mut expect = vec![held, four, eight, another];
        expect.sort_by_key(|card| card.id);
        assert_eq!(hand, expect);
        assert_eq!(state.deck.discard, vec![king]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.board, board);
        assert_eq!(state.players[1], other);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
    }

    #[test]
    fn test_penalty_draw_keeps_the_quick_discard_exemption() {
        let other = card(4, Suit::Hearts, Rank::Seven);
        let drawn = card(6, Suit::Hearts, Rank::Seven);
        let safe = card(9, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![other], false);
        let queen = state.deck.discard[0];
        state.deck.discard.push(drawn);

        assert!(state.apply(Action::TakeDiscard, 0));
        assert_eq!(state.drawn_card_id, Some(drawn.id));
        assert!(!state.apply(Action::DiscardCard(other), 0));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        state.deck.cards = vec![safe];
        assert!(state.apply(Action::DrawFromDeck, 0));

        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.drawn_card_id, Some(drawn.id));
        assert_eq!(state.players[0].hand, vec![other, drawn]);
        assert_eq!(state.deck.discard, vec![queen, safe]);
        assert_eq!(state.turn_counter, 0);

        assert!(state.apply(Action::DiscardCard(drawn), 0));
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[0].hand, vec![other]);
        assert_eq!(state.deck.discard, vec![queen, safe, drawn]);
        assert_eq!(state.round_number, 1);
        assert!(!state.players[0].is_on_board);
    }

    /// Seat 1 is off the board. The last card of `draw_under` is the next draw after the push.
    /// The push gives seat 1 a 7♥ and an 8♥. Both fit 5♥ 6♥ joker. Seat 0 drew the jack.
    fn push_heart_trap(held: Vec<Card>, draw_under: Vec<Card>) -> (GameState, Card, Card, Card) {
        let seven = card(10, Suit::Hearts, Rank::Seven);
        let eight = card(11, Suit::Hearts, Rank::Eight);
        let actor = card(12, Suit::Clubs, Rank::Jack);
        let king = card(13, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![king], false);
        let mut expect = held.clone();
        expect.push(seven);
        expect.push(eight);
        state.players[1].hand = held;
        let mut cards = draw_under;
        cards.push(actor);
        cards.push(eight);
        state.deck.cards = cards;
        state.deck.discard.push(seven);
        assert!(state.apply(Action::PushDiscard, 0));
        assert_eq!(state.players[1].hand, expect);
        assert_eq!(state.drawn_card_id, Some(actor.id));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert!(!state.round_over);
        (state, seven, eight, actor)
    }

    /// Push a 7♥ and an 8♥ onto the off-board seat. Both fit the heart run.
    /// The jack the pusher drew is the only quick discard. Discarding either pushed
    /// card enters penalty drawing. Three cards that fit stay, and the king is discarded.
    /// The pushed cards stay in the hand. A later discard of the 7♥ enters the penalty again.
    /// points and total_score stay as they were. The round does not end.
    #[test]
    fn test_pushed_penalty_trap() {
        let four = card(14, Suit::Hearts, Rank::Four);
        let other_seven = card(15, Suit::Hearts, Rank::Seven);
        let three = card(16, Suit::Hearts, Rank::Three);
        let safe = card(17, Suit::Clubs, Rank::King);
        let (mut state, seven, eight, actor) =
            push_heart_trap(vec![], vec![safe, three, other_seven, four]);
        let queen = state.deck.discard[0];
        let board = state.board.clone();
        let pusher = state.players[0].clone();
        assert_eq!(state.players[1].hand, vec![seven, eight]);
        assert_eq!(state.players[1].calculate_hand_penalty(), 10);

        let mut discard_penalty = state.clone();
        assert!(!discard_penalty.apply(Action::DiscardCard(eight), 1));
        assert_eq!(discard_penalty.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(discard_penalty.players[1].hand, vec![seven, eight]);
        assert_eq!(discard_penalty.players[0], pusher);
        assert_eq!(discard_penalty.board, board);
        assert_eq!(discard_penalty.deck, state.deck);
        assert!(!discard_penalty.round_over);
        assert_eq!(discard_penalty.players[1].points, 4);
        assert_eq!(discard_penalty.players[1].total_score, 9);

        let mut later = state.clone();
        later.advance_turn();
        assert_eq!(later.turn_counter, 1);
        assert_eq!(later.drawn_card_id, None);
        assert!(!later.apply(Action::DiscardCard(seven), 1));
        assert_eq!(later.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(later.players[1].hand, vec![seven, eight]);
        assert_eq!(later.board, board);

        let mut quick = state.clone();
        assert!(quick.apply(Action::DiscardCard(actor), 0));
        assert_eq!(quick.drawn_card_id, None);
        assert_eq!(quick.turn_phase, TurnPhase::Playing);
        assert!(!quick.apply(Action::DiscardCard(seven), 1));
        assert_eq!(quick.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(quick.players[1].hand, vec![seven, eight]);

        assert!(!state.apply(Action::DrawFromDeck, 1));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert!(!state.apply(Action::DiscardCard(seven), 1));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(1));
        assert_eq!(state.players[1].hand, vec![seven, eight]);
        assert_eq!(state.deck.discard, vec![queen]);
        assert_eq!(state.players[0], pusher);
        assert_eq!(state.board, board);

        let mut pusher_acts = state.clone();
        assert!(pusher_acts.apply(Action::DiscardCard(actor), 0));
        assert_eq!(pusher_acts.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(pusher_acts.penalty_seat, Some(1));
        assert_eq!(pusher_acts.players[1].hand, vec![seven, eight]);
        assert!(!pusher_acts.apply(Action::DrawFromDeck, 0));
        assert_eq!(pusher_acts.players[1].hand, vec![seven, eight]);
        assert_eq!(pusher_acts.deck.cards, state.deck.cards);
        assert!(pusher_acts.apply(Action::DrawFromDeck, 1));
        assert_eq!(
            pusher_acts.players[1].hand,
            vec![seven, eight, four, other_seven, three]
        );
        assert_eq!(pusher_acts.penalty_seat, None);
        assert_eq!(pusher_acts.turn_phase, TurnPhase::Playing);

        assert!(state.apply(Action::DrawFromDeck, 1));

        assert_eq!(
            state.players[1].hand,
            vec![seven, eight, four, other_seven, three]
        );
        assert_eq!(state.deck.discard, vec![queen, safe]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.board, board);
        assert_eq!(state.players[0], pusher);
        assert_eq!(state.drawn_card_id, Some(actor.id));
        assert!(!state.round_over);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(state.players[1].calculate_hand_penalty(), 25);
        assert_eq!(safe.get_penalty_value(), 10);

        let finished = state.clone();
        assert!(!state.apply(Action::DrawFromDeck, 1));
        assert_eq!(state.players, finished.players);
        assert_eq!(state.deck, finished.deck);
        assert_eq!(state.board, finished.board);
        assert_eq!(state.turn_phase, TurnPhase::Playing);

        assert!(!state.apply(Action::DiscardCard(seven), 1));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.players[1].hand, finished.players[1].hand);
        assert_eq!(state.deck, finished.deck);
        assert_eq!(state.board, board);
        assert!(!state.round_over);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
    }

    /// The pushed penalty is a joker. It fits the set of eights, so discarding it
    /// enters penalty drawing. The draw keeps a two. The joker's 20 stays in the hand.
    #[test]
    fn test_pushed_penalty_trap_wild_stays_in_the_hand() {
        let meld = vec![
            card(1, Suit::Spades, Rank::Eight),
            card(2, Suit::Clubs, Rank::Eight),
            card(3, Suit::Diamonds, Rank::Eight),
        ];
        let pushed = card(10, Suit::Hearts, Rank::Eight);
        let joker = card(11, Suit::None, Rank::Joker);
        let actor = card(12, Suit::Clubs, Rank::Jack);
        let two = card(14, Suit::Spades, Rank::Two);
        let safe = card(17, Suit::Clubs, Rank::King);
        let king = card(13, Suit::Spades, Rank::King);
        let mut state = board_with(vec![king], meld.clone(), false);
        state.players[1].hand.clear();
        state.deck.cards = vec![safe, two, actor, joker];
        state.deck.discard.push(pushed);
        let queen = state.deck.discard[0];

        assert!(state.apply(Action::PushDiscard, 0));
        assert_eq!(state.players[1].hand, vec![pushed, joker]);
        assert_eq!(state.drawn_card_id, Some(actor.id));
        assert_eq!(joker.get_penalty_value(), 20);
        assert_eq!(state.players[1].calculate_hand_penalty(), 25);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);

        assert!(!state.apply(Action::DiscardCard(joker), 1));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.players[1].hand, vec![pushed, joker]);
        assert_eq!(state.board, vec![meld.clone()]);
        assert_eq!(state.players[1].total_score, 9);

        assert!(state.apply(Action::DrawFromDeck, 1));

        assert_eq!(state.players[1].hand, vec![pushed, joker, two]);
        assert_eq!(state.players[1].calculate_hand_penalty(), 45);
        assert_eq!(state.deck.discard, vec![queen, safe]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.board, vec![meld]);
        assert!(!state.round_over);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(two.get_penalty_value(), 20);
    }

    /// Seat 1 already holds a king. Discarding a pushed card still enters penalty
    /// drawing. The king then leaves, and no card is drawn. The pushed cards stay.
    #[test]
    fn test_pushed_penalty_trap_safe_card_already_held() {
        let held = card(19, Suit::Spades, Rank::King);
        let (mut state, seven, eight, _) = push_heart_trap(vec![held], vec![]);
        let queen = state.deck.discard[0];
        let board = state.board.clone();
        assert_eq!(state.players[1].hand, vec![held, seven, eight]);

        let mut discard_penalty = state.clone();
        assert!(!discard_penalty.apply(Action::DiscardCard(eight), 1));
        assert_eq!(discard_penalty.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(discard_penalty.players[1].hand, vec![held, seven, eight]);

        assert!(!state.apply(Action::DiscardCard(seven), 1));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.players[1].hand, vec![held, seven, eight]);
        assert_eq!(state.deck.discard, vec![queen]);

        assert!(state.apply(Action::DiscardCard(held), 1));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[1].hand, vec![seven, eight]);
        assert_eq!(state.deck.discard, vec![queen, held]);
        assert_eq!(state.board, board);
        assert!(state.deck.cards.is_empty());
        assert!(!state.round_over);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(held.get_penalty_value(), 10);

        let finished = state.clone();
        assert!(!state.apply(Action::DrawFromDeck, 1));
        assert_eq!(state.players, finished.players);
        assert_eq!(state.deck, finished.deck);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert!(!state.round_over);
    }

    /// The penalty card is locked, so it can be discarded. The pushed 7♥ still fits,
    /// and discarding that 7♥ enters penalty drawing with both cards still in the hand.
    #[test]
    fn test_pushed_penalty_trap_locked_penalty_is_safe() {
        let seven = card(10, Suit::Hearts, Rank::Seven);
        let eight = locked(11, Suit::Hearts, Rank::Eight, 5);
        let actor = card(12, Suit::Clubs, Rank::Jack);
        let king = card(13, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![king], false);
        state.players[1].hand.clear();
        state.deck.cards = vec![actor, eight];
        state.deck.discard.push(seven);
        let queen = state.deck.discard[0];
        let board = state.board.clone();

        assert!(state.apply(Action::PushDiscard, 0));
        assert_eq!(state.players[1].hand, vec![seven, eight]);
        assert_eq!(state.players[1].hand[1].locked_until_turn, 5);

        let mut trapped = state.clone();
        assert!(!trapped.apply(Action::DiscardCard(seven), 1));
        assert_eq!(trapped.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(trapped.players[1].hand, vec![seven, eight]);
        assert_eq!(trapped.deck.discard, vec![queen]);
        assert_eq!(trapped.board, board);
        assert!(!trapped.round_over);

        assert!(state.apply(Action::DiscardCard(eight), 1));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[1].hand, vec![seven]);
        assert_eq!(state.deck.discard, vec![queen, eight]);
        assert_eq!(state.deck.discard.last().unwrap().locked_until_turn, 5);
        assert_eq!(state.board, board);
        assert!(!state.round_over);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(eight.get_penalty_value(), 5);
    }

    /// A seat that is already on the board can discard the pushed cards.
    /// The hand ends only when the last of those cards leaves. The round then stays over.
    #[test]
    fn test_pushed_penalty_trap_on_board_can_discard() {
        let (mut state, seven, eight, _) = push_heart_trap(vec![], vec![]);
        let queen = state.deck.discard[0];
        let board = state.board.clone();
        let pusher = state.players[0].clone();
        state.players[1].is_on_board = true;

        assert!(state.apply(Action::DiscardCard(seven), 1));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[1].hand, vec![eight]);
        assert_eq!(state.deck.discard, vec![queen, seven]);
        assert_eq!(state.board, board);
        assert_eq!(state.players[0], pusher);
        assert!(!state.round_over);
        assert!(state.players[1].is_on_board);

        assert!(state.apply(Action::DiscardCard(eight), 1));
        assert!(state.players[1].hand.is_empty());
        assert!(state.round_over);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.deck.discard, vec![queen, seven, eight]);
        assert_eq!(state.board, board);
        assert_eq!(state.players[0], pusher);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);

        let finished = state.clone();
        assert!(!state.apply(Action::DiscardCard(eight), 1));
        assert_eq!(state.players, finished.players);
        assert_eq!(state.deck, finished.deck);
        assert_eq!(state.board, finished.board);
        assert!(state.round_over);
    }

    /// The push took the last draw-pile cards. Penalty drawing recycles three cards
    /// that fit and discards the king that was left on top of them.
    #[test]
    fn test_pushed_penalty_trap_reshuffle() {
        let four = card(14, Suit::Hearts, Rank::Four);
        let three = card(16, Suit::Hearts, Rank::Three);
        let other_seven = card(18, Suit::Hearts, Rank::Seven);
        let safe = card(17, Suit::Clubs, Rank::King);
        let (mut state, seven, eight, _) = push_heart_trap(vec![], vec![]);
        let board = state.board.clone();
        let pusher = state.players[0].clone();
        state.deck.cards.clear();
        state.deck.discard = vec![four, three, other_seven, safe];

        assert!(!state.apply(Action::DiscardCard(seven), 1));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert!(state.apply(Action::DrawFromDeck, 1));

        let mut hand = state.players[1].hand.clone();
        hand.sort_by_key(|card| card.id);
        let mut expect = vec![seven, eight, four, three, other_seven];
        expect.sort_by_key(|card| card.id);
        assert_eq!(hand, expect);
        assert_eq!(state.deck.discard, vec![safe]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.board, board);
        assert_eq!(state.players[0], pusher);
        assert!(!state.round_over);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(safe.get_penalty_value(), 10);
        assert_eq!(state.players[1].calculate_hand_penalty(), 25);
    }

    #[test]
    fn test_round_victory_on_hit() {
        let eight_spades = card(1, Suit::Spades, Rank::Eight);
        let eight_clubs = card(2, Suit::Clubs, Rank::Eight);
        let joker = card(3, Suit::None, Rank::Joker);
        let eight_hearts = card(4, Suit::Hearts, Rank::Eight);
        let eight_diamonds = card(5, Suit::Diamonds, Rank::Eight);
        let meld = vec![eight_spades, eight_clubs, joker];
        let mut state = board_with(vec![eight_hearts, eight_diamonds], meld, true);
        let other = state.players[1].clone();
        assert!(!state.round_over);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![eight_hearts],
            }]),
            0,
        ));
        assert_eq!(state.players[0].hand, vec![eight_diamonds]);
        assert!(!state.round_over);
        assert_eq!(state.round_number, 1);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![eight_diamonds],
            }]),
            0,
        ));
        assert!(state.players[0].hand.is_empty());
        assert!(state.round_over);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1], other);
        assert!(state.players[0].is_on_board);
    }

    #[test]
    fn test_round_victory_on_play_meld() {
        let fours = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
        ];
        let fives = vec![
            card(4, Suit::Hearts, Rank::Five),
            card(5, Suit::Spades, Rank::Five),
            card(6, Suit::Clubs, Rank::Five),
        ];
        let mut state = table(
            fours.iter().chain(fives.iter()).copied().collect(),
            vec![card(7, Suit::Clubs, Rank::Ace)],
            vec![card(8, Suit::Diamonds, Rank::King)],
        );
        let other = state.players[1].clone();
        assert!(!state.round_over);

        assert!(state.apply(Action::PlayMeld(vec![fours, fives]), 0));

        assert!(state.players[0].hand.is_empty());
        assert!(state.round_over);
        assert!(state.players[0].is_on_board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1], other);
        assert!(!state.players[1].is_on_board);
    }

    #[test]
    fn test_round_victory_on_discard() {
        let seven = card(4, Suit::Hearts, Rank::Seven);
        let king = card(5, Suit::Spades, Rank::King);
        let mut state = heart_gap(vec![seven, king], true);
        let queen = state.deck.discard[0];
        let board = state.board.clone();
        let other = state.players[1].clone();
        assert!(!state.round_over);

        assert!(state.apply(Action::DiscardCard(king), 0));
        assert_eq!(state.players[0].hand, vec![seven]);
        assert!(!state.round_over);
        assert_eq!(state.turn_phase, TurnPhase::Playing);

        assert!(state.apply(Action::DiscardCard(seven), 0));
        assert!(state.players[0].hand.is_empty());
        assert!(state.round_over);
        assert_eq!(state.deck.discard, vec![queen, king, seven]);
        assert_eq!(state.board, board);
        assert_eq!(state.round_number, 1);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1], other);
        assert!(state.players[0].is_on_board);
    }

    /// One turn lays two sets, hits that set, hits an opponent run, and steals the joker
    /// from the set beside it. The 8♦ fits that set, so it cannot be discarded beforehand.
    /// After the lay-down it can. The counter stays. The joker's 20 stays in the hand.
    /// points and total_score stay as they were.
    #[test]
    fn test_multi_action_omniturn() {
        let eight_spades = card(1, Suit::Spades, Rank::Eight);
        let eight_clubs = card(2, Suit::Clubs, Rank::Eight);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_spades = card(4, Suit::Spades, Rank::Five);
        let six_spades = card(5, Suit::Spades, Rank::Six);
        let seven_spades = card(6, Suit::Spades, Rank::Seven);
        let run_eight = card(7, Suit::Spades, Rank::Eight);
        let four_hearts = card(10, Suit::Hearts, Rank::Four);
        let four_spades = card(11, Suit::Spades, Rank::Four);
        let four_clubs = card(12, Suit::Clubs, Rank::Four);
        let nine_hearts = card(13, Suit::Hearts, Rank::Nine);
        let nine_spades = card(14, Suit::Spades, Rank::Nine);
        let nine_clubs = card(15, Suit::Clubs, Rank::Nine);
        let four_diamonds = card(16, Suit::Diamonds, Rank::Four);
        let run_four = card(17, Suit::Spades, Rank::Four);
        let eight_hearts = card(19, Suit::Hearts, Rank::Eight);
        let eight_diamonds = card(20, Suit::Diamonds, Rank::Eight);
        let other = card(60, Suit::Clubs, Rank::King);
        let eights = vec![eight_spades, eight_clubs, joker];
        let spades = vec![five_spades, six_spades, seven_spades, run_eight];
        let fours = vec![four_hearts, four_spades, four_clubs];
        let nines = vec![nine_hearts, nine_spades, nine_clubs];
        let mut state = table(
            vec![
                four_hearts,
                four_spades,
                four_clubs,
                nine_hearts,
                nine_spades,
                nine_clubs,
                four_diamonds,
                run_four,
                eight_hearts,
                eight_diamonds,
            ],
            vec![other],
            vec![card(51, Suit::Clubs, Rank::Jack)],
        );
        state.board = vec![eights.clone(), spades.clone()];
        state.players[1].is_on_board = true;
        let queen = state.deck.discard[0];
        let draw = state.deck.cards.clone();
        let seat = state.players[1].clone();
        assert!(!state.players[0].is_on_board);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);

        let mut too_soon = state.clone();
        assert!(!too_soon.apply(Action::DiscardCard(eight_diamonds), 0));
        assert_eq!(too_soon.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(too_soon.penalty_seat, Some(0));
        assert_eq!(too_soon.players[0].hand, state.players[0].hand);
        assert_eq!(too_soon.board, state.board);
        assert!(!too_soon.round_over);
        assert_eq!(state.turn_phase, TurnPhase::Playing);

        let before_hit = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![eight_hearts],
            }]),
            0,
        ));
        assert_still(&before_hit, &state);
        assert!(!state.players[0].is_on_board);

        assert!(state.apply(Action::PlayMeld(vec![fours.clone(), nines.clone()]), 0));
        assert!(state.players[0].is_on_board);
        assert_eq!(
            state.players[0].hand,
            vec![four_diamonds, run_four, eight_hearts, eight_diamonds]
        );
        assert_eq!(
            state.board,
            vec![eights, spades, fours.clone(), nines.clone()]
        );
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 2,
                cards: vec![four_diamonds],
            }]),
            0,
        ));
        assert_eq!(
            state.board[2],
            vec![four_hearts, four_spades, four_clubs, four_diamonds]
        );
        assert_eq!(
            state.players[0].hand,
            vec![run_four, eight_hearts, eight_diamonds]
        );

        let before_run = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 1,
                cards: vec![eight_diamonds],
            }]),
            0,
        ));
        assert_still(&before_run, &state);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 1,
                cards: vec![run_four],
            }]),
            0,
        ));
        assert_eq!(
            state.board[1],
            vec![five_spades, six_spades, seven_spades, run_eight, run_four]
        );
        assert_eq!(state.players[0].hand, vec![eight_hearts, eight_diamonds]);
        assert_eq!(state.turn_counter, 0);

        let before_steal = state.clone();
        assert!(!state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: nine_hearts,
            }),
            0,
        ));
        assert_still(&before_steal, &state);

        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: eight_hearts,
            }),
            0,
        ));
        let stolen = state.players[0].hand[1];
        assert_eq!(stolen.id, joker.id);
        assert_eq!(stolen.locked_until_turn, 1);
        assert_eq!(state.players[0].hand, vec![eight_diamonds, stolen]);
        assert_eq!(
            state.board[0],
            vec![eight_spades, eight_clubs, eight_hearts]
        );
        assert_eq!(state.turn_counter, 0);
        assert!(!state.round_over);

        let before_locked = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert_still(&before_locked, &state);

        assert!(state.apply(Action::DiscardCard(eight_diamonds), 0));
        assert_eq!(state.players[0].hand, vec![stolen]);
        assert_eq!(state.players[0].hand[0].locked_until_turn, 1);
        assert_eq!(state.players[0].calculate_hand_penalty(), 20);
        assert_eq!(state.deck.discard, vec![queen, eight_diamonds]);
        assert_eq!(state.deck.cards, draw);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert!(!state.round_over);
        assert!(state.players[0].is_on_board);
        assert_eq!(state.players[1], seat);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(joker.get_penalty_value(), 20);
        assert_eq!(state.board[3], nines);

        state.advance_turn();
        assert_eq!(state.turn_counter, 1);
        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![stolen],
            }]),
            0,
        ));
        assert!(state.players[0].hand.is_empty());
        assert!(state.round_over);
        assert_eq!(
            state.board[0],
            vec![eight_spades, eight_clubs, eight_hearts, stolen]
        );
        assert_eq!(state.board[0][3].locked_until_turn, 1);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1], seat);
        assert_eq!(state.round_number, 1);
    }

    /// The 3♠ does not fit 5♠–8♠ until the 4♠ is on that run. That miss leaves the
    /// lay-down and the first hit as they were. The steal and the discard still finish
    /// the turn. The joker's 20 stays in the hand.
    #[test]
    fn test_multi_action_omniturn_second_hit_needs_the_first() {
        let five_spades = card(4, Suit::Spades, Rank::Five);
        let six_spades = card(5, Suit::Spades, Rank::Six);
        let seven_spades = card(6, Suit::Spades, Rank::Seven);
        let run_eight = card(7, Suit::Spades, Rank::Eight);
        let five_hearts = card(21, Suit::Hearts, Rank::Five);
        let five_diamonds = card(22, Suit::Diamonds, Rank::Five);
        let joker = card(23, Suit::None, Rank::Joker);
        let six_hearts = card(30, Suit::Hearts, Rank::Six);
        let six_spades_set = card(31, Suit::Spades, Rank::Six);
        let six_clubs = card(32, Suit::Clubs, Rank::Six);
        let nine_hearts = card(13, Suit::Hearts, Rank::Nine);
        let nine_diamonds = card(34, Suit::Diamonds, Rank::Nine);
        let nine_clubs = card(15, Suit::Clubs, Rank::Nine);
        let run_four = card(17, Suit::Spades, Rank::Four);
        let three_spades = card(18, Suit::Spades, Rank::Three);
        let five_clubs = card(19, Suit::Clubs, Rank::Five);
        let king = card(20, Suit::Diamonds, Rank::King);
        let spades = vec![five_spades, six_spades, seven_spades, run_eight];
        let fives = vec![five_hearts, five_diamonds, joker];
        let sixes = vec![six_hearts, six_spades_set, six_clubs];
        let nines = vec![nine_hearts, nine_diamonds, nine_clubs];
        let mut state = table(
            vec![
                six_hearts,
                six_spades_set,
                six_clubs,
                nine_hearts,
                nine_diamonds,
                nine_clubs,
                run_four,
                three_spades,
                five_clubs,
                king,
            ],
            vec![card(60, Suit::Clubs, Rank::Queen)],
            vec![card(51, Suit::Clubs, Rank::Jack)],
        );
        state.board = vec![spades.clone(), fives.clone()];
        state.players[1].is_on_board = true;
        let queen = state.deck.discard[0];
        let other = state.players[1].clone();

        assert!(state.apply(Action::PlayMeld(vec![sixes, nines.clone()]), 0));
        assert_eq!(
            state.players[0].hand,
            vec![run_four, three_spades, five_clubs, king]
        );

        let before_three = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![three_spades],
            }]),
            0,
        ));
        assert_still(&before_three, &state);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![run_four],
            }]),
            0,
        ));
        assert_eq!(
            state.board[0],
            vec![five_spades, six_spades, seven_spades, run_eight, run_four]
        );

        let before_king = state.clone();
        assert!(!state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![king],
            }]),
            0,
        ));
        assert_still(&before_king, &state);
        assert_eq!(state.board[0].len(), 5);

        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 0,
                cards: vec![three_spades],
            }]),
            0,
        ));
        assert_eq!(state.board[0].len(), 6);
        assert_eq!(state.players[0].hand, vec![five_clubs, king]);

        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 1,
                wild: joker,
                natural: five_clubs,
            }),
            0,
        ));
        let stolen = state.players[0].hand[1];
        assert_eq!(stolen.id, joker.id);
        assert_eq!(stolen.locked_until_turn, 1);
        assert_eq!(state.board[1], vec![five_hearts, five_diamonds, five_clubs]);
        assert_eq!(state.turn_counter, 0);

        assert!(state.apply(Action::DiscardCard(king), 0));
        assert_eq!(state.players[0].hand, vec![stolen]);
        assert_eq!(state.players[0].calculate_hand_penalty(), 20);
        assert_eq!(state.deck.discard, vec![queen, king]);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert!(!state.round_over);
        assert_eq!(state.board[3], nines);
        assert_eq!(state.players[1], other);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(joker.get_penalty_value(), 20);
    }

    /// Seat 1 is already trapped. Seat 0 still lays down, hits twice, steals, and
    /// discards. That discard does not clear the penalty, and seat 0 cannot draw.
    /// Seat 1 still draws. The joker's 20 stays in seat 0's hand.
    #[test]
    fn test_multi_action_omniturn_leaves_the_pushed_penalty() {
        let eight_spades = card(1, Suit::Spades, Rank::Eight);
        let eight_clubs = card(2, Suit::Clubs, Rank::Eight);
        let joker = card(3, Suit::None, Rank::Joker);
        let five_spades = card(4, Suit::Spades, Rank::Five);
        let six_spades = card(5, Suit::Spades, Rank::Six);
        let seven_spades = card(6, Suit::Spades, Rank::Seven);
        let run_eight = card(7, Suit::Spades, Rank::Eight);
        let four_hearts = card(10, Suit::Hearts, Rank::Four);
        let four_spades = card(11, Suit::Spades, Rank::Four);
        let four_clubs = card(12, Suit::Clubs, Rank::Four);
        let nine_hearts = card(13, Suit::Hearts, Rank::Nine);
        let nine_spades = card(14, Suit::Spades, Rank::Nine);
        let nine_clubs = card(15, Suit::Clubs, Rank::Nine);
        let four_diamonds = card(16, Suit::Diamonds, Rank::Four);
        let run_four = card(17, Suit::Spades, Rank::Four);
        let eight_hearts = card(19, Suit::Hearts, Rank::Eight);
        let eight_diamonds = card(20, Suit::Diamonds, Rank::Eight);
        let trapped = card(40, Suit::Hearts, Rank::Eight);
        let safe = card(41, Suit::Clubs, Rank::King);
        let eights = vec![eight_spades, eight_clubs, joker];
        let spades = vec![five_spades, six_spades, seven_spades, run_eight];
        let fours = vec![four_hearts, four_spades, four_clubs];
        let nines = vec![nine_hearts, nine_spades, nine_clubs];
        let mut state = table(
            vec![
                four_hearts,
                four_spades,
                four_clubs,
                nine_hearts,
                nine_spades,
                nine_clubs,
                four_diamonds,
                run_four,
                eight_hearts,
                eight_diamonds,
            ],
            vec![trapped],
            vec![safe],
        );
        state.board = vec![eights, spades];
        state.turn_phase = TurnPhase::PenaltyDrawing;
        state.penalty_seat = Some(1);
        let queen = state.deck.discard[0];
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[1].calculate_hand_penalty(), 5);

        assert!(state.apply(Action::PlayMeld(vec![fours, nines]), 0));
        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 2,
                cards: vec![four_diamonds],
            }]),
            0,
        ));
        assert!(state.apply(
            Action::HitMeld(vec![MeldHit {
                meld_index: 1,
                cards: vec![run_four],
            }]),
            0,
        ));
        assert!(state.apply(
            Action::StealWild(WildSteal {
                meld_index: 0,
                wild: joker,
                natural: eight_hearts,
            }),
            0,
        ));
        let stolen = state.players[0].hand[1];
        assert_eq!(stolen.id, joker.id);
        assert_eq!(stolen.locked_until_turn, 1);
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(1));

        assert!(state.apply(Action::DiscardCard(eight_diamonds), 0));
        assert_eq!(state.players[0].hand, vec![stolen]);
        assert_eq!(state.players[0].calculate_hand_penalty(), 20);
        assert_eq!(state.deck.discard, vec![queen, eight_diamonds]);
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(1));
        assert_eq!(state.players[1].hand, vec![trapped]);
        assert!(!state.apply(Action::DrawFromDeck, 0));
        assert_eq!(state.deck.cards, vec![safe]);
        assert_eq!(state.players[0].hand, vec![stolen]);

        assert!(state.apply(Action::DrawFromDeck, 1));
        assert_eq!(state.players[1].hand, vec![trapped]);
        assert_eq!(state.deck.discard, vec![queen, eight_diamonds, safe]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.round_number, 1);
        assert!(!state.round_over);
        assert!(state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[1].total_score, 9);
        assert_eq!(joker.get_penalty_value(), 20);
        assert_eq!(safe.get_penalty_value(), 10);
    }

    /// A push that only pops a stocked draw pile does not shuffle, so the rng stays idle.
    #[test]
    fn test_apply_with_rng_push_from_a_stocked_pile_leaves_the_rng_idle() {
        let under = card(1, Suit::Hearts, Rank::Four);
        let top = card(2, Suit::Spades, Rank::Ace);
        let start = card(3, Suit::Diamonds, Rank::Six);
        let penalty = card(4, Suit::Clubs, Rank::Five);
        let mut state = GameState::new(
            vec![Player::new(1, 0), Player::new(2, 1)],
            Deck {
                cards: vec![start, penalty],
                discard: vec![under, top],
            },
        );
        let mut rng = CountingRng::new(StdRng::seed_from_u64(1));

        assert!(state.apply_with_rng(Action::PushDiscard, 0, &mut rng));

        assert_eq!(rng.calls, 0);
        assert_eq!(state.players[1].hand, vec![top, penalty]);
        assert_eq!(state.players[0].hand, vec![start]);
        assert_eq!(state.deck.discard, vec![under]);
        assert!(state.deck.cards.is_empty());
        assert_eq!(state.drawn_card_id, Some(start.id));
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.turn_counter, 0);
        assert!(!state.round_over);
    }

    /// Take does not draw. Refusing a penalty draw outside that phase does not draw either.
    #[test]
    fn test_apply_with_rng_take_and_a_refused_draw_leave_the_rng_idle() {
        let mut state = heart_gap(vec![card(4, Suit::Hearts, Rank::Seven)], false);
        let queen = state.deck.discard[0];
        let mut rng = CountingRng::new(StdRng::seed_from_u64(1));

        assert!(state.apply_with_rng(Action::TakeDiscard, 0, &mut rng));

        assert_eq!(rng.calls, 0);
        assert_eq!(state.drawn_card_id, Some(queen.id));
        assert_eq!(
            state.players[0].hand,
            vec![card(4, Suit::Hearts, Rank::Seven), queen]
        );
        assert!(state.deck.discard.is_empty());

        let parked = state.clone();
        assert!(!state.apply_with_rng(Action::DrawFromDeck, 0, &mut rng));
        assert_eq!(rng.calls, 0);
        assert_eq!(state.players, parked.players);
        assert_eq!(state.deck, parked.deck);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.drawn_card_id, Some(queen.id));
    }

    /// An empty draw pile and four discard cards shuffle on the penalty draw, then on the actor's draw.
    /// The same seed repeats that order. Another seed that shuffles differently does not.
    #[test]
    fn test_apply_with_rng_push_reshuffle_follows_the_seed() {
        let ready = push_reshuffle_table();
        let (seed_a, seed_b) = seeds_whose_draws_differ(&ready.deck, 2);
        let mut left = ready.clone();
        let mut again = ready.clone();
        let mut right = ready.clone();
        let mut rng = CountingRng::new(StdRng::seed_from_u64(seed_a));

        assert!(left.apply_with_rng(Action::PushDiscard, 0, &mut rng));
        assert!(again.apply_with_rng(Action::PushDiscard, 0, &mut StdRng::seed_from_u64(seed_a),));
        assert!(right.apply_with_rng(Action::PushDiscard, 0, &mut StdRng::seed_from_u64(seed_b),));

        assert!(rng.calls > 0);
        assert_eq!(left.players, again.players);
        assert_eq!(left.deck, again.deck);
        assert_eq!(left.drawn_card_id, again.drawn_card_id);
        assert_push_matches_draw_with(&ready, &left, seed_a);
        assert_push_matches_draw_with(&ready, &right, seed_b);
        assert!(
            left.players[0].hand != right.players[0].hand
                || left.players[1].hand != right.players[1].hand
        );
        assert_eq!(left.turn_counter, 0);
        assert!(!left.round_over);
        assert_eq!(left.round_number, 1);
    }

    /// Penalty drawing keeps every recycled card that fits and discards the safe top.
    /// Hand order is the seeded draw order.
    #[test]
    fn test_apply_with_rng_penalty_reshuffle_follows_the_seed() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let four = card(6, Suit::Hearts, Rank::Four);
        let eight = card(7, Suit::Hearts, Rank::Eight);
        let another = card(8, Suit::Hearts, Rank::Seven);
        let king = card(9, Suit::Clubs, Rank::King);
        let mut state = heart_gap(vec![held], false);
        state.deck.cards.clear();
        state.deck.discard = vec![four, eight, another, king];
        assert!(!state.apply(Action::DiscardCard(held), 0));
        let ready = state.clone();
        let (seed_a, seed_b) = seeds_whose_draws_differ(&ready.deck, 3);

        let mut left = ready.clone();
        let mut again = ready.clone();
        let mut right = ready.clone();
        let mut rng = CountingRng::new(StdRng::seed_from_u64(seed_a));
        assert!(left.apply_with_rng(Action::DrawFromDeck, 0, &mut rng));
        assert!(again.apply_with_rng(Action::DrawFromDeck, 0, &mut StdRng::seed_from_u64(seed_a),));
        assert!(right.apply_with_rng(Action::DrawFromDeck, 0, &mut StdRng::seed_from_u64(seed_b),));

        assert!(rng.calls > 0);
        assert_eq!(left.players, again.players);
        assert_eq!(left.deck, again.deck);
        assert_eq!(left.turn_phase, TurnPhase::Playing);
        assert_eq!(left.penalty_seat, None);
        assert_penalty_keeps_drawn_prefix(&ready, &left, held, seed_a, king);
        assert_penalty_keeps_drawn_prefix(&ready, &right, held, seed_b, king);
        assert_ne!(left.players[0].hand, right.players[0].hand);
        assert_eq!(left.players[1], ready.players[1]);
        assert_eq!(left.board, ready.board);
        assert_eq!(left.turn_counter, 0);
        assert_eq!(left.round_number, 1);
    }

    /// The last two discard cards are shuffled. The safe card is discarded. The other goes to the next seat.
    #[test]
    fn test_apply_with_rng_penalty_last_two_follows_the_seed() {
        let held = card(4, Suit::Hearts, Rank::Seven);
        let king = card(9, Suit::Spades, Rank::King);
        let queen = card(10, Suit::Clubs, Rank::Queen);
        let bystander = card(70, Suit::Diamonds, Rank::Nine);
        let mut state = heart_gap(vec![held], false);
        state.players.push(Player::new(3, 2));
        state.players[2].hand = vec![bystander];
        state.deck.cards.clear();
        state.deck.discard = vec![king, queen];
        assert!(!state.apply(Action::DiscardCard(held), 0));
        let ready = state.clone();
        let (seed_a, seed_b) = seeds_whose_last_two_differ(&ready.deck);

        let mut left = ready.clone();
        let mut again = ready.clone();
        let mut right = ready.clone();
        let mut rng = CountingRng::new(StdRng::seed_from_u64(seed_a));
        assert!(left.apply_with_rng(Action::DrawFromDeck, 0, &mut rng));
        assert!(again.apply_with_rng(Action::DrawFromDeck, 0, &mut StdRng::seed_from_u64(seed_a),));
        assert!(right.apply_with_rng(Action::DrawFromDeck, 0, &mut StdRng::seed_from_u64(seed_b),));

        assert!(rng.calls > 0);
        assert_eq!(left.players, again.players);
        assert_eq!(left.deck, again.deck);
        assert_last_two_split(&ready, &left, held, bystander, seed_a);
        assert_last_two_split(&ready, &right, held, bystander, seed_b);
        assert!(
            left.deck.discard != right.deck.discard
                || left.players[1].hand != right.players[1].hand
        );
        assert_eq!(left.turn_phase, TurnPhase::Playing);
        assert_eq!(left.penalty_seat, None);
        assert_eq!(left.players[2].hand, vec![bystander]);
        assert_eq!(left.turn_counter, 0);
        assert_eq!(left.round_number, 1);
    }

    fn push_reshuffle_table() -> GameState {
        GameState::new(
            vec![Player::new(1, 0), Player::new(2, 1)],
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

    fn assert_push_matches_draw_with(before: &GameState, after: &GameState, seed: u64) {
        let mut deck = before.deck.clone();
        let mut rng = StdRng::seed_from_u64(seed);
        let discarded = deck.discard.pop().expect("push needs a top card");
        let penalty = one_from_draw_with(&mut deck, &mut rng);
        let start = one_from_draw_with(&mut deck, &mut rng);
        let mut next_hand = before.players[1].hand.clone();
        next_hand.push(discarded);
        next_hand.push(penalty);
        let mut actor_hand = before.players[0].hand.clone();
        actor_hand.push(start);
        assert_eq!(after.players[1].hand, next_hand);
        assert_eq!(after.players[0].hand, actor_hand);
        assert_eq!(after.deck, deck);
        assert_eq!(after.drawn_card_id, Some(start.id));
    }

    fn assert_penalty_keeps_drawn_prefix(
        before: &GameState,
        after: &GameState,
        held: Card,
        seed: u64,
        safe: Card,
    ) {
        let drawn = draw_n(&before.deck, seed, 4);
        assert_eq!(
            after.players[0].hand,
            vec![held, drawn[0], drawn[1], drawn[2]]
        );
        assert_eq!(after.deck.discard, vec![drawn[3]]);
        assert_eq!(drawn[3], safe);
        assert!(after.deck.cards.is_empty());
    }

    fn assert_last_two_split(
        before: &GameState,
        after: &GameState,
        held: Card,
        bystander: Card,
        seed: u64,
    ) {
        let (current, next) = last_two(&before.deck, seed);
        let mut next_hand = before.players[1].hand.clone();
        next_hand.push(next);
        assert_eq!(after.players[0].hand, vec![held]);
        assert_eq!(after.players[1].hand, next_hand);
        assert_eq!(after.players[2].hand, vec![bystander]);
        assert_eq!(after.deck.discard, vec![current]);
        assert!(after.deck.cards.is_empty());
    }

    fn one_from_draw_with(deck: &mut Deck, rng: &mut StdRng) -> Card {
        match deck.draw_with(rng) {
            TurnDraw::One(card) | TurnDraw::LastCard(card) => card,
            other => panic!("expected one card, got {other:?}"),
        }
    }

    fn draw_n(deck: &Deck, seed: u64, count: usize) -> Vec<Card> {
        let mut deck = deck.clone();
        let mut rng = StdRng::seed_from_u64(seed);
        let mut drawn = Vec::new();
        for _ in 0..count {
            drawn.push(one_from_draw_with(&mut deck, &mut rng));
        }
        drawn
    }

    fn last_two(deck: &Deck, seed: u64) -> (Card, Card) {
        let mut deck = deck.clone();
        match deck.draw_with(&mut StdRng::seed_from_u64(seed)) {
            TurnDraw::LastTwo { current, next } => (current, next),
            other => panic!("expected the last two cards, got {other:?}"),
        }
    }

    fn seeds_whose_draws_differ(deck: &Deck, count: usize) -> (u64, u64) {
        let first = draw_n(deck, 0, count);
        for seed in 1..64 {
            if draw_n(deck, seed, count) != first {
                return (0, seed);
            }
        }
        panic!("expected two seeds to draw in a different order");
    }

    fn seeds_whose_last_two_differ(deck: &Deck) -> (u64, u64) {
        let first = last_two(deck, 0);
        for seed in 1..64 {
            if last_two(deck, seed) != first {
                return (0, seed);
            }
        }
        panic!("expected two seeds to order the last two cards differently");
    }

    struct CountingRng<R> {
        inner: R,
        calls: usize,
    }

    impl<R> CountingRng<R> {
        fn new(inner: R) -> Self {
            Self { inner, calls: 0 }
        }
    }

    impl<R: RngCore> RngCore for CountingRng<R> {
        fn next_u32(&mut self) -> u32 {
            self.calls += 1;
            self.inner.next_u32()
        }

        fn next_u64(&mut self) -> u64 {
            self.calls += 1;
            self.inner.next_u64()
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            self.calls += 1;
            self.inner.fill_bytes(dest);
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.calls += 1;
            self.inner.try_fill_bytes(dest)
        }
    }
}
