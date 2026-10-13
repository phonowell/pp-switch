# Smoke acceptance

[Checkpoint](development-0.0.5.md) · [UI rules](../design/UI.md) ·
[Incremental verification](../CONTRIBUTING.md#incremental-verification)

## v0.0.5 acceptance matrix

Use isolated files and fake keys. Simplified Chinese covers functionality;
English, Traditional Chinese and Japanese cover style/property/pixels.
Keep this shared Mac's foreground input available to its user.

| Gate | Current scope / status |
| --- | --- |
| Format, all-target Clippy, Rust behavior, release bins/examples | [Default correction checks](measurements/v0.0.5/model-default-20261013/code-checks.json): 82 Rust / 18 Python tests; prior [prefill checks](measurements/v0.0.5/model-prefill-20261013/code-checks.json) and [rechecks](measurements/v0.0.5/model-prefill-20261013/rechecks.json) retain the earlier protection failure |
| Client writes, defaults, names, CCS/Sub2API, five Official routes | Isolated behavior tests; [previous baseline](measurements/v0.0.5/official-20261013/report.json) |
| Cocoa menu and Dock-format quit AppleEvent | Six real FLTK main/Settings/editor lifecycle scenarios; hidden native windows, no ordering or activation; missing-handler negative control |
| Actual Dock Quit with main/settings/editor windows | Pending dedicated native desktop |
| Claude Add/Edit/import → actual default; clear → direct URL/key Switch | Background production callbacks; Chinese native input pending |
| Default prefill: Add/Edit/import/Save/Cancel; clear then Switch | [Release background callbacks](measurements/v0.0.5/model-default-20261013/release-prefill.log) and read-only connection fixtures passed; pointer/pixel acceptance pending |
| Official/native ↔ Provider across five tabs | Pending Chinese native input; fixture write behavior separate |
| Tabs, blue text/badges, overflow/scroll, Settings/editor | Pending four-language, both-theme native pixel captures |
| Package, architecture, checksum, ≤5 MiB owned files | [Exact archive validated and installed](measurements/v0.0.5/model-default-20261013/report.json), 2,788,557 owned bytes |
| Diagrams, Markdown, links, translated usage, image provenance | Deterministic SVG parity and documentation checks |
| RSS, peak, cold interactive startup, update peak | Latest binary unmeasured; earlier memory failures disclosed |
| Windows/Linux/Intel native branch acceptance | Pending respective hosts |

Lifecycle tests are not pointer or visual/input acceptance. Illustrations cannot
substitute for native captures. Retain failures and append targeted corrections;
reuse only gates whose inputs are unchanged.

## Published v0.0.4 acceptance

[Archived matrix, hashes and captures](history/acceptance-0.0.4.md) ·
[Release evidence](verification.md#release-004). These apply to v0.0.4 only.
