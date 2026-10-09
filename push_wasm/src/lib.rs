//! WASM bridge for the Phase 1b debug table viewer.
//!
//! Build (from repo root, after the `wasm32-unknown-unknown` target is installed):
//! `wasm-pack build push_wasm --target web --out-dir ../viewer/pkg`
//!
//! `tick_game` is the JavaScript entry. It plays one point-averse turn and
//! returns that `GameState` as a JS object. Native tests call [`WasmGame::tick`]
//! and [`WasmGame::state_json`], which are the same step.

use push_core::game_state::GameState;
use push_core::profiles::{step_profile_turn, BotProfile};
use push_core::random_bot::new_two_seat_table;
use rand::rngs::StdRng;
use rand::SeedableRng;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

/// Smoke export so the viewer can confirm the WASM module loaded.
#[wasm_bindgen]
pub fn engine_name() -> String {
    push_core::engine_name().to_string()
}

/// One dealt table. Each [`Self::tick`] plays the current seat as point-averse.
#[wasm_bindgen]
pub struct WasmGame {
    state: GameState,
    rng: StdRng,
    seat: usize,
}

#[wasm_bindgen]
impl WasmGame {
    /// Deals two seats from `seed` and leaves seat 0 to play.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> WasmGame {
        let mut rng = StdRng::seed_from_u64(seed);
        let state = new_two_seat_table(&mut rng);
        WasmGame {
            state,
            rng,
            seat: 0,
        }
    }

    /// Plays one point-averse turn and returns the table as a JavaScript object.
    #[cfg(target_arch = "wasm32")]
    pub fn tick_game(&mut self) -> JsValue {
        self.tick();
        serde_wasm_bindgen::to_value(&self.state)
            .unwrap_or_else(|err| wasm_bindgen::throw_str(&err.to_string()))
    }
}

impl WasmGame {
    /// A table already in progress. `seed` is used for the next shuffle or draw.
    pub fn from_table(state: GameState, seed: u64, seat: usize) -> Self {
        Self {
            state,
            rng: StdRng::seed_from_u64(seed),
            seat,
        }
    }

    /// The seat that plays on the next tick.
    pub fn seat(&self) -> usize {
        self.seat
    }

    /// The table as it stands.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// One point-averse turn, then the match loop's advance.
    pub fn tick(&mut self) {
        let stepped = step_profile_turn(
            &mut self.state,
            self.seat,
            &mut self.rng,
            BotProfile::PointAverse,
        );
        self.seat = stepped.seat;
    }

    /// The table as a JSON string. `tick_game` uses these same fields as a JS object.
    pub fn state_json(&self) -> String {
        serde_json::to_string(&self.state).expect("GameState serializes")
    }
}
