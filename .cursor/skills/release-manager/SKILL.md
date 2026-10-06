---
name: release-manager
description: Establish lightweight release discipline: definition of merge-ready, versioning/changelog habits, and safe rollout/rollback practices.
---

# Release Manager — Ship Without Chaos

Use this skill when:
- You’re preparing to merge a feature branch
- You want to declare work “merge-ready” or “release-ready”
- You need consistent changelogs / release notes

Goal: make shipping predictable with minimal ceremony.

---

## Merge-ready checklist (generic)

1. **Scope**: change matches the intended branch/feature scope.
2. **Green**: evals/tests required by the repo are passing (Tier 1 minimum; Tier 2 if behavior demands it).
3. **Docs**: runbook/README/TEST_PLAN updates are in place if behavior changed.
4. **Rollback**: a safe revert path exists (single revert commit is ideal).

---

## Release notes template (short)

- **Summary**: 1–3 bullets of what changed and why
- **Risk**: 0–2 bullets (what could go wrong, how to detect)
- **Rollback**: how to undo (revert commit / restore previous version)

---

## Versioning (choose one and stick to it)

- **Plan coordinate** (default when a product has `PM_PLAN` and a user-visible build): `MAJOR.EPIC.COUNTER` plus optional `.FIX`. The counter is ships inside the epic, not the story id. Never go backwards. See [doc/VERSIONING.md](../../../doc/VERSIONING.md) and [pm-versioning.mdc](../../rules/pm-versioning.mdc).
- **SemVer** (libraries and tools): MAJOR.MINOR.PATCH
- **Date-based** (internal ops scripts): YYYY.MM.DD[.N]

Document the choice in `RELEASE.md`. A user-visible build must not go backwards under any scheme. Local `+BUILD` stays gitignored and out of the public version.

---

## GitHub operations (optional): GitHub CLI (`gh`) playbook

Use this section when you need boring, repeatable GitHub mechanics. Prefer **`gh`** over ad-hoc UI steps when it’s available.

### Safety / hygiene (always)

- Treat **`gh`/git output as sensitive** if it includes tokens or private URLs.
- Before publishing anything, run a quick **security-reviewer** pass if the change touched credentials, generated files, logs, or remote access.
- If a push fails with “too large” / HTTP 500 / unexpected disconnect, assume **history contains large blobs** until proven otherwise (`.gitignore` does not fix history).

### Common flows (examples — adjust names)

**Confirm auth**

```bash
gh auth status
```

**Create a new repo + push an existing local repo**

```bash
gh repo create OWNER/REPO --private --source . --remote origin --push
```

If you already have commits locally, this should set `origin` and push your current branch. If `origin` already exists, use `git remote -v` and push normally.

**Push all branches (after a clean history rewrite)**

```bash
git push -u origin --all
```

**Open a PR from the current branch**

```bash
gh pr create --fill
```

**Create a GitHub Release (on SWAT, after `main`, when RELEASE.md says this product cuts epic releases)**

```bash
gh release create v1.2.3 --notes-file RELEASE_NOTES.md
```

### History rewrite note (high risk / high leverage)

If you removed large files from history (e.g., ISOs), remember:

- Commit hashes change; anyone with old clones must **re-clone** or hard-reset.
- Coordinate `main`/release branches and CI expectations after rewrite.

