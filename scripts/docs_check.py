"""Check current Markdown links, UI translations and optional review evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]


def anchors(path):
    result, seen = set(), {}
    for title in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", path.read_text(), re.M):
        slug = re.sub(r"[^\w\- ]", "", title.lower()).replace(" ", "-")
        count = seen.get(slug, 0)
        seen[slug] = count + 1
        result.add(slug if count == 0 else f"{slug}-{count}")
    return result


def check(record_path=None):
    files = sorted([*ROOT.glob("*.md"), *ROOT.joinpath("docs").rglob("*.md"),
                    *ROOT.joinpath("design").rglob("*.md"),
                    ROOT / "assets/lucide/README.md", ROOT / "assets/clients/README.md"])
    errors, links = [], 0
    for path in files:
        content = path.read_text()
        if len(re.findall(r"^```", content, re.M)) % 2:
            errors.append(f"Unclosed fence: {path}")
        if not re.search(r"^# .+$", content, re.M):
            errors.append(f"Missing title: {path}")
        if any(line.rstrip() != line for line in content.splitlines()):
            errors.append(f"Trailing whitespace: {path.relative_to(ROOT)}")
        references = [raw for _, raw in re.findall(r"\[([^\]\n]+)\]\(([^)\n]+)\)", content)]
        references += re.findall(r'<(?:img|source)\b[^>]*\bsrc=[\"\']([^\"\']+)', content)
        for raw in references:
            target = raw.strip().strip("<>")
            if re.match(r"[a-zA-Z]+:", target):
                continue
            relative, _, anchor = target.partition("#")
            destination = (path.parent / unquote(relative)).resolve() if relative else path
            links += 1
            if not destination.exists():
                errors.append(f"Missing link: {path.relative_to(ROOT)} -> {target}")
            elif anchor and (destination.suffix != ".md" or unquote(anchor) not in anchors(destination)):
                errors.append(f"Missing anchor: {path.relative_to(ROOT)} -> {target}")
    for name, duplicate, settings, official in [
        ("user-guide.md", "Names must be unique", "Settings → Language", "## Official restoration"),
        ("user-guide-zh.md", "名称不能重复", "设置 → 语言", "## 官方还原"),
        ("user-guide-ja.md", "名前は一意", "設定 → 言語", "## 公式復元"),
    ]:
        text = (ROOT / "docs" / name).read_text()
        if official not in text or duplicate not in text or settings not in text or " (2)" not in text or "DSH_HOME" not in text or "sub2api" not in text.lower():
            errors.append(f"Missing translated UI rules: {name}")
    for name in ["README.md", "README_ZH.md", "README_JA.md", "README_DE.md"]:
        text = (ROOT / name).read_text()
        if "Lucide" not in text:
            errors.append(f"Missing icon/layout translation: {name}")
        if "docs/illustrations/v0.0.5/main-" not in text or "assets/screenshot" in text:
            errors.append(f"Missing versioned current schematic: {name}")
        if not all(token in text for token in ["0.0.4", "0.0.5", "Grok", "DSH", "Sub2API", " (2)"]):
            errors.append(f"Missing translated version/feature boundaries: {name}")
    active = [p for p in files if "history" not in p.parts]
    for path in active:
        if re.search(r"配置匹配|設定一致|#39795f|#78cca6", path.read_text(), re.I):
            errors.append(f"Retired badge label/color in current docs: {path.relative_to(ROOT)}")
    inventory = json.loads((ROOT / "docs/images.json").read_text())
    for name, expected in inventory["images"].items():
        image = ROOT / name
        if not image.is_file() or hashlib.sha256(image.read_bytes()).hexdigest() != expected["sha256"] or image.stat().st_size != expected["bytes"]:
            errors.append(f"Historical image changed: {name}")
    illustrations = json.loads((ROOT / "docs/illustrations/v0.0.5/manifest.json").read_text())
    for name, expected in illustrations["source_sha256"].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != expected:
            errors.append(f"Illustration source changed: {name}")
    for name, expected in illustrations["illustration_sha256"].items():
        path = ROOT / "docs/illustrations/v0.0.5" / name
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            errors.append(f"Illustration changed: {name}")
        svg = ET.parse(path).getroot()
        if "not a native" not in svg.find("{http://www.w3.org/2000/svg}desc").text:
            errors.append(f"Illustration missing evidence disclaimer: {name}")
    evidence = "not requested"
    if record_path:
        record = json.loads(record_path.read_text())
        digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
        for group in ["source_sha256", "screenshots_sha256"]:
            for name, expected in record[group].items():
                if digest(ROOT / name) != expected:
                    errors.append(f"Evidence hash mismatch: {name}")
        package = record["package"]
        if digest(ROOT / package["path"]) != package["sha256"]:
            errors.append("Current package hash mismatch")
        install = json.loads((record_path.parent / record["installation_record"]).read_text())
        if install["archive_sha256"] != package["sha256"]:
            errors.append("Installation/archive mismatch")
        destination = Path(install["destination"])
        for relative, expected in install["installed_sha256"].items():
            if digest(destination / relative) != expected:
                errors.append(f"Installed hash mismatch: {relative}")
        owned = sum((destination / relative).stat().st_size for relative in install["installed_sha256"])
        if owned != package["installed_bytes"]:
            errors.append("Installed byte count mismatch")
        evidence = "source/screenshots/package/installation hashes and byte count passed"
    if errors:
        raise ValueError("\n".join(errors))
    return {"markdown_files": len(files), "local_links_and_anchors": links,
            "translation_coverage": "4 READMEs / 3 guides aligned with versions/Settings/Lucide/names/DSH/imports/Official",
            "historical_images_unchanged": len(inventory["images"]),
            "versioned_illustrations": len(illustrations["illustration_sha256"]),
            "evidence": evidence, "exit_code": 0}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", type=Path)
    args = parser.parse_args()
    print(json.dumps(check(args.record), ensure_ascii=False, indent=2))
