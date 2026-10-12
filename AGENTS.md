# Pure Provider Switch — agent instructions

Keep the product limited to URL/key switching, UI language
and strictly user-triggered updates. Preserve user configuration and secrets;
never test writes against real client files. Commit only task-owned changes;
do not discard others' edits, rewrite history or force-push.

## Default workflow

1. **Locate:** run `rtk git status --short` and `rtk proxy git diff -- <target>`.
   For code tasks, use `rtk proxy codegraph explore <target>` to inspect the target
   and direct consumers. Returned source counts as read only for visible segments.
   Reuse unchanged source/graph evidence; investigate only specific missing relations.
2. **Change:** read only the relevant entries below. Change the PP contract first
   when intended behavior or scope changes; fixes that restore it need no spec rewrite.
   Prefer existing dependencies and the smallest implementation. Update affected docs.
3. **Verify:** code requires format, all-target Clippy, behavior tests and a release
   build; UI changes also require GUI acceptance. Pure docs require Markdown,
   link/anchor, translation and evidence checks. Reuse passed checks with unchanged
   inputs. Report results and unverified items; tie performance claims to measurements.

Full GUI functionality is tested in **Simplified Chinese only**; all four interface
languages receive style/property/pixel checks. Automate regressions first;
human screenshot review is the final fallback. After each correction, run only
checks affected by changed inputs and reuse the others' logs/hashes. Do not run
the full smoke suite for every correction; see [verification scope](CONTRIBUTING.md#incremental-verification).

Shell commands must use `rtk proxy`; filtered status is only an overview.
Rules, source, diffs, diagnostics and acceptance evidence need raw output and exit
codes. Empty filtered output or a success summary alone is not proof. For missing
output or tool failures, read [tool recovery](CONTRIBUTING.md#tool-and-output-recovery).
Documentation-only tasks do not require a code graph query.

## Read by task

| Task | Read |
| --- | --- |
| Client configuration writes | Relevant [Claude](SPEC.md#pp-03-claude-code), [Codex](SPEC.md#pp-04-codex) or [Gemini](SPEC.md#pp-05-gemini-cli) contract, plus [write limits](SPEC.md#pp-02-urlkey-writes), [secrets](SPEC.md#pp-08-secrets-and-network) and [recovery](SPEC.md#pp-09-writes-and-recovery). |
| Providers, paths or status | Relevant [PP-01](SPEC.md#pp-01-provider-crud), [PP-07](SPEC.md#pp-07-directories-and-overrides) or [PP-10](SPEC.md#pp-10-current-status). |
| Updates | [PP-15](SPEC.md#pp-15-manual-updates), plus [secrets](SPEC.md#pp-08-secrets-and-network). |
| UI or languages | [PP-14](SPEC.md#pp-14-minimal-interface) or [PP-13](SPEC.md#pp-13-languages), plus [UI rules and regression gates](design/UI.md); follow other PP entries for the behavior being changed. |
| Code validation | [Validation](CONTRIBUTING.md#validate-code-and-behavior); [build](CONTRIBUTING.md#build) only for setup. Add tests for behavior, not implementation details. |
| Documentation | [Ownership and checks](CONTRIBUTING.md#documentation-ownership). |
| Packaging, performance or release | Relevant [packaging](CONTRIBUTING.md#package-and-measure) or [publishing](CONTRIBUTING.md#publish-a-preview) chapter, [PP-11](SPEC.md#pp-11-performance-budgets) and [PP-12](SPEC.md#pp-12-public-evidence-and-ci); read measurement evidence when making claims. |
| Scope changes | [Purpose](SPEC.md#purpose), [boundaries](SPEC.md#product-boundaries) and the affected PP entries. |

Current UI: per-row Edit/Switch and in-window Settings; no hover tips, More menu,
content branding or bottom status/Switch. Use pinned Lucide icons, never handmade
substitutes or font glyphs. Preserve native OS chrome and the PPC app logo.
Chinese functional checks only; other locales style only. Reuse passed checks
when inputs are unchanged; visual review is the last fallback.
