# Checkbox design

Use this design by default for future checkboxes unless the user explicitly requests a different style. This is a separate control family from ordinary colored buttons, sidebar buttons, and dropdowns. Final geometry and pressed appearance approved in the launcher preview on 2026-10-14. This revision supersedes the earlier downward-offset pressed treatment.

Reference implementation: `apps/desktop/src/components/Checkbox.tsx` and the `.launch-checkbox` rules in `apps/desktop/src/styles/app.css`. Reuse the established styling rather than introducing a separate appearance for each page.

## Geometry and colors

| Element / state | Specification |
| --- | --- |
| Square | 22 × 22 CSS px, border-box, square corners, 2px border |
| Unchecked interior | `#171615`; inset shadow `2px 2px #0008` |
| Default border, checked or unchecked | `#8B8581`; never green merely because checked |
| Checked interior | `#3C8527`, matching the ordinary green button |
| Center geometry | 18 × 18px interior with symmetric 2px bevels leaves a 14 × 14px square center in both checked and pressed states |
| Checked bevel | Inset top/left `#639D52`, bottom/right `#2A641C`, 2px |
| Pointer hover / keyboard-visible focus / controller focus | White `#FFFFFF` border on the square only |
| Checked press | Interior `#2A641C`; inset top/left `#407433`, bottom/right `#558349`; fixed square geometry, no translation or height reduction |
| Disabled | 50% opacity; no activation |
| Label layout | Inherited typography/color; 8px gap from square; 8px control padding; transparent background |

## State and interaction rules

- Checked state is represented by green fill only. Do not draw a tick, checkmark, or other glyph.
- Hovering or focusing must not illuminate the whole label/control, add a whole-control outline, or change the label color.
- Keep the dark-green center square and aligned: no top gap, extra light-green strip, vertical translation, or asymmetric inset. Express depression with color and bevel changes only.
- The checked square itself has a green backing with no unchecked inset shadow; clip the decorative fill inside it so pressing cannot reveal a black seam.
- Keep the border and label stationary during a press; only the green interior darkens and reverses its bevel to appear recessed. Releasing an activation that unchecks the box restores the empty dark interior.
- Keep a single focusable control with an accessible name, `role="checkbox"`, and `aria-checked`. The decorative square is hidden from accessibility APIs.
- Preserve the shared navigation registration, controller confirmation, keyboard activation, disabled behavior, and existing focus boundaries. Styling must not add a separate input/navigation handler.
- Current pressed styling uses CSS `:active`. Programmatic controller `.click()` toggles the state but does not guarantee a visible held-press animation; do not claim hardware press animation has been verified.
