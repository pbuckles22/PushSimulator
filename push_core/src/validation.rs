//! Set/run validation and round requirements (Epics 1.4–1.5).

use crate::card::Card;

/// A set is three or more cards of one rank. Twos and jokers stand in for that rank.
///
/// Every natural card must share one rank. A group of only wilds is a set.
/// Two cards, or naturals of different ranks, are not.
pub fn validate_set(cards: &[Card]) -> bool {
    if cards.len() < 3 {
        return false;
    }

    let mut natural_rank = None;
    for card in cards {
        if card.is_wild() {
            continue;
        }
        match natural_rank {
            None => natural_rank = Some(card.rank),
            Some(rank) if card.rank == rank => {}
            Some(_) => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::card::{Card, Rank, Suit};
    use crate::validation::validate_set;

    fn card(id: u32, suit: Suit, rank: Rank) -> Card {
        Card {
            id,
            suit,
            rank,
            locked_until_turn: 0,
        }
    }

    #[test]
    fn test_validate_set_naturals() {
        let natural = [
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
        ];
        let mixed = [
            card(4, Suit::Hearts, Rank::Four),
            card(5, Suit::Spades, Rank::Four),
            card(6, Suit::Clubs, Rank::Five),
        ];

        let four_kind = [
            card(7, Suit::Hearts, Rank::Four),
            card(8, Suit::Spades, Rank::Four),
            card(9, Suit::Clubs, Rank::Four),
            card(10, Suit::Diamonds, Rank::Four),
        ];

        assert!(validate_set(&natural));
        assert!(!validate_set(&mixed));
        assert!(!validate_set(&natural[..2]));
        assert!(validate_set(&four_kind));
    }

    #[test]
    fn test_validate_set_wilds() {
        let with_wilds = [
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::None, Rank::Joker),
            card(3, Suit::Spades, Rank::Two),
        ];
        let all_jokers = [
            card(4, Suit::None, Rank::Joker),
            card(5, Suit::None, Rank::Joker),
            card(6, Suit::None, Rank::Joker),
        ];
        let conflict = [
            card(7, Suit::Hearts, Rank::Four),
            card(8, Suit::Clubs, Rank::Five),
            card(9, Suit::None, Rank::Joker),
        ];

        assert!(validate_set(&with_wilds));
        assert!(validate_set(&all_jokers));
        assert!(!validate_set(&conflict));
    }
}
