## Risk register (top 5)

This file is intentionally short. Keep only the highest leverage, current risks.

- **Risk**: Merge-ready gates drift from reality (agents think “green” without running the real gate)  
  **Impact**: High  
  **Likelihood**: Med  
  **Trigger**: PRs merge without the documented Tier 1 command; flaky CI ignored  
  **Mitigation**: keep `AGENT_HANDOFF.md` + `TEST_PLAN.md` aligned; enforce handoff checklist  
  **Rollback**: revert the merge; restore last known-good tag/commit

- **Risk**: Rule-book mistakes stay hidden because Phase 1b was never watched  
  **Impact**: High  
  **Likelihood**: Med  
  **Trigger**: iOS stories continue while no four-seat game has been watched on `viewer/`  
  **Mitigation**: Stories 1b.1–1b.5 in `doc/BACKLOG.md`. A mismatch becomes a `push_core` test. 1b does not block 3.3  
  **Rollback**: Fix the rule in `push_core` and rebuild the iOS xcframework and `push_wasm`

- **Risk**: Context bloat causes agent drift / incorrect assumptions  
  **Impact**: Med  
  **Likelihood**: High  
  **Trigger**: handoff notes exceed ~500 words; repeated long logs in chat; conflicting “truth” sources  
  **Mitigation**: use session-summarizer; keep durable truth in tracked docs; bootstrap with minimal context  
  **Rollback**: create a compressed handoff note; re-establish source-of-truth docs; re-scope work
