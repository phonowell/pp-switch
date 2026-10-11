# Verification evidence

## Local macOS evidence — 2026-10-12

Host: Apple Silicon ARM64, macOS 27.0.1. Rust 1.99.0, Apple clang 17.0.0,
CMake 4.4.4. Release flags: size optimization, LTO, one codegen unit, stripped
symbols, panic abort. No bundled fonts/images, browser, database or async runtime.

- Behavioral suite initially passed 19 tests; a subsequent regression covers
  Codex API-key auth files with null OAuth placeholders.
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

## Cross-platform status

CI is configured for native macOS ARM64, macOS Intel, Windows x64 and Linux x64:
format, lint, behavior tests, release build, GUI smoke, size check and RSS/startup
measurement. Successful runtime verification will be recorded after the actual
runs. Windows/Linux/Intel macOS have not been verified locally.

## Limits of evidence

- No live API calls are made; provider protocol/model compatibility is the user's
  existing client setup. Tests verify file changes, not a third-party service.
- GUI smoke invokes the real widgets/callbacks; it is not a physical mouse/keyboard
  or accessibility audit. Manual visual review covers the local screenshot.
- File locks coordinate pp-switch instances. Other applications need not honor
  them; comparing immediately before writes cannot eliminate all external races.
- Multi-file writes have journal recovery, not an atomic filesystem transaction
  spanning both Codex files.
