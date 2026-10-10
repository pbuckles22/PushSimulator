//! Swift calls this crate. The shoe size, the board, and the local hand come from `push_core`.
//! A drop applies `PlayMeld` or `HitMeld`. A game error or a refusal leaves the table.

use std::sync::{Arc, Mutex};

use push_core::actions::{validate_action, Action, MeldHit};
use push_core::card::{Card, Rank, Suit};
use push_core::deck::Deck;
use push_core::game_state::GameState;
use push_core::player::Player;
use push_core::resolution::ActionResolution;

mod snapshot;

pub use snapshot::{
    snapshot_from_state, CardRank, CardSnapshot, CardSuit, MeldSnapshot, TableSnapshot,
};

uniffi::setup_scaffolding!();

/// What Rust decided. An accept is the only verdict that changes the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum DropVerdict {
    Accepted,
    Refused,
    GameError,
}

/// One hit onto one meld already on the board.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct HitRequest {
    pub meld_index: u32,
    pub cards: Vec<CardSnapshot>,
}

/// The game Swift constructs. The shoe is the undealt deck. Drops run on `rules`.
#[derive(uniffi::Object)]
pub struct Game {
    shoe: push_core::game::Game,
    rules: Mutex<GameState>,
}

/// Classifies `action` and commits only an accept. A refusal or a game error
/// leaves `state` as it was. Play and hit do not draw.
pub fn apply_drop(state: &mut GameState, actor: usize, action: Action) -> DropVerdict {
    match validate_action(state, actor, &action) {
        ActionResolution::Invalid(_) => DropVerdict::GameError,
        ActionResolution::Rejected(_) => DropVerdict::Refused,
        ActionResolution::Accepted(_) => {
            let applied = state.apply(action, actor);
            assert!(applied, "an accepted drop commits");
            DropVerdict::Accepted
        }
    }
}

#[uniffi::export]
impl Game {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            shoe: push_core::game::Game::new(),
            rules: Mutex::new(empty_rules()),
        })
    }

    /// A set of three fours and a heart run. The undealt shoe stays 108.
    /// Seat 0 is on the board, so a hit is decided by the meld rules.
    #[uniffi::constructor]
    pub fn exhibit_set_and_run() -> Arc<Self> {
        Arc::new(Self {
            shoe: push_core::game::Game::new(),
            rules: Mutex::new(exhibit_rules()),
        })
    }

    /// Cards in the undealt shoe.
    pub fn get_deck_size(&self) -> u32 {
        self.shoe.get_deck_size()
    }

    /// Workspace version the screen shows.
    pub fn public_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// The shoe size, the round, the melds, and the local hand the screen draws.
    /// The shoe is the undealt deck, including after a drop.
    pub fn table_snapshot(&self) -> TableSnapshot {
        let state = self.rules.lock().expect("rules lock");
        let mut snap = snapshot::snapshot_from_state(&state);
        snap.deck_size = self.get_deck_size();
        snap
    }

    /// One `PlayMeld`. Seat 0 is the local seat.
    pub fn apply_play_meld(&self, groups: Vec<Vec<CardSnapshot>>) -> DropVerdict {
        let action = Action::PlayMeld(
            groups
                .into_iter()
                .map(|group| {
                    group
                        .into_iter()
                        .map(snapshot::card_from_snapshot)
                        .collect()
                })
                .collect(),
        );
        self.drop(action)
    }

    /// One `HitMeld`. Seat 0 is the local seat.
    pub fn apply_hit_meld(&self, hits: Vec<HitRequest>) -> DropVerdict {
        let action = Action::HitMeld(
            hits.into_iter()
                .map(|hit| MeldHit {
                    meld_index: hit.meld_index as usize,
                    cards: hit
                        .cards
                        .into_iter()
                        .map(snapshot::card_from_snapshot)
                        .collect(),
                })
                .collect(),
        );
        self.drop(action)
    }
}

impl Game {
    fn drop(&self, action: Action) -> DropVerdict {
        let mut state = self.rules.lock().expect("rules lock");
        apply_drop(&mut state, 0, action)
    }
}

fn empty_rules() -> GameState {
    GameState::new(vec![Player::new(0, 0), Player::new(1, 1)], Deck::new())
}

/// The painted exhibit, as a table the rules can accept or refuse.
///
/// The board and the hand are not dealt from the shoe. Seat 0 is already on
/// the board. Seat 1 holds a card the snapshot does not copy.
fn exhibit_rules() -> GameState {
    let painted = snapshot::exhibit_snapshot(0);
    let mut state = empty_rules();
    state.round_number = u8::try_from(painted.round_number).expect("round fits");
    state.board = painted
        .board
        .into_iter()
        .map(|meld| {
            meld.cards
                .into_iter()
                .map(snapshot::card_from_snapshot)
                .collect()
        })
        .collect();
    state.players[0].hand = painted
        .hand
        .into_iter()
        .map(snapshot::card_from_snapshot)
        .collect();
    state.players[0].is_on_board = true;
    state.players[1].hand = vec![Card {
        id: 90,
        suit: Suit::Hearts,
        rank: Rank::Ace,
        locked_until_turn: 0,
    }];
    state
}
