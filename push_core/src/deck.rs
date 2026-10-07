//! Deck generation, shuffle, draw (Epic 1.1).

use rand::seq::SliceRandom;
use rand::Rng;

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

    /// Reorders `cards` with `rng`. The same cards stay in the deck.
    pub fn shuffle_with(&mut self, rng: &mut impl Rng) {
        self.cards.shuffle(rng);
    }

    /// Reorders `cards` in place. The same cards stay in the deck.
    pub fn shuffle(&mut self) {
        self.shuffle_with(&mut rand::thread_rng());
    }

    /// Draws for the player whose turn it is, using `rng` when a pile is shuffled.
    ///
    /// A non-empty draw pile yields [`TurnDraw::One`]. An empty draw pile with three or more
    /// discard cards leaves the top card, shuffles the rest, and yields [`TurnDraw::One`].
    /// One leftover card yields [`TurnDraw::LastCard`]. Two leftover cards are shuffled and
    /// yielded as [`TurnDraw::LastTwo`]. Nothing left yields [`TurnDraw::Empty`].
    pub fn draw_with(&mut self, rng: &mut impl Rng) -> TurnDraw {
        if let Some(card) = self.cards.pop() {
            return TurnDraw::One(card);
        }
        match self.discard.len() {
            0 => TurnDraw::Empty,
            1 => TurnDraw::LastCard(self.discard.pop().expect("discard holds the last card")),
            2 => self.split_last_two(rng),
            _ => {
                self.reshuffle_discard(rng);
                TurnDraw::One(
                    self.cards
                        .pop()
                        .expect("a discard of three or more recycles at least two cards"),
                )
            }
        }
    }

    /// Draws for the player whose turn it is.
    ///
    /// A non-empty draw pile yields [`TurnDraw::One`]. An empty draw pile with three or more
    /// discard cards leaves the top card, shuffles the rest, and yields [`TurnDraw::One`].
    /// One leftover card yields [`TurnDraw::LastCard`]. Two leftover cards are shuffled and
    /// yielded as [`TurnDraw::LastTwo`]. Nothing left yields [`TurnDraw::Empty`].
    pub fn draw(&mut self) -> TurnDraw {
        self.draw_with(&mut rand::thread_rng())
    }

    /// Shuffles the last two discard cards and gives one to the current player and one to the next.
    fn split_last_two(&mut self, rng: &mut impl Rng) -> TurnDraw {
        self.cards.append(&mut self.discard);
        self.shuffle_with(rng);
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
    ///
    /// The caller has already required three or more discard cards.
    fn reshuffle_discard(&mut self, rng: &mut impl Rng) {
        let top = self
            .discard
            .pop()
            .expect("reshuffle runs when the discard holds three or more cards");
        self.cards.append(&mut self.discard);
        self.shuffle_with(rng);
        self.discard.push(top);
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

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
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);

        deck.shuffle_with(&mut rng);

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

    fn sample(id: u32, suit: Suit, rank: Rank) -> Card {
        Card {
            id,
            suit,
            rank,
            locked_until_turn: 0,
        }
    }

    fn ids(cards: &[Card]) -> Vec<u32> {
        cards.iter().map(|card| card.id).collect()
    }

    /// Same seed, same order. A different seed is a different order. The cards stay the same set.
    #[test]
    fn test_shuffle_with_seed_repeats_order() {
        let before = ids(&Deck::new().cards);
        let mut first = Deck::new();
        let mut second = Deck::new();
        let mut other = Deck::new();
        let mut rng_a = rand::rngs::StdRng::seed_from_u64(42);
        let mut rng_b = rand::rngs::StdRng::seed_from_u64(42);
        let mut rng_c = rand::rngs::StdRng::seed_from_u64(43);

        first.shuffle_with(&mut rng_a);
        second.shuffle_with(&mut rng_b);
        other.shuffle_with(&mut rng_c);

        let once = ids(&first.cards);
        assert_eq!(once, ids(&second.cards), "one seed repeats the order");
        assert_ne!(once, before, "this seed changes the order");
        assert_ne!(once, ids(&other.cards), "another seed is another order");
        assert_eq!(first.cards.len(), 108);
        let mut once_sorted = once.clone();
        let mut before_sorted = before.clone();
        once_sorted.sort_unstable();
        before_sorted.sort_unstable();
        assert_eq!(
            once_sorted, before_sorted,
            "a seeded shuffle keeps every card id"
        );
    }

    /// An empty discard stays empty. One discard card goes to the current player.
    /// Neither path puts cards back through a reshuffle.
    #[test]
    fn test_draw_with_empty_and_one_do_not_reshuffle() {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let mut deck = Deck {
            cards: Vec::new(),
            discard: Vec::new(),
        };

        assert_eq!(deck.draw_with(&mut rng), TurnDraw::Empty);
        assert!(deck.cards.is_empty());
        assert!(deck.discard.is_empty());

        let lone = sample(4, Suit::Hearts, Rank::Ace);
        deck.discard = vec![lone];
        assert_eq!(deck.draw_with(&mut rng), TurnDraw::LastCard(lone));
        assert!(deck.cards.is_empty());
        assert!(deck.discard.is_empty());
        assert_eq!(deck.draw_with(&mut rng), TurnDraw::Empty);
    }

    /// Exactly three discard cards yield one card from under the top. The top stays.
    /// The same seed draws the same card into the same remaining pile. The draw is not empty.
    #[test]
    fn test_draw_with_three_discard_follows_the_seed() {
        let under_a = sample(1, Suit::Hearts, Rank::Five);
        let under_b = sample(2, Suit::Spades, Rank::King);
        let top = sample(9, Suit::Diamonds, Rank::Ace);

        let play = |seed: u64| {
            let mut deck = Deck {
                cards: Vec::new(),
                discard: vec![under_a, under_b, top],
            };
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            let drawn = deck.draw_with(&mut rng);
            (drawn, ids(&deck.cards), deck.discard.clone())
        };

        let (drawn, pile, discard) = play(11);
        let TurnDraw::One(card) = drawn else {
            panic!("three discard cards yield one recycled card, got {drawn:?}");
        };
        assert_ne!(card.id, top.id);
        assert!(card.id == under_a.id || card.id == under_b.id);
        assert_eq!(discard, vec![top]);
        assert_eq!(pile.len(), 1);
        assert_ne!(pile[0], card.id);
        assert_ne!(pile[0], top.id);
        assert_eq!(play(11), (drawn, pile.clone(), discard.clone()));
    }

    /// The last two cards split the same way for one seed.
    #[test]
    fn test_draw_with_last_two_follows_the_seed() {
        let first = sample(1, Suit::Hearts, Rank::Five);
        let second = sample(2, Suit::Spades, Rank::King);
        let play = |seed: u64| {
            let mut deck = Deck {
                cards: Vec::new(),
                discard: vec![first, second],
            };
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            deck.draw_with(&mut rng)
        };

        let once = play(9);
        let TurnDraw::LastTwo { current, next } = once else {
            panic!("the last two cards are shuffled and split, got {once:?}");
        };
        let mut got = [current.id, next.id];
        got.sort_unstable();
        assert_eq!(got, [first.id, second.id]);
        assert_eq!(play(9), once);
    }

    /// Card ids survive a seeded shuffle, a full draw, and a reshuffle of three or more.
    /// Each seed repeats. An empty pile and a single leftover do not reshuffle.
    #[test]
    fn test_card_ids_survive_seeded_shuffle_draw_and_reshuffle() {
        let original = ids(&Deck::new().cards);

        let trace = |seed: u64| {
            let mut deck = Deck::new();
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            deck.shuffle_with(&mut rng);
            let shuffled = ids(&deck.cards);
            assert_ne!(shuffled, original, "seed {seed} changes the order");
            let mut shuffled_sorted = shuffled.clone();
            let mut original_sorted = original.clone();
            shuffled_sorted.sort_unstable();
            original_sorted.sort_unstable();
            assert_eq!(shuffled_sorted, original_sorted);

            let mut drawn = Vec::new();
            loop {
                match deck.draw_with(&mut rng) {
                    TurnDraw::One(card) => drawn.push(card),
                    TurnDraw::Empty => break,
                    other => panic!("a full draw pile drains one card at a time, got {other:?}"),
                }
            }
            assert_eq!(drawn.len(), original.len());
            assert!(deck.cards.is_empty());
            assert!(deck.discard.is_empty());
            assert_eq!(deck.draw_with(&mut rng), TurnDraw::Empty);

            let lone = drawn[0];
            deck.discard = vec![lone];
            assert_eq!(deck.draw_with(&mut rng), TurnDraw::LastCard(lone));
            assert!(deck.cards.is_empty());
            assert!(deck.discard.is_empty());

            let top = drawn[drawn.len() - 1];
            deck.discard = drawn;
            let recycled = match deck.draw_with(&mut rng) {
                TurnDraw::One(card) => card,
                other => panic!("three or more discard cards yield one card, got {other:?}"),
            };
            assert_ne!(recycled.id, top.id);
            assert_eq!(deck.discard, vec![top]);
            let mut left = ids(&deck.cards);
            left.push(recycled.id);
            left.push(top.id);
            left.sort_unstable();
            assert_eq!(left, original_sorted, "seed {seed} keeps every card id");

            (shuffled, recycled.id, ids(&deck.cards), top.id)
        };

        for seed in 0..24u64 {
            assert_eq!(trace(seed), trace(seed), "seed {seed} repeats");
        }
    }
}
