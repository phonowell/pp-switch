# Manual update review follow-up — 2026-10-14

This working-tree change addresses the review against [PP-15](../SPEC.md#pp-15-manual-updates).
It is not a published release or desktop acceptance claim. Existing unrelated
performance/documentation edits remain in the worktree.

| Review | Resolution |
| --- | --- |
| 1 — repeated checks | Cache successful metadata results for five minutes in the running app, including no update and cancelled installation. Explicit clicks reuse the offer; failures remain retryable. Recognize GitHub rate-limit responses separately. |
| 2 — inherited Claude environment | Cargo's test runner removes inherited client overrides before starting test threads. Background smoke also isolates fixture examples. Explicit environment-conflict subprocess tests retain their synthetic overrides. |
| 3 — untranslated status | The premise was incorrect: `Controls::message` already calls `i18n::message` and retains the source key for language changes. Add translation coverage tests and a clarifying comment. |
| 4 — inconsistent asset rejection | Invalid digest, URL, size and ambiguous duplicates all skip that release, with a comment and behavior tests. A valid alternative remains selectable. |
| 5 — release list truncation | Scan pages of 20 under one 45-second network deadline and a total 2-MiB metadata bound. Do not assume API creation order equals version order. An unfinished scan reports an error. |
| 6 — mismatched watchdog | Derive the 60-second UI watchdog from the shared 45-second network timeout plus 15 seconds. Consume completed workers immediately on the next poll; do not wait for the watchdog. |
| 7 — staging errors | Core errors already pass through the same localized UI message path; the stage language is saved for the helper. Preserve this design and test staging/error message translations and current-language installation dispatch. |
| 8 — redirect hosts | Extract the allowlist as a constant, keep strict HTTPS validation and report an unapproved host distinctly in all interface languages. |
| 9 — polling | Poll at 500 ms instead of 100 ms. No CPU or RSS improvement is claimed without measurements. |
| 10 — lifecycle coverage | Extract the actual worker lifecycle for background tests. Add Chinese callback acceptance for reentry, cancellation, staging error, helper-launch error and successful exit dispatch using isolated injected operations. |
| 11 — release changes | Add a localized Release notes action to installation confirmation, opening the selected version's repository release page only on a click. |
| 12 — positioning | Put the repeated-config-editing value proposition in the first sentence of all four READMEs. |
| 13 — paired asset naming | Document the filename contract at both Rust selection and Python assembly, and in Contributing. Validate the actual package with the updater. |
| 14 — failure contract | Extend PP-15 with cancellation, network/rate-limit, metadata bounds, permissions/disk/locked files, staging, helper launch, rollback conflict and failure/quit behavior. Align all three user guides. |
| 15 — recovery overhead | No change: `storage::read` returns immediately on a missing journal; parsing occurs only when it exists. A separate existence check would add another filesystem operation and could mask read errors. Recovery must remain under the write lock. |
| 16 — permission probe order | No change: the probe already precedes owned-file hashing and network work. It follows locating and validating the package, so an invalid package is rejected before a probe write. |

Background validation passed after targeted dependency rechecks:
format, all-target Clippy, 112 Rust tests, 30 script tests, release build,
documentation checks, icon parity, packaging and real archive validation.
An additional normal Cargo invocation with a synthetic `ANTHROPIC_BASE_URL`
passed all five PP-03 tests. No writes used real client files.

The first smoke run retained two missing-`fonttools` failures. Install the pinned
development dependency in an isolated `target/` virtual environment and rerun
only those gates; both passed. The original report and failures remain intact.
See the [combined verification record](measurements/update-review-20261014/verification.json),
[original smoke report](measurements/update-review-20261014/smoke-report.json),
and [targeted rechecks](measurements/update-review-20261014/targeted-rechecks.json)
for commands, exit codes, logs and source hashes.

Before committing, export only the staged changes and validate that isolated
tree: documentation and 24 script tests pass. The earlier 30-test worktree run
also included six preexisting performance-budget tests, which are not part of
this commit. Rust/build inputs match the previously validated worktree, so those
passed checks are reused. [Staged validation](measurements/update-review-20261014/staged-validation.json)
records that boundary; unrelated performance work remains uncommitted.

Desktop checks were skipped under the shared-workstation rule: new Chinese
callback/native confirmation acceptance, four-language light/dark confirmation
captures and visual/pixel review, release-notes browser delivery and helper
restart/quit remain unverified. Old captures cannot certify the new dialog.
Other native targets also require their CI hosts. No performance measurement or
public release was performed.
