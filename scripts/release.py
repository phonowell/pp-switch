"""Validate tag builds, assemble native evidence, and publish verified assets."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
from performance import BUDGET_VERSION, MEMORY_MIB, evaluate

TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin",
           "x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu")
ROOT = Path(__file__).resolve().parents[1]
VERSION = next(line.split('"')[1] for line in (ROOT / "Cargo.toml").read_text(encoding="utf-8").splitlines()
               if line.startswith("version ="))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def metrics(path, target=None):
    data = json.loads(path.read_text(encoding="utf-8"))
    samples = data["samples"]
    if not 1 <= len(samples) <= 20 or any(
            type(s[k]) not in (float, int) or not math.isfinite(s[k]) or not 0 < s[k] < 10**10
            for s in samples for k in ("startup_ms", "idle_rss_bytes", "peak_rss_bytes")):
        raise ValueError(f"Missing/invalid measurement samples: {path}")
    if (data["startup_max_ms"] != max(s["startup_ms"] for s in samples)
            or data["idle_max_bytes"] != max(s["idle_rss_bytes"] for s in samples)
            or data["peak_max_bytes"] != max(s["peak_rss_bytes"] for s in samples)):
        raise ValueError(f"Inconsistent measurements: {path}")
    if "budget_version" in data:
        if type(data["budget_version"]) is not int or data["budget_version"] != BUDGET_VERSION:
            raise ValueError(f"Unsupported budget version: {path}")
        if type(data.get("cold_start_verified")) is not bool or type(data.get("native_action_readiness")) is not bool:
            raise ValueError(f"Missing/invalid readiness provenance: {path}")
        if target is not None:
            system = "Darwin" if "apple" in target else "Windows" if "windows" in target else "Linux"
            if data.get("platform_system") != system:
                raise ValueError(f"Measurement platform does not match target: {path}")
        expected = evaluate(data["platform_system"], samples)
        for key, value in expected.items():
            if data.get(key) != value or (type(value) is bool and type(data.get(key)) is not bool):
                raise ValueError(f"Inconsistent budget field {key}: {path}")
        return data
    # Immutable older evidence keeps its original revision-1 interpretation.
    expected = (data["startup_max_ms"] <= 300 and data["idle_max_bytes"] <= 20 * 1024**2
                and data["peak_max_bytes"] <= 30 * 1024**2)
    if "cold_start_verified" in data or "native_action_readiness" in data:
        if type(data.get("cold_start_verified")) is not bool or type(data.get("native_action_readiness")) is not bool:
            raise ValueError(f"Missing/invalid readiness provenance: {path}")
        if data.get("measured_threshold_pass") is not expected:
            raise ValueError(f"Inconsistent measured thresholds: {path}")
        expected = expected and data["native_action_readiness"] and data["cold_start_verified"]
    if data["budget_pass"] is not expected:
        raise ValueError(f"Inconsistent budget result: {path}")
    return data


def assemble(artifacts, output):
    tag = os.environ["RELEASE_TAG"]
    if tag != f"v{VERSION}":
        raise ValueError("Tag must match Cargo.toml")
    output.mkdir(parents=True, exist_ok=False)
    evidence = {"version": VERSION, "tag": tag, "commit": os.environ["RELEASE_COMMIT"],
                "ci_run": os.environ["RELEASE_RUN_URL"], "signed": False, "platforms": []}
    notes = [f"Pure Provider Switch {VERSION}", "",
             "- Switch URLs, keys and explicitly supplied optional models for Claude, Codex, Gemini, Grok and shared DSH CLI/Web.",
             "- Row Edit/Switch, in-window Settings and scoped Provider-link previews with numeric name suffixes.",
             "- Each harness has an Official row to restore native routing while preserving credentials.",
             "- Claude imports read supplied model fields; Add/Edit/import suggest claude-sonnet-5 when absent. Models remain optional; clear the field to retain client models.",
             "- macOS application-menu and Dock-format Quit handling repaired; full window title, consistent harness icons and blue In use emphasis.",
             "- Single-snapshot Claude/DSH configuration status; raw measurements below.",
             "- Tag builds validate all four native targets before publishing.", "",
             "Packages are unsigned (macOS is not notarized). Routine PP-11 revision-2 budgets are reported below:",
             "OS caches are retained; routine readiness is an event-loop marker, not cold input latency.",
             "RSS includes system/shared pages. Update download/install peaks are unverified.", "",
             "Migration: saved providers remain readable. Save never applies client files; use the row's Switch action. Models must be supported by your endpoint.",
             "Codex CLI/App share configuration. DSH CLI/Web share initialized headless/web home profiles; external credential stores and unsupported overrides are refused.",
             "Official restores routing, not account login. Native Dock pointer clicks and true cold-start performance are not certified by these automated checks.", "",
             "| Target | Installed MiB | Idle / peak MiB | Readiness max ms | Editor peak MiB | Routine budget |",
             "| --- | ---: | ---: | ---: | ---: | --- |"]
    for target in TARGETS:
        root = artifacts / f"pp-switch-{target}"
        # Naming contract paired with src/update.rs::asset_name and package.py.
        name = f"pp-switch-v{VERSION}-{target}"
        report = json.loads((root / "package" / f"{name}.json").read_text(encoding="utf-8"))
        package = root / "package" / f"{name}.{'tar.gz' if 'linux' in target else 'zip'}"
        if (report["version"] != VERSION or report["target"] != target
                or not 0 < report["installed_bytes"] <= 5 * 1024**2
                or report["archive_bytes"] != package.stat().st_size
                or report["sha256"] != digest(package)):
            raise ValueError(f"Invalid native package: {target}")
        idle = metrics(root / "metrics.json", target)
        editor = metrics(root / "editor-metrics.json", target)
        if any(record.get("budget_version") != BUDGET_VERSION for record in (idle, editor)):
            raise ValueError(f"Current budget revision required for new release artifacts: {target}")
        if idle["scenario"] != "empty-window" or editor["scenario"] != "provider-editor":
            raise ValueError(f"Missing measurement scenario: {target}")
        report.update(measurement=idle, editor_measurement=editor,
                      functional_checks="passed (required native CI job)")
        native = root / "mac-performance" / "report.json"
        if native.exists():
            report["mac_native_performance"] = json.loads(native.read_text(encoding="utf-8"))
        evidence["platforms"].append(report)
        shutil.copy2(package, output / package.name)
        passed = idle["budget_pass"] and editor["budget_pass"]
        notes.append(f"| {target} | {report['installed_bytes']/1024**2:.2f} | "
                     f"{idle['idle_max_bytes']/1024**2:.2f} / {idle['peak_max_bytes']/1024**2:.2f} | "
                     f"{idle['startup_max_ms']:.2f} | {editor['peak_max_bytes']/1024**2:.2f} | "
                     f"{'Routine pass' if passed else '**FAIL**'} |")
    notes += ["", "PP-11 revision 2: installed size ≤5 MiB; cached readiness median ≤500 ms and maximum ≤1,000 ms,",
              "with at least three fresh processes per scenario. Total RSS limits (idle / peak):", "",
              *[f"- {'macOS' if system == 'Darwin' else system}: {idle} / {peak} MiB."
                for system, (idle, peak) in MEMORY_MIB.items()],
              "", "Budget failures remain disclosed. A routine pass does not certify strict cold startup or update peaks.",
              "See verification.json for budget revisions, exact limits and all samples.",
              "", f"Implementation: `{evidence['commit']}`. [Native CI]({evidence['ci_run']}).",
              "", "macOS 0.0.2 users must download manually once (the old updater rejects the icon).",
              "Move the app to Applications and open it. Language and manual updates are in Settings."]
    (output / "verification.json").write_text(json.dumps(evidence, indent=2) + "\n", encoding="utf-8")
    sums = [f"{digest(p)}  {p.name}" for p in sorted(output.iterdir())]
    (output / "SHA256SUMS.txt").write_text("\n".join(sums) + "\n", encoding="utf-8")
    (output / "release-notes.md").write_text("\n".join(notes) + "\n", encoding="utf-8")
    print(json.dumps(evidence, indent=2))


def publish(directory):
    tag = os.environ["RELEASE_TAG"]
    report = json.loads((directory / "verification.json").read_text(encoding="utf-8"))
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
        ["gh", "release", "view", tag, "--json", "assets"], cwd=ROOT))
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
