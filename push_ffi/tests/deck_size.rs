//! Chain: `Deck::new` → `push_core::Game::get_deck_size` → UniFFI `Game::get_deck_size`.

#[test]
fn test_deck_new_uniffi_game_get_deck_size_matches_the_core_shoe() {
    let ffi = push_ffi::Game::new();
    let core = push_core::game::Game::new();
    assert_eq!(ffi.get_deck_size(), core.get_deck_size());
    assert_eq!(ffi.get_deck_size(), 108);
    assert_eq!(ffi.get_deck_size(), ffi.get_deck_size());
    assert_ne!(ffi.get_deck_size(), 87);
}
