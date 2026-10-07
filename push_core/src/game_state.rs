//! The table: seats, piles, the round, and face-up melds.

use crate::card::Card;
use crate::deck::Deck;
use crate::player::Player;

/// Seats, piles, the round, and the face-up melds.
///
/// A new table is round 1 and the board is empty. `PlayMeld` reads `round_number`
/// and moves cards from one hand onto `board`. `HitMeld` adds cards from the
/// hand onto one or more of those melds, and only for a player who is on the board.
/// `StealWild` swaps a natural card for a wild on one of those melds. The stolen
/// wild locks until `turn_counter` plus one. A new table starts that counter at 0.
#[derive(Clone, Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub deck: Deck,
    pub round_number: u8,
    pub board: Vec<Vec<Card>>,
    pub turn_counter: u32,
}

impl GameState {
    pub fn new(players: Vec<Player>, deck: Deck) -> Self {
        Self {
            players,
            deck,
            round_number: 1,
            board: Vec::new(),
            turn_counter: 0,
        }
    }
}
