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

    /// Penalty points for one card. Ranks 3–9 score 5, a 10 through King scores 10,
    /// an ace scores 15, and a two or a joker scores 20.
    pub fn get_penalty_value(&self) -> u32 {
        match self.rank {
            Rank::Three
            | Rank::Four
            | Rank::Five
            | Rank::Six
            | Rank::Seven
            | Rank::Eight
            | Rank::Nine => 5,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
            Rank::Ace => 15,
            Rank::Two | Rank::Joker => 20,
        }
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

    #[test]
    fn test_score_card_pip() {
        let cases = [
            (Rank::Three, Suit::Hearts, 1, 0),
            (Rank::Four, Suit::Diamonds, 2, 1),
            (Rank::Five, Suit::Clubs, 3, 2),
            (Rank::Six, Suit::Spades, 4, 3),
            (Rank::Seven, Suit::Hearts, 11, 0),
            (Rank::Eight, Suit::Diamonds, 12, 4),
            (Rank::Nine, Suit::Clubs, 13, 5),
        ];

        for (rank, suit, id, locked_until_turn) in cases {
            let card = Card {
                id,
                suit,
                rank,
                locked_until_turn,
            };
            assert_eq!(card.get_penalty_value(), 5);
        }
    }

    #[test]
    fn test_score_card_face() {
        let ranks = [Rank::Ten, Rank::Jack, Rank::Queen, Rank::King];
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let mut id = 20u32;

        for (rank_index, rank) in ranks.iter().copied().enumerate() {
            for (suit_index, suit) in suits.iter().copied().enumerate() {
                let locked_until_turn = (rank_index + suit_index) as u32;
                let card = Card {
                    id,
                    suit,
                    rank,
                    locked_until_turn,
                };

                assert_eq!(card.get_penalty_value(), 10);
                assert_eq!(card.get_penalty_value(), 10);
                assert_eq!(card.id, id);
                assert_eq!(card.suit, suit);
                assert_eq!(card.rank, rank);
                assert_eq!(card.locked_until_turn, locked_until_turn);
                id += 1;
            }
        }
    }

    #[test]
    fn test_score_card_ace() {
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];

        for (index, suit) in suits.iter().copied().enumerate() {
            for copy in 0..2u32 {
                let id = 100 + (index as u32) * 2 + copy;
                let locked_until_turn = copy + 1;
                let card = Card {
                    id,
                    suit,
                    rank: Rank::Ace,
                    locked_until_turn,
                };

                assert_eq!(card.get_penalty_value(), 15);
                assert_eq!(card.get_penalty_value(), 15);
                assert_eq!(card.id, id);
                assert_eq!(card.suit, suit);
                assert_eq!(card.rank, Rank::Ace);
                assert_eq!(card.locked_until_turn, locked_until_turn);
            }
        }
    }

    #[test]
    fn test_score_card_wild() {
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let mut id = 200u32;

        for (index, suit) in suits.iter().copied().enumerate() {
            for copy in 0..2u32 {
                let locked_until_turn = (index as u32) + copy;
                let card = Card {
                    id,
                    suit,
                    rank: Rank::Two,
                    locked_until_turn,
                };

                assert_eq!(card.get_penalty_value(), 20);
                assert_eq!(card.get_penalty_value(), 20);
                assert!(card.is_wild());
                assert_eq!(card.id, id);
                assert_eq!(card.suit, suit);
                assert_eq!(card.rank, Rank::Two);
                assert_eq!(card.locked_until_turn, locked_until_turn);
                id += 1;
            }
        }

        let joker_suits = [Suit::None, Suit::Hearts, Suit::Diamonds, Suit::Spades];
        for (index, suit) in joker_suits.iter().copied().enumerate() {
            let locked_until_turn = index as u32 + 3;
            let card = Card {
                id,
                suit,
                rank: Rank::Joker,
                locked_until_turn,
            };

            assert_eq!(card.get_penalty_value(), 20);
            assert_eq!(card.get_penalty_value(), 20);
            assert!(card.is_wild());
            assert_eq!(card.id, id);
            assert_eq!(card.suit, suit);
            assert_eq!(card.rank, Rank::Joker);
            assert_eq!(card.locked_until_turn, locked_until_turn);
            id += 1;
        }
    }
}
