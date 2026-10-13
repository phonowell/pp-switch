# Pure Provider Switch (pp-switch) user guide

English · [简体中文](user-guide-zh.md) · [日本語](user-guide-ja.md)

pp-switch saves providers and switches **the configured base URL and API key**.
0.0.6 preview also manages explicit models and scoped native/API routing.
It does not manage accounts or translate protocols. Use a compatible model and
protocol. [Downloads](https://github.com/phonowell/pp-switch/releases).

## First switch

1. Initialize your client. 0.0.6 also supports the native routes described below.
2. Open pp-switch and choose a client tab at the top.
3. Click **Add**, enter a name, HTTP(S) base URL and nonempty key; review the optional model and click **Save**.
4. Click the row’s **Switch** icon after its pencil icon. Restart the client if it does not reload its files.

Save only changes the local provider list. The blue **In use** label means that the current
client URL, key **and supplied model settings** match that saved provider. Status is read from client
files, not a remembered active ID. Use any row's pencil button to edit. Cancel closes the dialog and discards its draft; saving never applies it. The switch button is disabled for a matching provider.
Delete removes the saved provider only; it does not revert the client.
Names must be unique within each client (trimmed, case-insensitive). Imported
name collisions receive the first available numeric suffix, such as `Name (2)`.
Identical connections may still be saved under different names; old duplicate names remain readable. All records matching the client's
URL, key and supplied model settings show **In use**; each remains independently editable.
Rows have no separate selection state. There is no bottom status/Switch area or hover tooltip.
The small gear beside the client tabs opens Settings in the same window; Back returns to the list.
The interface follows the system light/dark appearance, including open dialogs;
changing appearance preserves unsaved edits, text selection and focus.

**In use verifies only the supported user-level files.** It does not verify a running session or send a request. The new provider must support the retained model; a Claude unknown-model catalog warning alone does not mean that switching failed. Check a real request error separately from the startup model banner.

## Optional models (0.0.6 preview)

Models are optional, including for Claude. Model-less saved rows switch URL/key
directly and show In use when those fields match. Existing records and imports
keep explicit models first. Claude then uses its supplied Sonnet, Opus, then Haiku
mapping. Otherwise all five clients prefill from supported local configuration
only when both URL and key match; Claude tier aliases resolve through configured
mappings. No model is inherited from a different connection.

Claude Add/Edit/import fields fall back to the visible, editable `claude-sonnet-5`
when no specific model is available. This product suggestion is not a compatibility
guarantee; check that your provider supports it or replace it. Clearing it preserves
client models. Prefill is an unsaved draft: Save, then Switch to apply. Other clients'
new fields stay blank; DSH’s native-to-API transition uses `deepseek-flash` when blank.

Claude synchronizes root `model`, `env.ANTHROPIC_MODEL` and the Haiku/Sonnet/Opus
mappings. Expand mappings to override each tier. An existing
`ANTHROPIC_SMALL_FAST_MODEL` follows Haiku. Codex changes root `config.toml.model`;
Gemini synchronizes `.env.GEMINI_MODEL` and `settings.json.model.name` with
recoverable writes. Grok changes the selected custom profile’s model; creating
a PPS profile from native mode requires an explicit model. DSH changes the
common `agent-default-model` model. Clearing a saved model stops managing it;
it does not erase client values or restore previous settings.

No model catalogs, compatibility probes or context metadata are guessed.
Unknown-model warnings can remain. Old model-less records remain directly switchable.
These features are absent
from the 0.0.4 download.

## Client prerequisites and exact writes

| Client | User-level location | Owned values |
| --- | --- | --- |
| Claude Code | `~/.claude/settings.json` | `env.ANTHROPIC_BASE_URL` and existing `env.ANTHROPIC_AUTH_TOKEN` or `env.ANTHROPIC_API_KEY` |
| Codex | `~/.codex/config.toml` plus, where applicable, `~/.codex/auth.json` | Selected custom provider's `base_url`; existing `experimental_bearer_token` or `OPENAI_API_KEY` |
| Gemini CLI | `~/.gemini/.env` | `GOOGLE_GEMINI_BASE_URL` and `GEMINI_API_KEY` |
| Grok Build | `~/.grok/config.toml` | Selected custom profile `base_url` and inline `api_key` |
| DeepSeek Harness CLI/Web | `$DSH_HOME/cordis.patch.yml` and `.credentials.yaml` (default `~/.dsh`) | Shared route `baseURL` and existing `refs.<apiKeyEnv>` |

`~` is the user's home directory on each OS. Claude respects `CLAUDE_CONFIG_DIR`;
Codex respects `CODEX_HOME`; DSH respects `DSH_HOME`. Gemini settings also contain
the scoped auth selector and explicit model value.

**Claude:** if neither key field exists, use `ANTHROPIC_AUTH_TOKEN`. Both fields
present, `apiKeyHelper`, or active Bedrock/Vertex/Foundry modes are refused. pp-switch
does not manage OAuth login or convert cloud-provider authentication.

**Codex:** compatible custom routes retain their selector and protocol. A bearer
route must not also require OpenAI authentication; file-key routes require
existing API-key auth without active OAuth or external/keyring key storage. From
native login, PPS instead selects its reserved `pps` Responses/bearer table and
leaves OAuth credentials and credential-store settings untouched. A non-PPS
`pps` table, active profile or forced-login override prevents this transition.
External key sources such as `env_key` remain unsupported.

**Gemini:** native/unset, `oauth-personal` and `gemini-api-key` modes are supported.
Provider Switch sets `security.auth.selectedType` to `gemini-api-key`; cloud/Vertex
auth and conflicting alternate key/URL sources are refused. Explicit models
change `settings.json.model.name`; OAuth caches remain untouched.

**Grok:** existing inline-key custom profiles retain their selector and protocol.
From native mode, PPS selects a reserved `pps` Responses profile with an explicit
default model. Other profiles and `auth.json` stay intact.

**DSH:** initialize both `headless` and `web` profiles with `@deepseek-ai/dsh-base`.
The home patch applies after profile patches and is shared by CLI/Web. Supported
API routes preserve the existing key reference; from native account mode, PPS
selects `deepseek-official` and uses its existing reference (default
`DEEPSEEK_API_KEY`), creating only a missing version-1 API-key container. Conflicting
profile routes without a shared override and external credentials are refused.
Account records, manifests, profile patches and complete inherited catalogs
remain intact. Restart dsh if HMR is disabled. Launch patches, custom bundles and
separate process environments can override file-match status.

## Official restoration (0.0.6 preview)

Every client list begins with a fixed **Official** row, with Switch only. It
restores native routing, not a historical file backup. It makes no login or
network request: sign in through the client and restart it if needed.

- Claude removes its API URL/key/token, five managed model environment fields
  and root model. Native credentials, hooks, permissions, model catalogs and
  existing proxy/context settings remain. External helpers/cloud modes are refused.
- Codex clears root provider/model selection, API-only forced login and
  `auth.json.OPENAI_API_KEY`/API auth mode. OAuth tokens and inactive tables remain.
- Gemini clears managed URL/key/model environment entries, selects
  `oauth-personal`, and removes `model.name`; OAuth cache and other settings remain.
- Grok clears the default selector and a summary selector referencing the same
  profile. Inactive profile tables and native credentials remain.
- DSH selects `deepseek-account` / `deepseek-flash`, removes an account adapter’s
  custom endpoint through a complete common override, and preserves API/account
  credentials, model catalogs and both profile patches.

The Official badge checks native file routing/auth state only; it does not prove
that an account is logged in. Unsupported configuration is refused before writes.
Multi-file changes use the same recoverable transaction as Provider switching.

Only user files are edited. Project configuration, launch arguments and the
client's environment can take precedence. Overrides visible to pp-switch are
reported; another terminal may have a different environment. Resolve an override
in your launch environment yourself—the app does not edit shell profiles or
system environment variables.

## Provider entry

Add providers manually with name, URL and key. In 0.0.6 preview, browser
`ccswitch://v1/import` / `ppswitch://v1/import` Provider links open an unsaved
preview. Review the name, URL, masked key and optional models, then Save; client files are unchanged.
Remote configs and unrelated settings are never imported. An open draft is kept;
finish or cancel it and reopen the link. Settings → Register import links claims
only unowned schemes and preserves other installed handlers. Nothing registers
at startup. 0.0.4 downloads do not support import. Existing providers are retained.

Sub2API currently sends no model in its Claude links. If the same URL and key
already exist in supported Claude configuration, the preview prefills its model;
otherwise the preview suggests `claude-sonnet-5`, which you can replace or clear.
Saving alone never applies client files. Links containing
only Claude tier mappings use Sonnet, then Opus, then Haiku as the visible default.
Explicit defaults take priority. Usage scripts are discarded and never executed.

## Language and local storage

On Linux install Noto Sans CJK (Debian/Ubuntu: `sudo apt install fonts-noto-cjk`)
or WenQuanYi Zen Hei. Xft needs an installed face covering CJK; it does not
automatically fall back for missing glyphs. No font files ship with the app.

UI: English, Simplified Chinese, Traditional Chinese, Japanese. The first launch
uses the system language (unsupported → English). **Settings → Language** updates
labels, dialogs and status immediately, preserving drafts and the edited record.

| OS | pp-switch data directory |
| --- | --- |
| macOS | `~/Library/Application Support/pp-switch` |
| Windows | `%LOCALAPPDATA%\pp-switch` |
| Linux | `${XDG_DATA_HOME:-~/.local/share}/pp-switch` |

`providers.json` holds connections including keys; `ui.json` holds only language.
Files are restricted to the current user (Unix 0600 / Windows user ACL).
Masking **does not encrypt** secrets. Lock/recovery files coordinate writes;
invalid data is reported, not silently replaced. Back up the directory privately
if needed. Do not publish it or paste it into diagnostics.

## Manual update

macOS 0.0.2 users must download 0.0.3 manually once: the old strict file whitelist rejects the new app icon. 0.0.3 handles the icon in future updates.

1. Click **Settings → Check for updates**. There is no startup or periodic check.
2. If a newer compatible GitHub release is found, read its version and confirm
   **Install and restart**. Cancel makes no changes. Preview builds can receive
   newer previews. Save edits first; unsaved edits are lost on restart.
3. The app downloads only this repository's package for the current OS/CPU,
   verifies its GitHub SHA-256 digest, stages owned files and starts a temporary
   helper. The helper waits for exit, installs with backups and restarts.

Updates send no provider/key data and replace only application-owned files.
On installation failure, previous files are restored unless an external change
conflicts; in that case backups are preserved and a message is shown.

Run from the extracted package, not a lone copied binary. The installation must
be writable. Close other app windows and avoid editing its files while updating.
Read-only locations require moving the package to a user-writable location or
manual replacement. Network errors/rate limits can be retried manually. If restart
fails, reopen the app yourself. Packages are unsigned: GitHub digest verification
provides download integrity, not an independent publisher signature. macOS builds
are not Apple Developer-signed or notarized.

## Troubleshooting

| Symptom | Action |
| --- | --- |
| Unsupported or incomplete setup | Check the client prerequisites above and initialize through the client itself. A refused setup is left unchanged. |
| Client keeps using the previous provider | Restart the client; check project settings, launch arguments and environment overrides. |
| File changed or symlink refused | Inspect the destination before retrying. The app refuses to overwrite links or external edits. |
| Update check cannot contact GitHub / HTTP 403 | Retry manually later; an outbound address can exhaust GitHub's anonymous API quota. No GitHub login is required by the app. |
| Update install/restart fails | Keep any reported backups. Check package completeness and write access; reopen the app or download manually. |
| CJK text is missing on Linux | Install one of the CJK fonts listed above. |

Codex multi-file changes are recoverable, not globally atomic. Recovery runs on
next use; a conflict keeps the journal instead of overwriting external changes.
Closing the main window or choosing macOS Quit exits the process, including
dialogs; no helper remains resident. Current performance results and limitations live in
[verification](verification.md), and budget definitions in [SPEC PP-11](../SPEC.md#acceptance-contract).

[README](../README.md) · [Canonical English spec](../SPEC.md)
