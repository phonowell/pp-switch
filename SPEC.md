# pp-switch specification

Status: v0.0.3 implementation contract. Changes to behavior start here.

Release policy: 0.0.1 is a functional preview. The user authorized preview
delivery with disclosed performance failures on 2026-10-12. PP-11 remains the
optimization budget, not a condition for this preview to exist. Functional
checks and performance checks must report separate outcomes. Later optimization
must not silently relax the original thresholds or expand the product scope.

## Purpose

A small desktop application that saves API providers and switches only a
client's base URL and API key. Supported clients: Claude Code, Codex, Gemini CLI.
Supported desktop platforms: macOS (Apple Silicon / Intel), Windows x64, Linux x64.

## Product boundaries

- One main window: three client tabs, provider list and switch action. Add/edit/delete
  use a compact dialog; import, language and manual updates use the More menu.
- A provider has an ID, client, display name, base URL and API key. Nothing else.
- Closing the window exits the process. No tray, startup agent or background work.
- No models, OAuth management, proxy, protocol translation, failover, network probes,
  usage queries, scripts, MCP, prompts, skills or cloud sync.
- Providers must use the client's existing protocol, models and API-key mode.
- No command-line management interface. Development checks are separate tools.

## Acceptance contract

| ID | Requirement |
| --- | --- |
| PP-01 | Create, edit and delete a provider. Saving does not change client files. Names, HTTP(S) URLs and nonempty keys are validated; credentials are masked in the UI and excluded from diagnostics. |
| PP-02 | Apply modifies only the effective base URL and API key. Preserve all unrelated bytes, including comments, whitespace, ordering and line endings. Idempotent apply makes no file changes. |
| PP-03 | Claude: patch `env.ANTHROPIC_BASE_URL` and the existing `ANTHROPIC_AUTH_TOKEN` or `ANTHROPIC_API_KEY`. Default to `ANTHROPIC_AUTH_TOKEN` when neither exists. Refuse ambiguous credentials, external key helpers and active cloud-provider modes. |
| PP-04 | Codex: patch the currently selected custom provider's `base_url` in `config.toml`; patch its existing `experimental_bearer_token`, or `auth.json.OPENAI_API_KEY` for an existing file-based API-key setup. Do not create/change `model_provider`, auth mode, `wire_api`, OAuth tokens or models. Refuse unsupported key sources or an uninitialized provider. |
| PP-05 | Gemini: patch only `GOOGLE_GEMINI_BASE_URL` and `GEMINI_API_KEY` in `.env`. Require existing `security.auth.selectedType = gemini-api-key`; never change `settings.json`. Preserve unrelated dotenv lines. |
| PP-06 | Import `ccswitch://v1/import?resource=provider` and equivalent `ppswitch://` text. Support claude/codex/gemini, URL decoding, inline Base64 config (standard/URL-safe, padded/unpadded), JSON/TOML and explicit nonempty parameter precedence. Use the first endpoint. Extract only name/client/address/key; never apply imported extra fields. Reject other resources/apps, unsupported versions, remote-only config and malformed/incomplete input. Import previews the four editable fields; it does not apply. |
| PP-07 | Use client configuration-directory overrides where supported. Use platform-specific application-data storage for pp-switch. Do not modify shell profiles or system environment. Detect overrides visible to pp-switch; state that a separately launched client's environment may differ. |
| PP-08 | Persist secrets in user-only files (Unix 0600; Windows current-user ACL). No provider network requests, secret logging or automatic clipboard access. The only application network action is the explicit PP-15 update flow, which sends no provider data. Never claim that masking encrypts storage. |
| PP-09 | Before writing, validate every involved file. Refuse symbolic-link file destinations instead of replacing their links. Serialize pp-switch writes with an OS lock. Detect changes since reading. Replace each file via a same-directory temporary file; retain recovery data for multi-file operations. On failure/restart, roll back only files still matching this operation's output; refuse to overwrite external changes. A cross-file update is recoverable, not globally atomic. |
| PP-10 | Display current status by reading client configuration, rather than trusting a stored active-provider ID. Report incomplete/unsupported setup without silently fixing unrelated settings. Existing clients may require restart to reload configuration. |
| PP-11 | Installed application-owned files ≤5 MiB; steady idle total RSS ≤20 MiB; startup/import peak total RSS ≤30 MiB; cold interactive startup ≤300 ms. Measure a stripped release build on each platform, state hardware/method and report failures. These are budgets, not verified claims. No subprocess/runtime exclusions for memory. |
| PP-12 | Publish source, license, build instructions, compatibility limits and actual verification evidence. CI runs formatting, lint, behavioral tests and release builds on all three OSes. No platform is described as tested before evidence exists. |
| PP-13 | Match cc-switch's interface languages: English, Simplified Chinese, Traditional Chinese and Japanese. Detect the system language on first launch; unsupported locales fall back to English. A compact language selector updates labels, dialogs, status and user-facing errors immediately, preserving unsaved edits and selection. Persist only the selected language in a private local UI preference file. Invalid preferences are reported and never silently overwritten. README versions cover English, Simplified Chinese, Japanese and German; user guides cover English, Simplified Chinese and Japanese, with the same compatibility and measured-performance caveats. SPEC remains the single English behavior contract. |
| PP-14 | Follow the user-approved `design/minimal.html` layout with a flat native style: small title, three client tabs, one provider list with name/address, a selected-row edit action and a primary switch button. Show the active provider from current client files; disable switching an already matching provider. Add/edit use a three-field dialog; saving never applies. Import previews that dialog. Language and strictly manual updates live in More. Remove branding panels, nested cards, raised/inset effects and shadows. Keep visible contrast, keyboard focus, pressed/disabled states and masked credentials. Closing or cancelling an editor discards its unsaved draft without writing. Use system fonts and bounded vector drawing, without animations, new UI/runtime dependencies or background rendering. The only bitmap assets are the user-requested application icon: a flat blue pp monogram, generated from a small SVG for the window and OS package formats. Review real screenshots in all four languages and remeasure release size/RSS including the icon. PP-11 budgets remain unchanged. |
| PP-15 | Updates are strictly user-triggered: no startup, scheduled or background checks. A compact Check for updates action queries only this repository's GitHub Releases over verified HTTPS; preview builds may receive newer previews. Show version and request confirmation before download/install/restart, warning about unsaved edits. Keep the GUI responsive during network work. Select only the current OS/architecture package, bound download/extraction size and whitelist application-owned files. Verify the archive against GitHub's SHA-256 asset digest before staging. Reject malformed metadata, path traversal, symlinks, unsupported layouts and unwritable installations without editing provider/client files. A temporary helper waits for the app to exit, checks the destination has not changed, replaces owned files with backups, rolls back failed installs and restarts. Report failures in the selected language. Release assets remain unsigned; a GitHub digest verifies download integrity, not an independent publisher signature. |

## Implementation decisions

Rust core, FLTK candidate UI, a local JSON provider store. Use mature parsers;
do not write a general-purpose JSON/TOML parser. Patch only validated value
ranges. Keep the dependency graph small and omit unused FLTK features.

macOS 0.0.2 used a strict five-file update layout and cannot accept the new icon
resource in 0.0.3. Those users must download 0.0.3 manually once; its updater
includes the icon. Windows/Linux package layouts remain unchanged.

The only persistent state beyond providers is a bounded recovery journal, an
advisory lock and one UI language preference. No plugin system, daemon, database
or adapter SDK. User-triggered updates may use bounded temporary staging and
rollback files, plus a temporary copy of the app as an installation helper.
There is no permanent helper process. Translations are compiled text; the visual style uses native
FLTK drawing and system fonts, without an internationalization runtime.
Configuration files are read only on launch, client selection and user actions.

## Compatibility reference

cc-switch commit `8845b1ef799aa8810b864b122de6af5f4629de10`:

- [Deep-link parser](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/deeplink/parser.rs)
- [Provider config extraction](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/deeplink/provider.rs)
- [Claude projection](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/live/project/claude.rs)
- [Gemini projection](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/live/project/gemini.rs)

Only the Provider deep-link subset above is compatible. pp-switch does not share
cc-switch's database, routing state, full provider settings or OS URL association.
Links are pasted into the app; pp-switch registers no system URL scheme.

## Verification

Behavioral tests reference PP IDs and exercise real temporary files: unrelated
byte preservation, effective-provider selection, import fixtures, malformed
inputs, idempotency, permissions, rollback and concurrent-change recovery.
A GUI smoke harness exercises the same widgets/callbacks as the shipped window.
Record release size, actual process RSS and startup measurements in
`docs/verification.md`. Update this file before expanding any feature boundary.
