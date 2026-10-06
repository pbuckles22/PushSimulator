# Push: Data Structures & State Machine

**Status:** Placeholder — source file was missing.

The Google export named `Push_ Data Structures & State Machine.md` contained a duplicate of the Official Rulebook, not a data schema. The real schema is expected to define:

- `Card` (id, suit, rank, locked_until_turn)
- `Deck`, `Player`, `Meld`, `GameState`
- `Action` enum and `TurnPhase` state machine
- Round requirements and scoring table

See [GAME_RULES.md](GAME_RULES.md) for domain rules and [../BACKLOG.md](../BACKLOG.md) Epic 1.1+ for the TDD checklist that implies the schema. Rebuild this doc when the schema is authored (do not invent fields beyond the backlog until then).
