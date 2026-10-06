//! Deck generation, shuffle, draw (Epic 1.1).

use rand::seq::SliceRandom;

use crate::card::{Card, Rank, Suit};

/// What a draw gives the player whose turn it is.
///
/// Hands are not stored here. The caller places `current` on the current player and `next` on the next player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnDraw {
    /// The draw pile had a card, or a discard of three or more was recycled and one card was drawn.
    One(Card),
    /// The draw pile was empty and the discard held the only card left. It goes to the current player.
    LastCard(Card),
    /// The draw pile was empty and the discard held the last two cards, now shuffled.
    LastTwo { current: Card, next: Card },
    /// The draw pile and the discard are both empty.
    Empty,
}

/// A Push deck: two 52-card decks plus two jokers each.
///
/// `cards` is the draw pile; the last card is the top.
/// `discard` is the discard pile; the last card is the top.
/// An empty draw pile recycles a discard of three or more, leaving that top card.
/// One leftover discard card goes to the current player. Two leftover cards are shuffled
/// and split between the current player and the next player.
pub struct Deck {
    pub cards: Vec<Card>,
    pub discard: Vec<Card>,
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
        Self {
            cards,
            discard: Vec::new(),
        }
    }

    /// Reorders `cards` in place. The same cards stay in the deck.
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut rand::thread_rng());
    }

    /// Draws for the player whose turn it is.
    ///
    /// A non-empty draw pile yields [`TurnDraw::One`]. An empty draw pile with three or more
    /// discard cards leaves the top card, shuffles the rest, and yields [`TurnDraw::One`].
    /// One leftover card yields [`TurnDraw::LastCard`]. Two leftover cards are shuffled and
    /// yielded as [`TurnDraw::LastTwo`]. Nothing left yields [`TurnDraw::Empty`].
    pub fn draw(&mut self) -> TurnDraw {
        if let Some(card) = self.cards.pop() {
            return TurnDraw::One(card);
        }
        match self.discard.len() {
            0 => TurnDraw::Empty,
            1 => TurnDraw::LastCard(self.discard.pop().expect("discard holds the last card")),
            2 => self.split_last_two(),
            _ => {
                self.reshuffle_discard();
                match self.cards.pop() {
                    Some(card) => TurnDraw::One(card),
                    None => TurnDraw::Empty,
                }
            }
        }
    }

    /// Shuffles the last two discard cards and gives one to the current player and one to the next.
    fn split_last_two(&mut self) -> TurnDraw {
        self.cards.append(&mut self.discard);
        self.shuffle();
        let current = self
            .cards
            .pop()
            .expect("shuffled pair has a card for the current player");
        let next = self
            .cards
            .pop()
            .expect("shuffled pair has a card for the next player");
        TurnDraw::LastTwo { current, next }
    }

    /// Moves every discard card except the top into the draw pile and shuffles them.
    fn reshuffle_discard(&mut self) {
        let Some(top) = self.discard.pop() else {
            return;
        };
        if self.discard.is_empty() {
            self.discard.push(top);
            return;
        }
        self.cards.append(&mut self.discard);
        self.shuffle();
        self.discard.push(top);
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

        let TurnDraw::One(drawn) = deck.draw() else {
            panic!("a new deck has a card to draw");
        };

        assert_eq!(deck.cards.len(), 107);
        assert!(before.contains(&drawn.id));
        assert!(deck.cards.iter().all(|card| card.id != drawn.id));
    }

    #[test]
    fn test_deck_draw_empty() {
        let mut deck = Deck {
            cards: Vec::new(),
            discard: Vec::new(),
        };

        assert_eq!(deck.draw(), TurnDraw::Empty);
    }

    #[test]
    fn test_deck_draw_empty_reshuffles() {
        let under = [
            Card {
                id: 1,
                suit: Suit::Hearts,
                rank: Rank::Five,
                locked_until_turn: 0,
            },
            Card {
                id: 2,
                suit: Suit::Spades,
                rank: Rank::King,
                locked_until_turn: 0,
            },
            Card {
                id: 3,
                suit: Suit::Clubs,
                rank: Rank::Nine,
                locked_until_turn: 0,
            },
        ];
        let top = Card {
            id: 9,
            suit: Suit::Diamonds,
            rank: Rank::Ace,
            locked_until_turn: 0,
        };
        let mut discard = under.to_vec();
        discard.push(top);

        let mut deck = Deck {
            cards: Vec::new(),
            discard,
        };

        let TurnDraw::One(drawn) = deck.draw() else {
            panic!("empty draw recycles the discard pile minus its top card");
        };
        assert_ne!(drawn.id, top.id);
        assert!(under.iter().any(|card| card.id == drawn.id));

        assert_eq!(deck.discard.len(), 1);
        assert_eq!(deck.discard[0].id, top.id);

        let mut left: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();
        left.push(drawn.id);
        left.sort_unstable();
        let mut expect: Vec<u32> = under.iter().map(|card| card.id).collect();
        expect.sort_unstable();
        assert_eq!(left, expect);
        assert_eq!(deck.cards.len(), under.len() - 1);
    }

    #[test]
    fn test_deck_draw_last_card_goes_to_current() {
        let last = Card {
            id: 4,
            suit: Suit::Hearts,
            rank: Rank::Ace,
            locked_until_turn: 0,
        };
        let mut deck = Deck {
            cards: Vec::new(),
            discard: vec![last],
        };

        assert_eq!(deck.draw(), TurnDraw::LastCard(last));
        assert!(deck.cards.is_empty());
        assert!(deck.discard.is_empty());
        assert_eq!(deck.draw(), TurnDraw::Empty);
    }

    #[test]
    fn test_deck_draw_last_two_split() {
        let first = Card {
            id: 1,
            suit: Suit::Hearts,
            rank: Rank::Five,
            locked_until_turn: 0,
        };
        let second = Card {
            id: 2,
            suit: Suit::Spades,
            rank: Rank::King,
            locked_until_turn: 0,
        };
        let mut deck = Deck {
            cards: Vec::new(),
            discard: vec![first, second],
        };

        let TurnDraw::LastTwo { current, next } = deck.draw() else {
            panic!("the last two cards are shuffled and split");
        };
        let mut got = [current.id, next.id];
        got.sort_unstable();
        assert_eq!(got, [first.id, second.id]);
        assert!(deck.cards.is_empty());
        assert!(deck.discard.is_empty());
        assert_eq!(deck.draw(), TurnDraw::Empty);
    }
}
