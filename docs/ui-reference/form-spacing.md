# Form and settings spacing

User-approved on 2026-10-14. Apply this standard to all comparable settings,
management pages and forms, including future components and revisions.

- Use 16 CSS px between labels, explanatory paragraphs, control rows and adjacent
  settings groups. Use the shared `--settings-gap` token from app.css :root.
- Parent grid/flex `gap` owns the spacing. Reset paragraph and section margins;
  do not add child margins or padding on top of the parent's gap.
- Nested groups use the same gap. Their outer edges add no extra spacing, so
  crossing a group boundary has the same interval as rows inside a group.
- Place related action buttons to the right of their input or dropdown, centered
  vertically, with the same 16 CSS px gap. Preserve that gap when rows wrap.
- Keep dropdowns and ordinary input controls at the established compact width
  (300 CSS px, constrained by available width), rather than stretching full width.
- Measure between layout boxes, not glyph edges. Text line height and existing
  button bevels/press effects remain independent; do not offset them to compensate.
- Preserve shared keyboard/controller navigation and focus behavior.

Reference: .instances-page, .installation-details, .loader-settings,
.java-settings and .launch-settings in apps/desktop/src/styles/app.css.
Verify computed row gaps after changes, including wrapped text and narrow views.
This standard concerns form layout, not unrelated page padding or artwork geometry.

## Cross-page audit (2026-10-16)

- Error/status text inside a form has no independent padding. Page-level notices
  own the standard page inset; nested notices align with their associated control.
- Settings, account/content management, skins and placeholder pages use the same
  page inset. Dialogs and nested forms have zero paragraph margins and one shared
  gap, including delete confirmations and log viewers.
- Search labels occupy their own grid row; the input and action share a centered
  control row. Empty catalog containers add no gap. No-result/error first pages
  omit useless paging controls; pagination in flight preserves its focus owner.
- Content file actions reserve both update/delete columns, even when Update is
  absent. Long names wrap within the name column. Catalog rows may wrap without
  overflowing at the minimum supported viewport.
- Skin preview and action groups use the shared gap; arrow artwork remains the
  existing pixel design. Normal button bevel space is deliberately preserved.

## External and quoted content

Official articles/news, release announcements, resource descriptions and legal
notices follow the same local typography and spacing system. Preserve source
wording and semantic structure, but never import remote fonts or presentation.
Use `--ui-font` for prose and `--settings-gap` for section/paragraph separation;
compact list items may use 8px. Long identifiers and URLs must wrap. Tables must
fit the content width. Preserve original license text and credits. All links
must use registered navigation controls; long documents keep a registered scroll
owner with consistent keyboard/controller focus and close behavior.
