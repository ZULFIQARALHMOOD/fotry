# Responsive UI layout

Shared breakpoints, spacing, minimum target sizes and viewport margins live in
[`crates/ui-egui/src/layout.rs`](../crates/ui-egui/src/layout.rs). Use those helpers instead of
adding per-widget width thresholds.

## Buttons

- Make the visible control and its hit target fit the label; on touch devices use
  `layout::target_size` so the hit target is at least 44 points in both dimensions.
- Long or translated labels must wrap or truncate, and truncated labels need a tooltip with the
  complete text. Do not place labels with unbounded `layout_no_wrap` text.
- Put ribbon commands in the existing horizontally scrollable ribbon so overflow remains reachable.

## Panels

- Desktop panes use logical leading/trailing sides so their placement mirrors in RTL.
- At compact widths, panels must use the bounded overlay drawer pattern in `panes.rs`; don't add a
  fixed-width side panel that steals the document's remaining width.
- Scroll panel content vertically and constrain the drawer to `layout::drawer_size`.

## Dialogs and popups

- Constrain dialogs to `layout::dialog_area` and put long content in an internal scroll area.
- Use egui's constrained popup APIs where possible. For manually positioned popups, use
  `layout::popup_position` to flip at viewport edges and clamp the result.
- Keep all popup/dialog dimensions derived from the current content rectangle, which changes when
  the browser resizes for the on-screen keyboard.
- The web shell uses dynamic viewport units and safe-area insets; don't position app content from
  `100vh` or the physical screen edges.

The layout tests render LTR and RTL at 360×640, 768×1024, 1024×768 and 1920×1080 and check the
chrome, dialog bounds and clipping.
