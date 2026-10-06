# Monte Carlo strategy — PushSimulator

**Status:** Host and sample size are locked. Bot profiles are still Phase 2 (see [../BACKLOG.md](../BACKLOG.md) stories 2.1–2.4).

## Where the simulations run

Coding stays on the Windows PC. `push_sim` is a headless Rust binary. Rayon uses whatever cores the host has.

The first CPU host is the Mac. That is not the iOS app. Phase 3 (Xcode, UniFFI, SwiftUI) is a later switch. Phase 2 only needs Rust on the Mac.

1. Install Rust on the Mac once (`rustup`).
2. Pull this repo on the Mac.
3. Build there: `cargo build -p push_sim --release`.
4. Run the binary. Copy the CSV back (win rates, average turns, score spread). Remote Login can start that build and run over SSH.

Build the binary on the Mac. A Windows machine does not produce a Mac executable. A Linux EC2 instance is the later scale host: same crate, same Rayon loop, more cores when a laptop is not enough.

## How many games

One game is a full 5-round match.

| Games per matchup | Use |
|---|---|
| 1,000 | Prove a batch finishes. This is the Rayon test size in story 2.3. |
| 10,000 | First look. A gap of a few win-rate points is visible. |
| 100,000 | Rank bots. Story 2.4. A 1-point win-rate gap should not be noise. |
| 1,000,000 | Only when two bots are still inside half a percentage point. |

A win rate near 50% has a 95% margin of about ±3 points at 1,000 games, ±1 point at 10,000, and ±0.3 points at 100,000. Score spread waits on the first CSV; those win-rate margins are the ones that are known now.
