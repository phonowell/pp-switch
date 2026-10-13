"""Generate versioned main-window SVG schematics, never native acceptance captures."""
import argparse
import hashlib
from html import escape
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

from fontTools.pens.boundsPen import BoundsPen
from fontTools.svgLib.path import parse_path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs/illustrations/v0.0.5"
NS = "{http://www.w3.org/2000/svg}"
LABELS = {
    "en": ("Providers", "Add", "Official", "Native login", "Primary API", "Backup API", "In use", "Light / Dark · v0.0.5 schematic — not a native screenshot"),
    "zh": ("服务商", "添加", "官方", "原生登录", "日常连接", "备用连接", "使用中", "浅色 / 深色 · v0.0.5 界面示意图，非原生截图"),
    "ja": ("プロバイダー", "追加", "公式", "ネイティブログイン", "普段の接続", "予備の接続", "使用中", "ライト / ダーク · v0.0.5 模式図、ネイティブ画面の撮影ではありません"),
}


def palette(dark):
    source = (ROOT / "src/theme.rs").read_text()
    name = "DARK_PALETTE" if dark else "LIGHT"
    body = re.search(rf"const {name}: \[Rgb; 15\] = \[(.*?)\];", source, re.S).group(1)
    return ["#" + "".join(f"{int(n):02x}" for n in rgb)
            for rgb in re.findall(r"\((\d+), (\d+), (\d+)\)", body)]


def action(name, x, y, color, size=18):
    element = ET.parse(ROOT / "assets/lucide" / f"{name}.svg").getroot()
    children = "".join(ET.tostring(child, encoding="unicode") for child in element)
    # Imported paths retain Lucide's absolute two-pixel stroke at display size.
    return f'<g transform="translate({x} {y}) scale({size / 24})" fill="none" stroke="{color}" stroke-width="{48 / size}" stroke-linecap="round" stroke-linejoin="round">{children}</g>'


def brand(name, x, y, color):
    element = ET.parse(ROOT / "assets/clients" / f"{name}.svg").getroot()
    paths = list(dict.fromkeys(p.attrib["d"] for p in element.findall(NS + "path")))
    bounds = BoundsPen(None)
    for path in paths:
        parse_path(path, bounds)
    x0, y0, x1, y1 = bounds.bounds
    scale = 20 / max(x1 - x0, y1 - y0)
    tx, ty = 12 - (x0 + x1) * scale / 2, 12 - (y0 + y1) * scale / 2
    children = "".join(f'<path d="{escape(p, quote=True)}"/>' for p in paths)
    return f'<g transform="translate({x} {y}) scale({14 / 24})" fill="{color}"><g transform="translate({tx} {ty}) scale({scale})">{children}</g></g>'


def render(locale):
    heading, add, official, native, primary, backup, status, caption = LABELS[locale]
    svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="904" height="544" viewBox="0 0 904 544" role="img" aria-labelledby="title desc">',
           '<title id="title">Pure Provider Switch · v0.0.5</title>',
           f'<desc id="desc">{escape(caption)}. Schematic, not a native screenshot or acceptance evidence. Five harness tabs, Official and per-row actions. Fictional sample data.</desc>',
           '<g font-family="Helvetica, Arial, sans-serif">']
    for dark in [False, True]:
        p = palette(dark)
        svg.append(f'<g transform="translate({464 if dark else 0} 0)">')
        svg.append(f'<rect width="440" height="502" rx="8" fill="{p[0]}"/>')
        svg.append(f'<rect x="24" y="24" width="348" height="38" rx="7" fill="{p[8]}"/>')
        for i, (text, name) in enumerate(zip(["Claude", "Codex", "Gemini", "Grok", "DSH"], ["claude", "openai", "gemini", "grok", "deepseek"])):
            x = 24 + i * 348 / 5
            if i == 0:
                svg.append(f'<rect x="{x + 3}" y="27" width="{348 / 5 - 6}" height="32" rx="5" fill="{p[5]}" stroke="{p[9]}"/>')
            color = p[1] if i == 0 else p[2]
            left = x + (348 / 5 - (len(text) * 5.6 + 19)) / 2
            svg.append(brand(name, left, 36, color))
            svg.append(f'<text x="{left + 19}" y="47" font-size="11" fill="{color}">{text}</text>')
        svg.append(action("settings", 391, 34, p[2]))
        svg.append(f'<text x="24" y="105" font-size="15" font-weight="bold" fill="{p[1]}">{heading}</text>')
        svg.append(action("plus", 344, 90, p[2], 16))
        svg.append(f'<text x="368" y="103" font-size="12" fill="{p[2]}">{add}</text>')
        for i, (name, address) in enumerate([(official, native), (primary, "https://api.example"), (backup, "https://backup.example")]):
            y = 124 + i * 74
            active = i == 1
            name_color, address_color = (p[14], p[14]) if active else (p[1], p[2])
            clip = f'text-{locale}-{int(dark)}-{i}'
            svg.append(f'<defs><clipPath id="{clip}"><rect x="40" y="{y+12}" width="{220 if active else 289}" height="46"/></clipPath></defs>')
            svg.append(f'<g clip-path="url(#{clip})"><text x="40" y="{y+29}" font-size="14" font-weight="bold" fill="{name_color}">{name}</text>')
            svg.append(f'<text x="40" y="{y+50}" font-size="11" fill="{address_color}">{escape(address)}</text></g>')
            if active:
                svg.append(f'<text x="333" y="{y+38}" text-anchor="end" font-size="10" fill="{p[14]}">{status}</text>')
            if i:
                svg.append(action("pencil", 347, y + 25, p[2]))
            svg.append(action("arrow-right-left", 383, y + 25, p[12] if active else p[2]))
        svg.append('</g>')
    svg.append(f'<text x="452" y="530" text-anchor="middle" font-size="12" fill="#616a79">{escape(caption)}</text></g></svg>\n')
    return "\n".join(svg)


def generate():
    files = {f"main-{locale}.svg": render(locale) for locale in LABELS}
    sources = [ROOT / "src/theme.rs", Path(__file__),
               *sorted((ROOT / "assets/clients").glob("*.svg")),
               *sorted((ROOT / "assets/lucide").glob("*.svg"))]
    manifest = {"version": "0.0.5 development", "kind": "schematic, not native GUI evidence",
                "source_sha256": {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
                "illustration_sha256": {name: hashlib.sha256(content.encode()).hexdigest() for name, content in files.items()}}
    files["manifest.json"] = json.dumps(manifest, ensure_ascii=False, indent=2) + "\n"
    return files


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for name, content in generate().items():
        destination = OUTPUT / name
        if args.check:
            if not destination.exists() or destination.read_text() != content:
                raise SystemExit(f"Stale illustration: {destination}")
        else:
            destination.write_text(content)
    print("Three localized light/dark SVG schematics and provenance match")
