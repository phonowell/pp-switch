# Contributing to Pure Provider Switch

Agents start with [AGENTS.md](AGENTS.md), then read only the task's relevant
PP requirements and chapters here. [SPEC.md](SPEC.md) is the behavior contract:
update the relevant requirement before changing intended behavior, then implement
the smallest change and verify its observable result. Keep URL/key switching with explicit optional Provider models,
scoped native restoration and Provider import, language selection and strictly manual updates as the
entire product scope. Run the shell examples below through `rtk proxy`.

## Build

Install Rust ≥1.89, CMake, Python 3 and the platform C++ toolchain:

- macOS: Xcode Command Line Tools.
- Windows: Visual Studio C++ Build Tools with the Windows SDK; icons use `rc.exe`.
- Linux: the X11/fontconfig development libraries and CJK fonts listed in [CI](.github/workflows/ci.yml). Xvfb is needed for headless GUI checks.

```sh
cargo run --locked
cargo build --locked --release --bins --examples
```

When upgrading the native toolchain configuration, run
`cargo clean --release -p fltk-sys` once before rebuilding. CMake can retain an
older toolchain include even after Cargo notices the new filename. The release
cache must show `-Oz` on macOS and filesystem/print/SVG options OFF.

`cargo run` opens the actual app and uses your local data. Use the isolated
harnesses below for verification; never test writes against real client files.
Rust + FLTK is the UI implementation. Use existing dependencies, system fonts
and bounded native drawing. [UI rules](design/UI.md) define the layout; [versioned images](docs/images.md)
separate illustrative layouts from versioned native evidence.

## Shared workstation testing

`scripts/smoke.py` defaults to background checks: formatting, lint, unit/behavior
and detector tests, builds and package validation. It skips GUI activation, URL
opens, native input, appearance, restart and window measurements. The report
records skipped desktop gates and cannot certify full GUI acceptance.

On this developer's shared machine, preserve foreground focus, pointer and
keyboard input. Desktop checks below require a dedicated CI/desktop session or
an explicitly authorized idle period. Only then pass `--allow-desktop-input` to
`scripts/smoke.py`, `scripts/gui_smoke.py`, `scripts/measure.py`,
`scripts/protocol_smoke.py`, or directly invoked GUI examples. Dedicated CI sets
`CI=true`; do not spoof that value on a shared workstation. Wrappers and native
input/GUI/measurement examples refuse to launch by default. Style-only captures
also show windows and are subject to the same rule.

## Validate code and behavior

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release --bins --examples
python3 scripts/gui_smoke.py target/release/examples/ui_smoke target/gui.ppm
python3 -m unittest discover -s scripts -p '*_test.py'
target/release/examples/update_smoke
```

Windows binaries have an `.exe` suffix; use `python` if that is your Python
command. On headless Linux, prefix GUI and measurement commands with
`xvfb-run -a`, including the update smoke that restarts the fixture.

The harnesses use disposable provider/client files. GUI smoke exercises the
shipped callbacks in Simplified Chinese; it does not establish pointer hit testing.
On macOS run `python3 scripts/gui_smoke.py --timeout 180
target/release/examples/input_smoke target/input-smoke` for Cocoa mouse/key
events through the shipped widgets in Simplified Chinese, including per-row
actions and in-window Settings navigation. English, Traditional Chinese and Japanese use prepared
display states for screenshot/style review, without repeating functional flows.
Run `python3 scripts/visual_check.py target/input-smoke` on those captures to
check locale-invariant Lucide icons, row-action ink and complete active badges.
This moves the cursor inside this process's fixture windows and restores it on
normal exit. Run on an interactive desktop without using the mouse during the
check. A missing graphical session is a blocked check, never a pass.
Review the screenshots
after UI changes. `python3 scripts/protocol_smoke.py target/release/examples/protocol_smoke target/native-links`
checks cold/warm Apple URL events in an isolated bundle without changing defaults.
Add tests for PP behavior, not for implementation details.
Native CI validates macOS ARM64/Intel, Windows x64 and Linux x64 independently.

After a correction, rerun checks affected by its changed inputs and reuse the
others' retained commands/logs/hashes. In particular, a pixel-checker correction
uses the existing screenshots; it does not require another build or GUI run.

Install development-only `fonttools==4.60.1` for SVG/native icon parity.
`python3 scripts/smoke.py --allow-desktop-input` runs all desktop automated gates, including packaging,
native input checks on macOS, updater and both measurements. It writes an
isolated timestamped directory below `target/` containing exact commands, raw
logs, exit codes and `report.json`, and continues after failures to expose all
blocked gates. It does not install the app, change URL associations, tag or publish.
The [smoke matrix](docs/smoke.md) and [UI design](design/UI.md) define acceptance;
human screenshot inspection remains a separate mandatory gate.

Toolkit regression context: [native UI failure lessons](docs/history/native-ui-lessons.md).

## Incremental verification

Use the full smoke suite for a baseline or a change spanning multiple subsystems.
It is not the default loop for each correction. Record the changed inputs and
which passed gates remain applicable; retain the original failures and append
targeted rechecks rather than editing old results into passes.

| Changed inputs | Required recheck | Reuse when unchanged |
| --- | --- | --- |
| Client writes, storage or update behavior | Format/Clippy, affected behavior tests, release build and relevant isolated harness | Other subsystems and unrelated UI captures |
| UI drawing, layout or platform chrome | Format/Clippy, incremental release build, native style-only capture and pixel gates; screenshot fallback | Core tests, update helpers and performance measurements |
| UI input handling or callbacks | Above code gates and affected Chinese native input scenarios | Unchanged Chinese scenarios and other languages' functional flows; other languages receive style checks only |
| Pixel checker | Detector tests and checker on existing captures | Build, GUI execution, packaging and measurements |
| Package content | Package and exact archive validation | Existing verified executable when unchanged |
| Documentation only | Markdown, local links/anchors, translation coverage and evidence claims | All unchanged code/runtime gates |

For an icon/style-only correction, run:

```sh
python3 scripts/gui_smoke.py --timeout 60 target/release/examples/input_smoke target/style-review --style-only
python3 scripts/visual_check.py target/style-review
```

This prepares all four languages' display states without repeating functional
flows. Full input acceptance includes Chinese overflow/scroll badge checks;
style-only capture checks the visible active badge in each language. Remeasure
performance when changed inputs can affect the measured scenario or when making
a new performance claim, not after a documentation or pixel-checker correction.

For navigation, row actions or focus, use `input_smoke OUTPUT --interaction-only`
through the bounded GUI wrapper. It runs the affected Chinese native flow and
four-language style captures. For icon-only changes use `--style-only`.
For live appearance and both-theme styles use `--themes-only` (90-second wrapper),
then `visual_check.py` on `OUTPUT/light` and `OUTPUT/dark`. These change only the
fixture process appearance, never system settings. Reuse unaffected core/update
checks; macOS KVO introduces no idle appearance polling. Windows/Linux appearance
acceptance requires those hosts.

UI icons must come from the pinned [Lucide sources](assets/lucide/README.md).
`python3 scripts/lucide.py --check` verifies native-path parity (development-only
`fonttools==4.60.1`). Do not redraw approximate icons or substitute font glyphs.
Keep Lucide ISC/MIT notices in the packaged third-party notice file.
Client marks use one pinned [Lobe Icons revision](assets/clients/README.md).
`python3 scripts/client_icons.py --check` verifies normalized native path parity;
original SVGs and SHA-256 provenance are retained without a runtime SVG parser.

## Package and measure

For a local native build, replace `TARGET` with its Rust target triple:

```sh
python3 scripts/package.py TARGET --binary target/release/pp-switch
target/release/examples/update_smoke --archive target/package/PACKAGE
python3 scripts/measure.py target/release/examples/measure_ui --output target/metrics.json --enforce
python3 scripts/measure.py target/release/examples/measure_ui --editor --output target/editor-metrics.json --enforce
```

On Windows, use `target/release/pp-switch.exe`. Replace `PACKAGE` with the
generated ZIP or tar.gz filename. Packaging includes app metadata, the native
icon and license notices, and enforces the installed-size budget. The archive
smoke checks the actual package against the updater's whitelist and architecture.

Keep [PP-11](SPEC.md#pp-11-performance-budgets) thresholds unchanged. The measurement
script can fail after functional checks pass; preserve its JSON and report the
failure. Its event-loop marker does not certify cold interactive startup or
editor/update peaks. For macOS issue #1 use `python3 scripts/mac_performance.py
target/release/examples/mac_acceptance target/mac-performance` on a dedicated
desktop. It interleaves a compiled bare Cocoa window with PPS empty/editor/import
scenarios, measures total RSS and OS high-water marks, and requires a dispatched
Cocoa keyboard action plus a displayed frame for READY. Import exercises eight
near-256-KiB synthetic previews. Native termination tests cover main/Settings/editor
through NSApplication's Quit entry point; actual Dock clicks remain separate.
`python3 -m unittest scripts.native_quit_test -v` additionally runs real FLTK
main/Settings/editor lifecycle cases with test-only ordering and activation
suppressed. It sends menu and Dock-format quit AppleEvents and checks the real
delegate, termination, unchanged fixture files and unchanged foreground process.
The missing-handler negative control must reproduce `errAEEventNotHandled`.
These background tests do not replace visual or native pointer acceptance.
All commands, hashes and raw records are retained. Cached launches are explicitly
not a verified cold start: `budget_pass` remains false without that evidence.
Do not purge shared-machine caches; a separately documented reboot/first-launch
experiment is still required. `measure_config` compares separate vs single-snapshot
connection/model reads in the same release binary and synthetic files; cumulative
requested allocation bytes are not RSS or live heap.
[Verification](docs/verification.md) defines the method
and records current and historical evidence. Passing checks need not be rerun
when their inputs are unchanged.

## Documentation ownership

| Document | Owns |
| --- | --- |
| `README*.md` | Product introduction, differences, downloads, short usage and current performance summary; English / 简体中文 / 日本語 / Deutsch stay aligned. |
| `docs/user-guide*.md` | Setup, exact write locations, provider entry, language, local storage, manual updates and troubleshooting; English / 简体中文 / 日本語 stay aligned. |
| `SPEC.md` | The single English behavior contract and budget definitions. |
| `AGENTS.md` | Short agent workflow and task reading routes. |
| `CONTRIBUTING.md` | Build, validation, packaging, documentation maintenance and tool recovery. |
| `docs/verification.md` | Current evidence, measurement method and release summaries. |
| `docs/README.md`, `docs/images.md` | Documentation navigation and image provenance/version labels. |
| `docs/history/` | Superseded checkpoint narratives, acceptance and research; never current instructions. |
| `design/UI.md` | Current UI rules; generated SVG illustrations explain them without certifying native behavior. |
| `docs/measurements/` | Immutable raw results by version; keep historical files. |

`python3 scripts/docs_check.py` checks current Markdown links/anchors and
translated UI rules. Use `python3 design/render_docs.py --check` for deterministic illustration parity.
`--record PATH` verifies a matching current source/package/install report; archived
installations are records and must not be checked against today’s installed app.

For documentation-only changes, check local links and heading anchors, Markdown
structure, translation coverage and numerical claims against raw measurements.
Do not rebuild the app solely for wording changes. Update affected translations
together, keep sample credentials fake and distinguish document languages from
the four interface languages. Keep superseded narratives in `docs/history/` or at immutable Git tags. Keep raw
evidence paths stable. Replace stale README images with explicitly versioned
illustrations or current native captures; a schematic never proves GUI acceptance.

## Tool and output recovery

RTK and CodeGraph are required for code tasks. Use `rtk proxy` for raw shell
output; `rtk git status --short` is only a filtered overview. Retain actual exit
codes, stdout and stderr for validation. A filtered success summary or empty
output does not establish that a file is empty, a match is absent or a check passed.

- **Filtered read-only output:** repeat the narrow read through `rtk proxy`.
  Proxy output can still be truncated by the outer tool; narrow the requested
  range or capture a log and read it in bounded segments.
- **Missing mutation output:** inspect the existing log and resulting state.
  Never repeat a write, commit, push, publish or install solely to recover output.
- **Incomplete graph:** `explore` can omit source segments and consumers.
  Query the specific symbol or `rtk proxy codegraph node <target>` for the missing
  relation; use bounded `rg`, source and tests to fill a concrete evidence gap.
  Zero matches alone do not prove there are no consumers. Reuse unchanged results;
  run `rtk proxy codegraph sync` only when the index is stale.
- **Unavailable tool:** report the command and failure. Use bounded source/search
  evidence for what can proceed, and disclose the missing mandatory check;
  do not silently claim RTK or CodeGraph was used successfully.
- **Buffered interactive output:** use a PTY and drain output while the process
  runs. Do not wait for exit as the only way to retrieve evidence.

## Publish a preview

Update Cargo.toml and Cargo.lock together. Push a matching `vVERSION` tag to
trigger [.github/workflows/ci.yml](.github/workflows/ci.yml). It runs native
functional checks and builds all four packages from that tag. Missing evidence,
failed functions and oversized installed packages block publishing. RSS/readiness
budget failures remain warnings and are disclosed in raw evidence and release
notes; the thresholds do not change. Run `python3 -m unittest discover -s scripts
-p '*_test.py'` to check publication gates without making network writes.

[scripts/release.py](scripts/release.py) assembles four packages,
SHA256SUMS.txt and verification.json from that run. The release job creates a
draft preview, queries it with `gh release view` (the REST release-by-tag endpoint
returns 404 for drafts), checks all six uploaded sizes and GitHub SHA-256 digests,
then publishes. An existing release is never overwritten. A digest failure leaves
the draft for inspection; do not blindly rerun publication. Tag and Cargo
versions must match, and the workflow records the tag SHA and CI run URL.

Use native CI packages from the implementation commit; do not substitute an
untested local binary. Attach all four packages, `SHA256SUMS.txt` and
`verification.json`, and compare uploaded sizes/digests with the originals.
Record the implementation SHA, CI run and release tag in verification. Disclose
failed budgets, unsigned packages and migration limits in the release notes.
Exercise public download/checksum/staging in a disposable fixture, and record
network failures separately from authenticated metadata checks.

A successful `workflow_dispatch` run at the exact implementation SHA can also
supply those four native packages and checks. Reuse unchanged passing jobs;
retain failed attempts and rerun only the affected jobs. Confirm the complete
run succeeded before creating an immutable matching tag at that SHA. Assemble
with `RELEASE_TAG`, `RELEASE_COMMIT` and `RELEASE_RUN_URL` set to that tag, SHA and
successful run, then use the same draft/digest/publish flow. Record this reuse in
the evidence. Do not race manual publication with an active tag release job.

Only commit task-owned files. Preserve others' edits; do not rewrite history,
force-push or discard work. Keep secrets out of diagnostics and screenshots.
