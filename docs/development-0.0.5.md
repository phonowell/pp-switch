# v0.0.5 development checkpoint

Development is on `codex/v0.0.5`, based on the merged v0.0.4 commit `58a299c`.
This is a local macOS ARM64 checkpoint, not a published or installed release.
The primary remaining work is macOS memory and startup reduction; PP-11 budgets
are unchanged. Other native platforms and Intel CI have not run for this branch.

## Behavior and appearance

- Main title is **Pure Provider Switch**. The approved PPS SVG switch mark uses
  exact opaque `#365DD8` and white; package icons are regenerated without any
  runtime SVG/image dependency. macOS title bars continue to omit the icon.
- Provider rows have no selection circle or selected background. Edit and Switch
  target their own record; matching duplicates all show In use. Switching updates
  status without rebuilding the list, preserving its scroll position.
- Cold/warm macOS `ccswitch` / `ppswitch` URL events open an unsaved three-field
  preview. Save persists only; Cancel and incoming links preserve client files.
  An open draft cannot be overwritten. Only scoped Provider data is extracted;
  no remote configs or unrelated settings are applied.
- Settings provides explicit registration, preserving other installed handlers.
  Startup does not change URL defaults. Tests target temporary bundles directly;
  local default-handler registration is deliberately unverified.

[Native UI/title/import review](screenshots/v0.0.5/native-review.png) ·
[Native full title](screenshots/v0.0.5/full-title.png) ·
[Updated design preview](screenshots/v0.0.5/design-preview.png).

## Shared workstation testing

Local smoke now defaults to background checks, preserving the user's foreground
focus, pointer and keyboard. GUI activation, real URL opens, native input,
appearance, restart and window measurements require explicit desktop opt-in or
a dedicated CI session. Native GUI/input/measurement examples also refuse direct
launch without opt-in; never spoof `CI=true` or pass the flag without the user's
idle-period authorization. Skipped desktop gates are unverified, never passes.

[Background report](measurements/v0.0.5/review-20261013/background-report.json) ·
[Direct entry-point refusals](measurements/v0.0.5/review-20261013/desktop-guards.json).
8 background gates passed with 33 Rust / 11 Python tests, including no-child-launch
regressions. No foreground check was rerun after the user's request. The production
executable SHA is unchanged, so the earlier valid functional/style captures are
reused; the development measurement example now has an opt-in guard.

## Validation evidence

[Immutable full smoke report](measurements/v0.0.5/review-20261013/report.json) ·
[Final source, screenshot, executable and package hashes](measurements/v0.0.5/review-20261013/checkpoint.json).
Exact commands, raw output and exit codes are retained alongside these records.
33 Rust tests, 9 Python tests, format, all-target Clippy and release build passed.
Chinese native input, real cold/warm Apple URL delivery, callback import/save/
cancel, live appearance/draft preservation, four-language light/dark pixel gates,
updater/restart and final package whitelist/architecture validation passed.
The native captures and updated browser preview were visually inspected.

The full smoke run has 17 passing gates and 2 disclosed performance-budget
failures. After correcting palette quantization (which had averaged the solid
blue/white by one color level), the release was rebuilt and package validation
was repeated. Raw window RGBA and ICO stayed byte-identical; no measured runtime
input changed, so passing GUI/core/performance checks were reused. CodeGraph was
unavailable because this worktree has no index; bounded source/history reads
were used instead. No index or real-client test files were created.

## Package comparison

Same Mac M4 host, stripped ARM64 release builds, same packaging script layout.
The scoped link handler/parser is restored in 0.0.5; size remains below 5 MiB.

| Measurement | v0.0.4 baseline | v0.0.5 checkpoint |
| --- | --- | --- |
| Installed application-owned bytes | 2,544,096 | 2,498,449 |
| ZIP bytes | 1,349,523 | 1,285,871 |
| ICNS bytes | 105,585 | 42,291 |

Installed size is 45,647 bytes smaller (1.79%); ZIP is 63,652 bytes smaller
(4.72%). The final archive SHA-256 is recorded in the checkpoint JSON. The package
is only staged under `target/package`; no user installation or URL defaults changed.

## Memory and startup comparison

Mac M4, macOS 27.0.1, three fresh processes per scenario, OS caches not cleared;
50 ms RSS sampling and OS high-water mark. The first event-loop READY timer is a
readiness proxy, **not verified cold interactive startup**. Rows/editor use fake
credentials and isolated files. [Baseline empty](measurements/v0.0.5/review-20261013/baseline-empty.json) ·
[Baseline editor](measurements/v0.0.5/review-20261013/baseline-editor.json) ·
[Checkpoint empty](measurements/v0.0.5/review-20261013/metrics.json) ·
[Checkpoint editor](measurements/v0.0.5/review-20261013/editor-metrics.json).

| Scenario / measurement | v0.0.4 baseline | v0.0.5 checkpoint |
| --- | --- | --- |
| Empty idle max RSS MiB | 110.77 | 110.77 |
| Empty peak RSS MiB | 110.80 | 110.80 |
| Editor idle max RSS MiB | 129.00 | 128.77 |
| Editor peak RSS MiB | 129.05 | 128.80 |
| Empty READY median / max ms | 117.32 / 1,213.08 | 112.37 / 441.87 |
| Editor READY median / max ms | 112.28 / 331.96 | 127.11 / 151.26 |

Memory budgets fail, and the empty maximum readiness proxy exceeds 300 ms.
Small RSS/time differences are inconclusive; no material memory/startup gain is
claimed. The [baseline vmmap summary](measurements/v0.0.5/review-20261013/baseline-vmmap-summary.txt)
shows substantial shared framework mappings and graphics allocations. Its 32.1
MiB physical footprint is a different metric and cannot replace total RSS.
The older unexplained native hang remains unresolved; normal modal-close
regressions passing do not establish its original cause.

Next performance work must isolate launch/drawing/editor allocation costs using
repeatable measurements. Keep startup free of updates, URL-default registration,
font enumeration and background network work; retain the existing native size
flags. Any larger UI/toolkit change needs new evidence for appearance, IME, focus,
URL delivery and all supported clients before adoption.
