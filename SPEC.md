# pp-switch specification

Status: v0.0.1 implementation contract. Changes to behavior start here.

## Purpose

A small desktop application that saves API providers and switches only a
client's base URL and API key. Supported clients: Claude Code, Codex, Gemini CLI.
Supported desktop platforms: macOS (Apple Silicon / Intel), Windows x64, Linux x64.

## Product boundaries

- One window: client selector, provider list, add/edit/delete, switch, import link.
- A provider has an ID, client, display name, base URL and API key. Nothing else.
- Closing the window exits the process. No tray, startup agent or background work.
- No models, OAuth management, proxy, translation, failover, network probes,
  usage queries, scripts, MCP, prompts, skills, cloud sync or auto-updater.
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
| PP-08 | Persist secrets in user-only files (Unix 0600; Windows current-user ACL). No network requests, secret logging or automatic clipboard access. Never claim that masking encrypts storage. |
| PP-09 | Before writing, validate every involved file. Serialize pp-switch writes with an OS lock. Detect changes since reading. Replace each file via a same-directory temporary file; retain recovery data for multi-file operations. On failure/restart, roll back only files still matching this operation's output; refuse to overwrite external changes. A cross-file update is recoverable, not globally atomic. |
| PP-10 | Display current status by reading client configuration, rather than trusting a stored active-provider ID. Report incomplete/unsupported setup without silently fixing unrelated settings. Existing clients may require restart to reload configuration. |
| PP-11 | Installed application-owned files ≤5 MiB; steady idle total RSS ≤20 MiB; startup/import peak total RSS ≤30 MiB; cold interactive startup ≤300 ms. Measure a stripped release build on each platform, state hardware/method and report failures. These are budgets, not verified claims. No subprocess/runtime exclusions for memory. |
| PP-12 | Publish source, license, build instructions, compatibility limits and actual verification evidence. CI runs formatting, lint, behavioral tests and release builds on all three OSes. No platform is described as tested before evidence exists. |

## Implementation decisions

Rust core, FLTK candidate UI, a local JSON provider store. Use mature parsers;
do not write a general-purpose JSON/TOML parser. Patch only validated value
ranges. Keep the dependency graph small and omit unused FLTK features.

The only persistent operational state beyond providers is a bounded recovery
journal and an advisory lock. No plugin system, daemon, database or adapter SDK.
Configuration files are read only on launch, client selection and user actions.

## Compatibility reference

cc-switch commit `8845b1ef799aa8810b864b122de6af5f4629de10`:

- [Deep-link parser](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/deeplink/parser.rs)
- [Provider config extraction](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/deeplink/provider.rs)
- [Claude projection](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/live/project/claude.rs)
- [Gemini projection](https://github.com/farion1231/cc-switch/blob/8845b1ef799aa8810b864b122de6af5f4629de10/src-tauri/src/live/project/gemini.rs)

Only the Provider deep-link subset above is compatible. pp-switch does not share
cc-switch's database, routing state, full provider settings or OS URL association.
Links are pasted into the app; v0.0.1 registers no system URL scheme.

## Verification

Behavioral tests reference PP IDs and exercise real temporary files: unrelated
byte preservation, effective-provider selection, import fixtures, malformed
inputs, idempotency, permissions, rollback and concurrent-change recovery.
A GUI smoke harness exercises the same widgets/callbacks as the shipped window.
Record release size, actual process RSS and startup measurements in
`docs/verification.md`. Update this file before expanding any feature boundary.
