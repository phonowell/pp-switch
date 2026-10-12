# Verification evidence

[Product](../README.md) · [User guide](user-guide.md) · [Spec](../SPEC.md) · [Reproduce](../CONTRIBUTING.md)

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
empty isolated provider store. [measure.py](../scripts/measure.py) starts three
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
