//! Card primitives — Suit, Rank, Card (Epic 1.1).

/// Suit of a playing card. `None` is the suit of a joker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
    None,
}

/// Rank from two through ace, plus joker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Joker,
}

/// One card. `id` distinguishes copies of the same suit and rank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    pub id: u32,
    pub suit: Suit,
    pub rank: Rank,
    pub locked_until_turn: u32,
}

impl Card {
    /// Twos and jokers are wild.
    pub fn is_wild(&self) -> bool {
        self.rank == Rank::Joker || self.rank == Rank::Two
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suit_enum_instantiation() {
        let hearts = Suit::Hearts;
        let diamonds = Suit::Diamonds;
        let clubs = Suit::Clubs;
        let spades = Suit::Spades;
        let none = Suit::None;

        assert_eq!(hearts, Suit::Hearts);
        assert_ne!(hearts, diamonds);
        assert_ne!(diamonds, clubs);
        assert_ne!(clubs, spades);
        assert_ne!(spades, none);
    }

    #[test]
    fn test_rank_enum_instantiation() {
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
            Rank::Joker,
        ];

        assert_eq!(ranks.len(), 14);
        assert_eq!(ranks[0], Rank::Two);
        assert_eq!(ranks[13], Rank::Joker);
        assert_ne!(ranks[0], ranks[1]);
        assert_ne!(Rank::Ace, Rank::Joker);
    }

    #[test]
    fn test_card_struct_instantiation() {
        let card = Card {
            id: 7,
            suit: Suit::Hearts,
            rank: Rank::Ace,
            locked_until_turn: 3,
        };

        assert_eq!(card.id, 7);
        assert_eq!(card.suit, Suit::Hearts);
        assert_eq!(card.rank, Rank::Ace);
        assert_eq!(card.locked_until_turn, 3);
    }

    #[test]
    fn test_card_is_wild_true_for_joker() {
        let joker = Card {
            id: 1,
            suit: Suit::None,
            rank: Rank::Joker,
            locked_until_turn: 0,
        };

        assert!(joker.is_wild());
    }

    #[test]
    fn test_card_is_wild_true_for_two() {
        let two = Card {
            id: 2,
            suit: Suit::Hearts,
            rank: Rank::Two,
            locked_until_turn: 0,
        };

        assert!(two.is_wild());
    }

    #[test]
    fn test_card_is_wild_false_for_standard_card() {
        let three = Card {
            id: 3,
            suit: Suit::Hearts,
            rank: Rank::Three,
            locked_until_turn: 0,
        };

        assert!(!three.is_wild());
    }
}
