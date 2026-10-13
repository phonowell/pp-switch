# Native UI design and acceptance

This document is the current layout and state reference. [Generated illustrations](../docs/images.md)
explain the main window; they cannot certify native rendering or input. All
application-owned windows share these rules. OS title bars and menu placement
follow the host; screenshots alone never establish operability.

## Tokens

| Role | Light | Dark |
| --- | --- | --- |
| Window surface / ink / secondary text | `#f7f8fa` / `#282d38` / `#616a79` | `#1e2026` / `#ebeef5` / `#a9b1c0` |
| Accent / pressed accent | `#365dd8` / `#2e53c8` | `#4b6ee0` / `#3e5bbe` |
| In use name / address / badge | `#365dd8` | `#8faaff` |
| Border / field fill | `#e4e7ec` / `#f0f2f6` | `#3e4452` / `#272b34` |
| Selected client / selected border | `#edf1fb` / `#dbe3f5` | `#283452` / `#445984` |
| Hover / disabled fill | `#e9ecf1` | `#2e323c` |
| Disabled text / danger | `#747d8b` / `#ac4040` | `#858fa2` / `#f28484` |
| Body / label / caption / heading | 13 / 12 / 11 / 15 logical pixels | Same |
| Control / row corner radius | 7 / 8 logical pixels | Same |
| Page inset / action gap | 24 / 12 logical pixels | Same |

Use the same system sans family everywhere, with OS CJK fallback. User text must
not replace UI labels. No gradients, nested cards, inset/raised frames, application
shadows or animation. Credentials stay masked; sample screenshots use fake keys.
Follow system appearance with no theme selector or saved preference. Recolor
existing widgets using shared palette slots; preserve drafts, caret/selection,
keyboard focus and edited record. macOS observes effective appearance;
Windows reads the application light/dark preference; Linux uses exposed GNOME
color-scheme, GTK or KDE settings, falling back to light if unavailable. Non-macOS
preferences are checked once per second while the GUI runs; no resident helper.

## Components

- **Client tabs:** continuous neutral track, five equal cells in the 348-pixel track, selected
  inset surface and distinct focus/pressed feedback. Draw only the compact captions
  Claude / Codex / Gemini / Grok / DSH; no underlying full-name widget label.
- **Icons:** use the pinned [Lucide SVGs](../assets/lucide/README.md) for action
  icons, including the language chevron, and [client brand SVGs](../assets/clients/README.md) for tabs.
  Native paths are generated from these sources; no handmade substitutes or font
  glyphs. Rounded absolute 2-pixel strokes, 14-pixel client marks / 16–18-pixel action drawings, 30–32-pixel hit
  targets. The PPS application logo and OS-owned chrome remain separate.
- **Rows:** 69-pixel height, 5-pixel gap, name above address. Each row shows Edit
  then Switch. Rows have no selected state or selection circles. Switch targets
  its own record by ID, preserving list scroll and keyboard focus. URL/key and supplied model matches show In use and disable
  Switch, including duplicates. Edit still identifies each record independently.
  Matching names, addresses and badges all use the blue active-text color; ordinary names
  retain ink and ordinary addresses retain secondary text.
- **Official row:** fixed first row for all five clients, localized Official /
  Native login text, Switch only; no edit/delete or fake credentials. Shares
  row height, status/active text, clipping and scrollbar width. Status reflects
  native file routing, not whether the user is signed in.
- **Badges/text:** measure localized badge width, reserve both icon hit targets
  plus an 8-pixel gap before Edit. Long text clips within its own area. Scrollbar
  width reduces row width; it must never cover text or action targets.
- **Settings:** smaller gear right of tabs; replaces the main content in the same
  native window. Only Back appears in its top row; no redundant Settings heading.
  Back restores list/client. Contains Language, manual
  Check for updates and passive wrapped update status. Navigation remains usable
  during a worker; repeated update activation is disabled. Explicit import-link
  registration preserves other installed handlers. No automatic requests.
- **Fields/dialogs:** name, URL, masked key and optional Default model with a
  passive explanation of blank/default behavior. Model-less Claude rows switch URL/key directly. Claude has an expandable
  Haiku/Sonnet/Opus mapping section. A supplied default is inherited by blank
  mappings; without a default, blank values preserve their destinations. Editors
  and import previews retain explicit models; absent defaults use supplied Claude
  mappings or supported local configuration with the exact same URL/key, across
  all five clients. Claude Add/Edit/import fields fall back to an actual editable
  `claude-sonnet-5` value; clearing it opts out of model writes. This suggestion
  does not guarantee endpoint compatibility. Prefill stays an unsaved draft;
  other clients' new Providers keep blank model fields.
  Save
  stores only. Cancel/title-close discard drafts. Delete requires confirmation and
  never changes client configuration. Error messages preserve an invalid draft.
- **Focus/tooltips:** no hover tips anywhere. No control focuses on initial
  activation. Tab/Shift-Tab and actual user input begin normal focus navigation;
  focus remains visible. Editor opens with its name field focused.
- **Appearance:** follow system light/dark; recolor existing widgets while keeping
  drafts, cursor, selection and focus. No theme selector/preference.

## Window geometry

Main: 440 × 502, titled Pure Provider Switch. No in-content pp-switch heading, More menu, divider or bottom
Current/status/Switch area. OS title bar remains.

| Component | x / y | width / height |
| --- | --- | --- |
| Client track | 24 / 24 | 348 / 38 |
| Settings gear | 384 / 27 | 32 / 32; icon 18 |
| Provider heading | 24 / 89 | 180 / 20 |
| Add | 328 / 84 | 88 / 28 |
| Provider scroll | 24 / 124 | 392 / 354 |
| Row Edit / Switch | row right −75 / −39; row y +19 | 30 / 30; icon 18 |
| Settings Back | 24 / 24 | 88 / 32 |
| Language selector | 196 / 84 | 220 / 36 |
| Manual update | 24 / 156 | 392 / 36 |
| Register import links | 24 / 208 | 392 / 36 |
| Update / registration status | 24 / 260 | 392 / 108 |

Editor: 386px wide, four 338 × 40 fields. Non-Claude height 454px/actions y=394;
Claude collapsed height 492px/actions y=432, expanded height 578px/actions y=518,
with three 106 × 40 mapping fields in one row. Keep it inside the active screen's work area.
Messages use a
386-pixel width and grow to fit wrapped text. macOS title bars omit the PPS icon;
Windows/Linux retain it. Package icons remain on all platforms. The language
popup uses the existing borderless rounded menu frame and native shadow.

## Automated regression gates

[Native input](../src/ui/acceptance.rs) covers startup focus, keyboard Settings,
page/back navigation, unique-name validation, identical-connection Save, passive row clicks, a different row's
Switch, independent duplicate Edit, validation, save/cancel, delete confirmation,
five clients, optional model draft/save/cancel/switch and model-specific badges,
native scrolling/reachable actions, language preference, responsive
update worker/error recovery, tooltips, modal and main close. All files are fixtures.
Only Chinese runs functional flows. Other locales prepare style states.

[Pixel checks](../scripts/visual_check.py) compare Settings icon pixels across
locales, require visible Edit and Switch ink, a clean bottom margin and complete
badge silhouettes against a same-run reference, including overflow/scroll.
[Detector tests](../scripts/visual_test.py) reject removed icon ink, changed locale
icons, footer ink and covered badges. [Generator](../scripts/lucide.py) `--check`
requires parity with pinned SVGs. Palette tests require ≥4.5:1 text contrast.
Native appearance transitions preserve existing drafts/cursor/focus/edited record.
Four-language light and dark captures cover main, settings, fields and row states.

Reuse unchanged successful checks. For only icon/style changes use `--style-only`;
for the new navigation/actions use `--interaction-only`; for live appearance and
both-theme styles use `--themes-only`, each through the bounded GUI wrapper.
Capture rejects hidden windows before requesting a native context. Modal checks
use bounded watchdogs and normal error returns; visual review is the final fallback.

## Review matrix

Chinese functional acceptance only. All four UI languages: empty/client/rows/
active/overflow/scroll, Add/Edit, Settings, update busy/error; both light and dark.
Record current results and limitations in [smoke acceptance](../docs/smoke.md).
Historical More/footer/import captures are evidence of prior candidates, not the
current interface. Native Windows/Linux/Intel checks require their respective hosts.

Client tabs use Claude, Codex, Gemini, Grok and DSH labels (11px) with 14px
monochrome brand SVGs from one pinned Lobe Icons revision. Normalize visible
bounds to equal padding and center; preserve each original brand shape. Action
icons remain Lucide. Keep the 440px window and five equal tabs in the 348px track.
