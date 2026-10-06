# Versioning

The public version is the string a person checks to see that the build they are running is the one just produced. Agent rule: [.cursor/rules/pm-versioning.mdc](../.cursor/rules/pm-versioning.mdc). Record the scheme in [RELEASE.md](../RELEASE.md).

## Default for a product with PM_PLAN

`MAJOR.EPIC.COUNTER` and an optional `.FIX`.

| Segment | Meaning | When it changes |
|---------|---------|-----------------|
| **MAJOR** | Breaking generation named in RELEASE.md | Rare. Not every epic. |
| **EPIC** | Epic number in PM_PLAN | Moving to another epic |
| **COUNTER** | Ship count inside that epic | Every user-visible ship. **Not** the story id |
| **FIX** | Correction before the next ship | `.1`, `.2`, … |

**The number must never go backwards.**

Early epics often ship once per story, so the counter and the story id match. That is pace, not the rule. When one story takes many ships, the counter keeps climbing and the story id stays in PM_PLAN.

Worked example: story **4.2** opened at `1.4.2`, then later ships in that same story became `1.4.8`. Story **4.3** ships as **`1.4.9`**. `1.4.3` would roll the visible number backwards.

## When to bump

| Event | Public version | PM_PLAN |
|-------|----------------|---------|
| User-visible ship of a finished story | Raise the epic counter | Mark `[x]` in the **same** change |
| Mid-story ship, story still open | Raise the epic counter | Leave the story open |
| Fix before the next ship | Append `.1`, `.2`, … | Do not check off the next story |
| Docs or rules with no story id | No bump | No fake checkbox |

## Other schemes

Libraries may use SemVer. Ops scripts may use a date. Write the choice in RELEASE.md. A user-visible build still must not go backwards.

Local `+BUILD` (gitignored `build_number.txt`, informational version only) is not the public version. Do not commit it.

## Releases

Private or smoke builds: every user-visible ship.

GitHub Release: on epic close (**SWAT**), after that close is on `main`, when RELEASE.md says this product cuts releases. Tag the public version. Do not wait to be asked. Do not upload to a store until the product doc says it is distributable.
