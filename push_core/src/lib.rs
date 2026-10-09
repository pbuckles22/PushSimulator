//! Push core engine — pure Rust, headless.
//!
//! Phase 1 builds cards, deck, players, validation, actions, and GameState
//! via strict TDD (see `doc/sprints/SPRINT_1.md` and `doc/BACKLOG.md`).
//! Modules below are stubs until their first failing test lands.

pub mod actions;
pub mod card;
pub mod deck;
pub mod game_state;
pub mod latency;
pub mod legal_moves;
pub mod metrics;
pub mod player;
pub mod random_bot;
pub mod resolution;
pub mod validation;

#[cfg(test)]
mod legacy_oracle;

/// Crate is alive; replace with real API as Epic 1.1 ships.
pub fn engine_name() -> &'static str {
    "push_core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_name_is_push_core() {
        assert_eq!(engine_name(), "push_core");
    }
}
