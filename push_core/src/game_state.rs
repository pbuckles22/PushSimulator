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
/// wild locks until `turn_counter` plus one. A card whose lock is still ahead of
/// that counter cannot be played, hit, or used to replace a wild. A new table
/// starts that counter at 0. `drawn_card_id` is the card this turn's take or
/// push just put into the actor's hand. That card may be discarded even when
/// it fits a meld. `advance_turn` clears it.
#[derive(Clone, Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub deck: Deck,
    pub round_number: u8,
    pub board: Vec<Vec<Card>>,
    pub turn_counter: u32,
    pub drawn_card_id: Option<u32>,
}

impl GameState {
    pub fn new(players: Vec<Player>, deck: Deck) -> Self {
        Self {
            players,
            deck,
            round_number: 1,
            board: Vec::new(),
            turn_counter: 0,
            drawn_card_id: None,
        }
    }

    /// Moves the table on by one turn.
    ///
    /// Hands, the board, the piles, the round, and both scores stay as they were.
    /// A card locked until the old counter plus one can be played after this.
    /// The card drawn on the turn that just ended is no longer exempt from the
    /// safe-discard rule.
    pub fn advance_turn(&mut self) {
        self.turn_counter = self.turn_counter.saturating_add(1);
        self.drawn_card_id = None;
    }
}
