//! Action handlers: take, push, meld, hit, steal, discard (Epics 1.3–1.7).

use crate::card::{Card, Rank, Suit};
use crate::deck::{Deck, TurnDraw};
use crate::game_state::GameState;
use crate::player::Player;
use crate::validation::{card_can_be_played, check_round_requirements, validate_run, validate_set};

/// A player choice during a turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Move the discard pile's top card into the player's hand.
    TakeDiscard,
    /// Move the discard top to the next player, who also draws a penalty. The actor then draws.
    PushDiscard,
    /// Lay these melds down to get on the board for the current round.
    PlayMeld(Vec<Vec<Card>>),
    /// Add cards from the actor's hand onto melds already on the board.
    ///
    /// Each entry names one meld. One action can name several melds, and each of
    /// those melds receives only the cards listed for it.
    HitMeld(Vec<MeldHit>),
    /// Swap one natural card from the hand for a wild on a meld. The wild moves into the hand.
    StealWild(WildSteal),
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
            Self::PushDiscard => push_discard(players, actor_index, deck),
            Self::PlayMeld(_) => panic!("PlayMeld applies on GameState"),
            Self::HitMeld(_) => panic!("HitMeld applies on GameState"),
            Self::StealWild(_) => panic!("StealWild applies on GameState"),
        }
    }
}

impl GameState {
    /// Runs `action` for the player at `actor_index`.
    ///
    /// Take and push behave as [`Action::apply`]. A meld play returns false, and leaves
    /// the table as it was, when the round rejects the melds, a card is not in that hand,
    /// or that player is already on the board. A successful play appends the melds to
    /// `board`, removes those cards from the hand, and sets `is_on_board`.
    /// A steal returns false, and leaves the table as it was, when that player is off
    /// the board, the natural is still locked, or the natural is not the card the wild
    /// is standing in for. A successful
    /// steal puts the natural where the wild sat and moves the wild onto the end of the
    /// hand, locked until `turn_counter` plus one.
    /// A play or a hit returns false when a card in that action is still locked:
    /// `locked_until_turn` is ahead of `turn_counter`. The table stays as it was.
    pub fn apply(&mut self, action: Action, actor_index: usize) -> bool {
        match action {
            Action::TakeDiscard => {
                Action::TakeDiscard.apply(&mut self.players, actor_index, &mut self.deck);
                true
            }
            Action::PushDiscard => {
                Action::PushDiscard.apply(&mut self.players, actor_index, &mut self.deck);
                true
            }
            Action::PlayMeld(melds) => play_meld(self, actor_index, &melds),
            Action::HitMeld(hits) => hit_meld(self, actor_index, &hits),
            Action::StealWild(steal) => steal_wild(self, actor_index, &steal),
        }
    }
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
fn push_discard(players: &mut [Player], actor_index: usize, deck: &mut Deck) {
    assert!(players.len() >= 2, "Push is played with 2 or more players");
    let next_index = (actor_index + 1) % players.len();
    let discarded = deck.discard.pop().expect("push discard needs a top card");
    let penalty = draw_one(deck, "push penalty");
    let start = draw_one(deck, "push turn");
    players[next_index].hand.push(discarded);
    players[next_index].hand.push(penalty);
    players[actor_index].hand.push(start);
}

fn draw_one(deck: &mut Deck, why: &str) -> Card {
    match deck.draw() {
        TurnDraw::One(card) | TurnDraw::LastCard(card) => card,
        other => panic!("{why} expected one card, got {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::{Action, MeldHit, WildSteal};
    use crate::card::{Card, Rank, Suit};
    use crate::deck::Deck;
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

    use crate::game_state::GameState;

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
        state
    }

    fn assert_still(before: &GameState, after: &GameState) {
        assert_eq!(after.players, before.players);
        assert_eq!(after.board, before.board);
        assert_eq!(after.round_number, before.round_number);
        assert_eq!(after.deck, before.deck);
        assert_eq!(after.turn_counter, before.turn_counter);
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
        state.players[0].is_on_board = true;
        refuse(&state, 0, vec![fours, fives]);
        assert!(state.players[0].is_on_board);
        assert!(state.board.is_empty());
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
}
