//! Deck generation, shuffle, draw (Epic 1.1).

use rand::seq::SliceRandom;

use crate::card::{Card, Rank, Suit};

/// A Push deck: two 52-card decks plus two jokers each.
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let ranks = [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ];

        let mut cards = Vec::with_capacity(108);
        let mut id = 0u32;
        for _deck in 0..2 {
            for suit in suits {
                for rank in ranks {
                    cards.push(Card {
                        id,
                        suit,
                        rank,
                        locked_until_turn: 0,
                    });
                    id += 1;
                }
            }
            for _joker in 0..2 {
                cards.push(Card {
                    id,
                    suit: Suit::None,
                    rank: Rank::Joker,
                    locked_until_turn: 0,
                });
                id += 1;
            }
        }
        Self { cards }
    }

    /// Reorders `cards` in place. The same cards stay in the deck.
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::thread_rng());
    }

    /// Takes the top card. An empty deck returns `None`.
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_instantiation_count() {
        let deck = Deck::new();
        assert_eq!(deck.cards.len(), 108);
    }

    #[test]
    fn test_deck_contains_exact_wild_count() {
        let deck = Deck::new();
        let jokers = deck
            .cards
            .iter()
            .filter(|card| card.rank == Rank::Joker)
            .count();
        let twos = deck
            .cards
            .iter()
            .filter(|card| card.rank == Rank::Two)
            .count();

        assert_eq!(jokers, 2 * 2, "two decks, two jokers each");
        assert_eq!(twos, 2 * 4, "two decks, four twos each");
    }

    #[test]
    fn test_deck_shuffle() {
        let mut deck = Deck::new();
        let before: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();

        deck.shuffle();

        let after: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();
        assert_ne!(before, after, "shuffle changes card order");
        assert_eq!(deck.cards.len(), 108);

        let mut before_sorted = before.clone();
        let mut after_sorted = after.clone();
        before_sorted.sort_unstable();
        after_sorted.sort_unstable();
        assert_eq!(before_sorted, after_sorted, "shuffle keeps the same cards");
    }

    #[test]
    fn test_deck_draw_reduces_count() {
        let mut deck = Deck::new();
        let before: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();

        let drawn = deck.draw().expect("a new deck has a card to draw");

        assert_eq!(deck.cards.len(), 107);
        assert!(before.contains(&drawn.id));
        assert!(deck.cards.iter().all(|card| card.id != drawn.id));
    }

    #[test]
    fn test_deck_draw_empty() {
        let mut deck = Deck { cards: Vec::new() };

        assert!(deck.draw().is_none());
    }
}
