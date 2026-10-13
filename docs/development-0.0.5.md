# v0.0.5 development checkpoint

Development originated on `codex/v0.0.5`, based on merged v0.0.4 (`58a299c`).
The v0.0.5 tag was not released; these product changes ship in v0.0.6.
[Current published evidence](verification.md#release-006) supersedes this local checkpoint
for release acceptance. The records below describe their original binaries.
[Spec](../SPEC.md) owns behavior, [UI rules](../design/UI.md) presentation,
[guides](user-guide.md) usage. This page records historical evidence and open work.
The v0.0.5 tag failed at a CI test bootstrap; it remains unpublished and immutable.
The corrected v0.0.6 preview is published without changing the product feature scope.
[Failure record](measurements/v0.0.5/release-preparation-20261013/ci-rtk-failure.json).
Publication passed all four native targets and uploaded checksum verification;
the current download is v0.0.6. [Release preparation](measurements/v0.0.5/release-preparation-20261013/code-checks.json)
adds native acceptance assertions for visible Claude defaults and clearing before
direct Switch. Earlier acceptance logic still expected the removed compulsory
model dialog; that fixture and the packaged README were corrected before tagging.

## Behavior at the checkpoint

- Five harnesses: Claude, shared Codex CLI/App, Gemini, Grok and shared DSH CLI/Web.
- Per-row Edit/Switch, fixed Official first row, no independent row selection.
- Optional models including Claude, scoped native/API transitions.
- Connection-scoped model prefill across all five harnesses; Claude Add/Edit/import additionally uses visible `claude-sonnet-5` when no specific model exists. Explicit values win; clearing opts out and Save remains separate from Switch.
- CCS/Sub2API Provider preview, collision suffixes such as `Name (2)`, preserved drafts.
- Compact pinned brand tabs, Lucide actions, full title and PPS logo.
- Blue **In use / 使用中** names, addresses and badges; green status colors retired.
- macOS Quit explicitly handles the Dock AppleEvent and accepts Cocoa termination.

The badge checks supported user files, not a live session or request. Save stores
only. Official restores native routing while retaining credentials and unrelated
settings; it does not log in. Full behavior belongs in the spec, not this checklist.

## Verification at the checkpoint

[Default-model correction checks](measurements/v0.0.5/model-default-20261013/code-checks.json)
record format, all-target Clippy, 82 Rust tests, 18 Python tests and a release build.
The release background regression checks actual Claude Add/Edit/import field values,
explicit and same-connection priorities, Cancel/Save isolation, clearing the model
and direct model-less Switch. Other clients' new fields stay blank.
[Checkpoint candidate and installation](measurements/v0.0.5/model-default-20261013/report.json)
record exact package validation and 2,788,557 owned bytes. The exited application
was replaced with a private backup, without launching or writing client files.
[Reference evidence](measurements/v0.0.5/model-default-20261013/research.json)
records that cc-switch permits Claude imports without a model, while Sub2API
omits it. PPS's previous compulsory completion was removed; the visible editor
fallback is an editable suggestion, not universal endpoint compatibility.
Native pointer and pixel acceptance were pending for this checkpoint binary.

[Previous model-prefill code checks](measurements/v0.0.5/model-prefill-20261013/code-checks.json)
retain commands, logs and exit codes. The isolated background callback regression
covers Claude Edit/Switch/import/Save/Cancel and unchanged client fixture files;
backend fixtures cover all five clients, connection mismatches and explicit values.
Native pointer and pixel acceptance were pending for this checkpoint binary.
[Previous candidate and installation](measurements/v0.0.5/model-prefill-20261013/report.json)
record 82 Rust tests, 18 Python tests on recheck and release callback acceptance.
The initial Python run retains a foreground-process equality assertion failure;
disassembly identifies that guard, with the app-inactive assertion already passed.
The unchanged guard passed five targeted cases and the complete recheck. The
2,788,573-byte app was installed with a backup while PPS was exited, without
launching or writing client files.

[Quit-fix code checks](measurements/v0.0.5/quit-fltk-20261013/code-checks.json) retain
commands, logs and exit codes. The user's second failure revealed that real FLTK
startup skips Cocoa's default quit handler when NSApp already exists. New tests
use real FLTK widgets/delegate/wait across main, Settings and editor; test-only
suppression prevents native ordering and activation. A missing-handler negative
control reproduces -1708. These tests do not certify an actual Dock pointer click.
[Earlier isolated regression and staged candidate](measurements/v0.0.5/quit-docs-20261013/report.json)
are superseded for this lifecycle behavior; their raw evidence remains intact.
[Previous quit-fix candidate and installation](measurements/v0.0.5/quit-fltk-20261013/report.json)
record 80 Rust tests, 17 Python tests, 12 real FLTK release lifecycle scenarios
(six unbundled and six bundled) plus two missing-handler negative controls.
The application owns 2,788,557 bytes; its exact archive passed updater validation.
The standard user Applications bundle was replaced only while no PPS process was
running, with a private backup. No launch or client configuration write occurred.
[Previous Official/model baseline and installation](measurements/v0.0.5/official-20261013/report.json)
is historical for changed inputs; its installed package is 2.66 MiB.

[Claude live repair](measurements/v0.0.5/claude-live-20261013/report.json) verified
a compatible model through the existing proxy in isolated native Claude; the
user-authorized deployment preserved unrelated configuration. No further live
requests are needed for this correction.

## Workstation constraints and remaining work at the checkpoint

Preserve foreground focus, pointer, keyboard and running app drafts. Local smoke
defaults to background checks. Native GUI/input, URL open/restart and window
measurements need a dedicated desktop or explicit idle-period opt-in; never spoof
CI. Unrun gates are pending, never passes.

Pending: affected Chinese native interactions, four-language light/dark blue badge
pixels, actual Dock pointer acceptance, current whole-app memory/startup
measurements and native Windows/Linux/Intel branch checks. CodeGraph is unavailable
(no worktree index); bounded source and consumer reads were used instead.

The primary release goal remains macOS memory, startup and package size.
[Performance evidence](verification.md) establishes no material memory/startup gain;
PP-11 budgets remain unchanged. The older unexplained native hang is distinct from
this termination cancellation and remains unresolved.

## Earlier checkpoints

[Archived chronology](history/v0.0.5-checkpoints-20261013.md) preserves earlier
measurements, installation records and raw links. Counts and images apply only
to those candidates. [Image provenance](images.md).
