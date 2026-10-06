---
name: game-readiness
description: Validates a game or interactive product for playability, clarity, feedback, and scale-readiness. Use at milestones, before large scope changes, or when the user asks if the product feels ready or is ready to scale.
---

# Game Readiness — Project

Validates that the product feels **ready** (not just correct) and is **ready to scale** without costly rework. Apply at milestones or before major scope. Useful for games, puzzles, or any interactive product.

---

## When to apply

- User asks: "Is this ready?", "Ready to scale?", "Does this feel right?"
- Before locking large UX or content scope.
- At sprint or milestone boundaries when "done" needs a playability/readiness check.

---

## 1. Clarity and feedback

- **Goal visible** — User can understand the primary goal quickly (e.g. first 30 seconds).
- **Feedback** — Every meaningful action has clear feedback (visual and/or haptic).
- **Fairness** — No dead ends or confusing failure modes.

---

## 2. Feel (not dry or academic)

- **Responsiveness** — Input → reaction feels immediate.
- **Reward on success** — Success state feels like a win, not "dialog closed."
- **No clutter** — UI supports the core experience.

---

## 3. Scale-readiness

- **Pipeline** — Content/levels (if any) are validated (e.g. solvability, consistency).
- **Performance** — No blocking on UI thread; acceptable frame time.
- **Progress & persistence** — Progress (if applicable) is persisted and visible.

---

## Output format

Produce a short report: **Clarity** PASS/WARN/FAIL, **Feedback** PASS/WARN/FAIL, **Scale-readiness** PASS/WARN/FAIL, plus **Actions** (concrete fixes for each WARN/FAIL).
