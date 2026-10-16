# Pixel arrow buttons

User-approved on 2026-10-14. Reuse this design for future standalone directional
arrow controls unless the user requests otherwise. Do not replace it with
Unicode arrows or ordinary rectangular button chrome.

Reference implementation: RotationArrow in
apps/desktop/src/components/SkinPreview.tsx and .skin-rotate-arrow in
apps/desktop/src/styles/app.css.

- Stepped pixel silhouette with a near-black outline (#080808).
- Main fill is gray (#888); the lower diagonal has a darker bevel (#444).
- White highlights appear only on the topmost pixel and upper diagonal.
  The straight vertical edge stays the same gray as the body, with no white strip.
- Mirror the same geometry for left and right arrows.
- Render SVG with crispEdges. Current geometry is a 9 by 13 viewBox, displayed
  at 27 by 39 CSS pixels in a 44 by 48 pixel hit target.
- Transparent background. Mouse hover, keyboard focus, and controller focus
  must not introduce a square border, outline, background, or box shadow.
- Hover/focus brighten only the arrow (brightness 1.2).
- Controller focus replaces the existing black silhouette outline with white
  (#fff). Keep its geometry and thickness unchanged; do not add an inner ring,
  exterior stroke, or rectangular frame. Restore black when focus leaves.
- Pressing shifts the arrow down 2 pixels. Disabled controls do not activate.
- Keep accessible directional labels and the shared Control navigation wiring;
  decorative SVGs are hidden from accessibility APIs.
