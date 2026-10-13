# Pure Provider Switch (pp-switch) specification

Status: v0.0.5 development contract. Changes to behavior start here.

Release policy: functional previews may be published with disclosed performance
failures. Report functional and performance checks separately. PP-11 remains
the optimization budget; previews do not relax its thresholds or expand scope.

## Purpose

Full product name: Pure Provider Switch. Repository, executable and compact UI
name: pp-switch. Main window title: Pure Provider Switch. The application icon uses a white geometric S and two opposing sliders on the PPS blue rounded square (`#365DD8`), authored in SVG.
macOS window title bars show the title without an application icon. Windows/Linux window
icons and all platforms' installed application icons retain the PPS logo.

A small desktop application that saves API providers and switches only a
client's base URL and API key. Supported clients: Claude Code, Codex, Gemini CLI.
Supported desktop platforms: macOS (Apple Silicon / Intel), Windows x64, Linux x64.

## Product boundaries

- One main window: three client tabs, provider list and per-row edit/switch actions. Add/edit/delete
  use a compact dialog; a small gear opens an in-window settings page for language
  and strictly manual updates.
- A provider has an ID, client, display name, base URL and API key. Nothing else.
- Closing the window exits the process. No tray, startup agent or background work.
- No models, OAuth management, proxy, protocol translation, failover, network probes,
  usage queries, scripts, MCP, prompts, skills or cloud sync.
- Providers must use the client's existing protocol, models and API-key mode.
- No command-line management interface. Development checks are separate tools.

## Acceptance contract

### PP-01: Provider CRUD

Create, edit and delete a provider. Saving does not change client files. Names, HTTP(S) URLs and nonempty keys are validated; credentials are masked in the UI and excluded from diagnostics.

### PP-02: URL/key writes

Apply modifies only the effective base URL and API key. Preserve all unrelated bytes, including comments, whitespace, ordering and line endings. Idempotent apply makes no file changes.

### PP-03: Claude Code

Claude: patch `env.ANTHROPIC_BASE_URL` and the existing `ANTHROPIC_AUTH_TOKEN` or `ANTHROPIC_API_KEY`. Default to `ANTHROPIC_AUTH_TOKEN` when neither exists. Refuse ambiguous credentials, external key helpers and active cloud-provider modes.

### PP-04: Codex

Codex: patch the currently selected custom provider's `base_url` in `config.toml`; patch its existing `experimental_bearer_token`, or `auth.json.OPENAI_API_KEY` for an existing file-based API-key setup. Do not create/change `model_provider`, auth mode, `wire_api`, OAuth tokens or models. Refuse unsupported key sources or an uninitialized provider.

### PP-05: Gemini CLI

Gemini: patch only `GOOGLE_GEMINI_BASE_URL` and `GEMINI_API_KEY` in `.env`. Require existing `security.auth.selectedType = gemini-api-key`; never change `settings.json`. Preserve unrelated dotenv lines.

### PP-06: Provider links

Accept `ccswitch://v1/import` and `ppswitch://v1/import` links for
`resource=provider` only. Extract client, name, base URL and API key from direct
parameters or bounded inline JSON/TOML; discard unrelated settings. Never fetch
remote config URLs. Bound each link to 256 KiB and the native pending queue to
8 links. Show the three-field editor as an unsaved preview; only Save persists a
provider, and saving never applies client files. A new link must not replace an
open editor's draft; report that the user must finish/cancel and reopen the link.
Accept OS URL events on both cold and running macOS launches. Packaged macOS
apps declare both schemes. Settings offers explicit registration for otherwise
unowned schemes, preserving other installed applications' handlers. Startup
never changes URL defaults. URL arguments are delivery only, not CLI management.

### PP-07: Directories and overrides

Use client configuration-directory overrides where supported. Use platform-specific application-data storage for pp-switch. Do not modify shell profiles or system environment. Detect overrides visible to pp-switch; state that a separately launched client's environment may differ.

### PP-08: Secrets and network

Persist secrets in user-only files (Unix 0600; Windows current-user ACL). No provider network requests, secret logging or automatic clipboard access. The only application network action is the explicit PP-15 update flow, which sends no provider data. Never claim that masking encrypts storage.

### PP-09: Writes and recovery

Before writing, validate every involved file. Refuse symbolic-link file destinations instead of replacing their links. Serialize pp-switch writes with an OS lock. Detect changes since reading. Replace each file via a same-directory temporary file; retain recovery data for multi-file operations. On failure/restart, roll back only files still matching this operation's output; refuse to overwrite external changes. A cross-file update is recoverable, not globally atomic.

### PP-10: Current status

Display current status by reading client configuration, rather than trusting a stored active-provider ID. Allow duplicate saved providers, including identical records; every record for the selected client matching the effective URL and key displays In use and cannot be switched again. Editing identifies individual records; rows have no independent selection state. There is no bottom Current/status or global Switch area; status is represented by row badges. Setup errors are reported when switching is attempted. Report incomplete/unsupported setup without silently fixing unrelated settings. Existing clients may require restart to reload configuration.

### PP-11: Performance budgets

The primary v0.0.5 work is reducing macOS memory, startup time and package size,
with comparable release-build measurements. Installed application-owned files ≤5 MiB; steady idle total RSS ≤20 MiB; startup/editor peak total RSS ≤30 MiB; cold interactive startup ≤300 ms. Measure a stripped release build on each platform, state hardware/method and report failures. These are budgets, not verified claims. No subprocess/runtime exclusions for memory.

### PP-12: Public evidence and CI

Publish source, license, build instructions, compatibility limits and actual verification evidence. CI runs formatting, lint, behavioral tests and release builds on all three OSes. No platform is described as tested before evidence exists.

Pushing a version tag matching Cargo.toml automatically validates and builds all
four native packages and publishes a GitHub preview release, checksums and
verification.json from that same commit. Functional/size failures block publishing;
RSS/startup budget failures remain visible in raw evidence and release notes.
Publishing must not use partial or stale artifacts or replace an existing release.

### PP-13: Languages

Match cc-switch's interface languages: English, Simplified Chinese, Traditional Chinese and Japanese. Detect the system language on first launch; unsupported locales fall back to English. A compact language selector updates labels, dialogs, status and user-facing errors immediately, preserving unsaved edits and keyboard focus. Persist only the selected language in a private local UI preference file. Invalid preferences are reported and never silently overwritten. README versions cover English, Simplified Chinese, Japanese and German; user guides cover English, Simplified Chinese and Japanese, with the same compatibility and measured-performance caveats. SPEC remains the single English behavior contract.

### PP-14: Minimal interface

Follow [design/UI.md](design/UI.md): three client tabs and a smaller Settings
gear at the top, a provider list below, and visible Edit then Switch icon buttons
on each row. Do not show an in-content product heading, More menu, bottom status
or global Switch button. Rows have no selection circle or selected background; Edit and Switch each
target their own record by ID. Matching records show In use and have
disabled Switch. Add/edit use a three-field dialog; saving never applies.
Settings replaces the list within the same window and contains language, import-link registration and
manual updates, with a Back action. No hover tooltips anywhere. Initial activation
focuses no control; later keyboard navigation retains visible focus. Preserve
pressed/disabled states, contrast, masked credentials and unsaved drafts. Closing
or cancelling an editor discards its draft without writing. Use system fonts and
bounded vector icons, without new UI dependencies or background rendering.
The application icon remains the flat blue PPS switch mark in OS packages; macOS
window titles omit it, while other platforms retain their window icons.

Follow system light/dark appearance without a theme selector or saved preference.
Recolor existing widgets without losing drafts, text selection or focus. Chinese native
functional acceptance and all four languages' light/dark style checks must cover
row actions, settings, localized badge visibility and keyboard navigation.
PP-11 budgets remain unchanged.

### PP-15: Manual updates

Updates are strictly user-triggered: no startup, scheduled or background checks. A compact Check for updates action queries only this repository's GitHub Releases over verified HTTPS; preview builds may receive newer previews. Show version and request confirmation before download/install/restart, warning about unsaved edits. Keep the GUI responsive during network work. Select only the current OS/architecture package, bound download/extraction size and whitelist application-owned files. Verify the archive against GitHub's SHA-256 asset digest before staging. Reject malformed metadata, path traversal, symlinks, unsupported layouts and unwritable installations without editing provider/client files. A temporary helper waits for the app to exit, checks the destination has not changed, replaces owned files with backups, rolls back failed installs and restarts. Report failures in the selected language. Release assets remain unsigned; a GitHub digest verifies download integrity, not an independent publisher signature.

## Implementation decisions

Rust core, FLTK native UI and a local JSON provider store. Use mature parsers;
do not write a general-purpose JSON/TOML parser. Patch only validated value
ranges. Keep the dependency graph small and omit unused FLTK features.

Inside the app's data directory, the only persistent state beyond providers is a
bounded recovery journal, an advisory lock and one UI language preference. No
plugin system, daemon, database or adapter SDK. User-triggered updates may use bounded temporary staging and
rollback files, plus a temporary copy of the app as an installation helper.
There is no permanent helper process. Translations are compiled text. Native
FLTK drawing uses system fonts, without an internationalization runtime.
Configuration files are read only on launch, client selection and user actions.

## Compatibility reference

cc-switch commit `8845b1ef799aa8810b864b122de6af5f4629de10`:

- Historical reference: [Deep-link parser](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/deeplink/parser.rs)
- [Provider config extraction](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/deeplink/provider.rs)
- [Claude projection](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/live/project/claude.rs)
- [Gemini projection](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/live/project/gemini.rs)

pp-switch does not share cc-switch's database, routing state or full provider
settings. Provider-link compatibility is limited to the PP-06 fields and preview flow.

## Verification

Behavioral tests reference PP IDs and exercise real temporary files: unrelated
byte preservation, effective-provider selection, malformed
inputs, idempotency, permissions, rollback and concurrent-change recovery.
A GUI smoke harness exercises the same widgets/callbacks as the shipped window.
Record release size, actual process RSS and startup measurements in
[verification](docs/verification.md). Build and validation commands live in
[CONTRIBUTING.md](CONTRIBUTING.md). Update this contract before changing behavior.
