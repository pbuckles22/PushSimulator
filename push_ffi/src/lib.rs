//! Swift calls this crate. The shoe size comes from `push_core`.

use std::sync::Arc;

uniffi::setup_scaffolding!();

/// The game Swift constructs. The shoe is the one `push_core` deals from.
#[derive(uniffi::Object)]
pub struct Game {
    inner: push_core::game::Game,
}

#[uniffi::export]
impl Game {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: push_core::game::Game::new(),
        })
    }

    /// Cards in the undealt shoe.
    pub fn get_deck_size(&self) -> u32 {
        self.inner.get_deck_size()
    }
}
