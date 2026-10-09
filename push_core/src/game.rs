//! A new game and the shoe it starts with.

use crate::deck::Deck;

/// A new game. The shoe is a full [`Deck::new`] and no cards have been dealt.
pub struct Game {
    deck: Deck,
}

impl Game {
    pub fn new() -> Self {
        Self { deck: Deck::new() }
    }

    /// How many cards are in the shoe. A new game has not dealt any of them.
    pub fn get_deck_size(&self) -> u32 {
        self.deck.cards.len() as u32
    }
}
