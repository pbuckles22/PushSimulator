---
name: visual-match
description: Match UI graphics to reference examples (examples/ screenshots and specs). Use when drawing or changing visuals so the result matches the examples without back-and-forth.
---

# Visual match — Project

Use this skill when implementing or changing UI graphics (layout, grid, colors, strokes, icons) so the result **matches the reference examples** in `examples/` (screenshots, PDFs, specs). Ensures consistent look and alignment with design.

---

## When to apply

- Drawing or changing custom graphics (canvas, DOM, native views, or your UI toolkit).
- Changing layout, grid, background color, or palette.
- User says graphics "don't match the examples" or "make it look like the examples."

---

## Reference look

1. **Background** — Per spec or examples (e.g. solid color, contrast).
2. **Grid / structure** — Alignment, spacing, dots or lines per reference.
3. **Paths / shapes** — Stroke width, dot-to-dot or shape rules from examples.
4. **Colors** — Palette from design or examples; one source of truth.

---

## Implementation rules

- **Layout** — Sizing and scaling so content fits and tap coordinates match (logical space).
- **Stroke / line** — Width and style per reference; primary visual is the line/shape, not decoration.
- **Alignment** — Elements start/end on grid points or specified anchors.
- **Background** — Solid or specified; no unintended gradients unless in examples.

---

## Output

- **PASS** — Rendered UI matches the reference look.
- **FAIL** — Wrong proportions, colors, or layout; apply rules above and re-render.
