//! Action handlers: take, push, meld, hit, steal, discard (Epics 1.3–1.7).

use crate::card::Card;
use crate::deck::{Deck, TurnDraw};
use crate::game_state::GameState;
use crate::player::Player;
use crate::validation::{check_round_requirements, validate_run, validate_set};

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
}

/// Cards from the hand added onto one meld already on the board.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeldHit {
    pub meld_index: usize,
    pub cards: Vec<Card>,
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

/// Moves verified melds from the actor's hand onto the board.
fn play_meld(state: &mut GameState, actor_index: usize, melds: &[Vec<Card>]) -> bool {
    if state.players[actor_index].is_on_board {
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
    use crate::actions::{Action, MeldHit};
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
        state
    }

    fn assert_still(before: &GameState, after: &GameState) {
        assert_eq!(after.players, before.players);
        assert_eq!(after.board, before.board);
        assert_eq!(after.round_number, before.round_number);
        assert_eq!(after.deck, before.deck);
    }

    fn refuse(state: &GameState, actor: usize, melds: Vec<Vec<Card>>) {
        let mut next = state.clone();
        assert!(!next.apply(Action::PlayMeld(melds), actor));
        assert_still(state, &next);
    }

    /// Two sets of three, a second four of hearts that stays, and a lookalike four in the draw pile.
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
        let state = table(
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
}
