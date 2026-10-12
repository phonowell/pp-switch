"""Check current Markdown links, UI translations and optional review evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import re
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
    files = sorted([*ROOT.glob("*.md"), *ROOT.joinpath("docs").glob("*.md"),
                    *ROOT.joinpath("design").glob("*.md"),
                    ROOT / "assets/lucide/README.md"])
    errors, links = [], 0
    for path in files:
        content = path.read_text()
        if len(re.findall(r"^```", content, re.M)) % 2:
            errors.append(f"Unclosed fence: {path}")
        if not re.search(r"^# .+$", content, re.M):
            errors.append(f"Missing title: {path}")
        for _, raw in re.findall(r"\[([^\]\n]+)\]\(([^)\n]+)\)", content):
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
    for name, duplicate, settings in [
        ("user-guide.md", "Duplicate saved records", "Settings → Language"),
        ("user-guide-zh.md", "允许重复记录", "设置 → 语言"),
        ("user-guide-ja.md", "重複保存", "設定 → 言語"),
    ]:
        text = (ROOT / "docs" / name).read_text()
        if duplicate not in text or settings not in text:
            errors.append(f"Missing translated UI rules: {name}")
    for name in ["README.md", "README_ZH.md", "README_JA.md", "README_DE.md"]:
        if "Lucide" not in (ROOT / name).read_text():
            errors.append(f"Missing icon/layout translation: {name}")
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
            "translation_coverage": "4 READMEs / 3 guides aligned with Settings/Lucide/duplicates",
            "evidence": evidence, "exit_code": 0}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--record", type=Path)
    args = parser.parse_args()
    print(json.dumps(check(args.record), ensure_ascii=False, indent=2))
