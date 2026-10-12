# v0.0.4 smoke acceptance

User accepted the installed redesign on 2026-10-12 and authorized publication. Functional GUI checks run only in
**简体中文**. English, 繁體中文 and 日本語 use prepared style states. Every test
uses isolated client/provider files and fake keys; installed user data is preserved.

Design: [native UI](../design/UI.md) · [Lucide sources](../assets/lucide/README.md).

| Check | Result |
| --- | --- |
| Startup no-focus, Tab/Space Settings, same-window page/Back | Chinese native input passes |
| Add/Save only; duplicate records retained | Chinese native input passes |
| Select one row; Switch a different row; every URL/key match active/disabled | Chinese native input passes |
| Each duplicate edited independently; title-close discards draft | Chinese native input passes |
| Invalid name/URL/key; draft preserved and no write | Chinese native input passes |
| Edit Save/Cancel; Delete confirm/cancel without changing client files | Chinese native input passes |
| All client tabs and record-specific Claude/Codex/Gemini switches | Chinese native input passes |
| Native scroll and visible Edit action | Chinese native input passes |
| Language preference; asynchronous Update error and responsive Back/navigation | Chinese native input passes |
| All tooltip properties absent; message Escape/title close and main close | Chinese native input passes |
| Light/dark live recoloring; draft/cursor/focus/selection preserved | Chinese appearance input passes |
| Four-language icons, badges, overflow/scroll, Settings/editor styles | Both-theme pixel/style checks; final evidence below |
| Format/all-target Clippy, Rust behavior, Python detector/release tests | 28 Rust and 8 Python tests passed; final affected checks below |
| Release build, package layout/architecture/checksum/size | Final evidence below |

The native interaction run is `target/settings-lucide-review-20261012`, exit 0.
Appearance run: `target/settings-lucide-themes-20261012`, exit 0. Both use Lucide
SVG-derived paths. Subsequent Add text and selector-corner corrections change
only drawing; final checks reuse these passing functional results and capture
styles again without repeating the functional flow.

[Historical menu baseline](measurements/v0.0.4/menu-review-20261012.json),
[interaction baseline](measurements/v0.0.4/interaction-review-20261012.json) and
[appearance installation](measurements/v0.0.4/install-review-20261012T105643Z.json)
remain unchanged. Import/More/footer scenarios are obsolete, not current gates.
The removed native URL registration CI step previously failed on Intel; that
failure remains historical. All four native CI check jobs now pass; see [release evidence](verification.md#release-004).

The prior installed candidate's unknown hang trigger remains unresolved. Its
[raw sample](measurements/v0.0.4/hang-callgraph-20261012.txt) shows a nested event
loop wait after mouse release. Normal modal/title-close checks pass; this does
not establish the original incident's root cause or guarantee it cannot recur.

No performance suite was rerun and no improvement is claimed. Historical macOS
RSS budgets failed (empty 109.50/109.53 MiB idle/peak; obsolete import scenario
127.33/127.36 MiB). Budgets remain unchanged; future measurements use an editor
scenario instead of the removed import preview. CodeGraph is unavailable (no index).

## Final redesign evidence

[Checks, source/screenshot/package hashes](measurements/v0.0.4/settings-lucide-review-20261012.json) ·
[Installation/backup manifest](measurements/v0.0.4/install-review-20261012T113722Z.json).

Final four-language light/dark styles pass, including language-popup rounded
clipping/native shadow, borderless edge pixels and clean selector corners.
[Light Chinese main](screenshots/v0.0.4/settings-lucide-20261012/light/zh-active.png) ·
[Dark Chinese main](screenshots/v0.0.4/settings-lucide-20261012/dark/zh-active.png) ·
[Settings](screenshots/v0.0.4/settings-lucide-20261012/dark/zh-settings.png).

Installed/opened at `/Users/mimiko/Applications/pp-switch.app`, process 48644 at
verification; previous bundle retained as `.pp-switch-backup-20261012T113722Z`.
Final archive SHA-256:
`e1536046f9154ef22e24bab500a27096fb8ee8f2c18832317f48211e43b5d29d`.
Installed owned bytes: 2,544,096; archive bytes: 1,349,523. The new Chinese callback and isolated editor measurement harnesses also run and
exit normally; the latter collects no new performance samples. All installed files
match the archive; PPC package icon and Lucide notices remain, no URL schemes.
User acceptance, four native CI check jobs and public upload/download verification are complete.
[Acceptance record](measurements/v0.0.4/user-acceptance-20261012.json).
