"""Validate tag builds, assemble native evidence, and publish verified assets."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin",
           "x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu")
ROOT = Path(__file__).resolve().parents[1]
VERSION = next(line.split('"')[1] for line in (ROOT / "Cargo.toml").read_text().splitlines()
               if line.startswith("version ="))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def metrics(path):
    data = json.loads(path.read_text())
    samples = data["samples"]
    if not 1 <= len(samples) <= 20 or any(
            not isinstance(s[k], (float, int)) or not 0 < s[k] < 10**10
            for s in samples for k in ("startup_ms", "idle_rss_bytes", "peak_rss_bytes")):
        raise ValueError(f"Missing/invalid measurement samples: {path}")
    expected = (data["startup_max_ms"] <= 300 and data["idle_max_bytes"] <= 20 * 1024**2
                and data["peak_max_bytes"] <= 30 * 1024**2)
    if data["budget_pass"] is not expected:
        raise ValueError(f"Inconsistent budget result: {path}")
    if (data["startup_max_ms"] != max(s["startup_ms"] for s in samples)
            or data["idle_max_bytes"] != max(s["idle_rss_bytes"] for s in samples)
            or data["peak_max_bytes"] != max(s["peak_rss_bytes"] for s in samples)):
        raise ValueError(f"Inconsistent measurements: {path}")
    return data


def assemble(artifacts, output):
    tag = os.environ["RELEASE_TAG"]
    if tag != f"v{VERSION}":
        raise ValueError("Tag must match Cargo.toml")
    output.mkdir(parents=True, exist_ok=False)
    evidence = {"version": VERSION, "tag": tag, "commit": os.environ["RELEASE_COMMIT"],
                "ci_run": os.environ["RELEASE_RUN_URL"], "signed": False, "platforms": []}
    notes = [f"Pure Provider Switch {VERSION}", "",
             "- Native URL/key switcher for Claude Code, Codex and Gemini CLI.",
             "- OS Provider import links; existing ccswitch handlers are preserved during registration.",
             "- Smaller native build and fewer provider/key copies; raw measurements below.",
             "- Tag builds validate all four native targets before publishing.", "",
             "Packages are unsigned (macOS is not notarized). Full PP-11 is not certified:",
             "OS caches are retained; readiness is an event-loop marker, not cold input latency.",
             "RSS includes system/shared pages. Update download/install peaks are unverified.", "",
             "| Target | Installed MiB | Idle / peak MiB | Readiness max ms | Import peak MiB | Budget |",
             "| --- | ---: | ---: | ---: | ---: | --- |"]
    for target in TARGETS:
        root = artifacts / f"pp-switch-{target}"
        name = f"pp-switch-v{VERSION}-{target}"
        report = json.loads((root / "package" / f"{name}.json").read_text())
        package = root / "package" / f"{name}.{'tar.gz' if 'linux' in target else 'zip'}"
        if (report["version"] != VERSION or report["target"] != target
                or not 0 < report["installed_bytes"] <= 5 * 1024**2
                or report["archive_bytes"] != package.stat().st_size
                or report["sha256"] != digest(package)):
            raise ValueError(f"Invalid native package: {target}")
        idle = metrics(root / "metrics.json")
        imported = metrics(root / "import-metrics.json")
        if idle["scenario"] != "empty-window" or imported["scenario"] != "import-preview":
            raise ValueError(f"Missing measurement scenario: {target}")
        report.update(measurement=idle, import_measurement=imported,
                      functional_checks="passed (required native CI job)")
        evidence["platforms"].append(report)
        shutil.copy2(package, output / package.name)
        passed = idle["budget_pass"] and imported["budget_pass"]
        notes.append(f"| {target} | {report['installed_bytes']/1024**2:.2f} | "
                     f"{idle['idle_max_bytes']/1024**2:.2f} / {idle['peak_max_bytes']/1024**2:.2f} | "
                     f"{idle['startup_max_ms']:.2f} | {imported['peak_max_bytes']/1024**2:.2f} | "
                     f"{'Script pass' if passed else '**FAIL**'} |")
    notes += ["", "Budget failures are disclosed; thresholds remain 5 MiB installed, 20 MiB idle RSS,",
              "30 MiB startup/import peak RSS and 300 ms readiness. See verification.json for all samples.",
              "", f"Implementation: `{evidence['commit']}`. [Native CI]({evidence['ci_run']}).",
              "", "macOS 0.0.2 users must download manually once (the old updater rejects the icon).",
              "Move the app to Applications, open it, and use More → Register import links as needed."]
    (output / "verification.json").write_text(json.dumps(evidence, indent=2) + "\n")
    sums = [f"{digest(p)}  {p.name}" for p in sorted(output.iterdir())]
    (output / "SHA256SUMS.txt").write_text("\n".join(sums) + "\n")
    (output / "release-notes.md").write_text("\n".join(notes) + "\n")
    print(json.dumps(evidence, indent=2))


def publish(directory):
    tag = os.environ["RELEASE_TAG"]
    report = json.loads((directory / "verification.json").read_text())
    if tag != f"v{VERSION}" or report["tag"] != tag:
        raise ValueError("Release tag/evidence mismatch")
    # Creation fails if the release already exists; never overwrite released assets.
    assets = sorted(p for p in directory.iterdir() if p.name != "release-notes.md")
    if len(assets) != 6:
        raise ValueError("Expected four packages, checksums and verification.json")
    subprocess.run(["gh", "release", "create", tag, "--verify-tag", "--draft", "--prerelease",
                    "--title", f"pp-switch {VERSION}", "--notes-file", str(directory / "release-notes.md"),
                    *map(str, assets)], check=True, cwd=ROOT)
    remote = json.loads(subprocess.check_output(
        ["gh", "api", f"repos/phonowell/pp-switch/releases/tags/{tag}"], cwd=ROOT))
    uploaded = {asset["name"]: asset for asset in remote["assets"]}
    if len(uploaded) != len(assets):
        raise ValueError("Uploaded asset count mismatch; draft kept for inspection")
    for path in assets:
        asset = uploaded[path.name]
        if asset["size"] != path.stat().st_size or asset.get("digest") != f"sha256:{digest(path)}":
            raise ValueError(f"Uploaded asset mismatch: {path.name}; draft kept for inspection")
    subprocess.run(["gh", "release", "edit", tag, "--draft=false"], check=True, cwd=ROOT)
    print(f"Published {tag}: all six uploaded sizes and SHA-256 digests verified")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check-tag")
    parser.add_argument("--event")
    parser.add_argument("--check-metrics", nargs="+")
    parser.add_argument("--artifacts", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--publish", type=Path)
    args = parser.parse_args()
    if args.check_tag is not None:
        if args.event == "tag" and args.check_tag != f"v{VERSION}":
            raise SystemExit(f"Tag {args.check_tag} does not match v{VERSION}")
        print(f"Version/tag validation passed: {VERSION} ({args.event})")
    elif args.check_metrics:
        for path in args.check_metrics:
            data = metrics(Path(path))
            print(f"{path}: valid evidence; budget_pass={data['budget_pass']}")
            if not data["budget_pass"]:
                print(f"::warning file={path}::PP-11 budget failed; see retained raw measurements")
    elif args.artifacts and args.output:
        assemble(args.artifacts, args.output)
    elif args.publish:
        publish(args.publish)
    else:
        parser.error("select a validation, assembly or publish action")


if __name__ == "__main__":
    main()
