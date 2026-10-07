//! Set/run validation and round requirements (Epics 1.4–1.5).

use std::collections::HashSet;

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

/// Round minimums for getting on the board.
///
/// A meld counts only when it is a set or a run. One meld fills one requirement.
/// Four or more wilds are both, and still fill only one. A meld may be larger
/// than the minimum. An extra meld fails, and so does a card used twice.
/// Six fours may be two sets of three.
///
/// - Round 1: two sets of at least 3
/// - Round 2: one set of at least 3 and one run of at least 4
/// - Round 3: two runs of at least 4
/// - Round 4: three sets of at least 3
/// - Round 5: one set of at least 3 and one run of at least 7
pub fn check_round_requirements<M: AsRef<[Card]>>(round_number: u8, melds: &[M]) -> bool {
    let Some(needs) = round_requirements(round_number) else {
        return false;
    };
    if melds.is_empty()
        || melds.iter().any(|meld| !legal_meld(meld.as_ref()))
        || !each_card_once(melds)
    {
        return false;
    }

    let mut used = vec![false; melds.len()];
    for need in &needs {
        let mut filled = 0;
        for exclusive in [true, false] {
            for (index, meld) in melds.iter().enumerate() {
                if used[index] || filled == need.count {
                    continue;
                }
                let cards = meld.as_ref();
                if !is_requirement(cards, need) {
                    continue;
                }
                if exclusive && fills_other(cards, &needs, need.kind) {
                    continue;
                }
                used[index] = true;
                filled += 1;
            }
        }
        if filled < need.count {
            return false;
        }
    }
    used.iter().all(|was_used| *was_used)
}

fn each_card_once<M: AsRef<[Card]>>(melds: &[M]) -> bool {
    let mut seen = HashSet::new();
    melds
        .iter()
        .flat_map(|meld| meld.as_ref())
        .all(|card| seen.insert(card.id))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MeldKind {
    Set,
    Run,
}

struct Requirement {
    kind: MeldKind,
    min_len: usize,
    count: usize,
}

fn round_requirements(round_number: u8) -> Option<Vec<Requirement>> {
    match round_number {
        1 => Some(vec![Requirement {
            kind: MeldKind::Set,
            min_len: 3,
            count: 2,
        }]),
        2 => Some(vec![
            Requirement {
                kind: MeldKind::Set,
                min_len: 3,
                count: 1,
            },
            Requirement {
                kind: MeldKind::Run,
                min_len: 4,
                count: 1,
            },
        ]),
        3 => Some(vec![Requirement {
            kind: MeldKind::Run,
            min_len: 4,
            count: 2,
        }]),
        4 => Some(vec![Requirement {
            kind: MeldKind::Set,
            min_len: 3,
            count: 3,
        }]),
        5 => Some(vec![
            Requirement {
                kind: MeldKind::Set,
                min_len: 3,
                count: 1,
            },
            Requirement {
                kind: MeldKind::Run,
                min_len: 7,
                count: 1,
            },
        ]),
        _ => None,
    }
}

fn legal_meld(cards: &[Card]) -> bool {
    validate_set(cards) || validate_run(cards)
}

fn is_requirement(cards: &[Card], need: &Requirement) -> bool {
    if cards.len() < need.min_len {
        return false;
    }
    match need.kind {
        MeldKind::Set => validate_set(cards),
        MeldKind::Run => validate_run(cards),
    }
}

fn fills_other(cards: &[Card], needs: &[Requirement], kind: MeldKind) -> bool {
    needs
        .iter()
        .any(|need| need.kind != kind && is_requirement(cards, need))
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
    use crate::validation::{check_round_requirements, validate_run, validate_set};

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
    fn test_validate_run_two_starts_low_and_length_bounds() {
        let two_low = [
            card(1, Suit::Hearts, Rank::Two),
            card(2, Suit::Hearts, Rank::Three),
            card(3, Suit::Hearts, Rank::Four),
            card(4, Suit::Hearts, Rank::Five),
        ];
        let two_cannot_fill_two_holes = [
            card(10, Suit::Hearts, Rank::Two),
            card(11, Suit::Hearts, Rank::Three),
            card(12, Suit::Hearts, Rank::Five),
            card(13, Suit::Hearts, Rank::Seven),
        ];
        let king_ace_extended = [
            card(20, Suit::Hearts, Rank::King),
            card(21, Suit::Hearts, Rank::Ace),
            card(22, Suit::None, Rank::Joker),
            card(23, Suit::Spades, Rank::Two),
        ];
        let full_suit = [
            card(30, Suit::Hearts, Rank::Ace),
            card(31, Suit::Hearts, Rank::Two),
            card(32, Suit::Hearts, Rank::Three),
            card(33, Suit::Hearts, Rank::Four),
            card(34, Suit::Hearts, Rank::Five),
            card(35, Suit::Hearts, Rank::Six),
            card(36, Suit::Hearts, Rank::Seven),
            card(37, Suit::Hearts, Rank::Eight),
            card(38, Suit::Hearts, Rank::Nine),
            card(39, Suit::Hearts, Rank::Ten),
            card(40, Suit::Hearts, Rank::Jack),
            card(41, Suit::Hearts, Rank::Queen),
            card(42, Suit::Hearts, Rank::King),
        ];
        let mut past_max = full_suit.to_vec();
        past_max.push(card(50, Suit::None, Rank::Joker));

        assert!(validate_run(&two_low));
        assert!(!validate_run(&two_cannot_fill_two_holes));
        assert!(validate_run(&king_ace_extended));
        assert!(validate_run(&full_suit));
        assert!(!validate_run(&past_max));
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

    fn set_of(id: u32, rank: Rank, suits: &[Suit]) -> Vec<Card> {
        suits
            .iter()
            .enumerate()
            .map(|(offset, suit)| card(id + offset as u32, *suit, rank))
            .collect()
    }

    fn heart_run(id: u32, ranks: &[Rank]) -> Vec<Card> {
        suited_run(id, Suit::Hearts, ranks)
    }

    fn suited_run(id: u32, suit: Suit, ranks: &[Rank]) -> Vec<Card> {
        ranks
            .iter()
            .enumerate()
            .map(|(offset, rank)| card(id + offset as u32, suit, *rank))
            .collect()
    }

    fn wilds(id: u32, count: usize) -> Vec<Card> {
        (0..count)
            .map(|offset| {
                let offset = offset as u32;
                if offset % 2 == 0 {
                    card(id + offset, Suit::None, Rank::Joker)
                } else {
                    card(id + offset, Suit::Clubs, Rank::Two)
                }
            })
            .collect()
    }

    #[test]
    fn test_round_1_minimum_rejection() {
        let one_set = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let oversized = set_of(
            10,
            Rank::Five,
            &[Suit::Hearts, Suit::Spades, Suit::Clubs, Suit::Diamonds],
        );

        assert!(!check_round_requirements(1, &[&one_set]));
        assert!(!check_round_requirements(1, &[&oversized]));
        assert!(!check_round_requirements::<&Vec<Card>>(1, &[]));
    }

    #[test]
    fn test_round_1_minimum_acceptance() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let fives = set_of(10, Rank::Five, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);

        assert!(check_round_requirements(1, &[&fours, &fives]));
    }

    #[test]
    fn test_round_1_exceeding_minimum() {
        let fours = set_of(
            1,
            Rank::Four,
            &[Suit::Hearts, Suit::Spades, Suit::Clubs, Suit::Diamonds],
        );
        let fives = set_of(
            10,
            Rank::Five,
            &[Suit::Hearts, Suit::Spades, Suit::Clubs, Suit::Diamonds],
        );
        let sixes = set_of(20, Rank::Six, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let sevens = set_of(30, Rank::Seven, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let eights = set_of(40, Rank::Eight, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let five_wide = vec![
            card(50, Suit::Hearts, Rank::Nine),
            card(51, Suit::Spades, Rank::Nine),
            card(52, Suit::Clubs, Rank::Nine),
            card(53, Suit::Diamonds, Rank::Nine),
            card(54, Suit::Hearts, Rank::Nine),
        ];
        let run = heart_run(60, &[Rank::Three, Rank::Four, Rank::Five, Rank::Six]);

        assert!(check_round_requirements(1, &[&fours, &fives]));
        assert!(check_round_requirements(1, &[&five_wide, &sixes]));
        assert!(!check_round_requirements(1, &[&sixes, &sevens, &eights]));
        assert!(!check_round_requirements(1, &[&fours, &fives, &run]));
    }

    #[test]
    fn test_round_requirements_reused_card_rejection() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let shared = card(1, Suit::Hearts, Rank::Four);
        let other = vec![
            shared,
            card(10, Suit::Spades, Rank::Five),
            card(11, Suit::Clubs, Rank::Five),
        ];

        assert!(!check_round_requirements(1, &[&fours, &fours]));
        assert!(!check_round_requirements(1, &[&fours, &other]));
    }

    #[test]
    fn test_round_1_non_set_rejection() {
        let mixed = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Spades, Rank::Four),
            card(3, Suit::Clubs, Rank::Five),
        ];
        let also_mixed = vec![
            card(4, Suit::Hearts, Rank::Six),
            card(5, Suit::Spades, Rank::Seven),
            card(6, Suit::Clubs, Rank::Eight),
        ];
        let fours = set_of(10, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let run = heart_run(20, &[Rank::Four, Rank::Five, Rank::Six, Rank::Seven]);
        let garbage = vec![
            card(30, Suit::Hearts, Rank::Nine),
            card(31, Suit::Spades, Rank::Nine),
        ];
        let fives = set_of(40, Rank::Five, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);

        assert!(!check_round_requirements(1, &[&mixed, &also_mixed]));
        assert!(!check_round_requirements(1, &[&fours, &run]));
        assert!(!check_round_requirements(1, &[&fours, &fives, &garbage]));
    }

    #[test]
    fn test_round_1_wild_sets_acceptance() {
        let with_wilds = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::None, Rank::Joker),
            card(3, Suit::Spades, Rank::Two),
        ];
        let jokers = vec![
            card(4, Suit::None, Rank::Joker),
            card(5, Suit::None, Rank::Joker),
            card(6, Suit::None, Rank::Joker),
        ];

        assert!(check_round_requirements(1, &[&with_wilds, &jokers]));
    }

    #[test]
    fn test_round_1_same_rank_sets_acceptance() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let more_fours = set_of(
            10,
            Rank::Four,
            &[Suit::Diamonds, Suit::Hearts, Suit::Spades],
        );

        assert!(check_round_requirements(1, &[&fours, &more_fours]));
    }

    #[test]
    fn test_round_2_set_and_run() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let fives = set_of(10, Rank::Five, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let run = heart_run(20, &[Rank::Four, Rank::Five, Rank::Six, Rank::Seven]);
        let short_run = heart_run(30, &[Rank::Four, Rank::Five, Rank::Six]);
        let wilds = vec![
            card(40, Suit::None, Rank::Joker),
            card(41, Suit::None, Rank::Joker),
            card(42, Suit::None, Rank::Joker),
            card(43, Suit::Clubs, Rank::Two),
        ];
        let longer = heart_run(
            50,
            &[Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven],
        );

        assert!(check_round_requirements(2, &[&fours, &run]));
        assert!(check_round_requirements(2, &[&fours, &longer]));
        assert!(!check_round_requirements(2, &[&fours, &fives]));
        assert!(!check_round_requirements(2, &[&fours, &short_run]));
        assert!(!check_round_requirements(2, &[&wilds]));
        assert!(check_round_requirements(2, &[&wilds, &run]));
        assert!(check_round_requirements(2, &[&wilds, &fours]));
        assert!(!check_round_requirements(2, &[&fours, &fives, &run]));
    }

    #[test]
    fn test_round_3_two_runs() {
        let hearts = heart_run(1, &[Rank::Four, Rank::Five, Rank::Six, Rank::Seven]);
        let diamonds = vec![
            card(10, Suit::Diamonds, Rank::Eight),
            card(11, Suit::Diamonds, Rank::Nine),
            card(12, Suit::Diamonds, Rank::Ten),
            card(13, Suit::Diamonds, Rank::Jack),
        ];
        let long = heart_run(
            20,
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
        let fours = set_of(30, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);

        assert!(check_round_requirements(3, &[&hearts, &diamonds]));
        assert!(check_round_requirements(3, &[&long, &diamonds]));
        assert!(!check_round_requirements(3, &[&long]));
        assert!(!check_round_requirements(3, &[&hearts, &fours]));
        assert!(!check_round_requirements(3, &[&hearts, &diamonds, &long]));
    }

    #[test]
    fn test_round_4_three_sets() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let fives = set_of(
            10,
            Rank::Five,
            &[Suit::Hearts, Suit::Spades, Suit::Clubs, Suit::Diamonds],
        );
        let sixes = set_of(20, Rank::Six, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let sevens = set_of(30, Rank::Seven, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);

        assert!(check_round_requirements(4, &[&fours, &fives, &sixes]));
        assert!(!check_round_requirements(
            4,
            &[&fours, &fives, &sixes, &sevens]
        ));
        assert!(!check_round_requirements(4, &[&fours, &fives]));
    }

    #[test]
    fn test_round_5_set_and_run_of_seven() {
        let fours = set_of(
            1,
            Rank::Four,
            &[Suit::Hearts, Suit::Spades, Suit::Clubs, Suit::Diamonds],
        );
        let run7 = heart_run(
            10,
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
        let run8 = heart_run(
            20,
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
        let run6 = heart_run(
            30,
            &[
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
            ],
        );
        let gapped = vec![
            card(40, Suit::Hearts, Rank::Three),
            card(41, Suit::Hearts, Rank::Four),
            card(42, Suit::Hearts, Rank::Five),
            card(43, Suit::Hearts, Rank::Six),
            card(44, Suit::Hearts, Rank::Eight),
            card(45, Suit::Hearts, Rank::Nine),
            card(46, Suit::Hearts, Rank::Ten),
        ];
        let wild_run = vec![
            card(50, Suit::Hearts, Rank::Four),
            card(51, Suit::None, Rank::Joker),
            card(52, Suit::Hearts, Rank::Six),
            card(53, Suit::Hearts, Rank::Seven),
            card(54, Suit::Hearts, Rank::Eight),
            card(55, Suit::Hearts, Rank::Nine),
            card(56, Suit::Hearts, Rank::Ten),
        ];
        let seven_wilds = vec![
            card(60, Suit::Hearts, Rank::Two),
            card(61, Suit::Spades, Rank::Two),
            card(62, Suit::Clubs, Rank::Two),
            card(63, Suit::Diamonds, Rank::Two),
            card(64, Suit::None, Rank::Joker),
            card(65, Suit::None, Rank::Joker),
            card(66, Suit::None, Rank::Joker),
        ];

        assert!(check_round_requirements(5, &[&fours, &run7]));
        assert!(check_round_requirements(5, &[&fours, &run8]));
        assert!(check_round_requirements(5, &[&fours, &wild_run]));
        assert!(!check_round_requirements(5, &[&fours, &run6]));
        assert!(!check_round_requirements(5, &[&fours, &gapped]));
        assert!(!check_round_requirements(5, &[&seven_wilds]));
        assert!(check_round_requirements(5, &[&fours, &seven_wilds]));
    }

    #[test]
    fn test_round_requirements_unknown_round_rejection() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let fives = set_of(10, Rank::Five, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);

        assert!(!check_round_requirements(0, &[&fours, &fives]));
        assert!(!check_round_requirements(6, &[&fours, &fives]));
        assert!(!check_round_requirements(255, &[&fours, &fives]));
    }

    #[test]
    fn test_round_1_six_fours_need_two_melds() {
        let six = set_of(
            1,
            Rank::Four,
            &[
                Suit::Hearts,
                Suit::Spades,
                Suit::Clubs,
                Suit::Diamonds,
                Suit::Hearts,
                Suit::Spades,
            ],
        );
        let left = six[..3].to_vec();
        let right = six[3..].to_vec();

        assert!(!check_round_requirements(1, &[&six]));
        assert!(check_round_requirements(1, &[&left, &right]));
        assert!(check_round_requirements(1, &[&right, &left]));
    }

    #[test]
    fn test_round_requirements_pair_empty_and_duplicate_id() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let pair = vec![
            card(20, Suit::Hearts, Rank::Five),
            card(21, Suit::Spades, Rank::Five),
        ];
        let empty: Vec<Card> = vec![];
        let doubled = vec![
            card(30, Suit::Hearts, Rank::Six),
            card(30, Suit::Spades, Rank::Six),
            card(31, Suit::Clubs, Rank::Six),
        ];
        let shared = card(1, Suit::Hearts, Rank::Four);
        let run = vec![
            shared,
            card(40, Suit::Hearts, Rank::Five),
            card(41, Suit::Hearts, Rank::Six),
            card(42, Suit::Hearts, Rank::Seven),
        ];

        assert!(validate_set(&doubled));
        assert!(!check_round_requirements(1, &[&fours, &pair]));
        assert!(!check_round_requirements(1, &[&pair, &pair]));
        assert!(!check_round_requirements(1, &[&fours, &empty]));
        assert!(!check_round_requirements(1, &[&doubled, &fours]));
        assert!(!check_round_requirements(2, &[&fours, &run]));
    }

    #[test]
    fn test_four_wilds_fill_one_requirement() {
        let group_a = wilds(1, 4);
        let group_b = wilds(10, 4);
        let group_c = wilds(20, 4);
        let fours = set_of(40, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let run = heart_run(50, &[Rank::Four, Rank::Five, Rank::Six, Rank::Seven]);

        assert!(check_round_requirements(1, &[&group_a, &fours]));
        assert!(!check_round_requirements(1, &[&group_a]));
        assert!(check_round_requirements(1, &[&group_a, &group_b]));
        assert!(!check_round_requirements(
            1,
            &[&group_a, &group_b, &group_c]
        ));

        assert!(check_round_requirements(2, &[&run, &fours]));
        assert!(check_round_requirements(2, &[&group_a, &group_b]));
        assert!(!check_round_requirements(
            2,
            &[
                &run,
                &heart_run(60, &[Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack])
            ]
        ));
        assert!(!check_round_requirements(2, &[&fours, &run, &group_a]));
        assert!(!check_round_requirements(
            2,
            &[
                &fours,
                &heart_run(70, &[Rank::Four, Rank::Six, Rank::Seven, Rank::Eight])
            ]
        ));

        assert!(check_round_requirements(3, &[&group_a, &run]));
        assert!(check_round_requirements(3, &[&group_a, &group_b]));
        assert!(!check_round_requirements(
            3,
            &[&group_a, &group_b, &group_c]
        ));
        assert!(!check_round_requirements(4, &[&group_a, &group_b]));
        assert!(check_round_requirements(4, &[&group_a, &group_b, &group_c]));
        assert!(!check_round_requirements(5, &[&group_a, &group_b]));
    }

    #[test]
    fn test_round_3_eight_cards_need_two_melds() {
        let low = heart_run(1, &[Rank::Three, Rank::Four, Rank::Five, Rank::Six]);
        let high = heart_run(10, &[Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten]);
        let mut eight = low.clone();
        eight.extend(high.iter().copied());
        let short = heart_run(20, &[Rank::Jack, Rank::Queen, Rank::King]);
        let ace_low = vec![
            card(30, Suit::Hearts, Rank::Ace),
            card(31, Suit::Clubs, Rank::Two),
            card(32, Suit::Hearts, Rank::Three),
            card(33, Suit::Hearts, Rank::Four),
        ];
        let ace_high = suited_run(
            40,
            Suit::Spades,
            &[Rank::Jack, Rank::Queen, Rank::King, Rank::Ace],
        );
        let wrap = vec![
            card(50, Suit::Diamonds, Rank::King),
            card(51, Suit::Diamonds, Rank::Ace),
            card(52, Suit::Clubs, Rank::Two),
            card(53, Suit::Diamonds, Rank::Three),
        ];
        let shared = card(1, Suit::Hearts, Rank::Three);
        let overlap = vec![
            shared,
            card(60, Suit::Hearts, Rank::Four),
            card(61, Suit::Hearts, Rank::Five),
            card(62, Suit::Hearts, Rank::Six),
        ];

        assert!(check_round_requirements(3, &[&low, &high]));
        assert!(check_round_requirements(3, &[&high, &low]));
        assert!(!check_round_requirements(3, &[&eight]));
        assert!(!check_round_requirements(3, &[&low, &short]));
        assert!(check_round_requirements(3, &[&ace_low, &ace_high]));
        assert!(!check_round_requirements(3, &[&ace_low, &wrap]));
        assert!(!check_round_requirements(3, &[&low, &overlap]));
    }

    #[test]
    fn test_round_4_a_run_is_not_a_set() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let fives = set_of(10, Rank::Five, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let sixes = set_of(20, Rank::Six, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let run = heart_run(30, &[Rank::Four, Rank::Five, Rank::Six, Rank::Seven]);
        let other = heart_run(40, &[Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack]);
        let third = suited_run(
            50,
            Suit::Spades,
            &[Rank::Three, Rank::Four, Rank::Five, Rank::Six],
        );
        let nine = set_of(
            60,
            Rank::Nine,
            &[
                Suit::Hearts,
                Suit::Spades,
                Suit::Clubs,
                Suit::Diamonds,
                Suit::Hearts,
                Suit::Spades,
                Suit::Clubs,
                Suit::Diamonds,
                Suit::Hearts,
            ],
        );
        let shared = card(1, Suit::Hearts, Rank::Four);
        let overlap = vec![
            shared,
            card(80, Suit::Spades, Rank::Four),
            card(81, Suit::Clubs, Rank::Four),
        ];

        assert!(check_round_requirements(4, &[&sixes, &fives, &fours]));
        assert!(!check_round_requirements(4, &[&fours, &fives, &run]));
        assert!(!check_round_requirements(4, &[&run, &other, &third]));
        assert!(!check_round_requirements(4, &[&nine]));
        assert!(!check_round_requirements(4, &[&fours, &fives, &overlap]));
    }

    #[test]
    fn test_round_5_ace_runs_and_one_meld() {
        let fours = set_of(1, Rank::Four, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let more = set_of(10, Rank::Five, &[Suit::Hearts, Suit::Spades, Suit::Clubs]);
        let seven_nines = set_of(
            20,
            Rank::Nine,
            &[
                Suit::Hearts,
                Suit::Spades,
                Suit::Clubs,
                Suit::Diamonds,
                Suit::Hearts,
                Suit::Spades,
                Suit::Clubs,
            ],
        );
        let ace_low = vec![
            card(40, Suit::Hearts, Rank::Ace),
            card(41, Suit::Clubs, Rank::Two),
            card(42, Suit::Hearts, Rank::Three),
            card(43, Suit::Hearts, Rank::Four),
            card(44, Suit::Hearts, Rank::Five),
            card(45, Suit::Hearts, Rank::Six),
            card(46, Suit::Hearts, Rank::Seven),
        ];
        let ace_high = suited_run(
            50,
            Suit::Spades,
            &[
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ],
        );
        let unsorted = vec![
            card(60, Suit::Diamonds, Rank::Ace),
            card(61, Suit::Diamonds, Rank::Queen),
            card(62, Suit::Diamonds, Rank::Eight),
            card(63, Suit::Diamonds, Rank::Jack),
            card(64, Suit::Diamonds, Rank::Nine),
            card(65, Suit::Diamonds, Rank::King),
            card(66, Suit::Diamonds, Rank::Ten),
        ];
        let wrap = vec![
            card(70, Suit::Hearts, Rank::King),
            card(71, Suit::Hearts, Rank::Ace),
            card(72, Suit::Clubs, Rank::Two),
            card(73, Suit::Hearts, Rank::Three),
            card(74, Suit::Hearts, Rank::Four),
            card(75, Suit::Hearts, Rank::Five),
            card(76, Suit::Hearts, Rank::Six),
        ];
        let mixed = vec![
            card(80, Suit::Hearts, Rank::Eight),
            card(81, Suit::Hearts, Rank::Nine),
            card(82, Suit::Diamonds, Rank::Ten),
            card(83, Suit::Hearts, Rank::Jack),
            card(84, Suit::Hearts, Rank::Queen),
            card(85, Suit::Hearts, Rank::King),
            card(86, Suit::Hearts, Rank::Ace),
        ];
        let other_run = suited_run(
            90,
            Suit::Clubs,
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
        let short = heart_run(100, &[Rank::Four, Rank::Five, Rank::Six, Rank::Seven]);
        let wild_set = wilds(110, 4);
        let both_a = wilds(120, 7);
        let both_b = wilds(130, 7);
        let both_c = wilds(140, 7);

        assert!(check_round_requirements(5, &[&ace_high, &fours]));
        assert!(check_round_requirements(5, &[&fours, &ace_low]));
        assert!(check_round_requirements(5, &[&fours, &unsorted]));
        assert!(check_round_requirements(5, &[&wild_set, &ace_high]));
        assert!(check_round_requirements(5, &[&seven_nines, &ace_high]));
        assert!(check_round_requirements(5, &[&both_a, &both_b]));
        assert!(!check_round_requirements(5, &[&seven_nines, &fours]));
        assert!(!check_round_requirements(5, &[&seven_nines]));
        assert!(!check_round_requirements(5, &[&seven_nines, &short]));
        assert!(!check_round_requirements(5, &[&fours, &ace_high, &more]));
        assert!(!check_round_requirements(5, &[&ace_high, &other_run]));
        assert!(!check_round_requirements(5, &[&fours, &wrap]));
        assert!(!check_round_requirements(5, &[&fours, &mixed]));
        assert!(!check_round_requirements(5, &[&both_a, &both_b, &both_c]));
        assert!(!check_round_requirements(0, &[&fours, &ace_high]));
        assert!(!check_round_requirements(6, &[&fours, &ace_high]));
    }
}
