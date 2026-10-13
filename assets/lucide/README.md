# Lucide UI icons

Original SVGs from [Lucide](https://github.com/lucide-icons/lucide/tree/70562c1ee1c4fdcf736fe97bc893fb8511927934/icons),
commit `70562c1ee1c4fdcf736fe97bc893fb8511927934` (2026-10-09).
Settings, Pencil, Arrow right left, Arrow left, Plus and
Chevron down cover every application-drawn UI icon. The PPC application logo and
OS-owned window chrome are separate. Licenses: [ISC/MIT](LICENSE), also included
in every package's [third-party notices](../../THIRD_PARTY_NOTICES.txt).

Generate [native paths](../../src/ui/lucide.rs) with
`python3 scripts/lucide.py`; add `--check` for SVG/native parity.
Development-only dependency: `fonttools==4.60.1`. It converts SVG arcs to cubic
curves. Neither the parser nor an SVG/image runtime ships with the application.
Render 16–18 pixel icons with a consistent absolute 2-pixel rounded stroke,
the shared theme palette and no locale-dependent font glyphs.
