# Contributing to PushSimulator

Tracked documentation is the source of truth — not chat history alone.

## Start here (reading order)

1. [README.md](README.md)
2. **[doc/PROJECT_STATUS.md](doc/PROJECT_STATUS.md)**
3. [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md) · [doc/ARCHITECTURE.md](doc/ARCHITECTURE.md)
4. [PM_PLAN.md](PM_PLAN.md) · [doc/sprints/SPRINT_1.md](doc/sprints/SPRINT_1.md)
5. [AGENT_HANDOFF.md](AGENT_HANDOFF.md) — `cargo test`, wasm-pack
6. [TEST_PLAN.md](TEST_PLAN.md)
7. [doc/requirements/GAME_RULES.md](doc/requirements/GAME_RULES.md)

Ship commands: **UCPH**, **CMPH**, **CMPHD**, **SWAT** — [.cursor/rules/wrap-on-command.mdc](.cursor/rules/wrap-on-command.mdc).

## Local session notes vs GitHub

| Location | On GitHub? | Purpose |
|----------|------------|---------|
| `doc/PROJECT_STATUS.md` | **Yes** | Current state |
| `AGENT_HANDOFF.md` → Current state | **Yes** | Agent snapshot |
| `PM_PLAN.md` | **Yes** | Phases |
| `.cursor/handoff/*` except `README.md` and `_template.md` | **No** | Local diary. The directory stays. |
| `doc/handoff/*` except `README.md` | **No** | Same |

## Development setup

```bash
cd PushSimulator
# Install Rust: https://rustup.rs
cargo test -p push_core
```

Merge-ready gate: `cargo test -p push_core` (expand as the workspace grows). Compiling stack — count every compile (`testing.mdc` → *Compile counter*).

## Pull request expectations

- [ ] Scope matches PROJECT_STATUS / Sprint 1 / BACKLOG
- [ ] Gate green
- [ ] No secrets
- [ ] Milestone ships update PROJECT_STATUS + AGENT_HANDOFF Current state
