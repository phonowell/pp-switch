# pp-switch user guide

English · [简体中文](user-guide-zh.md) · [日本語](user-guide-ja.md)

pp-switch saves providers and switches **only the effective base URL and API
key**. It does not initialize clients, select models, change authentication modes
or translate protocols. Use a provider compatible with your client's existing
model and protocol. [Downloads](https://github.com/phonowell/pp-switch/releases).

## First switch

1. Configure your client for a supported API-key setup using the client itself.
2. Open pp-switch and choose a client tab at the top.
3. Click **Add**, enter a name, HTTP(S) base URL and nonempty key; click **Save**.
4. Click **Switch**. Restart the client if it does not reload its files.

Save only changes the local provider list. The **In use** label means that the current
client URL **and key** match that saved provider. Status is read from client
files, not a remembered active ID. Use the selected row's pencil button to edit. Cancel closes the dialog and discards its draft; saving never applies it. The switch button is disabled for a matching provider.
Delete removes the saved provider only; it does not revert the client.

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
Use Codex itself to initialize a compatible setup; importing a link does not do it.

**Gemini:** select API-key authentication with `/auth` first.
`security.auth.selectedType` must already be `gemini-api-key`. OAuth and Vertex
setups are refused. `settings.json` remains unchanged.

Only user files are edited. Project configuration, launch arguments and the
client's environment can take precedence. Overrides visible to pp-switch are
reported; another terminal may have a different environment. Resolve an override
in your launch environment yourself—the app does not edit shell profiles or
system environment variables.

## Import existing links

Open **More → Import link**, paste into the masked field, then click **Preview**:

```text
ccswitch://v1/import?resource=provider&app=claude&name=Example&endpoint=https%3A%2F%2Fapi.example.com&apiKey=sk-example
```

`ppswitch://` also works. Supported app values are `claude`, `codex`, `gemini`.
Direct fields and Base64 inline JSON/TOML are supported (standard/URL-safe,
padded/unpadded). Explicit nonempty parameters take precedence; use the first
endpoint. Only name/client/URL/key enter the editor. Models, scripts, tools and
other fields are discarded. Review, **Save**, then **Switch**.

Importing is a preview: no client files change and no provider is saved yet.
Remote-only configs, other resources, unsupported app/version and incomplete
input are refused. There is no shared cc-switch database or OS link association.
The app reads the clipboard only if you paste normally; it does not monitor it.
Treat real links as credentials; the example above is fake.

## Language and local storage

On Linux install Noto Sans CJK (Debian/Ubuntu: `sudo apt install fonts-noto-cjk`)
or WenQuanYi Zen Hei. Xft needs an installed face covering CJK; it does not
automatically fall back for missing glyphs. No font files ship with the app.

UI: English, Simplified Chinese, Traditional Chinese, Japanese. The first launch
uses the system language (unsupported → English). **More → Language** updates
labels, dialogs and status immediately, preserving drafts and selected provider.

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

1. Click **More → Check for updates**. There is no startup or periodic check.
2. If a newer compatible GitHub release is found, read its version and confirm
   **Install and restart**. Cancel makes no changes. Preview builds can receive
   newer previews. Save edits first; unsaved edits are lost on restart.
3. The app downloads only this repository's package for the current OS/CPU,
   verifies its GitHub SHA-256 digest, stages owned files and starts a temporary
   helper. The helper waits for exit, installs with backups and restarts.

No provider/key data is sent. Archives are capped at 10 MiB, installed owned files
at 5 MiB. Traversal, links, unexpected files and mismatched executables are
rejected. Provider/client files are outside the installer. On installation
failure, previous files are restored unless an external change conflicts; in
that case backups are preserved and a message is shown.

Run from the extracted package, not a lone copied binary. The installation must
be writable. Close other app windows and avoid editing its files while updating.
Read-only locations require moving the package to a user-writable location or
manual replacement. Network errors/rate limits can be retried manually. If restart
fails, reopen the app yourself. Packages are unsigned: GitHub digest verification
provides download integrity, not an independent publisher signature. macOS builds
are not Apple Developer-signed or notarized.

## Limits, recovery and performance

A refused setup remains unchanged. Fix the underlying client setup through the
client, then retry. Stale-file or symlink errors require checking the destination;
pp-switch refuses to overwrite links/external changes. Multi-file Codex writes
are recoverable, not globally atomic. Recovery runs on the next app operation;
a conflict keeps the journal rather than overwriting someone else's edits.

Closing the window exits the process. There is no tray or persistent helper.
Original budgets: installed ≤5 MiB, idle RSS ≤20 MiB, startup/import peak ≤30 MiB,
cold interactive ≤300 ms. **These are targets. macOS memory has exceeded them.**
[Measurements](verification.md) document platform/version results, hardware and
raw data. Fresh-process event-loop readiness is not a cold-start or input-latency
measurement; import-stress peaks remain unverified. Do not infer current release
performance from historical figures or another OS.

[README](../README.md) · [Canonical English spec](../SPEC.md)
