//! A new game reports the undealt shoe.
//!
//! Chain: `Deck::new` → `Game::get_deck_size`. Two decks, four suits, thirteen
//! ranks, and four jokers are 108 cards. Nothing has been dealt.

use push_core::deck::Deck;
use push_core::game::Game;

/// Chain: `Deck::new` → `Game::get_deck_size`.
#[test]
fn test_deck_new_game_get_deck_size_matches_the_undealt_shoe() {
    let shoe = Deck::new();
    let game = Game::new();

    assert_eq!(shoe.cards.len(), 108);
    assert_eq!(game.get_deck_size(), shoe.cards.len() as u32);
    assert_eq!(game.get_deck_size(), 108);
    let mut ids: Vec<u32> = shoe.cards.iter().map(|card| card.id).collect();
    ids.sort_unstable();
    assert_eq!(ids, (0..108).collect::<Vec<_>>());
}

#[test]
fn test_get_deck_size_stays_the_full_shoe_when_asked_again() {
    let game = Game::new();
    assert_eq!(game.get_deck_size(), 108);
    assert_eq!(game.get_deck_size(), 108);
    // A dealt two-seat table has taken 21 cards. This count is the shoe before that.
    assert_ne!(game.get_deck_size(), 87);
}

#[test]
fn test_two_new_games_report_the_same_deck_size() {
    assert_eq!(Game::new().get_deck_size(), Game::new().get_deck_size());
}
