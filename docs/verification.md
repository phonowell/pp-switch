# Verification evidence

[Documentation](README.md) · [Acceptance](smoke.md) · [Reproduce](../CONTRIBUTING.md)

## Release: 0.0.6

[Published preview](https://github.com/phonowell/pp-switch/releases/tag/v0.0.6): implementation `74ee93fedb2fc2c8deed846cb15d7310190a5a16`.
[Four native CI targets](https://github.com/phonowell/pp-switch/actions/runs/38393449261) passed format, all-target Clippy, behavior,
packaging, GUI and update checks. macOS additionally passed Chinese native input,
Provider URL delivery, six native termination scenarios and four-language
light/dark pixel checks. Reviewed native captures show the actual Claude default,
readable blue active rows and Settings without a redundant heading.
[Retained job logs and reports](measurements/v0.0.6/native-ci-20261013/run.json) ·
[Native capture provenance](screenshots/v0.0.6/aarch64-apple-darwin/provenance.json).

The first Intel attempt failed while capturing a validation dialog. Its
[raw failure and targeted recheck](measurements/v0.0.6/native-capture-20261013/report.json)
are retained. Only that job was rerun at the same implementation SHA, with all
assertions intact; its complete native gates passed on the second attempt.
[Publication provenance](measurements/v0.0.6/release-process.json) identifies the
four successful artifacts reused before creating the immutable matching tag.

Grok and shared DSH CLI/Web, scoped Official rows, optional models, Sub2API
previews and numeric name suffixes are included. Save remains separate from Switch.
Claude Add/Edit/import fields suggest `claude-sonnet-5` when no explicit,
mapped or same-connection model exists; clearing opts out of model writes.
That suggestion does not guarantee endpoint compatibility.

| Native runner | Installed bytes | Idle / peak MiB | Readiness max ms | Editor peak MiB | Both script budgets |
| --- | ---: | ---: | ---: | ---: | --- |
| macOS 15 · ARM64 | 2,788,597 | 48.55 / 48.59 | 180.01 | 51.72 | Fail |
| macOS 15 · Intel | 3,094,789 | 32.41 / 32.41 | 136.63 | 35.86 | Fail |
| Windows Server 2022 · x64 | 3,276,633 | 15.21 / 15.60 | 44.09 | 16.16 | Fail |
| Ubuntu 22.04 · x64 · Xvfb | 3,563,651 | 21.20 / 21.38 | 19.28 | 22.38 | Fail |

[Raw release evidence](measurements/v0.0.6/verification.json) ·
[Public download and checksum validation](measurements/v0.0.6/publication-verification.json).
All installed packages fit 5 MiB. macOS total RSS exceeds the original memory
budget; native Cocoa comparison, import stress and action-readiness samples are
retained. A bare Cocoa window is diagnostic, not a feature-equivalent baseline.
Cold startup and update peaks remain unverified; no overall memory/startup gain
is established. Packages are unsigned and macOS is not notarized. Actual Dock
pointer clicks and local physical Windows/Linux/Intel desktops remain untested.
Performance issue #1 stays open.

The v0.0.5 tag remains an unpublished failed candidate. Its
[test-bootstrap failure](measurements/v0.0.5/release-preparation-20261013/ci-rtk-failure.json)
and the [blue pixel-detector correction](measurements/v0.0.6/pixel-checker-20261013/report.json)
remain recorded without rewriting tag history. The
[v0.0.5 local development checkpoint](development-0.0.5.md) retains earlier
installations and pending checks for those specific binaries. Release CI resolves
the automated native gates for v0.0.6; historical evidence is not today's installation.

## Release: 0.0.4

[Published release](https://github.com/phonowell/pp-switch/releases/tag/v0.0.4):
implementation `47d83267796d901db333ec06d0e523dda900917b`.
[Four native CI checks](https://github.com/phonowell/pp-switch/actions/runs/38245799635)
passed. [User acceptance](measurements/v0.0.4/user-acceptance-20261012.json) and
[archived matrix](history/acceptance-0.0.4.md) describe this version only.

| Native runner | Installed bytes | Idle / peak MiB | READY median / max ms | Editor peak MiB | Both script budgets |
| --- | ---: | ---: | ---: | ---: | --- |
| macOS ARM64 | 2,544,128 | 48.44 / 48.50 | 173.12 / 205.80 | 51.17 | Fail |
| macOS Intel | 2,803,992 | 31.97 / 31.98 | 154.56 / 155.76 | 34.86 | Fail |
| Windows x64 | 2,915,551 | 15.08 / 15.45 | 37.82 / 42.34 | 15.91 | Pass |
| Linux x64 | 3,158,143 | 16.96 / 19.27 | 21.04 / 24.52 | 22.25 | Fail |

[Raw CI measurements](measurements/v0.0.4/verification.json) ·
[Asset/download validation](measurements/v0.0.4/publication-verification.json).
macOS exceeds memory budgets. Linux passes the empty window but exceeds editor
idle RSS. Cold interactive startup and update peaks are unverified. Packages are
unsigned; macOS is not notarized. Anonymous update metadata returned rate-limit
403; authenticated fixture metadata was recorded separately. The app uses no
GitHub credentials.

## Measurement method and limits

[measure_ui](../examples/measure_ui.rs) runs a production window/editor with
isolated files and fake credentials. [measure.py](../scripts/measure.py) starts
three fresh processes, samples total RSS every 50 ms and samples idle after one
second. READY is the first event-loop timer, not verified cold interactive readiness;
OS caches remain. Peak is the OS high-water mark. Installed size includes all
owned files: executable, metadata, icons and license notices. Shared pages and
system fonts count toward RSS; physical footprint and allocations are other metrics.

CI uses separate VMs, with Xvfb on Linux; results are not controlled cross-platform
comparisons. Minimum OS declarations do not prove every version was tested.
Local Windows/Linux/Intel physical desktops remain untested. Chinese GUI flows
cover functionality; four languages cover styles. Fixture tests do not certify
live model/protocol compatibility. The separately authorized [Claude live diagnosis](measurements/v0.0.5/claude-live-20261013/report.json)
is a bounded exception, not a provider availability benchmark.

## Screenshots

[Image inventory](images.md) separates current illustrations from native captures.
README SVGs are schematics, not GUI evidence. Historical capture paths and bytes
remain unchanged, including original labels/colors.

## Historical releases

[Archived 0.0.1–0.0.4 narratives](history/verification-through-0.0.4.md).
Old package sizes, test counts and measurements are not current. The earlier
unexplained 0.0.4 hang remains unresolved; its sample does not identify the trigger.

## Community sources

[Dated cc-switch research](history/verification-through-0.0.4.md#community-sources)
records scope motivation and limits. Individual reports do not establish a current
universal defect, endorsement or comparative performance result.
