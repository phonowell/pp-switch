# Client brand SVGs

All five monochrome marks come from Lobe Icons
[`67c955577316c56c276dd406c4c5b7c2b9715b4b`](https://github.com/lobehub/lobe-icons/tree/67c955577316c56c276dd406c4c5b7c2b9715b4b/packages/static-svg/icons).
[Source hashes](sources.json) pin the original SVG bytes. This replaces the older
cc-switch-extracted set with one complete upstream revision.
These Lobe Icons shapes are MIT licensed; see [LICENSE](LICENSE).

Claude Code uses Claude, Codex uses OpenAI, Gemini CLI uses Gemini,
Grok Build uses Grok. DeepSeek identifies DSH CLI/Web.
Brand names and marks identify their respective clients.
Lucide provides action icons; it deliberately excludes brand logos.

`scripts/client_icons.py` compiles SVG path geometry into native filled paths.
Every mark is centered in a common 20-of-24 unit visual box with equal padding.
Only the first Gemini path is needed for monochrome rendering; its repeated
color-gradient overlays share exactly the same geometry. The application uses
its current text color, with no runtime SVG parser, image decoder or network.
