//! Swift calls this crate. The shoe size, the board, and the local hand come from `push_core`.

use std::sync::Arc;

mod snapshot;

pub use snapshot::{
    snapshot_from_state, CardRank, CardSnapshot, CardSuit, MeldSnapshot, TableSnapshot,
};

uniffi::setup_scaffolding!();

/// Which board `table_snapshot` returns. The shoe is always the undealt deck.
enum TableKind {
    Empty,
    Exhibit,
}

/// The game Swift constructs. The shoe is the one `push_core` deals from.
#[derive(uniffi::Object)]
pub struct Game {
    inner: push_core::game::Game,
    kind: TableKind,
}

#[uniffi::export]
impl Game {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: push_core::game::Game::new(),
            kind: TableKind::Empty,
        })
    }

    /// A set of three fours and a heart run. The undealt shoe stays 108.
    #[uniffi::constructor]
    pub fn exhibit_set_and_run() -> Arc<Self> {
        Arc::new(Self {
            inner: push_core::game::Game::new(),
            kind: TableKind::Exhibit,
        })
    }

    /// Cards in the undealt shoe.
    pub fn get_deck_size(&self) -> u32 {
        self.inner.get_deck_size()
    }

    /// Workspace version the screen shows.
    pub fn public_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// The shoe size, the round, the melds, and the local hand the screen draws.
    pub fn table_snapshot(&self) -> TableSnapshot {
        let deck_size = self.get_deck_size();
        match self.kind {
            TableKind::Empty => snapshot::empty_snapshot(deck_size),
            TableKind::Exhibit => snapshot::exhibit_snapshot(deck_size),
        }
    }
}
