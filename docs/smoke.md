# Smoke acceptance

## Working-tree update review — 2026-10-14

[Review resolutions and evidence](update-review-20261014.md): background format,
all-target Clippy, 112 Rust tests, 30 script tests, release build, documentation,
icon parity and package/archive validation passed, including targeted rechecks.
The new updater lifecycle and release-notes confirmation require fresh Chinese
native acceptance and four-language light/dark style/pixel review. These desktop
gates were skipped to preserve shared-workstation input; published-release GUI
results below do not certify the changed callbacks/dialog.

[Published evidence](verification.md#release-006) · [UI rules](../design/UI.md) ·
[Incremental verification](../CONTRIBUTING.md#incremental-verification)

## v0.0.6 acceptance matrix

Use isolated files and fake keys. Simplified Chinese covers functionality;
English, Traditional Chinese and Japanese cover style/property/pixels.
Native checks ran in [dedicated CI](https://github.com/phonowell/pp-switch/actions/runs/38393449261), preserving this Mac's foreground input.
The [revision-2 reassessment](verification.md#performance-budget-revision-2)
changes performance acceptance only; the release's original reports remain intact.

| Gate | Status |
| --- | --- |
| Format, all-target Clippy, behavior, release bins/examples | Passed on all four native targets |
| Models, five Official routes, names, CCS/Sub2API | Isolated behavior tests passed |
| Chinese native mouse/key, per-row actions, Settings and model fields | Passed on both macOS targets |
| Cold/warm Provider URL delivery | Passed on both macOS targets |
| Menu and Dock-format quit across main/Settings/editor | Real native lifecycle checks passed; actual Dock pointer click unverified |
| Four-language light/dark styles, overflow, scrolling and blue badges | Pixel/property checks passed on both macOS targets; selected captures visually reviewed |
| Package architecture, size, checksum and update installation/restart | Passed for all four final native archives; public checks recorded in verification |
| Bare Cocoa comparison, native action readiness and bounded import stress | Samples retained on both macOS targets; cold start unverified |
| Routine RSS, peak and cached readiness (PP-11 revision 2) | Retained v0.0.6 samples pass retrospective reassessment; no new measurements |
| Strict cold interactive startup and update peak | Unverified separate investigations; original revision-1 failures retained |
| Documentation and illustrations | Markdown/link/translation and deterministic SVG checks passed |
| Local physical Windows/Linux/Intel desktop | Unverified; CI is separate evidence |

Illustrations remain versioned v0.0.5 schematics of the unchanged layout, not native
captures. Lifecycle tests do not certify actual Dock pointer clicks. Retain earlier
failures and rechecks; only unchanged inputs may reuse evidence.

## Published v0.0.4 acceptance

[Archived matrix, hashes and captures](history/acceptance-0.0.4.md) ·
[Release evidence](verification.md#release-004). These apply to v0.0.4 only.
