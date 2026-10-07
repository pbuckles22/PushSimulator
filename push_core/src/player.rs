//! Player hand, board flag, score (Epic 1.2).

use crate::card::Card;
use crate::deck::Deck;

/// One seat. A new player has no cards, is not on the board, and has 0 points and a 0 total score.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    pub id: u32,
    pub seat_index: u32,
    pub points: u32,
    pub total_score: u32,
    pub is_on_board: bool,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(id: u32, seat_index: u32) -> Self {
        Self {
            id,
            seat_index,
            points: 0,
            total_score: 0,
            is_on_board: false,
            hand: Vec::new(),
        }
    }

    /// Sum of each card's penalty. The hand and both scores stay as they are.
    pub fn calculate_hand_penalty(&self) -> u32 {
        self.hand.iter().map(|card| card.get_penalty_value()).sum()
    }

    /// Adds this hand's penalty onto the historical total. The hand and `points` stay as they are.
    pub fn add_hand_penalty_to_total(&mut self) {
        self.total_score += self.calculate_hand_penalty();
    }
}

/// Deals 10 cards to each player, one card at a time around the table.
///
/// `players` is the deal order. The first player receives the top card, the next player
/// receives the next card, and so on for 10 passes. The discard pile stays as it is.
/// Push is played with 2 or more players.
pub fn deal_initial_hands(players: &mut [Player], deck: &mut Deck) {
    assert!(players.len() >= 2, "Push is played with 2 or more players");
    for _pass in 0..10 {
        for player in players.iter_mut() {
            let card = deck
                .cards
                .pop()
                .expect("initial deal takes a card from the draw pile");
            player.hand.push(card);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Rank, Suit};
    use crate::deck::Deck;

    #[test]
    fn test_player_instantiation() {
        let player = Player::new(7, 1);
        assert_eq!(player.id, 7);
        assert_eq!(player.seat_index, 1);
        assert_eq!(player.points, 0);
        assert!(!player.is_on_board);
    }

    #[test]
    fn test_player_hand_starts_empty() {
        let player = Player::new(1, 0);
        assert!(player.hand.is_empty());
    }

    #[test]
    fn test_deal_initial_hand() {
        let mut deck = Deck::new();
        let before = deck.cards.len();
        let tops: Vec<Card> = deck.cards.iter().rev().take(20).copied().collect();
        let mut players = [Player::new(1, 0), Player::new(2, 1)];

        deal_initial_hands(&mut players, &mut deck);

        assert_eq!(players[0].hand.len(), 10);
        assert_eq!(players[1].hand.len(), 10);
        assert_eq!(deck.cards.len(), before - 20);
        for pass in 0..10 {
            assert_eq!(players[0].hand[pass], tops[pass * 2]);
            assert_eq!(players[1].hand[pass], tops[pass * 2 + 1]);
        }
        for (player, id, seat) in [(&players[0], 1, 0), (&players[1], 2, 1)] {
            assert_eq!(player.id, id);
            assert_eq!(player.seat_index, seat);
            assert_eq!(player.points, 0);
            assert!(!player.is_on_board);
        }
        assert!(deck.discard.is_empty());
        assert!(players.iter().all(|player| {
            player
                .hand
                .iter()
                .all(|card| deck.cards.iter().all(|left| left.id != card.id))
        }));
    }

    #[test]
    fn test_deal_initial_hand_three_players_one_card_at_a_time() {
        let mut deck = Deck::new();
        let tops: Vec<Card> = deck.cards.iter().rev().take(30).copied().collect();
        let mut players = [Player::new(1, 0), Player::new(2, 1), Player::new(3, 2)];

        deal_initial_hands(&mut players, &mut deck);

        assert_eq!(deck.cards.len(), 108 - 30);
        for index in 0..30 {
            assert_eq!(players[index % 3].hand[index / 3], tops[index]);
        }
    }

    #[test]
    fn test_deal_initial_hand_refuses_one_player() {
        let mut deck = Deck::new();
        let before = deck.cards.len();
        let mut players = [Player::new(1, 0)];
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            deal_initial_hands(&mut players, &mut deck);
        }));
        let message = panicked.expect_err("one player cannot be dealt a hand");
        let text = message
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| message.downcast_ref::<&str>().copied())
            .unwrap_or("");
        assert!(text.contains("2 or more players"), "{text}");
        assert_eq!(deck.cards.len(), before);
        assert!(deck.discard.is_empty());
        assert!(players[0].hand.is_empty());
        assert_eq!(players[0].points, 0);
        assert!(!players[0].is_on_board);
    }

    #[test]
    fn test_deal_initial_hand_short_draw_pile() {
        let mut deck = Deck::new();
        let pile: Vec<Card> = deck.cards.split_off(deck.cards.len() - 3);
        deck.cards = pile.clone();
        let mut players = [Player::new(1, 0), Player::new(2, 1)];

        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            deal_initial_hands(&mut players, &mut deck);
        }));

        let message = panicked.expect_err("a short draw pile cannot finish a 10-card deal");
        let text = message
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| message.downcast_ref::<&str>().copied())
            .unwrap_or("");
        assert!(
            text.contains("initial deal takes a card from the draw pile"),
            "{text}"
        );
        assert_eq!(players[0].hand, vec![pile[2], pile[0]]);
        assert_eq!(players[1].hand, vec![pile[1]]);
        assert!(deck.cards.is_empty());
        assert!(deck.discard.is_empty());
        assert_eq!(players[0].points, 0);
        assert_eq!(players[1].points, 0);
        assert_eq!(players[0].total_score, 0);
        assert_eq!(players[1].total_score, 0);
    }

    fn penalty_hand() -> Vec<Card> {
        vec![
            Card {
                id: 1,
                suit: Suit::Hearts,
                rank: Rank::Four,
                locked_until_turn: 3,
            },
            Card {
                id: 2,
                suit: Suit::Spades,
                rank: Rank::Jack,
                locked_until_turn: 0,
            },
            Card {
                id: 3,
                suit: Suit::Diamonds,
                rank: Rank::Ace,
                locked_until_turn: 7,
            },
            Card {
                id: 4,
                suit: Suit::None,
                rank: Rank::Joker,
                locked_until_turn: 1,
            },
        ]
    }

    #[test]
    fn test_calculate_hand_total() {
        let mut player = Player::new(1, 0);
        let hand = penalty_hand();
        player.hand = hand.clone();

        assert_eq!(player.calculate_hand_penalty(), 50);
        assert_eq!(player.calculate_hand_penalty(), 50);
        assert_eq!(player.hand, hand);
        assert_eq!(player.points, 0);
        assert_eq!(player.total_score, 0);
        assert!(!player.is_on_board);
    }

    #[test]
    fn test_add_to_total_score() {
        let mut player = Player::new(1, 0);
        let hand = penalty_hand();
        player.hand = hand.clone();
        assert_eq!(player.total_score, 0);
        assert_eq!(player.calculate_hand_penalty(), 50);

        player.add_hand_penalty_to_total();

        assert_eq!(player.total_score, 50);
        assert_eq!(player.points, 0);
        assert_eq!(player.hand, hand);
        assert!(!player.is_on_board);
        assert_eq!(player.id, 1);
        assert_eq!(player.seat_index, 0);
    }
}
