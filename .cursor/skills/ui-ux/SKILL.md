---
name: ui-ux
description: UI/UX review and alignment. Use when writing or reviewing screens, animations, haptics, layout, or design tokens; or when adding/changing UX to match project requirements and accessibility.
---

# UI/UX — Project

Use this skill when implementing or reviewing UI: screens, widgets, animations, haptics, layout, colors, tap targets, and feedback. Ensures alignment with doc/requirements/ (if present) and DESIGN_SYSTEM.md.

---

## When to apply

- Adding or changing screens, widgets, or layout.
- Implementing or changing animations or haptics.
- Choosing or changing colors, palette, or appearance.
- User asks for a "UI review," "UX check," or "design compliance."

## Review passes

1. **Requirements alignment** — Does the implementation match doc/requirements or DESIGN_SYSTEM?
2. **Feedback and affordance** — Does every user action have appropriate feedback (visual and/or haptic)?
3. **Accessibility and touch** — Minimum tap target 44×44 px where applicable; contrast and information not by color alone.
4. **Performance** — No blocking work on UI thread.
5. **Edge cases** — Empty state, loading, error states.

## Output

- **PASS** — Matches requirements; no change needed.
- **WARN** — Improvement suggested; not blocking.
- **FAIL** — Does not meet requirement; must fix. Cite requirement and give concrete fix.
