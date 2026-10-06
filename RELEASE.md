## Release / merge discipline (lightweight)

Keep releases/merges boring and reversible.

### Merge-ready (minimum)

Document your real gate in `AGENT_HANDOFF.md` and `TEST_PLAN.md`, then treat it as mandatory:

- Tier 1 is green (fast feedback)
- If the stack compiles: every documented compile ran (test/Debug **and** Release) and the local +BUILD counter incremented; skip the counter when nothing compiles
- Tier 2 is run when behavior demands integration/E2E validation
- Tracked docs updated when workflow/expectations change
- Rollback path is clear (a revert commit is usually sufficient)

### Version

Default for a product with `PM_PLAN`: `MAJOR.EPIC.COUNTER` plus optional `.FIX`. The counter is ships inside the epic, not the story id. Never go backwards. See [doc/VERSIONING.md](doc/VERSIONING.md).

Libraries may use SemVer. Ops scripts may use a date. Write the choice here when you adopt one. Local `+BUILD` stays gitignored.

GitHub Release: on **SWAT**, after that close is on `main`, when this file says the product cuts epic releases. Do not upload to a store until a product doc says it is distributable.

### Rollback

- Prefer a single revert commit per change
- If a change affects your “stable” line, revert immediately and re-run the required validation tier(s)
