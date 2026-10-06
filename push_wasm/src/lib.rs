//! WASM bridge for the Phase 1b debug table viewer.
//!
//! Build (from repo root, after Phase 1b starts):
//! `wasm-pack build push_wasm --target web --out-dir ../viewer/pkg`

use wasm_bindgen::prelude::*;

/// Smoke export so the viewer can confirm the WASM module loaded.
#[wasm_bindgen]
pub fn engine_name() -> String {
    push_core::engine_name().to_string()
}
