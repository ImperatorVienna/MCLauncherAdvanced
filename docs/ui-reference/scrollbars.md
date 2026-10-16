# Scrollbars

Use the Minecraft in-game scrollbar reference approved on 2026-10-14 for pages,
dialogs, dropdown lists, and tabs.

- Pure black track, no surrounding bevel or border.
- Square light-gray thumb (#c0c0c0), flush across the track width.
- Dark-gray (#808080) right and bottom inset shading, 2 CSS pixels.
- 12 CSS pixel track width (or height for horizontal scrollbars).
- Minimum thumb length 32 CSS pixels; actual length follows native scroll extent.
- No arrow buttons, rounded corners, or inset black gutters around the thumb.
- Hover lightens the face to #d0d0d0; dragging darkens it to #b0b0b0.
- Preserve native wheel, dragging, keyboard, and controller scroll behavior.
- WebKit/Blink use scrollbar pseudo-elements; other engines retain native
  geometry with the same light-gray-on-black color fallback.

Reference implementation: scrollbar rules in apps/desktop/src/styles/app.css.
