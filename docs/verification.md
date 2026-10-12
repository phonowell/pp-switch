# Verification evidence

## 0.0.3 local validation — 2026-10-12

The user approved `design/minimal.html`, then requested a flat style and an app
icon, with PPC lettering. Full name: Pure Provider Switch; short name: pp-switch.
The production window is now 440 × 502; editing and import use compact
native dialogs. More contains import, the four languages and manual updates.
No new application dependency or runtime was added. The SVG icon produces
OS icon formats; the window uses 16 KiB of raw RGBA with image decoders disabled.
macOS adds a 105,585-byte ICNS resource; Windows embeds ICO through the installed
SDK and Linux uses the window icon. macOS 0.0.2 cannot accept the new icon in its
fixed update layout and needs a one-time manual download. Windows/Linux retain
their package file layouts.

Local host: Apple M4, 16 GiB, macOS 27.0.1. Format, all-target Clippy with warnings
denied, 30 behavior/localization/update tests, release build and all-four-language
GUI acceptance passed. The GUI checks exercise add/save without applying,
matching-provider disabled state, selection without opening an editor, cancel
without saving, import preview, tabs, preserved drafts/selections on language
change, rereading an external client edit on selection and closing every window. Real screenshots were reviewed; opaque row
backgrounds fixed corner artifacts. All data and client paths were disposable.
The actual release archive passed updater layout/architecture validation, and
the real helper installed, restarted and cleaned up in a temporary fixture.

Installed application-owned files including notices and icon: 2,630,386 bytes
(2.51 MiB), passing the 5 MiB size budget. Local idle maximum total RSS was
112.17 MiB; OS peak was 115.45 MiB. Both memory budgets still fail. Fresh-process
event-loop readiness was 405.48 / 110.37 / 89.32 ms, also failing the 300 ms
maximum budget. This is a local development run with OS caches retained,
not a controlled comparison or a cold/input-readiness result.
[Raw local data](measurements/v0.0.3/macos-m4-local.json).

## 0.0.3 cross-platform release evidence

Implementation `021f8208e75fc37963b46674c10f89e2e252cfb9`,
[CI run 38201751965](https://github.com/phonowell/pp-switch/actions/runs/38201751965).
All four native runners passed format, Clippy, behavior (30 tests on Unix;
28 on Windows), release build, four-language GUI, real helper/install/restart
and actual package validation. Reviewed CJK screenshots on Linux and Windows.
The Windows executable contains ICON and GROUP_ICON resources (group ID 1,
seven sizes from 16 to 256 px, byte-matching the final PPC ICO); its imports contain no dynamic MSVC runtime.
macOS ICNS was decoded successfully by the native icon tool and is listed in
the bundle's Info.plist. The application icon is the only bundled bitmap asset.

| Native runner | Installed bytes | Idle RSS MiB | Peak RSS MiB | Startup median / max ms | Script budget |
| --- | ---: | ---: | ---: | ---: | --- |
| macOS 15 · ARM64 | 2,630,402 | 66.64 | 66.67 | 122.08 / 151.19 | Memory fail |
| macOS 15 · Intel | 2,864,402 | 38.40 | 38.41 | 255.64 / 299.35 | Memory fail |
| Windows Server 2022 · x64 | 2,943,921 | 15.00 | 15.38 | 39.90 / 43.28 | Pass |
| Ubuntu 22.04 · x64 · Xvfb | 3,167,905 | 17.19 | 19.23 | 17.14 / 17.89 | Pass |

Windows and Linux pass this benchmark. macOS exceeds both RSS budgets, so the
overall CI remains red; the original thresholds were not changed. These are
fresh processes on separate GitHub VMs with OS caches retained, not controlled
before/after comparisons. This does not certify PP-11 cold/input readiness,
import stress or update-download/extraction peak memory. Shared pages and system
fonts are included in total RSS.

Raw results: [ARM64 macOS](measurements/v0.0.3/aarch64-apple-darwin.json),
[Intel macOS](measurements/v0.0.3/x86_64-apple-darwin.json),
[Windows](measurements/v0.0.3/x86_64-pc-windows-msvc.json),
[Linux](measurements/v0.0.3/x86_64-unknown-linux-gnu.json).
Release assets include archive SHA-256 sums and a consolidated verification JSON.

### Published release verification

Published [v0.0.3](https://github.com/phonowell/pp-switch/releases/tag/v0.0.3)
as a public prerelease at documentation commit
`911db7b0c598e95d2b63fe2f4353020f98e0ae54`; executable source is the implementation
commit above. All six uploaded asset sizes and GitHub SHA-256 digests match the
final local files. The real updater downloaded the public macOS ARM64 archive,
verified its digest and staged the six owned bundle files in a disposable fixture.
The installed application and real provider/client files were untouched.

The local anonymous metadata check returned HTTP 403 with
`X-RateLimit-Remaining: 0`: the current outbound address had exhausted GitHub's
60-request anonymous API quota. This live check did not pass. Release metadata
for the download/staging smoke was obtained through authenticated `gh`; the
application's archive download remained unauthenticated. The application does
not use the user's GitHub credentials and reports this check failure for retry
later. No automatic retries or background checks were added.


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
