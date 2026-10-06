# Test plan — PushSimulator

## Tier 1: Fast feedback

```bash
cargo test -p push_core
```

Optional full workspace:

```bash
cargo test
```

---

## Integration chain (1..X together)

Headless tests in `push_core` that run shipped behaviors **together** through real engine APIs. Rule: [.cursor/rules/integration-chain.mdc](.cursor/rules/integration-chain.mdc).

Suggested after Epic 1.8: Epic 1.9 chains (see [doc/PLAN_COMMENTARY.md](doc/PLAN_COMMENTARY.md)). Until then, Crucible scenarios in BACKLOG Epic 1.8 are the first multi-step suites.

```bash
cargo test -p push_core
```

---

## Tier 2: Integration / E2E

| Surface | When | Command / check |
|---------|------|-----------------|
| WASM viewer | Phase 1b | Build with `wasm-pack`, serve `viewer/`, smoke Load WASM + bot play |
| iOS | Phase 3 | Xcode + UniFFI smoke |
| Multiplayer | Phase 4 | WS handshake + room round-trip |

```bash
wasm-pack build push_wasm --target web --out-dir ../viewer/pkg
```

---

**Handoff:** Commands mirrored in [AGENT_HANDOFF.md](AGENT_HANDOFF.md).

**Compile counter:** Count every merge-ready compile (test/Debug and Release). Local `build_number.txt` (gitignored).

**Evidence sink:** TBD when a durable runtime log exists. Until then, CI/`cargo test` is the gate. Optional viewer hook: `viewer/test_status.json` for live red/green panel.
