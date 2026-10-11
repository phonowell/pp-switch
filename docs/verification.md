# Verification evidence

## 0.0.2 local validation — 2026-10-12

Apple M4 (Mac16,1), 16 GiB, macOS 27.0.1, Rust 1.99.0 / Apple clang 17 / CMake 4.4.4.
The new native window and four compiled locale catalogs keep the FLTK backend;
manual updates add HTTPS/TLS, hashing and archive parsing. No WebView, bundled
fonts/images, database, async runtime, tray or startup updater.

- Formatting, all-target Clippy with warnings denied, 21 core behavior tests,
  2 locale/preference tests and 7 update tests (6 on Windows) passed locally.
- Four-language GUI smoke passed the real save/switch/import/selection/close
  callbacks, persisted language changes and preservation of unsaved drafts.
- Reviewed actual English, Simplified/Traditional Chinese and Japanese windows.
  Native dropdowns initially left square-border artifacts; custom vector drawing
  and an opaque background corrected them. Linux CI screenshots revealed missing
  CJK glyphs despite installed fonts; explicitly selecting installed CJK faces
  fixes Xft's lack of per-glyph fallback. Screenshots contain dummy data only.
- Real temporary installer/helper smoke waited for the old process, replaced
  package resources and automatically reopened the updated fixture. It does not
  load the real user store or client files. A separate explicit read-only GitHub
  HTTPS check succeeded (no newer release at the time).
- Update tests cover release ordering/platform selection, invalid digests/URLs,
  archive layout/traversal/links/architecture, successful filesystem installation,
  a failure after the first replacement, rollback conflict preserving external
  edits/backups and refusal of tampered staged data or changed destinations.
- Regenerated third-party notices from locked dependency sources, including TLS
  and archive libraries. No user client configuration was changed.

Local measurement of the final visual layout (cached OS pages, system locale):

| Metric | Observed | Original budget |
| --- | ---: | --- |
| Installed app-owned bytes including notices | approximately 2.39 MiB | ≤5 MiB: pass |
| Idle maximum total RSS | 132.69 MiB | ≤20 MiB: **fail** |
| Peak total RSS | 133.06 MiB | ≤30 MiB: **fail** |
| Fresh-process event-loop readiness | 411.34 / 110.55 / 113.45 ms | ≤300 ms: maximum **fail** |

[Raw local data](measurements/v0.0.2/macos-m4-local.json). RSS increased from
approximately 121 MiB in 0.0.1; this iteration does not establish a macOS memory
improvement. The UI grew and now displays the system-selected language; no
controlled attribution of the additional residency has been made. Keep the
original budgets and optimize from measured evidence.

The local measurement predates final updater-only archive hardening; the
production window/idle path is identical. Release CI measurements below apply
to the exact published build. Update-download/extraction peak RSS is not part
of this idle benchmark and has not been separately certified.

## 0.0.2 cross-platform release evidence

Implementation `3e271b0012b160b320f4117dddc3e54cbcad861d`, [CI run 38161367066](https://github.com/phonowell/pp-switch/actions/runs/38161367066).
All four native runners passed format, Clippy, behavior, release build, four-language
GUI, real installer/helper/restart and actual archive round-trip checks.

| Native runner | Functional checks | Installed bytes | Idle RSS MiB | Peak RSS MiB | Startup median / max ms | Script budget |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| macOS 15 · ARM64 | Pass, 30 tests | 2,508,446 | 70.88 | 70.94 | 113.83 / 139.72 | Memory fail |
| macOS 15 · Intel | Pass, 30 tests | 2,742,766 | 42.76 | 42.84 | 165.68 / 169.85 | Memory fail |
| Windows Server 2022 · x64 | Pass, 28 tests | 2,888,618 | 16.03 | 16.57 | 42.36 / 44.61 | Pass |
| Ubuntu 22.04 · x64 · Xvfb | Pass, 30 tests | 3,145,146 | 20.99 | 21.00 | 8.99 / 9.49 | Idle RSS fail |

Installed size includes metadata and all notices. Windows passes this benchmark;
macOS exceeds idle/peak budgets, and Linux exceeds the 20 MiB idle threshold by
about 1 MiB after using an installed CJK face. The budget was not rounded up or
relaxed. System fonts are not bundled, but their resident memory is included.
The overall CI remains red for these disclosed performance failures.

Raw results: [ARM64 macOS](measurements/v0.0.2/aarch64-apple-darwin.json),
[Intel macOS](measurements/v0.0.2/x86_64-apple-darwin.json),
[Windows](measurements/v0.0.2/x86_64-pc-windows-msvc.json),
[Linux](measurements/v0.0.2/x86_64-unknown-linux-gnu.json).
Reviewed the CJK rendering in the real CI screenshots; additional [Linux](../assets/screenshot-linux-zh.png)
and [Windows](../assets/screenshot-windows-zh.png) views are retained in the repo.
CI artifacts retain all four interface languages for each platform.

After publication, the production HTTPS download/checksum/extraction/staging flow
was exercised anonymously against the real 0.0.2 macOS ARM64 GitHub asset from
a disposable app fixture. It passed; the fixture installation and all user/client
files remained unchanged. This complements the native helper/install/restart
tests on all four runners; it is not a physical-click or live-service audit.

Archive SHA-256 sums and a consolidated verification JSON are release attachments.
These builds are unsigned, and no independent publisher signature is claimed.
This benchmark does not certify full PP-11: cold/input readiness, import stress
and update download/install peak memory remain unverified.


## 0.0.1 historical evidence


## Local macOS evidence — 2026-10-12

Host: Apple M4 (Mac16,1), 16 GiB RAM, ARM64, macOS 27.0.1. Rust 1.99.0, Apple clang 17.0.0,
CMake 4.4.4. Release flags: size optimization, LTO, one codegen unit, stripped
symbols, panic abort. No bundled fonts/images, browser, database or async runtime.

- Behavioral suite passed 21 tests, including Codex API-key auth files with null
  OAuth placeholders and refusal to replace symbolic-link destinations.
- Formatting and Clippy (all targets, warnings denied) passed.
- Native GUI harness passed save, switch, new, import preview, list selection
  and close using isolated temporary client files. All normal callbacks ran.
- A screenshot review found status-text redraw artifacts; using an opaque
  status background corrected them. Screenshot contains example data only.
- No real client configuration was edited during testing.

### First release measurement (before status redraw correction)

| Metric | Observed | Budget |
| --- | --- | --- |
| Installed application-owned files | 1,174,323 bytes (1.12 MiB) | ≤5 MiB: pass |
| Compressed macOS package | 561,512 bytes | Informational only |
| Idle total RSS, maximum median across 3 processes | 126,828,544 bytes (120.95 MiB) | ≤20 MiB: **fail** |
| OS peak RSS | 126,861,312 bytes (120.98 MiB) | ≤30 MiB: **fail** |
| Fresh-process event-loop readiness | 455.54 / 143.26 / 152.34 ms | ≤300 ms: first process **fail** |

The measurement example uses the exact production window/event loop with an
isolated temporary home, an event-loop readiness marker and a 3-second exit
timer. The observer samples total process RSS every 50 ms; idle samples begin
after one second. macOS peak RSS comes from `wait4`, not the sample maximum.
There are no application subprocesses. OS caches are not cleared: these are
fresh-process starts, not a claim of disk-cold startup or measured input latency.

A separate `vmmap -summary` diagnostic reported physical footprint 44.8 MiB
(peak 52.3 MiB), including backing surfaces and CoreGraphics buffers. This is
not substituted for RSS and does not meet the earlier budget either. The
diagnostic itself changes residency and is not the release benchmark.

Performance is still under investigation. Do not advertise a 20 MiB application.
CI enforces the original RSS/startup thresholds and publishes the raw JSON even
when they fail. Package creation precedes memory checks so failures remain
reviewable without presenting a passing performance result.

### Final local preview validation

Implementation commit `b4ccb7f3f3c3792052399bedf6607afa1a718a8b`:
format, Clippy, 21 behavioral tests, release build and GUI smoke passed.
The installed package includes all third-party notices: 1,382,503 bytes (1.32 MiB).
Final local measurement: idle maximum 126,812,160 bytes (120.94 MiB), peak
126,844,928 bytes (120.97 MiB), readiness 524.42 / 153.54 / 136.31 ms.
Memory and maximum-startup budgets **fail**. The median alone is not used to
declare startup success.

A measurement-only Cocoa timer close could leave FLTK waiting for a native
event after hiding the final window. The harness now bounds that final wait;
this does not add polling or background work to the production app. GUI smoke
uses the regular close path and passed both locally and in macOS CI.

## Cross-platform status

Native GitHub-hosted runners cover macOS ARM64 (`macos-15`), macOS Intel
(`macos-15-intel`), Windows x64 (`windows-2022`) and Linux x64 (`ubuntu-22.04`, Xvfb).
Each runs formatting, Clippy, behavioral tests, release build, GUI smoke,
installed-size enforcement and a separate memory/startup check. Windows, Linux
and Intel macOS have not been run locally.

Final implementation [CI run 38155786150](https://github.com/phonowell/pp-switch/actions/runs/38155786150)
at `b4ccb7f3f3c3792052399bedf6607afa1a718a8b`:

| Native runner | Format / lint / behavior / build / GUI | Installed bytes | Idle RSS MiB | Peak RSS MiB | Startup median / max ms | Measurement budget |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| macOS 15.7.9 ARM64 | Pass, 21 behavioral tests | 1,382,535 | 68.08 | 68.09 | 91.02 / 100.58 | Memory fail |
| macOS 15.7.9 Intel | Pass, 21 behavioral tests | 1,404,543 | 40.24 | 40.25 | 218.78 / 222.67 | Memory fail |
| Windows Server 2022 x64 | Pass, 20 behavioral tests | 1,457,860 | 15.10 | 15.48 | 42.39 / 43.22 | Pass |
| Ubuntu 22.04 x64, Xvfb | Pass, 21 behavioral tests | 1,595,555 | 8.77 | 19.10 | 17.00 / 17.31 | Pass |

Installed size includes the executable, package metadata and all distributed
licenses/notices. Memory includes shared resident pages; it is not heap usage.
CI machines are shared GitHub-hosted VMs, not a controlled hardware comparison.
Windows and Linux results do not stand in for the unmeasured client-desktop setups.

Raw three-process results: [ARM64 macOS](measurements/v0.0.1/aarch64-apple-darwin.json),
[Intel macOS](measurements/v0.0.1/x86_64-apple-darwin.json),
[Windows](measurements/v0.0.1/x86_64-pc-windows-msvc.json),
[Linux](measurements/v0.0.1/x86_64-unknown-linux-gnu.json) and
[local M4](measurements/v0.0.1/macos-m4-local.json).
Release attachments include archive SHA-256 sums and a consolidated verification
JSON with the package sizes, hashes, raw metrics and implementation commit.

The final Windows executable's PE import directory contains system DLLs only;
it imports no `VCRUNTIME`, `MSVCP` or `MSVCR` DLL. Rust and the nested FLTK CMake
project both use the static CRT. macOS packages declare macOS 11 as their minimum;
actual native runtime evidence covers macOS 15 CI and macOS 27 on the local M4,
not every intervening OS version.

The preview keeps budget failures visible in CI; a red overall workflow can
mean a performance failure after all functional checks passed. It does not
mean the memory threshold was waived. The complete PP-11 budget is not certified.

## Next performance work

Keep PP-11 unchanged. Measure an empty Cocoa window and pp-switch under identical
conditions before changing the UI backend; the initial local Cocoa baseline was
about 79.8 MiB RSS. Attribute the additional resident pages and backing surfaces,
then retain only changes that improve repeated measurements without changing
the provider-only scope. Extend the harness to import/load stress and true
interactive readiness before claiming those budgets pass.

## Limits of evidence

- No live API calls are made; provider protocol/model compatibility is the user's
  existing client setup. Tests verify file changes, not a third-party service.
- GUI smoke invokes the real widgets/callbacks; it is not a physical mouse/keyboard
  or accessibility audit. Manual visual review covers the local screenshot.
- File locks coordinate pp-switch instances. Other applications need not honor
  them; comparing immediately before writes cannot eliminate all external races.
- Multi-file writes have journal recovery, not an atomic filesystem transaction
  spanning both Codex files.
- The memory benchmark covers launch and idle with an empty isolated provider
  store. Import/load stress peaks and truly cold interactive startup remain
  unverified; passing this script does not prove the whole PP-11 requirement.
