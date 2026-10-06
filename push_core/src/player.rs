//! Player hand, board flag, score (Epic 1.2).

use crate::card::Card;

/// One seat. A new player has no cards, is not on the board, and has 0 points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    pub id: u32,
    pub seat_index: u32,
    pub points: u32,
    pub is_on_board: bool,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(id: u32, seat_index: u32) -> Self {
        Self {
            id,
            seat_index,
            points: 0,
            is_on_board: false,
            hand: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
