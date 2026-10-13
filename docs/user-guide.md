# Pure Provider Switch (pp-switch) user guide

English · [简体中文](user-guide-zh.md) · [日本語](user-guide-ja.md)

pp-switch saves providers and switches **only the effective base URL and API
key**. It does not initialize clients, select models, change authentication modes
or translate protocols. Use a provider compatible with your client's existing
model and protocol. [Downloads](https://github.com/phonowell/pp-switch/releases).

## First switch

1. Configure your client for a supported API-key setup using the client itself.
2. Open pp-switch and choose a client tab at the top.
3. Click **Add**, enter a name, HTTP(S) base URL and nonempty key; click **Save**.
4. Click the row’s **Switch** icon after its pencil icon. Restart the client if it does not reload its files.

Save only changes the local provider list. The **In use** label means that the current
client URL **and key** match that saved provider. Status is read from client
files, not a remembered active ID. Use any row's pencil button to edit. Cancel closes the dialog and discards its draft; saving never applies it. The switch button is disabled for a matching provider.
Delete removes the saved provider only; it does not revert the client.
Duplicate saved records are allowed. All records matching the client's
URL and key show **In use**; each remains independently editable.
Rows have no separate selection state. There is no bottom status/Switch area or hover tooltip.
The small gear beside the client tabs opens Settings in the same window; Back returns to the list.
The interface follows the system light/dark appearance, including open dialogs;
changing appearance preserves unsaved edits, text selection and focus.

## Client prerequisites and exact writes

| Client | User-level location | Owned values |
| --- | --- | --- |
| Claude Code | `~/.claude/settings.json` | `env.ANTHROPIC_BASE_URL` and existing `env.ANTHROPIC_AUTH_TOKEN` or `env.ANTHROPIC_API_KEY` |
| Codex | `~/.codex/config.toml` plus, where applicable, `~/.codex/auth.json` | Selected custom provider's `base_url`; existing `experimental_bearer_token` or `OPENAI_API_KEY` |
| Gemini CLI | `~/.gemini/.env` | `GOOGLE_GEMINI_BASE_URL` and `GEMINI_API_KEY` |

`~` is the user's home directory on each OS. Claude respects `CLAUDE_CONFIG_DIR`;
Codex respects `CODEX_HOME`. Gemini's `settings.json` is read, never changed.

**Claude:** if neither key field exists, use `ANTHROPIC_AUTH_TOKEN`. Both fields
present, `apiKeyHelper`, or active Bedrock/Vertex/Foundry modes are refused. pp-switch
does not manage OAuth login or convert cloud-provider authentication.

**Codex:** an existing `model_provider` must select a custom `model_providers`
entry. `env_key` is unsupported. A bearer-token setup must not also have
`requires_openai_auth=true`. File-based API-key auth requires that flag to be
true, an existing nonempty `auth.json.OPENAI_API_KEY`, no active OAuth credentials,
and `cli_auth_credentials_store` absent or `file`. External/keyring stores are
unsupported. The app never changes selectors, `wire_api`, models or auth mode.
Use Codex itself to initialize a compatible setup.

**Gemini:** select API-key authentication with `/auth` first.
`security.auth.selectedType` must already be `gemini-api-key`. OAuth and Vertex
setups are refused. `settings.json` remains unchanged.

Only user files are edited. Project configuration, launch arguments and the
client's environment can take precedence. Overrides visible to pp-switch are
reported; another terminal may have a different environment. Resolve an override
in your launch environment yourself—the app does not edit shell profiles or
system environment variables.

## Provider entry

Add providers manually with name, URL and key. In 0.0.5 development, browser
`ccswitch://v1/import` / `ppswitch://v1/import` Provider links open an unsaved
preview. Review the name, URL and masked key, then Save; client files are unchanged.
Remote configs and unrelated settings are never imported. An open draft is kept;
finish or cancel it and reopen the link. Settings → Register import links claims
only unowned schemes and preserves other installed handlers. Nothing registers
at startup. 0.0.4 downloads do not support import. Existing providers are retained.

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
Closing the main window exits the process, including dialogs; no helper remains
resident. Current performance results and limitations live in
[verification](verification.md), and budget definitions in [SPEC PP-11](../SPEC.md#acceptance-contract).

[README](../README.md) · [Canonical English spec](../SPEC.md)
