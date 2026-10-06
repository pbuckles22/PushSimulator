## Technical debt (tracked backlog)

This is the durable home for technical debt across sessions. Handoff notes can mention debt, but anything that persists should be recorded here.

### Cadence

- **Every handoff**: run the tech-debt-evaluator skill and record “Do first” items in the handoff note.
- **Promote persistent debt**: if a “Do first” item persists across 2+ handoffs (or blocks work), add it here and rank it.

---

## Fix now

(Blocking, unsafe, or no-rollback debt.)

- (none)

## Fix soon

(High ROI; frequent pain; not blocking.)

- (none)

## Accept for now

(Isolated + workaround + revisit trigger.)

- **Tests / Low:** `test_deck_shuffle` uses `thread_rng` and asserts the order changed (`push_core/src/deck.rs`). A 108-card identity shuffle is not a practical failure. Revisit if that test flakes.
- **Scope / next:** The two open 1.1.4 lines in `doc/BACKLOG.md` (empty-deck reshuffle from the discard pile, minus the top card) block Epic 1.2. `test_deck_draw_empty` still expects `None` when there is no discard pile to recycle.
- **Code / Low:** `push_core` declares `serde` and does not use it yet. Leave it for a later serialize story.

---

## ROI rubric (quick)

Score each: Impact (0–2) + Frequency (0–2) + RiskReduction (0–2) + Effort (0–2, reverse scale). Sort descending.

