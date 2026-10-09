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

- **Liveness:** Closed on `main` at `a4708f2`. Seeds 1..=10000 finished in 213.42s with 0 stalls at 8,000 turns. Seed 1 scores 340 and 80.

- Older closes: **1.2.4.2** closed the seeded shuffle, the card-id property test, the uncalled reshuffle arms, the extra `Player::new` chains, CI for `cargo test -p push_core`, and the unused `serde` dependency. Epics 1.4–1.8 added no new Do-first item.

## Accept for now

(Isolated + workaround + revisit trigger.)

- **Docs / Low:** `docs/` is the original Google export and still says an empty draw pile always leaves the discard top. The living rule is `doc/requirements/GAME_RULES.md`. Leave the export as the snapshot.
- **Sim / Low:** `--players` copies seat 1 into every later seat. A table cannot give three seats three different profiles. Revisit when a match needs a mixed field.

---

## ROI rubric (quick)

Score each: Impact (0–2) + Frequency (0–2) + RiskReduction (0–2) + Effort (0–2, reverse scale). Sort descending.

