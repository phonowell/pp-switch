<div align="center">

<img src="assets/icon.svg" width="72" height="72" alt="pp-switch icon">

# pp-switch

**Pure Provider Switch. Your setup stays yours.**

Switch API base URLs and keys for Claude Code, Codex and Gemini CLI.

[![CI](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml/badge.svg)](https://github.com/phonowell/pp-switch/actions/workflows/ci.yml)
[![MIT](https://img.shields.io/badge/license-MIT-465bab.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/macOS%20%7C%20Windows%20%7C%20Linux-native-465bab)

**English · [简体中文](README_ZH.md) · [日本語](README_JA.md) · [Deutsch](README_DE.md)**

[Download](https://github.com/phonowell/pp-switch/releases) · [Guide](docs/user-guide.md) · [Measured performance](docs/verification.md) · [Spec](SPEC.md)

</div>

![pp-switch: a compact flat native window with client tabs, providers and one switch action](assets/screenshot.png)

## Why pp-switch?

**You spent time getting your client right. Changing API services should not make you do it again.**

Your models, reasoning settings, project permissions, MCP and skills already work. You just need a different endpoint and key. pp-switch saves those connections and patches exactly those two values, preserving unrelated configuration bytes—including comments and formatting. It refuses unsupported setups instead of silently rebuilding them.

**A switcher should finish when you do.** One window, no tray, no startup agent, no provider probes or usage polling. Close it and the process exits. Updates happen only when you click **Check for updates** and confirm installation. Rust + FLTK, native vector drawing, system fonts; no bundled WebView, database or JavaScript runtime.

This need is visible in the community: [cc-switch #7649](https://github.com/farion1231/cc-switch/issues/7649) asks for a Pure/Lite mode and says “CC Switch only manages fields it owns. Everything else belongs to the target application/user.” pp-switch takes that ownership principle and narrows it to URL + key. The discussion is motivation, not an endorsement. [Research notes](docs/verification.md#community-sources) record the sources and their limits.

## Small scope, clear differences

| | pp-switch | cc-switch |
| --- | --- | --- |
| Purpose | URL/key switching for an already configured client | Broader provider and client management |
| Writes | Only the effective base URL and API key | More configuration, according to enabled features |
| Provider links | Outside scope | Full deep-link workflows |
| Models, routing, usage, MCP, prompts, skills | Outside the product boundary | Offers corresponding management features |
| Lifecycle | Close window → process exits | Supports tray and broader workflows |
| Updates | Explicit check → confirm → download, install, restart | Its own update workflow |

[cc-switch](https://github.com/farion1231/cc-switch) is a better fit when you want a full management dashboard. pp-switch is an independent project for people who have already configured their clients and want a small, predictable switching tool.

## Download

**0.0.4 passed user acceptance on 2026-10-12 and is published.** Chinese functional GUI and four-language styles are verified; see [acceptance](docs/smoke.md).

**[v0.0.4 functional preview](https://github.com/phonowell/pp-switch/releases/tag/v0.0.4)**. macOS memory remains over budget; [actual measurements](docs/verification.md) are public.

| Platform | Package | Run |
| --- | --- | --- |
| macOS · Apple Silicon | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.4/pp-switch-v0.0.4-aarch64-apple-darwin.zip) | Extract; move `.app` to Applications |
| macOS · Intel | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.4/pp-switch-v0.0.4-x86_64-apple-darwin.zip) | Same |
| Windows · x64 | [ZIP](https://github.com/phonowell/pp-switch/releases/download/v0.0.4/pp-switch-v0.0.4-x86_64-pc-windows-msvc.zip) | Extract; run `pp-switch.exe` |
| Linux · x64 | [tar.gz](https://github.com/phonowell/pp-switch/releases/download/v0.0.4/pp-switch-v0.0.4-x86_64-unknown-linux-gnu.tar.gz) | Extract; run `pp-switch` on a desktop |

No Rust or Node.js installation required. macOS packages declare macOS 11+ and are **not Apple Developer-signed or notarized**. Windows requires Windows 10+. Linux requires X11/XWayland, X11/fontconfig libraries and a system font covering your selected language. Native test environments are listed in [verification](docs/verification.md); a declared minimum is not evidence of testing every OS version.

macOS 0.0.2 users: download this version manually once. The older updater rejects the new icon resource; 0.0.3 includes it in the update layout.

## Use

1. Choose Claude Code, Codex or Gemini CLI.
2. Click **Add**, enter name/base URL/key and **Save**.
3. Click that row's Switch icon, after Edit. Restart the client if needed.

Each row has Lucide Edit and Switch icons; selection alone never applies.
The smaller gear beside the tabs opens Settings for language and manual updates.
All hover tips, the in-content heading, bottom status/Switch area and Provider-link
import are removed. Matching duplicates all show In use. The interface follows
system light/dark appearance. Saving only changes the local store. UI languages:
**English, 简体中文, 繁體中文 and 日本語**, initially matching the system.


| Client | Existing setup required | Fields changed |
| --- | --- | --- |
| Claude Code | API-key setup without external key helper, cloud-provider mode or ambiguous credentials | `ANTHROPIC_BASE_URL` + existing key/token field in `settings.json` |
| Codex | Selected custom provider, initialized API-key auth | Selected provider's `base_url` + existing bearer token or `auth.json.OPENAI_API_KEY` |
| Gemini CLI | API-key mode already selected through `/auth` | `GOOGLE_GEMINI_BASE_URL` + `GEMINI_API_KEY` in `.env` |

No model/auth-mode/protocol changes or first-time client setup. User-level files only; project config, launch arguments and another terminal's environment may override them. `CLAUDE_CONFIG_DIR` and `CODEX_HOME` are respected. [Setup and troubleshooting →](docs/user-guide.md)

## Compatibility and trust

- Provider-link import and URL registration are outside the current candidate scope; existing saved providers are retained.
- Secrets stay in local user-only files (Unix 0600 / Windows current-user ACL). Masking is **not encryption**. No automatic clipboard reads, secret diagnostics or provider requests.
- Updates require your confirmation and a writable installation. Only this repository's release packages are downloaded and SHA-256 checked. Packages are unsigned; provider/client files are outside the updater. [Update details](docs/user-guide.md#manual-update).
- Full behavior contract: [SPEC.md](SPEC.md). Exact write locations and storage paths: [guide](docs/user-guide.md).

## Performance, with evidence

**0.0.4 native CI measurements:**

| Native CI | Installed MiB | Idle RSS MiB | Script budget |
| --- | ---: | ---: | --- |
| macOS 15 · ARM64 | 2.43 | 48.44 | Fail |
| macOS 15 · Intel | 2.67 | 31.97 | Fail |
| Windows Server 2022 · x64 | 2.78 | 15.08 | Pass |
| Ubuntu 22.04 · x64 · Xvfb | 3.01 | 16.96 | Fail |

Windows passes both scenarios; macOS exceeds memory budgets. Linux passes the empty-window script but exceeds the editor idle budget. Cold interactive startup and update peaks remain unverified. [SPEC PP-11](SPEC.md#acceptance-contract) · [Raw data](docs/verification.md#release-004).

## Contribute

Start with [SPEC.md](SPEC.md); build, validation and release instructions live in
[CONTRIBUTING.md](CONTRIBUTING.md).

## Acknowledgments and license

Thanks to [cc-switch](https://github.com/farion1231/cc-switch) for its open-source protocol and implementation references, and to [fltk-rs](https://github.com/fltk-rs/fltk-rs) / [FLTK](https://www.fltk.org) for the native UI toolkit. pp-switch is not affiliated with cc-switch.

[MIT](LICENSE). Dependencies retain their own [licenses and notices](THIRD_PARTY_NOTICES.txt).
