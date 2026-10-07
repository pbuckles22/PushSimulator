//! Set/run validation and round requirements (Epics 1.4–1.5).

use crate::card::{Card, Rank};

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

/// A run is four or more cards of one suit in rank order. Twos and jokers fill missing ranks.
///
/// An ace is low or high. The run does not wrap from the king through the ace to the low cards.
/// Four wilds are a run. Three cards, a gap, a repeated rank, or two suits are not.
pub fn validate_run(cards: &[Card]) -> bool {
    if cards.len() < 4 || cards.len() > 13 {
        return false;
    }

    let mut suit = None;
    let mut naturals = Vec::new();
    for card in cards {
        if card.is_wild() {
            continue;
        }
        match suit {
            None => suit = Some(card.suit),
            Some(expected) if card.suit == expected => {}
            Some(_) => return false,
        }
        if naturals.contains(&card.rank) {
            return false;
        }
        naturals.push(card.rank);
    }

    if naturals.is_empty() {
        return true;
    }

    fits(&naturals, cards.len(), true) || fits(&naturals, cards.len(), false)
}

/// Ace-high uses 2..=14. Ace-low uses 1..=13. A window that needs both ends is a wrap.
fn fits(naturals: &[Rank], total: usize, ace_high: bool) -> bool {
    let (floor, ceiling) = if ace_high { (2, 14) } else { (1, 13) };
    let mut min_rank = ceiling;
    let mut max_rank = floor;
    for rank in naturals {
        let value = rank_value(*rank, ace_high);
        if value < floor || value > ceiling {
            return false;
        }
        min_rank = min_rank.min(value);
        max_rank = max_rank.max(value);
    }

    let span = usize::from(max_rank - min_rank + 1);
    if span > total {
        return false;
    }
    let extra = total - span;
    let before = usize::from(min_rank - floor);
    let after = usize::from(ceiling - max_rank);
    extra <= before + after
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

#[cfg(test)]
mod tests {
    use crate::card::{Card, Rank, Suit};
    use crate::validation::{validate_run, validate_set};

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

    #[test]
    fn test_validate_run_naturals() {
        let natural = [
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Hearts, Rank::Five),
            card(3, Suit::Hearts, Rank::Six),
            card(4, Suit::Hearts, Rank::Seven),
        ];
        let mixed_suits = [
            card(5, Suit::Hearts, Rank::Four),
            card(6, Suit::Hearts, Rank::Five),
            card(7, Suit::Hearts, Rank::Six),
            card(8, Suit::Diamonds, Rank::Seven),
        ];
        let too_short = [
            card(9, Suit::Hearts, Rank::Four),
            card(10, Suit::Hearts, Rank::Five),
            card(11, Suit::Hearts, Rank::Six),
        ];
        let gap = [
            card(12, Suit::Hearts, Rank::Four),
            card(13, Suit::Hearts, Rank::Five),
            card(14, Suit::Hearts, Rank::Seven),
            card(15, Suit::Hearts, Rank::Eight),
        ];
        let five = [
            card(16, Suit::Hearts, Rank::Three),
            card(17, Suit::Hearts, Rank::Four),
            card(18, Suit::Hearts, Rank::Five),
            card(19, Suit::Hearts, Rank::Six),
            card(20, Suit::Hearts, Rank::Seven),
        ];
        let seven = [
            card(21, Suit::Hearts, Rank::Three),
            card(22, Suit::Hearts, Rank::Four),
            card(23, Suit::Hearts, Rank::Five),
            card(24, Suit::Hearts, Rank::Six),
            card(25, Suit::Hearts, Rank::Seven),
            card(26, Suit::Hearts, Rank::Eight),
            card(27, Suit::Hearts, Rank::Nine),
        ];
        let unsorted = [
            card(28, Suit::Hearts, Rank::Seven),
            card(29, Suit::Hearts, Rank::Five),
            card(30, Suit::Hearts, Rank::Four),
            card(31, Suit::Hearts, Rank::Six),
        ];
        let duplicate_rank = [
            card(32, Suit::Hearts, Rank::Four),
            card(33, Suit::Hearts, Rank::Four),
            card(34, Suit::Hearts, Rank::Five),
            card(35, Suit::Hearts, Rank::Six),
        ];

        assert!(validate_run(&natural));
        assert!(!validate_run(&mixed_suits));
        assert!(!validate_run(&too_short));
        assert!(!validate_run(&gap));
        assert!(validate_run(&five));
        assert!(validate_run(&seven));
        assert!(validate_run(&unsorted));
        assert!(!validate_run(&duplicate_rank));
    }

    #[test]
    fn test_validate_run_wilds() {
        let with_wilds = [
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::None, Rank::Joker),
            card(3, Suit::Hearts, Rank::Six),
            card(4, Suit::Spades, Rank::Two),
        ];
        let two_holes = [
            card(5, Suit::Hearts, Rank::Four),
            card(6, Suit::Hearts, Rank::Six),
            card(7, Suit::Hearts, Rank::Eight),
            card(8, Suit::None, Rank::Joker),
        ];
        let two_wilds_fill_two_holes = [
            card(9, Suit::Hearts, Rank::Four),
            card(10, Suit::None, Rank::Joker),
            card(11, Suit::None, Rank::Joker),
            card(12, Suit::Hearts, Rank::Seven),
        ];
        let wild_does_not_fix_suit = [
            card(13, Suit::Hearts, Rank::Four),
            card(14, Suit::Diamonds, Rank::Five),
            card(15, Suit::Hearts, Rank::Six),
            card(16, Suit::None, Rank::Joker),
        ];
        let all_jokers = [
            card(17, Suit::None, Rank::Joker),
            card(18, Suit::None, Rank::Joker),
            card(19, Suit::None, Rank::Joker),
            card(20, Suit::None, Rank::Joker),
        ];
        let three_jokers = [
            card(21, Suit::None, Rank::Joker),
            card(22, Suit::None, Rank::Joker),
            card(23, Suit::None, Rank::Joker),
        ];

        assert!(validate_run(&with_wilds));
        assert!(!validate_run(&two_holes));
        assert!(validate_run(&two_wilds_fill_two_holes));
        assert!(!validate_run(&wild_does_not_fix_suit));
        assert!(validate_run(&all_jokers));
        assert!(!validate_run(&three_jokers));
    }

    #[test]
    fn test_ace_placement_low() {
        let low = [
            card(1, Suit::Hearts, Rank::Ace),
            card(2, Suit::Clubs, Rank::Two),
            card(3, Suit::Hearts, Rank::Three),
            card(4, Suit::Hearts, Rank::Four),
        ];
        let hole_where_the_deuce_goes = [
            card(5, Suit::Hearts, Rank::Ace),
            card(6, Suit::Hearts, Rank::Three),
            card(7, Suit::Hearts, Rank::Four),
            card(8, Suit::Hearts, Rank::Five),
        ];
        let five = [
            card(9, Suit::Hearts, Rank::Ace),
            card(10, Suit::Clubs, Rank::Two),
            card(11, Suit::Hearts, Rank::Three),
            card(12, Suit::Hearts, Rank::Four),
            card(13, Suit::Hearts, Rank::Five),
        ];
        let too_short = [
            card(14, Suit::Hearts, Rank::Ace),
            card(15, Suit::Clubs, Rank::Two),
            card(16, Suit::Hearts, Rank::Three),
        ];

        assert!(validate_run(&low));
        assert!(!validate_run(&hole_where_the_deuce_goes));
        assert!(validate_run(&five));
        assert!(!validate_run(&too_short));
    }

    #[test]
    fn test_ace_placement_high() {
        let high = [
            card(1, Suit::Hearts, Rank::Jack),
            card(2, Suit::Hearts, Rank::Queen),
            card(3, Suit::Hearts, Rank::King),
            card(4, Suit::Hearts, Rank::Ace),
        ];
        let five = [
            card(5, Suit::Hearts, Rank::Ten),
            card(6, Suit::Hearts, Rank::Jack),
            card(7, Suit::Hearts, Rank::Queen),
            card(8, Suit::Hearts, Rank::King),
            card(9, Suit::Hearts, Rank::Ace),
        ];
        let too_short = [
            card(10, Suit::Hearts, Rank::Queen),
            card(11, Suit::Hearts, Rank::King),
            card(12, Suit::Hearts, Rank::Ace),
        ];

        assert!(validate_run(&high));
        assert!(validate_run(&five));
        assert!(!validate_run(&too_short));
    }

    #[test]
    fn test_ace_wrap_rejection() {
        let wrap = [
            card(1, Suit::Hearts, Rank::King),
            card(2, Suit::Hearts, Rank::Ace),
            card(3, Suit::Clubs, Rank::Two),
            card(4, Suit::Hearts, Rank::Three),
        ];
        let wrap_same_suit_two = [
            card(5, Suit::Hearts, Rank::King),
            card(6, Suit::Hearts, Rank::Ace),
            card(7, Suit::Hearts, Rank::Two),
            card(8, Suit::Hearts, Rank::Three),
        ];
        let low_ace_plus_king = [
            card(9, Suit::Hearts, Rank::Ace),
            card(10, Suit::Clubs, Rank::Two),
            card(11, Suit::Hearts, Rank::Three),
            card(12, Suit::Hearts, Rank::King),
        ];

        assert!(!validate_run(&wrap));
        assert!(!validate_run(&wrap_same_suit_two));
        assert!(!validate_run(&low_ace_plus_king));
    }
}
