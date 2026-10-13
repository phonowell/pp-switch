# Native UI failure lessons

Archived toolkit lessons; use the [current UI rules](../../design/UI.md) and [acceptance matrix](../smoke.md) for active gates.


For Choice popups, set `menu_frame` explicitly. `down_frame` affects selection
paint, not the popup window; using it alone loses native corner clipping/shadow.
Capture/inspect the actual popup and assert its native properties. Paint the
parent background before a rounded custom control to avoid stale corner pixels.

More/footer observations below are historical toolkit lessons; those UI
components were removed. Apply relevant lessons to the remaining language popup.

These observations apply to FLTK/fltk-sys 1.5.23 and the tested macOS build.
Verify them again when upgrading the toolkit. Observable behavior is the gate;
source inspection or callback invocation alone is insufficient.

| Failure | Cause and correction | Automated guard |
| --- | --- | --- |
| More ignored left click | `NoBox` makes MenuButton a right-click context menu. Use `FlatBox` with custom drawing. | Actual left-click/Space/Escape events; a NoBox negative control must fail |
| Language marker or update disable state did nothing | This version's `MenuExt::set_mode` binding ignores its item index. Use `at(index).set_flag(flags)`. | Selected radio value and active/inactive native item assertions |
| Rounded drawing still produced a rectangular menu | An opaque rectangular native window remains rectangular. macOS menus need transparent background and layer corner clipping, while retaining `hasShadow`. | Native radius/clipping/shadow assertions, transparent corner and opaque center pixels; border-edge pixels |
| Three dots differed between languages | Use fixed integer-position filled circles rather than a font glyph or outlined floating-point circles. | Compare More interior pixels across all four locales and require filled centers |
| In use was cut off | Edit draws after the row and covered the badge's last 2 pixels. Measure localized text and leave an 8-pixel gap before Edit. | Compare rendered badge ink with a complete same-system reference, including narrower/scrolling rows |
| Screenshot crashed input_smoke | Escape closes the whole popup; a second Escape can hide its parent. Cocoa `make_current` assumes a mapped window. | One Escape, parent visibility assertion, capture guard and normal error exit |
| Exact-color text checks failed on valid captures | Cocoa text rendering changes paint RGB; antialiased edges also depend on selected/unselected backgrounds. | Compare same-run solid glyph ink silhouettes; test detector sensitivity against missing ink |
| More focused on initial activation | FLTK Group focuses its first interactive child. Accept initial focus at the main window until user input, preserving normal subsequent navigation. | Native initial-focus and Tab/Space assertions in all locale captures / Chinese input |
| Undesigned status dialog/tooltip | An output field had a release callback and copied its text into a tooltip. Replace it with a passive Frame. | Tooltip property and native click window/modal counts; watchdog fails an unexpected nested dialog |

When a personal app hangs, preserve a bounded process sample before ending the
identified process if possible. A main thread waiting inside a nested event loop
does not by itself prove a deadlock or identify its trigger. Record unknown causes
as unresolved; test native title-close and Escape dismissal and parent recovery.
Do not reproduce using the user's real keys/configuration. Duplicate active badges
are intentional: client URL/key cannot identify one saved record among identical
records. Never arbitrarily choose one active ID or silently delete duplicates.

macOS title-icon removal must stay behind `cfg(target_os = "macos")`; Windows/
Linux window icons and installed package icons remain. Popup styling must target
only the application's menu frame, preserving normal native window chrome.
Layer PNGs prove rounded transparency but exclude the external compositor
shadow; native properties supply that part of the evidence.

