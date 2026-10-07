# Handoff notes

**Committed in this repo:** `_template.md` and this README — no secrets.

**Session narratives (usually gitignored):**

- **`doc/handoff/NNNN-HANDOFF-YYYY-MM-DD_HHmm.md`** — prefer. Body = **Receiver brief**. Last line is **Filename**. Shape: [`_template.md`](_template.md).
- **`.cursor/handoff/NNNN-handoff-YYYY-MM-DD_HHmm.md`** — copy of the same brief.

UCPH and CMPH **chat** must paste that brief. **First line:** `You are the agent to execute the below hand off.` Brief body is second person (**You** …). Every brief includes this line verbatim, between **Next steps** and **Measured**: `write thorough tests. red, integration, find missing ones of both kinds.` A git-only note with no chat is incomplete. Gates (PASS/WARN) are SWAT only.

**Naming rules (do not overwrite history):**

- **`NNNN`**: zero-padded monotonic serial (**0001**, **0002**, …) that always increases.
- **`YYYY-MM-DD_HHmm`**: local timestamp in 24h time.
- **Never reuse** an `NNNN` and **never edit** a previous handoff file in place—append a new file.

**Pick the next serial (PowerShell) — `.cursor/handoff/`:**

```powershell
$dir = Join-Path $PSScriptRoot '.'
$next = 1
Get-ChildItem -LiteralPath $dir -Filter '*-handoff-*.md' -File -ErrorAction SilentlyContinue |
  ForEach-Object {
    if ($_.Name -match '^(\d{4})-handoff-\d{4}-\d{2}-\d{2}_\d{4}\.md$') {
      [int]$n = $matches[1]
      if ($n -ge $next) { $next = $n + 1 }
    }
  }
'{0:0000}-handoff-YYYY-MM-DD_HHmm.md' -f $next
```

See [`.gitignore`](../.gitignore). Files in this folder are ignored except `README.md` and `_template.md`. The same rule keeps `doc/handoff/README.md`. Add a `!` exception if another file in either folder should be tracked.

**Product state on GitHub:** [doc/PROJECT_STATUS.md](../../doc/PROJECT_STATUS.md), [PM_PLAN.md](../../PM_PLAN.md), [AGENT_HANDOFF.md](../../AGENT_HANDOFF.md) → *Current state*. **Norm:** [CONTRIBUTING.md](../../CONTRIBUTING.md). Promote session decisions to tracked docs before merge.

Ship cadence is `.cursor/rules/wrap-on-command.mdc`. The review swarm (`.cursor/rules/handoff-checklist.mdc`) runs on SWAT, not on every UCPH or CMPH.
