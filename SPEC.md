# Pure Provider Switch (pp-switch) specification

Status: v0.0.6 preview contract. Changes to behavior start here.

Release policy: functional previews may be published with disclosed performance
failures. Report functional and performance checks separately. PP-11 defines
versioned performance budgets; changing them requires baseline evidence and an
explicit contract revision, never a silent relaxation for a preview.

## Purpose

Full product name: Pure Provider Switch. Repository, executable and compact UI
name: pp-switch. Main window title: Pure Provider Switch. The application icon uses a white geometric S and two opposing sliders on the PPS blue rounded square (`#365DD8`), authored in SVG.
macOS window title bars show the title without an application icon. Windows/Linux window
icons and all platforms' installed application icons retain the PPS logo.

A small desktop application that saves API providers and switches a
client's base URL and API key, plus explicitly configured optional models. Supported clients: Claude Code, Codex, Gemini CLI, Grok Build and DeepSeek Harness (CLI/Web).
Supported desktop platforms: macOS (Apple Silicon / Intel), Windows x64, Linux x64.

## Product boundaries

- One main window: client tabs, provider list and per-row edit/switch actions. Add/edit/delete
  use a compact dialog; a small gear opens an in-window settings page for language
  and strictly manual updates.
- A provider has an ID, client, display name, base URL, API key and optional model settings defined by PP-18.
- Closing the window exits the process. No tray, startup agent or background work.
- No model discovery, catalog management, OAuth management, proxy, protocol translation, failover, network probes,
  usage queries, scripts, MCP, prompts, skills or cloud sync.
- Providers use the existing protocol. PP-19 permits scoped native/API-key transitions; preserve models unless explicitly configured or restoring Official.
- No command-line management interface. Development checks are separate tools.

## Acceptance contract

### PP-01: Provider CRUD

Create, edit and delete a provider. Saving does not change client files. Names are unique within each client after trimming and case folding. Imports preview the first unused numeric suffix ` (2)`, ` (3)`, etc., within the 128-byte name limit; saving rechecks uniqueness under the storage lock. Editing excludes the same record ID. Existing duplicate records remain readable and are never silently renamed. Names, HTTP(S) URLs and nonempty keys are validated; credentials are masked in the UI and excluded from diagnostics.

### PP-02: URL/key writes

Apply modifies only the effective base URL and API key, plus optional PP-18 model fields explicitly configured on the Provider and the scoped PP-19 native/API routing changes. Preserve all unrelated bytes, including comments, whitespace, ordering and line endings. Idempotent apply makes no file changes.

### PP-03: Claude Code

Claude: patch `env.ANTHROPIC_BASE_URL` and the existing `ANTHROPIC_AUTH_TOKEN` or `ANTHROPIC_API_KEY`. Default to `ANTHROPIC_AUTH_TOKEN` when neither exists. Refuse ambiguous credentials, external key helpers and active cloud-provider modes.

### PP-04: Codex

Codex: patch the currently selected custom provider's `base_url` in `config.toml`; patch its existing `experimental_bearer_token`, or `auth.json.OPENAI_API_KEY` for an existing file-based API-key setup. For an existing custom route preserve selection and auth mode. From the native route, select a reserved `pps` Responses/bearer table without changing OAuth tokens or credential-store settings; refuse a pre-existing unrelated `pps` table and active profile overrides. Models follow PP-18. Refuse unsupported key sources. PP-19 defines restoration to the native route.

### PP-05: Gemini CLI

Gemini: patch `GOOGLE_GEMINI_BASE_URL` and `GEMINI_API_KEY` in `.env`. Allow `gemini-api-key`, `oauth-personal` or an unset native mode; set `gemini-api-key` for Provider use and refuse cloud modes. Only an explicitly configured PP-18 model may change `settings.json.model.name`. Preserve unrelated dotenv lines and settings bytes.

### PP-06: Provider links

Accept `ccswitch://v1/import` and `ppswitch://v1/import` links for
`resource=provider` only. Extract client, name, base URL and API key from direct
parameters or bounded inline JSON/TOML, including optional PP-18 model fields;
discard unrelated settings. For a Claude link containing only tier mappings, use
Sonnet, then Opus, then Haiku as its visible default. A supplied default always
wins. When a Claude link contains no models, use the visible PP-18 editor default.
Blank fields keep PP-18 preservation semantics. Never fetch
remote config URLs. Bound each link to 256 KiB and the native pending queue to
8 links. Show the provider editor as an unsaved preview, including supplied models; only Save persists a
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

Display configuration status by reading supported user-level client files, rather than trusting a stored active-provider ID. Allow duplicate connection settings under different names; every record for the selected client matching the configured URL, key and supplied PP-18 model fields displays In use and cannot be switched again. This is a file match, not verification of a running session, connection, credentials or model compatibility. Project settings, launch arguments and a separately launched client's environment can override these files. Editing identifies individual records; rows have no independent selection state. There is no bottom Current/status or global Switch area; status is represented by row badges. Setup errors are reported when switching is attempted. Report incomplete/unsupported setup without silently fixing unrelated settings. Existing clients may require restart to reload configuration. Documentation must distinguish provider configuration from live usage and explain that retained model settings must be supported by the new provider.

### PP-11: Performance budgets

Budget revision 2 (2026-10-14) uses platform-specific total RSS limits:

| Platform | Steady idle total RSS | Startup/editor peak total RSS |
| --- | ---: | ---: |
| macOS (Apple Silicon / Intel) | ≤160 MiB | ≤192 MiB |
| Windows x64 | ≤32 MiB | ≤48 MiB |
| Linux x64 | ≤32 MiB | ≤48 MiB |

Installed application-owned files remain ≤5 MiB. Routine launch readiness with
OS caches retained must have a median ≤500 ms and maximum ≤1,000 ms over at least
three fresh processes per scenario. Empty-window and provider-editor scenarios
are required. Readiness is the first event-loop marker unless native input and
a displayed frame are explicitly verified; always label which was measured.
Native import stress uses the same limits when measured. One-process diagnostic
records cannot certify a budget; interleaved native records are evaluated together.

Strict cold interactive startup and update download/install peaks are separate,
unverified investigations until dedicated experiments establish their baselines;
they do not force every routine measurement to fail. A routine budget pass never
certifies them. State hardware, OS, renderer, method and scenario, retain raw
samples, and use comparable stripped release builds. Total RSS includes shared
system mappings; physical footprint, allocations and subtraction of a bare-window
baseline are diagnostics, never substitutes. No subprocess/runtime exclusions.

The [budget rationale](docs/verification.md#performance-budget-revision-2)
uses both native CI and local macOS measurements. These are regression ceilings,
not verified efficiency claims. Continue reducing controllable application costs;
investigate same-host increases rather than spending all available headroom.
Reports record the budget revision and limits. Preserve historical failures under
their original contract; any reassessment is separate and does not establish an
implementation improvement. Revise budgets only with new baseline evidence.

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

Follow [design/UI.md](design/UI.md): client tabs and a smaller Settings
gear at the top, a provider list below, and visible Edit then Switch icon buttons
on each row. Do not show an in-content product heading, More menu, bottom status
or global Switch button. Rows have no selection circle or selected background; Edit and Switch each
target their own record by ID. Matching records show In use and have
disabled Switch. Add/edit show name, URL, masked key and an optional default model;
Matching row names, addresses and In use badges share the blue active-text color
for visible emphasis; there is no green status role. PP-19 Official rows precede
saved Providers and expose Switch only.
Claude mappings use an expandable section. Saving never applies.
Settings replaces the list within the same window and contains language, import-link registration and
manual updates, with a Back action and no redundant Settings page heading. No hover tooltips anywhere. Initial activation
focuses no control; later keyboard navigation retains visible focus. Preserve
pressed/disabled states, contrast, masked credentials and unsaved drafts. Closing
or cancelling an editor discards its draft without writing. Client tabs pair compact brand labels (Claude, Codex, Gemini, Grok, DSH) with
pinned brand SVG shapes, rendered in the current text color. Use the pinned Lobe Icons
brand marks for clients and Lucide for actions. Use system fonts and
bounded vector icons, without new UI dependencies or background rendering.
The application icon remains the flat blue PPS switch mark in OS packages; macOS
window titles omit it, while other platforms retain their window icons.

Follow system light/dark appearance without a theme selector or saved preference.
Recolor existing widgets without losing drafts, text selection or focus. Chinese native
functional acceptance and all four languages' light/dark style checks must cover
row actions, settings, localized badge visibility and keyboard navigation.
UI changes remain subject to the current PP-11 budgets.

### PP-15: Manual updates

Updates are strictly user-triggered: no startup, scheduled or background checks. A compact Check for updates action queries only this repository's GitHub Releases over verified HTTPS; preview builds may receive newer previews. Show version and request confirmation before download/install/restart, warning about unsaved edits. Keep the GUI responsive during network work. Select only the current OS/architecture package, bound download/extraction size and whitelist application-owned files. Verify the archive against GitHub's SHA-256 asset digest before staging. Reject malformed metadata, path traversal, symlinks, unsupported layouts and unwritable installations without editing provider/client files. A temporary helper waits for the app to exit, checks the destination has not changed, replaces owned files with backups, rolls back failed installs and restarts. Report failures in the selected language. Release assets remain unsigned; a GitHub digest verifies download integrity, not an independent publisher signature.

After a successful check (including no update or a cancelled installation), reuse
its result for five minutes in the running app; another explicit click may show
the cached offer but must not request metadata again. Failed checks remain
retryable. Fetch releases in pages of 20, including later pages, within a shared
45-second network deadline and 2-MiB metadata budget. If the scan cannot finish,
report a failure rather than claiming the current version is latest. Skip a
release with invalid or ambiguous matching assets; never install it. Provide
the selected version's repository release-notes link before confirmation.

Report GitHub rate limiting distinctly from connection failure. HTTPS redirect
hosts must be explicitly allowed; a rejected host gets a specific localized
message. Poll workers at 500 ms; the UI watchdog is the network deadline plus
15 seconds and resets when staging begins. Cancellation, network/timeout,
checksum, disk-space, permission or locked-file failures restore the update
action. Before replacement, failures leave the installed app unchanged. During
replacement, restore backups; if external changes prevent rollback, preserve
backups and report the conflict. Never quit on staging/helper-launch failure;
quit only after the helper starts successfully. All failures preserve
providers, credentials and client configuration.

### PP-16: Grok Build

Grok Build: patch only `base_url` and existing `api_key` in the custom
`[model.<profile>]` selected by `[models].default` in `~/.grok/config.toml`.
For an existing custom route, require an initialized API-key profile; refuse `env_key`, `auth_provider`,
`env_http_headers` and `mtls_cert_dir` credential sources. Preserve other
profiles, profile selection, API backend, headers, MCP and
`auth.json` unchanged. From an unselected native route, select a reserved `pps` inline-key profile using the Provider's explicit default model; refuse collisions. PP-19 defines restoration; never edit OAuth credentials.
Accept the cc-switch client identifier `grokbuild` and its `grok-build`,
`grok_build`, `grok` aliases in bounded Provider links. Inline TOML and JSON
wrapping a TOML `config` extract URL/key and the optional upstream model from the selected profile; a single
unselected profile may be imported, but ambiguous multiple profiles are refused.

### PP-17: DeepSeek Harness CLI/Web

One DSH tab and Provider list controls both CLI and Web through the common
`$DSH_HOME/cordis.patch.yml` (default `~/.dsh`), loaded after their profile patches.
Do not create profiles or modify their manifests, bundles or individual patches.
Require initialized `headless` and `web` profiles with the base bundle. Support
an explicit, unambiguous shared API-key route or the PP-19 native account route, with matching effective
provider and credential reference in both profiles. An untouched native
`deepseek-official` route may inherit the base bundle's API-key defaults; PPS
may add minimal common connection overrides for that existing route.

For `llm-deepseek` or `llm-pi-ai`, change only the route's `baseURL` and its
existing `refs.<apiKeyEnv>` in `$DSH_HOME/.credentials.yaml`. Common overrides
may be minimally added for supported values inherited from a profile. Preserve
provider selection except for PP-19 transitions, wire protocol, catalogs, account/OAuth records, unrelated
patches and credentials. Optional models follow PP-18. Refuse external credential
stores, conflicting selections/references, disabled/replaced target entries,
unsupported patch operations, malformed YAML, duplicate keys/entry IDs, aliases,
anchors, tags and unsupported destination scalars. Never execute YAML expressions.
Refuse relevant inherited environment overrides visible to PPS. Separately
launched environment, custom bundle overrides, `--patch` and session-specific
model selections remain outside file-match status. Shared changes affect every
profile inheriting this home layer; CLI/Web are not separate PPS Providers.
Use PP-09 recovery for the common-patch/credential update. Reload may require
restarting dsh when its HMR is disabled.

### PP-18: Optional Provider models

Persist an optional `models` object with `model`, and Claude-only `haiku`,
`sonnet`, `opus` overrides. Old records without it retain URL/key-only behavior.
Empty editor inputs mean no explicit value, not deletion. Models are optional for
Claude too: model-less saved records can switch URL/key directly and display In use
when their managed fields match. Saving never applies; a separate Switch applies.
The low-level URL/key patch contract remains available for legacy fixtures. Validate each supplied
model as 1–512 UTF-8 bytes with no whitespace or control characters; never
query a model catalog or test credentials. Save/cancel/import preview do not
modify client files. The editor explains exactly which values follow the default.
For existing records and import previews, keep explicit Provider/link values.
When its default is absent, Claude may use its supplied Sonnet/Opus/Haiku mapping
(in that order); otherwise prefill from supported client files only when both
URL and key match that Provider. Resolve Claude tier aliases through their
configured mappings. Apply the same connection-scoped default prefill to all five
clients. Never copy models from a different connection. Claude editors (including
new Providers and model-less import previews) fall back to the visible, editable
`claude-sonnet-5` when no explicit/mapped/same-connection model exists. This is a
product default, not a compatibility guarantee or catalog query. Clearing it
opts out of model writes. Other clients retain connection-scoped prefill and their
existing native/API transition rules. Prefill is an unsaved draft; Save and a
separate Switch remain required.

For Claude, a supplied default synchronizes top-level `model` and
`env.ANTHROPIC_MODEL`. Its Haiku/Sonnet/Opus mappings follow that default unless
explicitly overridden. With no default, patch only supplied mappings; leave
other model settings intact. Update existing `ANTHROPIC_SMALL_FAST_MODEL` only
when setting the Haiku mapping. Preserve modelPicker, modelOverrides, context
window, reasoning, plugins and all other settings. Do not invent `behavesAs` or
claim an unknown-model catalog warning has been fixed.

Codex changes only top-level `config.toml.model` (minimally add it if absent).
Gemini synchronizes `.env.GEMINI_MODEL` and `settings.json.model.name`, using
PP-09 recovery for the two-file update. Grok changes only the selected custom
profile's existing `model` string; PP-19 native transitions may also change `[models].default`. DSH changes the common `agent-default-model.config.model`; preserve its selected provider except for PP-19 native transitions, and preserve all model catalogs. Apply validates model
destinations before any writes; malformed or ambiguous destinations are refused.

In use badges must match URL/key and every model destination this Provider
would write. A Provider without model settings ignores model differences. A
different supplied model must keep Switch available even for matching URL/key.
Project settings, launch arguments and process environment remain outside this
file-match status. Detect relevant model environment overrides visible to PPS
only when applying optional model settings; never change shell configuration.

### PP-19: Fixed Official items

Claude, Codex, Gemini, Grok and DSH each have a fixed first Official row, with Switch
only, no editing/deletion and no fake Provider record or credential. Official restores
native connection/login mode, not a historical whole-file backup or a login.
The user may need to sign in and restart the client. No network request is made.

Claude removes only `env.ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN`,
`ANTHROPIC_API_KEY`, the five PP-18 model env fields and top-level `model`.
External helpers/cloud modes remain unsupported. Preserve hooks, permissions,
modelPicker/modelOverrides and native credential files. Codex removes root
`model_provider`, `model`, API-only `forced_login_method`, and
`auth.json.OPENAI_API_KEY` plus API-only `auth_mode`; keep OAuth tokens,
keyring/login files and inactive custom provider tables. Refuse active profile
routing overrides. Switching from native uses a dedicated bearer table so it
never needs to overwrite native login credentials.

Gemini removes `GOOGLE_GEMINI_BASE_URL`, `GEMINI_BASE_URL`, `GEMINI_API_KEY`,
`GOOGLE_API_KEY`, `GEMINI_MODEL`, sets `security.auth.selectedType` to
`oauth-personal`, and removes `settings.json.model.name`. Preserve OAuth cache,
other settings and comments. Grok removes `[models].default` and a
`session_summary` selector only when it references that same custom profile;
keep inactive profiles and `auth.json`. Returning to a new PPS Grok profile
requires an explicit default model; do not guess a model.

Validate all destinations and visible relevant environment overrides before
writes; use PP-09 locking, conflict detection, permissions and recovery. Missing
native config does not prove login but may be initialized minimally. Official
badges derive from native file routing/auth state, never a saved active ID. An
ordinary native model selection does not imply a third-party connection. Status
cannot certify credentials or override precedence in separately launched clients.
DSH Official selects `deepseek-account` / `deepseek-flash` in the common
`agent-default-model` patch, clears only the account adapter's inherited `baseURL`
into a common complete config, and preserves account records, API-key references,
catalogs and both profile patches. Require initialized base-bundle CLI/Web
profiles. From account mode, a Provider selects `deepseek-official` with the
explicit default (or the native `deepseek-flash` default when blank), changes
`llm-deepseek.baseURL` and its existing local API-key reference (default `DEEPSEEK_API_KEY`), initializing
only a missing version-1 `refs` API-key container while preserving account data.
Normal API-key routes keep existing selectors/references. No login credential
stash, account manager or permanent additional PPS state.

## Implementation decisions

Rust core, FLTK native UI and a local JSON provider store. Use mature parsers;
do not write a general-purpose JSON/TOML parser. Patch only validated value
ranges. Keep the dependency graph small and omit unused FLTK features.

Inside the app's data directory, the only persistent state beyond providers is a
bounded recovery journal, an advisory lock and UI language preferences. No
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

Grok Build compatibility and brand assets reference cc-switch commit
`d9a62313c8a276216ed40245c638753606831e95`. DeepSeek Harness is absent from
that revision; the shared CLI/Web integration separately references deepseek-harness
`d743267388641bc76f17c45ce8b4c231aed1d32c`.
