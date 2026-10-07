# Handoff (optional, local)

Files inside `doc/handoff/` and `.cursor/handoff/` are gitignored. `README.md` stays tracked, and `.cursor/handoff/_template.md` stays tracked, so the directories stay in git.

## Tracked source of truth (norm)

1. [CONTRIBUTING.md](../CONTRIBUTING.md)
2. [PROJECT_STATUS.md](../PROJECT_STATUS.md)
3. [PM_PLAN.md](../PM_PLAN.md)

Update **PROJECT_STATUS.md** and [AGENT_HANDOFF.md](../AGENT_HANDOFF.md) when phases ship — in the same PR as the code.

Every Receiver brief includes this line verbatim, between **Next steps** and **Measured**: `write thorough tests. red, integration, find missing ones of both kinds.`
