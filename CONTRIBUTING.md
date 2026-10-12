# Contributing to Pure Provider Switch

Start with [SPEC.md](SPEC.md). It is the behavior contract: update the relevant
PP requirement before changing behavior, then implement the smallest change and
verify its observable result. Keep URL/key switching, scoped Provider import,
language selection and strictly manual updates as the entire product scope.

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
target/release/examples/update_smoke
```

Windows binaries have an `.exe` suffix; use `python` if that is your Python
command. On headless Linux, prefix GUI and measurement commands with
`xvfb-run -a`, including the update smoke that restarts the fixture.

The harnesses use disposable provider/client files. GUI smoke exercises the
shipped widgets and callbacks in all four languages; review the screenshots
after UI changes. Add tests for PP behavior, not for implementation details.
Native CI validates macOS ARM64/Intel, Windows x64 and Linux x64 independently.

## Package and measure

For a local native build, replace `TARGET` with its Rust target triple:

```sh
python3 scripts/package.py TARGET --binary target/release/pp-switch
target/release/examples/update_smoke --archive target/package/PACKAGE
python3 scripts/measure.py target/release/examples/measure_ui --output target/metrics.json --enforce
```

On Windows, use `target/release/pp-switch.exe`. Replace `PACKAGE` with the
generated ZIP or tar.gz filename. Packaging includes app metadata, the native
icon and license notices, and enforces the installed-size budget. The archive
smoke checks the actual package against the updater's whitelist and architecture.

Keep [PP-11](SPEC.md#acceptance-contract) thresholds unchanged. The measurement
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
| `CONTRIBUTING.md` | Build, validation, packaging and documentation maintenance. |
| `docs/verification.md` | Versioned evidence, measurement method, limitations and dated community sources. |
| `docs/measurements/` | Immutable raw results by version; keep historical files. |

For documentation-only changes, check local links and heading anchors, Markdown
structure, translation coverage and numerical claims against raw measurements.
Do not rebuild the app solely for wording changes. Update affected translations
together, keep sample credentials fake and distinguish document languages from
the four interface languages. Use Git tags and release pages for superseded
development narratives rather than copying them into current instructions.

## Publish a preview

Use native CI packages from the implementation commit; do not substitute an
untested local binary. Attach all four packages, `SHA256SUMS.txt` and
`verification.json`, and compare uploaded sizes/digests with the originals.
Record the implementation SHA, CI run and release tag in verification. Disclose
failed budgets, unsigned packages and migration limits in the release notes.
Exercise public download/checksum/staging in a disposable fixture, and record
network failures separately from authenticated metadata checks.

Only commit task-owned files. Preserve others' edits; do not rewrite history,
force-push or discard work. Keep secrets out of diagnostics and screenshots.
