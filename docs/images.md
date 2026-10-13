# Image inventory and provenance

Images have one of three roles. Their role and version must be visible in the
surrounding text; a picture cannot establish behavior beyond the recorded tests.

| Role | Source | Meaning |
| --- | --- | --- |
| Current layout, illustrated during v0.0.5 development | [English](illustrations/v0.0.5/main-en.svg) · [中文](illustrations/v0.0.5/main-zh.svg) · [日本語](illustrations/v0.0.5/main-ja.svg) | Reused for the unchanged v0.0.6 layout; paired light/dark **schematics**, not native screenshots or acceptance evidence |
| Product logo / pinned icon sources | [PPS logo](../assets/icon.svg) · [Lucide](../assets/lucide/README.md) · [Lobe brand marks](../assets/clients/README.md) | Canonical source assets; application icons and native paths are derived |
| v0.0.6 native CI captures | [Active row](screenshots/v0.0.6/aarch64-apple-darwin/zh-active.png) · [Add](screenshots/v0.0.6/aarch64-apple-darwin/zh-add.png) · [Settings](screenshots/v0.0.6/aarch64-apple-darwin/zh-settings.png) · [Dark](screenshots/v0.0.6/aarch64-apple-darwin/dark-zh-active.png) · [Provenance](screenshots/v0.0.6/aarch64-apple-darwin/provenance.json) | Isolated fake-provider fixtures; lossless PNG conversions, no retouching. Native automated checks are separate from local physical desktop acceptance. |
| Historical capture evidence | [Versioned captures](screenshots) · [Hash inventory](images.json) | Recorded candidates only; original paths, bytes, labels and colors retained |

## Current illustrations

[Generator](../design/render_docs.py) reads the actual theme palette and imports
pinned SVG shapes. Run `python3 design/render_docs.py` with the existing development
fonttools dependency; `--check` checks reproducibility. [Source/image hashes](illustrations/v0.0.5/manifest.json)
record provenance. German README uses the English illustration, matching the
available UI languages; Traditional Chinese remains covered by native style gates.

The diagram shows five compact tabs, fixed Official row, per-row Edit/Switch and
blue In use/使用中. Sample URLs use `.example`; no credentials are shown. Native
OS chrome is omitted. Rendering a diagram in a headless browser is documentation
review only; it does not certify the application, text clipping or native input.

## Historical images

| Group | Provenance / limitation |
| --- | --- |
| Unversioned `assets/screenshot*.png` | Legacy v0.0.3 gallery, linked only by [archived verification](history/verification-through-0.0.4.md#screenshots); removed from current README heroes |
| `screenshots/v0.0.4/` | Timestamped native candidates, including retired More/footer/import states; [archived acceptance](history/acceptance-0.0.4.md) identifies final released captures |
| `screenshots/v0.0.5/` | Early three-harness design/title/import review, before model/Official/blue-status changes; [archived checkpoint](history/v0.0.5-checkpoints-20261013.md#behavior-and-appearance) |

The older green badges remain only as historical evidence. They are not tokens
in the current design system. The obsolete interactive design page is [archived](history/minimal-0.0.4.html).
The duplicate logo draft was byte-identical and removed; `assets/icon.svg` remains
canonical. Keep actual app icons and source licenses. Do not regenerate or retouch
historical capture evidence to make it resemble the latest UI.
