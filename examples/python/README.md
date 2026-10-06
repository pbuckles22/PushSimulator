# Python (JIT)

This folder exists as a **pointer**, not a boilerplate. AgenticTemplate stays stack-agnostic until a project actually needs Python.

When you adopt Python in a variant repo:

1. Document layout and commands in that repo's `.cursor/skills/DEV_GUIDE.md`.
2. Fill in `TEST_PLAN.md` and `AGENT_HANDOFF.md` with real merge-ready commands (e.g. `python -m pytest -q`).
3. Add CI only when the project has tests — same command as local gate.
4. See **When you adopt Python (JIT)** in [.cursor/skills/DEV_GUIDE.md](../../.cursor/skills/DEV_GUIDE.md).

Working example: [dj-library-tools](https://github.com/pbuckles22/dj-library-tools).
