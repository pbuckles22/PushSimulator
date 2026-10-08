# Story 2.2.1: Refactor Move Generation & Combinatorics

## 1. The Problem: Allocation Overhead & Combinatorics Explosion

Currently, a release-mode five-round game takes a median of ~5.6 seconds, with massive outliers (e.g., 61.7 seconds). At the median, 100,000 games would take roughly 6.5 CPU-days before parallelization—far from the project’s goal.

There are three primary bottlenecks:

**A. The Allocation Trap (Cloning to Validate)**
In `push_core/src/legal_moves.rs`, the `accepts` function tests move legality by cloning the entire `GameState` and applying the move. Rust has no garbage collector; the cost is entirely in allocating, copying, and dropping these vectors.

**B. Materialized Combinatorics Explosion**
Play generation explores `(groups + 1)^hand_size` (up to `4^n` in round 4). Hit generation enumerates `2^n` subsets per board meld and then combines them. Even if validation is fast, materializing these into a `Vec<Action>` will cause massive memory bloat. A hack (`if hand.len() > 11`) currently bypasses the generator entirely.

**C. Panic Boundaries & Loose State Mutations**
`GameState::apply` can panic on malformed `Take` or `Push` calls rather than cleanly returning `false`. Furthermore, penalty transitions inherently mutate state on rejection, confusing the definition of a "failed" action.

## 2. The Solution: Lazy Validation & Deterministic Commits

To eliminate cloning and massive allocations without breaking rule fidelity, we will implement shared, lazy validation and deterministic stochastic commits.

1. **`validate_action` & `ActionResolution` Enum:**
   Validation will use a shared `validate_action(state, actor, action) -> ActionResolution` path.
   * `Accepted(ActionPlan)`: The move is fully legal. `commit` mutates state. Returns `true`.
   * `Rejected(RejectionPlan)`: The move is illegal but triggers a penalty (e.g., unsafe discard). `commit` mutates state (enters `PenaltyDrawing`). Returns `false`.
   * `Invalid(GameError)`: Nonsense move. No mutation. Returns `false`.

2. **Deterministic RNG & `apply_with_rng`:**
   Because draws and reshuffles are stochastic, validation cannot pre-calculate exact outcomes. `ActionPlan` represents the *intent*. We will expose `apply_with_rng(action, actor, &mut impl Rng)` to execute the intent deterministically.

3. **Policy-Specific Targeted Searches:**
   Bots will select actions via policy-specific targeted searches that do not traverse every legal action. Reservoir sampling is explicitly avoided as it still performs exponential work. `generate_legal_moves` will expose an iterator/visitor API rather than a materialized `Vec`, retaining the `Vec` wrapper strictly for compatibility testing.

4. **Sequential Capacity Model:**
   `push_is_legal` will no longer clone the `Deck`. We will implement a non-mutating `has_draw_capacity(count)` that accurately models sequential draws, correctly rejecting `LastTwo` boundaries without mutating piles.

## 3. Execution Plan (Micro-Wins)

Execute the following using strict Red/Green/Refactor TDD.

### Phase 0: RNG Plumbing & Legacy Oracle Isolation

* [x] **Introduce `apply_with_rng`:** Mechanically introduce `apply_with_rng` to the current state machine.
* [x] **Route Simulations:** Route all seeded simulations and tests through this new deterministic path.
* [x] **Isolate the Deep Oracle:** Rename the introduced path to `legacy_apply_with_rng` and preserve the deep legacy paths (`GameState::apply`, `accepts`, `assign_plays`, `hit_moves`, `additions`) into a new `legacy_oracle` module under `#[cfg(test)]`. *Do not copy random-bot grouping functions unless evaluating bot behavior.*

### Phase 1: Safe Validation Architecture

* [ ] **Sequential Capacity Model:** Implement `Deck::has_draw_capacity(count: usize) -> bool`. This must correctly model sequential draws/reshuffles to reject boundaries.
* [ ] **Resolution Types:** Define `ActionPlan` (intents), `RejectionPlan` (penalty state mutations), `GameError`, and the `ActionResolution` enum.
* [ ] **Panic Boundary Tests:** Write explicit tests for invalid actors, fewer than two players, empty discards, and insufficient drawable cards. Prove they return `Invalid` rather than panicking.

### Phase 2: Deterministic Locks & `apply_with_rng`

* [ ] **Implement `validate_action`:** Write the shared validation pathway without cloning.
* [ ] **Implement `apply_with_rng`:** Refactor mutation to consume the validated plan.
* [ ] **State Mutation Mapping:** Ensure `Accepted` → commit → returns `true`; `Rejected` → commit rejection effects → returns `false`; `Invalid` → no mutation → returns `false`.
* [ ] **Lock 1 (Deep Legality & State Match):** Compare `validate_action`/`ActionResolution` and resulting states (penalty transitions, draws, locks, round completion) of the new `apply_with_rng` against the `legacy_apply_with_rng` oracle using identically seeded RNGs. Test invalid/panic states separately.

### Phase 3: Lazy Generation & Admissible Pruning

* [ ] **Targeted Pruning Locks:** Write specific tests to prove pruning admissibility against wild cards, ace-high/low runs, duplicate cards, all-wild melds, and rounds requiring repeated meld types.
* [ ] **Lazy Iterator API:** Refactor generation to use an iterator/callback API. Update bots to use targeted searches instead of generating all combinations.
* [ ] **Lock 2 (Completeness via Multisets):** Compare the exact output of the new generator against the old generator on bounded hands. The equality assertion **must** compare them as multisets to prevent sets from concealing duplicate generation.

### Phase 4: Cargo Benchmarks & Throughput Gates

* [ ] **Reference Machine Profile:** Define the performance baseline using recorded CPU/OS/Rust metadata on a stable reference machine. GitHub-hosted runners are used only for correctness or benchmark trends, not hard latency gates.
* [ ] **Stage 1 (Allocation Instrumentation):** Set up allocation instrumentation separately from timing benchmarks. Track maximum generated-action/allocation counts for a 14-card fixture.
* [ ] **Stage 2 (Component Benchmarks):** Use `push_sim/benches/*.rs` or `src/bin/bench.rs` for timing validation, play generation, and hit generation independently on the 14-card fixture. Record median and p95/p99 tail latency. Establish numeric throughput gates *after* this component profiling.
* [ ] **Stage 3 (Throughput Gate):** Run five distinct batches of 1,000 headless games. Confirm the post-profiling numeric gates are reliably met.
* [ ] **Safely Remove Fallback:** Only after the Stage 3 throughput gate reliably passes, remove the `if state.players[actor].hand.len() > 11` fallback from `random_bot.rs`. Explicitly require a full `cargo test -p push_core` integration-chain run and a benchmark rerun to guarantee zero regressions.
