# Verification evidence

[Documentation](README.md) · [Acceptance](smoke.md) · [Reproduce](../CONTRIBUTING.md)

## Development checkpoint: 0.0.5

The branch is not published. [Current checkpoint](development-0.0.5.md) separates
background validation from pending native acceptance. Earlier captures do not
certify later Grok/DSH, model, Official, compact-tab or blue-status changes.

The `v0.0.5` preview tag points to `80c460c0c543da703e77225b8f66a5b220b9ad2d`.
[Tag-driven native CI](https://github.com/phonowell/pp-switch/actions/runs/38391365065)
failed at the macOS test bootstrap because it depended on the local-only `rtk`
wrapper. [Failure and correction](measurements/v0.0.5/release-preparation-20261013/ci-rtk-failure.json).
That tag remains immutable and unpublished. The next preview is v0.0.6, retaining
the v0.0.5 product changes and correcting the portable test bootstrap. Native CI
and public package verification must complete before publication.

The [first v0.0.6 preflight](https://github.com/phonowell/pp-switch/actions/runs/38391812689)
passed Windows/Linux and macOS native input, URL delivery and termination, then
exposed a stale green-only pixel detector. The checker now isolates blue badges
from blue row text and normalizes antialiasing against the captured background.
[Both macOS screenshot sets pass the targeted correction](measurements/v0.0.6/pixel-checker-20261013/report.json).
Light/dark CI gates and final publication remain pending.

The [current installed candidate](measurements/v0.0.5/model-default-20261013/report.json)
has 2,788,557 owned bytes (2.66 MiB), 82 Rust tests and 18 Python tests.
It removes compulsory Claude model entry and verifies actual Add/Edit/import
prefill plus blank-model switching through background production callbacks.
Exact archive validation passed; no user client files or foreground input were changed.
The [earlier Official baseline](measurements/v0.0.5/official-20261013/report.json)
is historical for these inputs.
The second Dock Quit failure supersedes the earlier isolated Cocoa regression.
Real FLTK startup with a pre-created NSApp skips `finishLaunching`, leaving the
default quit AppleEvent unhandled (`errAEEventNotHandled`, -1708). PPS now registers
that event explicitly and accepts Cocoa termination. Background tests exercise
the shipped widgets, real delegate and wait across main/Settings/editor, with
native presentation suppressed. Removing the handler reproduces the failure.
Actual Dock pointer acceptance remains pending.
[Original quit-fix code checks](measurements/v0.0.5/quit-fltk-20261013/code-checks.json).

The [earlier Mac M4 comparison](history/v0.0.5-checkpoints-20261013.md#memory-and-startup-comparison)
found empty idle RSS 110.77 MiB for both versions and editor idle RSS 129.00 / 128.77 MiB.
These are earlier candidates, not today's binary. Memory budgets failed; no material
startup or memory gain was established. [Single-snapshot reads](measurements/v0.0.5/official-20261013/config-read-summary.json)
reduce cumulative requested allocations in fixtures, not measured whole-app RSS.

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
