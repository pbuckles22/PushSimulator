# Agent handoff — PushSimulator

## Purpose

**Push** card-game engine and simulator: pure Rust core, WASM debug table (Phase 1b), Monte Carlo (Phase 2), SwiftUI + UniFFI iOS (Phase 3), Axum multiplayer (Phase 4). Agentic layer from [AgenticTemplate](https://github.com/pbuckles22/AgenticTemplate).

**Repo:** https://github.com/pbuckles22/PushSimulator

---

## Source of truth

- **Status:** [doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md)
- **Architecture:** [doc/ARCHITECTURE.md](doc/ARCHITECTURE.md)
- **Plan commentary:** [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md)
- **Backlog / Sprint 1:** [doc/BACKLOG.md](doc/BACKLOG.md), [doc/sprints/SPRINT_1.md](doc/sprints/SPRINT_1.md)
- **Rules:** [doc/requirements/GAME_RULES.md](doc/requirements/GAME_RULES.md)
- **Scope / phases:** [PM_PLAN.md](PM_PLAN.md)
- **Skills:** [.cursor/skills/](.cursor/skills/)

## Current state

- **Ship:** 1.4.2 is on **`main` at `da56999`**. `validate_run` accepts four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run. `feature/1.4.2-validate-runs` is kept. 1.2.4.3 is on **`main` at `b70cfa2`**. A short draw pile deals the cards it has, then panics. `feature/1.2.4.3-deal-short-deck` is kept. 1.4.1 Validating Sets is on **`main` at `62220b3`**. `validate_set` accepts three or more cards of one rank. Twos and jokers fill that rank. A group of only wilds is a set. 1.3.2 is on **`main` at `1500b02`**. `Action::PushDiscard` gives the next player the top discard and the top draw-pile card. The pushing player then draws. Epic 1.3 is complete. `feature/1.4.1-validate-sets` and `feature/1.3.2-push-discard` are kept. `main` before these lands was 1.3.1 at `1dfdbc6` (TakeDiscard content `6a00eff`). `feature/1.3.1-take-discard` is kept. Epic 1.2 stays inside that history at `b7e4b93` (tip `2272506`). `feature/1.2.6-hand-total` is kept. Earlier 1.2 branches stay: `feature/1.2.5-wild-scoring`, `feature/1.2.4-face-ace-scoring`, `feature/1.2.3-pip-scoring`, `feature/1.2.2-deal-hands`, `feature/1.2.1-player-init`. Epic 1.1 stays inside that history. `feature/1.1.4-empty-reshuffle` is kept. `feature/1.1.4-shuffle-draw` is kept. Prior park: `origin/feature/1.1.2-wild-cards @ 71c50c7`.
- **Phase:** 1 — Rust core. **Next:** Epic 1.2 is complete. Epic 1.4 stays open until SWAT. 1.4.2 `validate_run` accepts four or more cards of one suit in order. Twos and jokers fill gaps. An ace is low or high. King, ace, a two, and a three is not a run. 1.2.4.3 is on `main` at `b70cfa2`: a short draw pile deals the cards it has, then panics. 1.2.4.2 seeds `Deck::shuffle_with` and `Deck::draw_with`. Card ids survive shuffle, draw, and a reshuffle of three or more. Epic 1.2 is complete. 1.4.1 `validate_set` accepts three or more cards of one rank. Twos and jokers fill that rank. A group of only wilds is a set. Two cards, or two different natural ranks, are not. 1.3.2 `Action::PushDiscard` moves the discard top onto the next player's hand, then that player draws the top of the draw pile. The pushing player draws the next card. Cards under the pushed top stay. `points` and `total_score` stay 0. 1.3.1 `Action::TakeDiscard` moves the discard pile's top card onto the end of the player's hand. Cards under that top stay. The draw pile stays. `points` and `total_score` stay 0. 1.2.6 `calculate_hand_penalty` sums the hand. A hand of 4, Jack, Ace, and Joker is 50. `add_hand_penalty_to_total` adds that onto `total_score`. `points` stays 0. 1.2.5 returns 20 for a two and a joker. 1.2.4.1 returns 10 for a 10 through King and 15 for an ace. 1.2.3 returns 5 for ranks 3 through 9. 1.2.2 deals one card at a time to 2 or more players, 10 cards each. A one-player deal is refused. The discard pile stays put. 1.2.1 `Player::new` starts at 0 points, `is_on_board = false`, empty hand, `total_score` 0.
- **Version:** no bump (workspace 0.1.0; not a user-visible build).
- **Client path:** SwiftUI + UniFFI (not Flutter).
- **Handoff:** The next session gets **`You are the agent to execute the below hand off.`** as first line; brief is second person.

## Run and test

Requires Rust (`rustup`). Add `%USERPROFILE%\.cargo\bin` to PATH on Windows if `cargo` is missing.

```bash
cargo test -p push_core
cargo test
wasm-pack build push_wasm --target web --out-dir ../viewer/pkg
cargo run -p push_sim
```

Viewer: `cd viewer && npx --yes serve .`

**Merge-ready gate:** `cargo test -p push_core`

## Conventions

- Game rules live only in `push_core`.
- Strict TDD for Epic 1.x; integration chains after Epic 1.8.
- **Handoff protocol:** First chat line **`You are the agent to execute the below hand off.`** Body uses **You** …; never tell the receiver to write a handoff. Template: [`.cursor/handoff/_template.md`](.cursor/handoff/_template.md).

## Git workflow

1. Integration branch: **`main`**
2. Feature branches: `feature/<story-id>-short-topic` — merge only after **CMPH**
3. Gate before push: `cargo test -p push_core`
4. **UCPH** pushes feature branch only; **CMPH** merges to `main`

## Handoff protocol

UCPH/CMPH → Receiver brief for the **next** agent (see template). SWAT → handoff-checklist + review swarm.

## Epic close (SWAT)

Follow [.cursor/rules/epic-close.mdc](.cursor/rules/epic-close.mdc) only on **SWAT** or the epic’s last CMPH.
