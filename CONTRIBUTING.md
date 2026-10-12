# Contributing to Pure Provider Switch

Agents start with [AGENTS.md](AGENTS.md), then read only the task's relevant
PP requirements and chapters here. [SPEC.md](SPEC.md) is the behavior contract:
update the relevant requirement before changing intended behavior, then implement
the smallest change and verify its observable result. Keep URL/key switching,
scoped Provider import, language selection and strictly manual updates as the
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

`cargo run` opens the actual app and uses your local data. Use the isolated
harnesses below for verification; never test writes against real client files.
Rust + FLTK is the UI implementation. Use existing dependencies, system fonts
and bounded native drawing. [The approved layout](design/minimal.html) remains
the visual reference; [current screenshots](docs/verification.md#screenshots)
show the shipped implementation.

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
shipped widgets and callbacks in all four languages; review the screenshots
after UI changes. Add tests for PP behavior, not for implementation details.
Native CI validates macOS ARM64/Intel, Windows x64 and Linux x64 independently.

On macOS also run `python3 scripts/protocol_smoke.py
target/release/examples/protocol_smoke target/native-url`. This opens a disposable
app bundle via real cold/warm Apple URL events, checks preview-only handling,
draft preservation and malformed input, then unregisters only that fixture.
It never changes the user's default handlers or real provider/client files.

## Package and measure

For a local native build, replace `TARGET` with its Rust target triple:

```sh
python3 scripts/package.py TARGET --binary target/release/pp-switch
target/release/examples/update_smoke --archive target/package/PACKAGE
python3 scripts/measure.py target/release/examples/measure_ui --output target/metrics.json --enforce
python3 scripts/measure.py target/release/examples/measure_ui --import --output target/import-metrics.json --enforce
```

On Windows, use `target/release/pp-switch.exe`. Replace `PACKAGE` with the
generated ZIP or tar.gz filename. Packaging includes app metadata, the native
icon and license notices, and enforces the installed-size budget. The archive
smoke checks the actual package against the updater's whitelist and architecture.

Keep [PP-11](SPEC.md#pp-11-performance-budgets) thresholds unchanged. The measurement
script can fail after functional checks pass; preserve its JSON and report the
failure. Its event-loop marker does not certify cold interactive startup or
import/update peaks. [Verification](docs/verification.md) defines the method
and records current and historical evidence. Passing checks need not be rerun
when their inputs are unchanged.

## Documentation ownership

| Document | Owns |
| --- | --- |
| `README*.md` | Product introduction, differences, downloads, short usage and current performance summary; English / 简体中文 / 日本語 / Deutsch stay aligned. |
| `docs/user-guide*.md` | Setup, exact write locations, import, language, local storage, manual updates and troubleshooting; English / 简体中文 / 日本語 stay aligned. |
| `SPEC.md` | The single English behavior contract and budget definitions. |
| `AGENTS.md` | Short agent workflow and task reading routes. |
| `CONTRIBUTING.md` | Build, validation, packaging, documentation maintenance and tool recovery. |
| `docs/verification.md` | Versioned evidence, measurement method, limitations and dated community sources. |
| `docs/measurements/` | Immutable raw results by version; keep historical files. |

For documentation-only changes, check local links and heading anchors, Markdown
structure, translation coverage and numerical claims against raw measurements.
Do not rebuild the app solely for wording changes. Update affected translations
together, keep sample credentials fake and distinguish document languages from
the four interface languages. Use Git tags and release pages for superseded
development narratives rather than copying them into current instructions.

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
draft preview, checks all six uploaded sizes and GitHub SHA-256 digests, then
publishes. An existing release is never overwritten. A digest failure leaves
the draft for inspection; do not blindly rerun publication. Tag and Cargo
versions must match, and the workflow records the tag SHA and CI run URL.

Use native CI packages from the implementation commit; do not substitute an
untested local binary. Attach all four packages, `SHA256SUMS.txt` and
`verification.json`, and compare uploaded sizes/digests with the originals.
Record the implementation SHA, CI run and release tag in verification. Disclose
failed budgets, unsigned packages and migration limits in the release notes.
Exercise public download/checksum/staging in a disposable fixture, and record
network failures separately from authenticated metadata checks.

Only commit task-owned files. Preserve others' edits; do not rewrite history,
force-push or discard work. Keep secrets out of diagnostics and screenshots.
