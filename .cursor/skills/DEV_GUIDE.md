# DEV_GUIDE — Project

## Tech stack

**You define it.** This template is stack-agnostic. Document languages, frameworks, and runtime here as you adopt them (for example: TypeScript + npm for an Edge extension, Rust, Python, etc.).

## Architecture

Describe how you organize source, tests, and integration or E2E assets. Point to real paths in this repo (for example `src/`, `extension/`, `tests/`).

## Conventions

- Prefer pure functions for business logic where possible.
- See AGENT_HANDOFF.md for run/test commands and source of truth.

### Compile counter (only if this repo compiles)

If merge-ready **compiles**, count every compile (test/Debug **and** Release). Gitignored `build_number.txt` (seed `build_number.txt.example` = `0`); inject `+N` into the binary informational version when the toolchain allows. Skip IDE/design-time. **Skip this whole section** for scripts, bookmarklets, twitchurl*, DJ tools, docs-only.

C# reference implementations: DerailValleyModv2, LurkWall.

### Evidence sink (only if this repo has a runtime)

Named events go in a **durable file** documented in TEST_PLAN.md. Human opens it; agent Reads it. Debugger-only Trace is not evidence. **Skip** for this template, docs-only, and stacks with no runtime.

C# references: DerailValleyModv2 (`Player.log` `T2 …`), LurkWall (`%LocalAppData%\LurkWall\lurkwall.log`).

---

## When you adopt Python (JIT — add when a project needs it)

Do **not** add Python scaffolding to this template preemptively. When a real Python project uses AgenticTemplate, document **that project's** layout in its own `DEV_GUIDE.md` and wire commands into `TEST_PLAN.md` / `AGENT_HANDOFF.md`.

Patterns proven in production (add only what the project uses):

| Need | JIT addition | Notes |
|------|--------------|-------|
| Tests | `pytest` + `tests/` + `pyproject.toml` with `[tool.pytest.ini_options] pythonpath` | Run as `python -m pytest -q` (works when `pytest` is not on PATH) |
| Dev deps | `requirements-dev.txt` or `[project.optional-dependencies] dev` | Keep runtime deps minimal |
| CI | `.github/workflows/ci.yml` running the same command as local merge-ready | Verify with `gh run watch` after push — see github-feature-workflow skill |
| Cross-platform tests | Mock platform keys; set both `HOME` and `USERPROFILE` for tilde tests | Linux CI will not parse `C:\` paths — gate OS-specific tests with `skipif` |

Reference implementation: [dj-library-tools](https://github.com/pbuckles22/dj-library-tools) (CLI, pytest, no PRs, upstream syncs skills from this template).
