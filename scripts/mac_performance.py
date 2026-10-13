"""Issue #1: same-session bare Cocoa/PPS total RSS and native readiness evidence.

Requires dedicated desktop. Never purges global caches or touches client files.
Cold startup remains explicitly unverified; record a separate reboot experiment.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--allow-desktop-input", action="store_true")
    args = parser.parse_args()
    if not args.allow_desktop_input and os.environ.get("CI") != "true":
        raise SystemExit("SKIPPED: Cocoa/PPS measurements require a dedicated desktop")
    if platform.system() != "Darwin":
        raise SystemExit("macOS-only measurement")
    if not 1 <= args.runs <= 20:
        raise SystemExit("runs must be between 1 and 20")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    commands = []

    def run(command, label):
        result = subprocess.run(command, capture_output=True, text=True, timeout=180)
        (output / f"{label}.log").write_text(result.stdout + result.stderr)
        commands.append({"command": command, "exit_code": result.returncode, "log": f"{label}.log"})
        if result.returncode:
            (output / "commands.json").write_text(json.dumps(commands, indent=2) + "\n")
            raise SystemExit(f"{label} failed; see {output / (label + '.log')}")
        return result.stdout

    bare = output / "measure_cocoa"
    run(["clang", "-Oz", "-fobjc-arc", str(ROOT / "examples/measure_cocoa.m"), "-framework", "Cocoa", "-o", str(bare)], "build-cocoa")
    binaries = {"cocoa": bare, "empty": args.binary.resolve(), "editor": args.binary.resolve(), "import": args.binary.resolve()}
    evidence = {name: [] for name in binaries}
    for index in range(args.runs):
        # Interleave comparison processes in the same session, reversing order.
        order = list(binaries) if index % 2 == 0 else list(reversed(binaries))
        for scenario in order:
            label = f"{scenario}-{index}"
            record = output / f"{label}.json"
            command = [sys.executable, str(ROOT / "scripts/measure.py"), str(binaries[scenario]), "--native-ready", "--runs", "1", "--output", str(record), "--allow-desktop-input"]
            if scenario != "cocoa":
                command += ["--scenario", scenario]
            run(command, label)
            evidence[scenario].append(json.loads(record.read_text()))
    for scenario in ("quit-main", "quit-settings", "quit-editor", "quit-dock-main", "quit-dock-settings", "quit-dock-editor"):
        text = run([str(args.binary.resolve()), f"--scenario={scenario}", "--allow-desktop-input"], scenario)
        if "NATIVE QUIT ACCEPTED" not in text or "QUIT FIXTURES UNCHANGED" not in text:
            raise SystemExit(f"{scenario}: no native termination/fixture proof")
    report = {
        "issue": "https://github.com/phonowell/pp-switch/issues/1",
        "os": platform.platform(), "machine": platform.machine(),
        "hardware": subprocess.check_output(["sysctl", "-n", "hw.model", "hw.memsize", "machdep.cpu.brand_string"], text=True).strip(),
        "boot_session": subprocess.check_output(["sysctl", "-n", "kern.boottime"], text=True).strip(),
        "binaries": {name: {"path": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()} for name, binary in binaries.items()},
        "commands": commands, "measurements": evidence,
        "cold_start_verified": False,
        "limitations": ["OS caches retained; no true cold-start acceptance", "Cocoa baseline is a blank window, not a feature-equivalent UI", "Native terminate entry point checked; actual Dock/menu clicks require separate acceptance", "Total RSS includes shared mappings; baseline subtraction is diagnostic, not the budget metric"],
    }
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"Evidence: {output / 'report.json'}; cold startup unverified")


if __name__ == "__main__":
    main()
