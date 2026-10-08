//! The table: seats, piles, the round, and face-up melds.

use rand::Rng;

use crate::card::Card;
use crate::deck::Deck;
use crate::player::{deal_initial_hands, Player};

/// Where a turn is. Penalty drawing continues until a safe card is discarded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnPhase {
    /// The player may play, hit, steal, and discard.
    Playing,
    /// An off-board discard was refused because the card fits a meld.
    PenaltyDrawing,
}

/// Seats, piles, the round, and the face-up melds.
///
/// A new table is round 1 and the board is empty. `PlayMeld` reads `round_number`
/// and moves cards from one hand onto `board`. `HitMeld` adds cards from the
/// hand onto one or more of those melds, and only for a player who is on the board.
/// `StealWild` swaps a natural card for a wild on one of those melds. The stolen
/// wild locks until `turn_counter` plus one. A card whose lock is still ahead of
/// that counter cannot be played, hit, or used to replace a wild. A new table
/// starts that counter at 0. `drawn_card_id` is the card this turn's take or
/// push just put into the actor's hand. That card may be discarded even when
/// it fits a meld. `advance_turn` clears it and returns the phase to playing.
/// A new table starts in [`TurnPhase::Playing`]. `penalty_seat` is the off-board
/// player whose discard of a playable card opened that phase. Another seat's
/// discard does not close it, and only that seat may draw. `round_over` starts false.
/// A play, a hit, or a discard that leaves the actor's hand empty sets it.
/// `advance_to_next_round` then adds each remaining hand onto `total_score` and deals again.
#[derive(Clone, Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub deck: Deck,
    pub round_number: u8,
    pub board: Vec<Vec<Card>>,
    pub turn_counter: u32,
    pub drawn_card_id: Option<u32>,
    pub turn_phase: TurnPhase,
    /// The seat that must draw until it can discard. `None` while the phase is playing.
    pub penalty_seat: Option<usize>,
    pub round_over: bool,
}

impl GameState {
    pub fn new(players: Vec<Player>, deck: Deck) -> Self {
        Self {
            players,
            deck,
            round_number: 1,
            board: Vec::new(),
            turn_counter: 0,
            drawn_card_id: None,
            turn_phase: TurnPhase::Playing,
            penalty_seat: None,
            round_over: false,
        }
    }

    /// Moves the table on by one turn.
    ///
    /// Hands, the board, the piles, the round, and both scores stay as they were.
    /// A card locked until the old counter plus one can be played after this.
    /// The card drawn on the turn that just ended is no longer exempt from the
    /// safe-discard rule. A penalty draw does not carry into the next turn.
    pub fn advance_turn(&mut self) {
        self.turn_counter = self.turn_counter.saturating_add(1);
        self.drawn_card_id = None;
        self.turn_phase = TurnPhase::Playing;
        self.penalty_seat = None;
    }

    /// Scores the hands and deals the next round. `rng` shuffles the full shoe.
    ///
    /// Returns false while the round is still open. An empty hand does not start
    /// the next round until `round_over` is set. The table stays as it was.
    ///
    /// The player who went out adds 0. Every card still in a hand is added onto
    /// that seat's `total_score`. `points` stays as it was. A card on the board,
    /// in the draw pile, or on the discard pile is not scored. A locked card in
    /// a hand still counts.
    ///
    /// The shoe is the hands in seat order, then each meld, then the draw pile,
    /// then the discard pile. Every lock is cleared. That shoe is shuffled, dealt
    /// ten cards at a time, and the next card starts the discard pile. The board
    /// is empty. Nobody is on the board. The turn counter is 0. The phase is
    /// playing. `round_over` is false. `round_number` is one higher.
    pub fn advance_to_next_round_with(&mut self, rng: &mut impl Rng) -> bool {
        if !self.round_over {
            return false;
        }
        for player in &mut self.players {
            player.add_hand_penalty_to_total();
        }
        let mut shoe = Vec::new();
        for player in &mut self.players {
            shoe.append(&mut player.hand);
            player.is_on_board = false;
        }
        for meld in &mut self.board {
            shoe.append(meld);
        }
        self.board.clear();
        shoe.append(&mut self.deck.cards);
        shoe.append(&mut self.deck.discard);
        for card in &mut shoe {
            card.locked_until_turn = 0;
        }
        self.deck.cards = shoe;
        self.deck.shuffle_with(rng);
        deal_initial_hands(&mut self.players, &mut self.deck);
        let starter = self
            .deck
            .cards
            .pop()
            .expect("the next round starts a discard pile");
        self.deck.discard.push(starter);
        self.round_number = self.round_number.saturating_add(1);
        self.turn_counter = 0;
        self.drawn_card_id = None;
        self.turn_phase = TurnPhase::Playing;
        self.penalty_seat = None;
        self.round_over = false;
        true
    }

    /// Scores the hands and deals the next round.
    ///
    /// Same table as [`Self::advance_to_next_round_with`], shuffled with the thread rng.
    pub fn advance_to_next_round(&mut self) -> bool {
        self.advance_to_next_round_with(&mut rand::thread_rng())
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::*;
    use crate::card::{Card, Rank, Suit};
    use crate::deck::Deck;
    use crate::player::{deal_initial_hands, Player};

    fn take_suited(cards: &mut Vec<Card>, suit: Suit, rank: Rank) -> Card {
        let index = cards
            .iter()
            .position(|card| card.suit == suit && card.rank == rank)
            .expect("the deck contains this suit and rank");
        cards.remove(index)
    }

    fn with_lock(mut card: Card, locked_until_turn: u32) -> Card {
        card.locked_until_turn = locked_until_turn;
        card
    }

    /// Hands in seat order, then each meld, then the draw pile, then the discard pile.
    /// Locks are cleared before the shuffle that deals the next round.
    fn next_round_deal(state: &GameState, seed: u64) -> (Vec<Vec<Card>>, Vec<Card>, Card) {
        let mut shoe = Vec::new();
        for player in &state.players {
            shoe.extend(player.hand.iter().copied());
        }
        for meld in &state.board {
            shoe.extend(meld.iter().copied());
        }
        shoe.extend(state.deck.cards.iter().copied());
        shoe.extend(state.deck.discard.iter().copied());
        for card in &mut shoe {
            card.locked_until_turn = 0;
        }
        let mut deck = Deck {
            cards: shoe,
            discard: Vec::new(),
        };
        deck.shuffle_with(&mut StdRng::seed_from_u64(seed));
        let mut players: Vec<Player> = (0..state.players.len())
            .map(|seat| Player::new(seat as u32, seat as u32))
            .collect();
        deal_initial_hands(&mut players, &mut deck);
        let starter = deck
            .cards
            .pop()
            .expect("the next round starts a discard pile");
        let hands = players.into_iter().map(|player| player.hand).collect();
        (hands, deck.cards, starter)
    }

    fn cards_of(state: &GameState) -> Vec<Card> {
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

    fn assert_same_table(left: &GameState, right: &GameState) {
        assert_eq!(left.players, right.players);
        assert_eq!(left.deck, right.deck);
        assert_eq!(left.board, right.board);
        assert_eq!(left.round_number, right.round_number);
        assert_eq!(left.turn_counter, right.turn_counter);
        assert_eq!(left.drawn_card_id, right.drawn_card_id);
        assert_eq!(left.turn_phase, right.turn_phase);
        assert_eq!(left.penalty_seat, right.penalty_seat);
        assert_eq!(left.round_over, right.round_over);
    }

    /// Seat 0 went out. Seat 1 still holds 4, Jack, Ace, and a locked joker (50).
    /// A locked two sits on the board. A locked king sits on the discard. Neither is scored.
    fn finished_round() -> (GameState, u32, u32) {
        let mut deck = Deck::new();
        let four = take_suited(&mut deck.cards, Suit::Hearts, Rank::Four);
        let jack = take_suited(&mut deck.cards, Suit::Spades, Rank::Jack);
        let ace = take_suited(&mut deck.cards, Suit::Diamonds, Rank::Ace);
        let joker = with_lock(take_suited(&mut deck.cards, Suit::None, Rank::Joker), 3);
        let eight_spades = take_suited(&mut deck.cards, Suit::Spades, Rank::Eight);
        let eight_clubs = take_suited(&mut deck.cards, Suit::Clubs, Rank::Eight);
        let two = with_lock(take_suited(&mut deck.cards, Suit::Hearts, Rank::Two), 8);
        let king = with_lock(take_suited(&mut deck.cards, Suit::Diamonds, Rank::King), 2);
        deck.discard = vec![king];
        let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
        players[0].points = 4;
        players[1].points = 4;
        players[0].total_score = 9;
        players[1].total_score = 9;
        players[0].is_on_board = true;
        players[1].is_on_board = true;
        players[1].hand = vec![four, jack, ace, joker];
        let mut state = GameState::new(players, deck);
        state.board = vec![vec![eight_spades, eight_clubs, two]];
        state.round_over = true;
        state.turn_counter = 6;
        state.turn_phase = TurnPhase::PenaltyDrawing;
        state.penalty_seat = Some(1);
        state.drawn_card_id = Some(four.id);
        state.round_number = 1;
        assert_eq!(state.players[1].calculate_hand_penalty(), 50);
        assert_eq!(state.players[0].calculate_hand_penalty(), 0);
        assert_eq!(two.get_penalty_value(), 20);
        assert_eq!(king.get_penalty_value(), 10);
        (state, joker.id, two.id)
    }

    #[test]
    fn test_round_transition_refuses_while_the_round_is_open() {
        let (mut state, _, _) = finished_round();
        state.round_over = false;
        let before = state.clone();

        assert!(!state.advance_to_next_round_with(&mut StdRng::seed_from_u64(11)));
        assert!(!state.advance_to_next_round());
        assert_same_table(&state, &before);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].total_score, 9);
        assert!(state.players[0].hand.is_empty());
        assert_eq!(state.players[1].hand[3].locked_until_turn, 3);
    }

    #[test]
    fn test_round_transition_and_wipe() {
        let (mut state, joker_id, two_id) = finished_round();
        let original = cards_of(&state);
        assert_eq!(original.len(), 108);
        let (hands, draw, starter) = next_round_deal(&state, 11);

        assert!(state.advance_to_next_round_with(&mut StdRng::seed_from_u64(11)));

        assert_eq!(state.players[0].hand, hands[0]);
        assert_eq!(state.players[1].hand, hands[1]);
        assert_eq!(state.deck.cards, draw);
        assert_eq!(state.deck.discard, vec![starter]);
        assert!(state.board.is_empty());
        assert_eq!(state.players[0].hand.len(), 10);
        assert_eq!(state.players[1].hand.len(), 10);
        assert_eq!(state.deck.cards.len(), 87);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].total_score, 59);
        assert_eq!(state.players[0].id, 1);
        assert_eq!(state.players[1].id, 2);
        assert_eq!(state.players[0].seat_index, 0);
        assert_eq!(state.players[1].seat_index, 1);
        assert!(!state.players[0].is_on_board);
        assert!(!state.players[1].is_on_board);
        assert_eq!(state.round_number, 2);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.drawn_card_id, None);
        let dealt = cards_of(&state);
        assert!(dealt.iter().all(|card| card.locked_until_turn == 0));
        assert_eq!(
            dealt
                .iter()
                .find(|card| card.id == joker_id)
                .map(|card| card.locked_until_turn),
            Some(0)
        );
        assert_eq!(
            dealt
                .iter()
                .find(|card| card.id == two_id)
                .map(|card| card.locked_until_turn),
            Some(0)
        );
        let mut dealt_ids: Vec<u32> = dealt.iter().map(|card| card.id).collect();
        let mut original_ids: Vec<u32> = original.iter().map(|card| card.id).collect();
        dealt_ids.sort_unstable();
        original_ids.sort_unstable();
        assert_eq!(dealt_ids, original_ids);

        let dealt_table = state.clone();
        assert!(!state.advance_to_next_round_with(&mut StdRng::seed_from_u64(99)));
        assert_same_table(&state, &dealt_table);
    }

    #[test]
    fn test_round_transition_adds_the_next_hands_onto_the_totals() {
        let (mut state, _, _) = finished_round();
        assert!(state.advance_to_next_round_with(&mut StdRng::seed_from_u64(11)));
        let first_totals = (state.players[0].total_score, state.players[1].total_score);
        let next_penalties = (
            state.players[0].calculate_hand_penalty(),
            state.players[1].calculate_hand_penalty(),
        );
        assert_ne!(next_penalties, (0, 0));
        state.round_over = true;
        let (hands, draw, starter) = next_round_deal(&state, 12);

        assert!(state.advance_to_next_round_with(&mut StdRng::seed_from_u64(12)));

        assert_eq!(state.players[0].hand, hands[0]);
        assert_eq!(state.players[1].hand, hands[1]);
        assert_eq!(state.deck.cards, draw);
        assert_eq!(state.deck.discard, vec![starter]);
        assert!(state.board.is_empty());
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(
            state.players[0].total_score,
            first_totals.0 + next_penalties.0
        );
        assert_eq!(
            state.players[1].total_score,
            first_totals.1 + next_penalties.1
        );
        assert_eq!(state.round_number, 3);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert!(cards_of(&state)
            .iter()
            .all(|card| card.locked_until_turn == 0));
    }

    #[test]
    fn test_round_transition_three_seats_from_round_five() {
        let mut deck = Deck::new();
        let ace = take_suited(&mut deck.cards, Suit::Hearts, Rank::Ace);
        let two = with_lock(take_suited(&mut deck.cards, Suit::Spades, Rank::Two), 4);
        let king = with_lock(take_suited(&mut deck.cards, Suit::Clubs, Rank::King), 1);
        let jack = take_suited(&mut deck.cards, Suit::Hearts, Rank::Jack);
        deck.discard = vec![jack];
        let mut players = vec![Player::new(1, 0), Player::new(2, 1), Player::new(3, 2)];
        players[0].points = 4;
        players[1].points = 4;
        players[2].points = 7;
        players[0].total_score = 9;
        players[1].total_score = 9;
        players[2].total_score = 2;
        players[0].is_on_board = true;
        players[1].is_on_board = true;
        players[2].is_on_board = false;
        players[1].hand = vec![ace];
        players[2].hand = vec![two];
        let mut state = GameState::new(players, deck);
        state.board = vec![vec![king]];
        state.round_over = true;
        state.round_number = 5;
        state.turn_counter = 12;
        state.turn_phase = TurnPhase::PenaltyDrawing;
        state.penalty_seat = Some(2);
        state.drawn_card_id = Some(ace.id);
        assert_eq!(state.players[1].calculate_hand_penalty(), 15);
        assert_eq!(state.players[2].calculate_hand_penalty(), 20);
        assert_eq!(king.get_penalty_value(), 10);
        assert_eq!(jack.get_penalty_value(), 10);
        let (hands, draw, starter) = next_round_deal(&state, 11);

        assert!(state.advance_to_next_round_with(&mut StdRng::seed_from_u64(11)));

        assert_eq!(state.players[0].hand, hands[0]);
        assert_eq!(state.players[1].hand, hands[1]);
        assert_eq!(state.players[2].hand, hands[2]);
        assert_eq!(state.deck.cards, draw);
        assert_eq!(state.deck.discard, vec![starter]);
        assert!(state.board.is_empty());
        assert!(state.players.iter().all(|player| player.hand.len() == 10));
        assert_eq!(state.deck.cards.len(), 77);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[1].points, 4);
        assert_eq!(state.players[2].points, 7);
        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].total_score, 24);
        assert_eq!(state.players[2].total_score, 22);
        assert_eq!(state.players[2].id, 3);
        assert_eq!(state.players[2].seat_index, 2);
        assert!(state.players.iter().all(|player| !player.is_on_board));
        assert_eq!(state.round_number, 6);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.drawn_card_id, None);
        assert_eq!(two.locked_until_turn, 4);
        assert!(cards_of(&state)
            .iter()
            .all(|card| card.locked_until_turn == 0));
        assert_eq!(cards_of(&state).len(), 108);
    }

    #[test]
    fn test_advance_to_next_round_deals_when_the_round_is_over() {
        let (mut state, joker_id, _) = finished_round();
        let before = cards_of(&state);

        assert!(state.advance_to_next_round());

        assert_eq!(state.players[0].total_score, 9);
        assert_eq!(state.players[1].total_score, 59);
        assert_eq!(state.players[0].points, 4);
        assert_eq!(state.players[1].points, 4);
        assert!(state.board.is_empty());
        assert_eq!(state.players[0].hand.len(), 10);
        assert_eq!(state.players[1].hand.len(), 10);
        assert_eq!(state.deck.discard.len(), 1);
        assert_eq!(state.deck.cards.len(), 87);
        assert_eq!(state.round_number, 2);
        assert!(!state.round_over);
        assert_eq!(state.turn_counter, 0);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
        assert_eq!(state.penalty_seat, None);
        assert_eq!(state.drawn_card_id, None);
        assert!(state.players.iter().all(|player| !player.is_on_board));
        let dealt = cards_of(&state);
        assert!(dealt.iter().all(|card| card.locked_until_turn == 0));
        assert!(dealt.iter().any(|card| card.id == joker_id));
        let mut dealt_ids: Vec<u32> = dealt.iter().map(|card| card.id).collect();
        let mut before_ids: Vec<u32> = before.iter().map(|card| card.id).collect();
        dealt_ids.sort_unstable();
        before_ids.sort_unstable();
        assert_eq!(dealt_ids, before_ids);
    }
}
