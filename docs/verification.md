# Verification evidence

[Product](../README.md) · [User guide](user-guide.md) · [Spec](../SPEC.md) · [Reproduce](../CONTRIBUTING.md)

## Review candidate: 0.0.4

User accepted the installed redesign on 2026-10-12 and authorized commit,
push and publication. [Acceptance record](measurements/v0.0.4/user-acceptance-20261012.json).
Native release CI and public upload verification are pending; downloads below
remain 0.0.3 until publication completes. Earlier timestamped review records
remain unchanged.

**Settings/row-action redesign accepted.**
The current candidate removes all hover tips, the in-content heading, More,
bottom status/global Switch, Provider-link import and URL handlers/declarations.
Every row has Lucide Edit then Switch; Switch targets that row. The small gear
opens an in-window page for language and strictly manual updates. Matching
duplicates all show In use. Existing providers/configuration are preserved.

Chinese native input passes for the changed flows; other locales receive only
prepared styles. System appearance transitions preserve drafts/cursor/focus/
selection. Final light/dark style, package and installation evidence is recorded
in [smoke acceptance](smoke.md). No claim that the previous unexplained hang is
fixed, no fresh performance claim, and no native certification of these changes
on Windows/Linux/Intel. Unchanged core/update-helper evidence is reused.

### Current redesign package

[Full current record](measurements/v0.0.4/settings-lucide-review-20261012.json) ·
[Installation manifest](measurements/v0.0.4/install-review-20261012T113722Z.json).
Installed/opened in `/Users/mimiko/Applications/pp-switch.app`; old bundle backed up.
ARM64 archive SHA-256 `e1536046f9154ef22e24bab500a27096fb8ee8f2c18832317f48211e43b5d29d`;
2,544,096 installed bytes, 1,349,523 archive bytes. Size budget passes. No new RSS
or startup measurements were taken. [Current main](screenshots/v0.0.4/settings-lucide-20261012/light/zh-active.png) ·
[Current Settings](screenshots/v0.0.4/settings-lucide-20261012/dark/zh-settings.png).

### Historical candidate checks

The following menu/footer/import results describe earlier 2026-10-12 candidates,
not the redesigned UI. Raw reports and screenshots remain immutable.

**Local UI correction passes; release review remains incomplete.** The menu is a
7-pixel rounded rectangle without a border, retaining its native system shadow.
The More glyph uses three fixed filled circles, and active badges reserve space
before Edit. macOS title bars have no PPC icon; Windows/Linux window icons and
installed application icons remain. The native input harness no longer requests
a drawing context for a hidden window or intentionally panics on test failure.

[Smoke matrix](smoke.md) · [UI design](../design/UI.md) ·
[Raw commands/results/hashes](measurements/v0.0.4/menu-review-20261012.json).
Chinese full functionality passed; other languages receive style checks only.
Native assertions cover title icons, menu shadow and rounded clipping; transparent
corner pixels verify the actual shape. Pixel regression gates reject menu borders,
locale-dependent More glyphs and covered badge text. Real pre-fix captures fail
these detectors. Human screenshot review is the final fallback.

The base release run passed format, all-target Clippy, 32 Rust tests, 6 Python
tests, Chinese callback/native input, targeted cold/warm URL delivery,
update-helper wait/install/restart/cleanup, packaging and exact archive checks.
The popup-rounding correction passed incremental format/Clippy/build,
four-language style-only native capture, pixel checks and exact new archive
validation. Unchanged functional/core evidence was reused; it was not rerun.
Actionlint passes. CodeGraph remains unavailable (no index).

The subsequent user review found startup More focus and an undesigned bottom
status dialog/tooltip. Both are corrected. Duplicate records are explicitly
allowed: all URL/key matches display In use, with independent selection/editing.
[Targeted evidence](measurements/v0.0.4/interaction-review-20261012.json) records
incremental format/Clippy/release, Chinese native input for these cases and modal/
title-close recovery, four-language style/pixel checks and exact archive validation.
Unchanged full-flow/core/URL/update/performance evidence is reused. New screenshots:
[startup](screenshots/v0.0.4/interaction-review-20261012/zh-initial-focus.png) ·
[duplicate active rows](screenshots/v0.0.4/interaction-review-20261012/zh-duplicate-active.png).

During that review the previous installed build became unresponsive. The retained
[sample](measurements/v0.0.4/hang-callgraph-20261012.txt) shows waiting in a nested
event loop after mouse release; its stripped addresses do not identify the callback.
The user does not recall the trigger. SIGTERM ended the identified process, with
no residual process. The exact root cause remains unresolved; passing dismissal
checks and removal of the status dialog do not establish a fix for that incident.

[Candidate CI](https://github.com/phonowell/pp-switch/actions/runs/38210816784)
finished: Windows, Linux and macOS ARM passed; macOS Intel failed Native URL events
with `Registration acceptance needs a clean CI user`; release was skipped. That
run tests HEAD, not these working-tree UI changes. The existing handler path was
not printed, so its cause remains unresolved. Four-platform assembly, clean-user
registration/preservation/untargeted-open acceptance and remote upload validation
remain release-preparation limits.

### Local package and measurements

Historical interaction-review ARM64 archive:
`target/package/pp-switch-v0.0.4-aarch64-apple-darwin.zip`, SHA-256
`a0990f681819026f55c1abf526e57e8b3899af1e3e412cbc9b179561b14757c1`.
Installed owned files: 2,557,439 bytes; archive: 1,356,647 bytes. This local review
build was installed and opened at `/Users/mimiko/Applications/pp-switch.app`,
preserving the previous bundle. [Interaction installation](measurements/v0.0.4/install-review-20261012T102057Z.json)
and [earlier installation](measurements/v0.0.4/install-review-20261012T095740Z.json)
retain exact manifests/backups. The old `target/review-v0.0.4/` archive is historical.

Apple M4, 16 GiB, macOS 27.0.1, busy desktop with retained OS caches. The latest
measurements precede the popup/focus/status corrections; they are reused without
making a new performance claim. **Both RSS budgets fail.**

| Scenario | Idle / peak maximum MiB | Readiness median / maximum ms | Evidence |
| --- | ---: | ---: | --- |
| Empty window | 109.50 / 109.53 | 152.70 / 422.37 | [Raw](measurements/v0.0.4/macos-m4-menu-review-empty.json) |
| Import preview | 127.33 / 127.36 | 171.93 / 214.24 | [Raw](measurements/v0.0.4/macos-m4-menu-review-import.json) |

These uncontrolled development-desktop results establish no before/after
improvement. Historical [empty](measurements/v0.0.4/macos-m4-local.json),
[import](measurements/v0.0.4/macos-m4-local-import.json) and
[blocked-session smoke](measurements/v0.0.4/smoke-input-audit-20261012.json)
remain unchanged. Performance acceptance is not complete.

### Candidate screenshots

All use fake data. Layer menu PNGs preserve actual corner transparency; system
shadow is verified by native properties and lies outside these content captures.

| View | Evidence |
| --- | --- |
| Rounded menu | [简体中文](screenshots/v0.0.4/menu-review-20261012/zh-menu.png) · [English](screenshots/v0.0.4/menu-review-20261012/en-menu.png) · [繁體中文](screenshots/v0.0.4/menu-review-20261012/zh-TW-menu.png) · [日本語](screenshots/v0.0.4/menu-review-20261012/ja-menu.png) |
| Mac title bar | [简体中文](screenshots/v0.0.4/menu-review-20261012/zh-chrome.png) · [English](screenshots/v0.0.4/menu-review-20261012/en-chrome.png) |
| Complete active badges | [Four languages](screenshots/v0.0.4/menu-review-20261012/review-active.png) |
| Editor / import | [Edit](screenshots/v0.0.4/menu-review-20261012/review-edit.png) · [Import](screenshots/v0.0.4/menu-review-20261012/review-import.png) |
| Error / delete / update | [Error](screenshots/v0.0.4/menu-review-20261012/review-invalid-url.png) · [Delete](screenshots/v0.0.4/menu-review-20261012/review-delete-confirm.png) · [Update](screenshots/v0.0.4/menu-review-20261012/review-update-confirm.png) |

## Current release: 0.0.3

[Public functional preview](https://github.com/phonowell/pp-switch/releases/tag/v0.0.3),
verified on 2026-10-12. Implementation:
`021f8208e75fc37963b46674c10f89e2e252cfb9`. Release tag points to documentation
commit `911db7b0c598e95d2b63fe2f4353020f98e0ae54`.
[Native CI run 38201751965](https://github.com/phonowell/pp-switch/actions/runs/38201751965).

**Functional checks passed on all four runners. Windows/Linux pass the current
measurement script; macOS fails both RSS budgets. Full PP-11 is not certified.**
The original thresholds remain in [SPEC PP-11](../SPEC.md#acceptance-contract).

| Native runner | Installed bytes | Idle RSS MiB | Peak RSS MiB | Readiness median / max ms | Script budget |
| --- | ---: | ---: | ---: | ---: | --- |
| macOS 15 · ARM64 | 2,630,402 | 66.64 | 66.67 | 122.08 / 151.19 | Memory fail |
| macOS 15 · Intel | 2,864,402 | 38.40 | 38.41 | 255.64 / 299.35 | Memory fail |
| Windows Server 2022 · x64 | 2,943,921 | 15.00 | 15.38 | 39.90 / 43.28 | Pass |
| Ubuntu 22.04 · x64 · Xvfb | 3,167,905 | 17.19 | 19.23 | 17.14 / 17.89 | Pass |

Raw CI results: [macOS ARM64](measurements/v0.0.3/aarch64-apple-darwin.json),
[macOS Intel](measurements/v0.0.3/x86_64-apple-darwin.json),
[Windows](measurements/v0.0.3/x86_64-pc-windows-msvc.json),
[Linux](measurements/v0.0.3/x86_64-unknown-linux-gnu.json).

### What passed

- Formatting, all-target Clippy with warnings denied, behavior/localization/update tests (30 on Unix, 28 on Windows) and release builds.
- Four-language GUI smoke: save without applying, matching-provider disabled state, row selection, edit/cancel, import preview, tabs, draft/selection preservation on language change, external configuration reread and closing all windows. All files were disposable.
- Real update helper/install/restart/cleanup and actual archive layout/architecture validation on each platform.
- Reviewed native CJK screenshots. Windows ICON/GROUP_ICON resources contain seven sizes from 16–256 px, byte-matching the PPC ICO; no dynamic MSVC runtime is imported. Native `iconutil` decoded macOS ICNS, and bundles contain the correct icon and Info.plist entry.

The flat native window is 440 × 502, with compact editors and a More menu.
No new UI runtime dependency was added. The window icon uses 16 KiB RGBA with
image decoders disabled; macOS bundles a 105,585-byte ICNS. Package layout adds
that resource only on macOS. [macOS 0.0.2 migration](user-guide.md#manual-update)
requires a one-time manual download.

### Local macOS result

Apple M4, 16 GiB, macOS 27.0.1; the same functional checks passed locally.
Installed owned files: 2,630,386 bytes (2.51 MiB). Maximum idle RSS: 112.17 MiB;
OS peak: 115.45 MiB. Readiness samples: 405.48 / 110.37 / 89.32 ms.
Memory and maximum-readiness budgets fail.
[Raw local result](measurements/v0.0.3/macos-m4-local.json).
This development-machine run is not a controlled comparison with CI or older
versions and does not establish a performance improvement.

### Published assets and live update check

All six uploaded asset sizes and GitHub SHA-256 digests matched the final files:
four packages, `SHA256SUMS.txt` and `verification.json`. The real updater
downloaded the public macOS ARM64 archive, verified its digest and staged the
six owned bundle files in a disposable fixture. Installation and real
provider/client files were untouched.

The anonymous metadata check returned HTTP 403 with
`X-RateLimit-Remaining: 0` on 2026-10-12: the outbound address had exhausted
GitHub's 60-request anonymous quota. **That live check did not pass.** Metadata
for the download/staging smoke was obtained through authenticated `gh`; the
app's package download remained unauthenticated. The app does not use the
user's GitHub credentials and reports check failures for later manual retry.
Packages are unsigned; the digest verifies integrity, not an independent
publisher signature.

### Screenshots

All screenshots use sample data and are excluded from application packages.

| View | Screenshot |
| --- | --- |
| macOS · English / 简体中文 / 繁體中文 / 日本語 | [English](../assets/screenshot.png) · [简体中文](../assets/screenshot-zh.png) · [繁體中文](../assets/screenshot-zh-TW.png) · [日本語](../assets/screenshot-ja.png) |
| Compact editor · 简体中文 | [Editor](../assets/screenshot-edit-zh.png) |
| Native Linux / Windows · 简体中文 | [Linux](../assets/screenshot-linux-zh.png) · [Windows](../assets/screenshot-windows-zh.png) |

## Measurement method and limits

[measure_ui](../examples/measure_ui.rs) runs the production window with an
empty isolated provider store, or the actual provider editor with fake data using
`--editor`. Historical import measurements retain their original scenario label. [measure.py](../scripts/measure.py) starts three
fresh processes, samples total RSS every 50 ms and begins idle sampling after
one second. Readiness is the first event-loop timer's `READY` marker. OS caches
are retained. Peak is the OS high-water mark (`wait4` on macOS, `VmHWM` on Linux,
`GetProcessMemoryInfo` on Windows). Installed size includes the executable,
metadata, icon and license notices. Shared resident pages and system-font
memory are included in RSS; physical footprint or heap use is not a substitute.

Native CI uses separate GitHub VMs and Linux/Xvfb. Results are not controlled
cross-platform or before/after comparisons. Minimum OS declarations do not
prove runtime support on every OS version. Windows, Linux and Intel macOS
have not been tested on local physical desktops.

- True cold interactive readiness, input latency, import/load stress and update-download/install peak memory remain unverified. A passing script is not full PP-11 acceptance.
- GUI smoke invokes real widgets/callbacks; it is not a physical mouse/keyboard or accessibility audit.
- No live provider calls are made. Tests verify file changes, not service/model/protocol compatibility.
- Locks coordinate pp-switch instances; other software may ignore them. Pre-write comparisons cannot eliminate every external race. Codex multi-file writes have recovery, not a globally atomic transaction.

## Historical releases

Historical raw JSON remains versioned; full release-era narratives are preserved
at their tags. Use the current section above for 0.0.3 claims.

| Release | Implementation / native CI | Functional result | Script budget | Evidence |
| --- | --- | --- | --- | --- |
| [0.0.2](https://github.com/phonowell/pp-switch/releases/tag/v0.0.2) | `3e271b0012b160b320f4117dddc3e54cbcad861d` · [38161367066](https://github.com/phonowell/pp-switch/actions/runs/38161367066) | All runners pass; 30 Unix / 28 Windows tests | Windows pass; macOS memory and Linux idle RSS fail | [Raw data](measurements/v0.0.2) · [Tagged report](https://github.com/phonowell/pp-switch/blob/v0.0.2/docs/verification.md) |
| [0.0.1](https://github.com/phonowell/pp-switch/releases/tag/v0.0.1) | `b4ccb7f3f3c3792052399bedf6607afa1a718a8b` · [38155786150](https://github.com/phonowell/pp-switch/actions/runs/38155786150) | All runners pass; 21 Unix / 20 Windows tests | Windows/Linux pass; macOS memory fail | [Raw data](measurements/v0.0.1) · [Tagged report](https://github.com/phonowell/pp-switch/blob/v0.0.1/docs/verification.md) |

## Community sources

Public cc-switch discussions reviewed on **2026-10-12** motivated the narrow
scope. These are individual, dated reports, not a representative survey,
endorsement or comparison benchmark. They do not establish defects in every
current cc-switch version or show that pp-switch is faster.

| Source | Report or request | Relevance to pp-switch |
| --- | --- | --- |
| [#7649: Pure / Lite mode](https://github.com/farion1231/cc-switch/issues/7649) | “对于只希望使用 CC Switch 来「切换供应商 / API / 模型」的用户来说，管理范围有些过大。” Also: “CC Switch only manages fields it owns. Everything else belongs to the target application/user.” | Own only effective URL/key values; exclude the request's broader model/routing/usage scope. |
| [#4142](https://github.com/farion1231/cc-switch/issues/4142) | A Windows user on 3.16.2 reported losing Codex settings after toggling local routing and needing to redo sandbox/project trust settings. | Preserve unrelated bytes and refuse unsupported setups; this is a version-specific report. |
| [#4581](https://github.com/farion1231/cc-switch/issues/4581) | A macOS user on 3.16.3 reported WebContent memory growth while the app stayed resident and described opening it only when needed. | Close the window → exit; publish our own budget failures as well. |
| [PR #4693](https://github.com/farion1231/cc-switch/pull/4693) | Proposed destroying the WebView on entering tray mode; open and unmerged when reviewed. | Upstream also addresses lifecycle costs; its dated proposal is not evidence of a released fix. |
