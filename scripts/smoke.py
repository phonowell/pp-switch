"""Run every local smoke gate, keep raw logs, and never publish or install locally."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    output = (args.output or root / "target" / ("smoke-" + stamp)).resolve()
    output.mkdir(parents=True, exist_ok=False)
    cargo = shutil.which("cargo") or str(Path.home() / ".cargo/bin/cargo")
    env = os.environ.copy()
    env["PATH"] = str(Path(cargo).parent) + os.pathsep + env.get("PATH", "")
    rust_host = subprocess.check_output([str(Path(cargo).with_name("rustc")), "-vV"], env=env, text=True)
    target = next(line.split(": ", 1)[1] for line in rust_host.splitlines() if line.startswith("host:"))
    suffix = ".exe" if os.name == "nt" else ""
    examples = root / "target/release/examples"
    python = sys.executable
    package = root / "target/package" / ("pp-switch-v0.0.4-" + target + (".tar.gz" if "linux" in target else ".zip"))
    report = {"started_utc": stamp, "platform": platform.platform(), "target": target,
              "published": False, "checks": [], "source_sha256": {}}
    sources = [root / "Cargo.toml", root / "Cargo.lock", root / "build.rs"]
    for directory in ("src", "examples", "scripts", "design"):
        sources.extend(path for path in (root / directory).rglob("*") if path.is_file() and "__pycache__" not in path.parts)
    report["source_sha256"] = {str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest() for path in sources}

    def run(name, command, timeout=180, graphical=False):
        print(f"START {name}", flush=True)
        start = time.monotonic()
        log = output / (name + ".log")
        with log.open("w") as stream:
            stream.write("COMMAND " + json.dumps([str(part) for part in command]) + "\n")
            stream.flush()
            try:
                result = subprocess.run(command, cwd=root, env=env, stdout=stream, stderr=subprocess.STDOUT, timeout=timeout)
                code = result.returncode
            except subprocess.TimeoutExpired:
                code = 124
                stream.write("\nTIMEOUT: check did not complete.\n")
        raw = log.read_text(errors="replace")
        state = "passed" if code == 0 else "failed"
        if graphical and any(marker in raw for marker in ("Connection invalid", "Connection Invalid", "graphical session is unavailable", "hiservices-xpcservice")):
            state = "blocked"
        if name.startswith("measure") and "budget failed" in raw:
            state = "budget-failed"
        report["checks"].append({"name": name, "status": state, "exit_code": code,
                                 "seconds": round(time.monotonic() - start, 2), "log": str(log)})
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"{state.upper()} {name} (exit {code}); {log}", flush=True)
        return code == 0

    run("lucide-parity", [python, "scripts/lucide.py", "--check"])
    run("format", [cargo, "fmt", "--check"])
    run("clippy", [cargo, "clippy", "--locked", "--all-targets", "--", "-D", "warnings"])
    run("behavior", [cargo, "test", "--locked"])
    run("release-tests", [python, "-m", "unittest", "discover", "-s", "scripts", "-p", "*_test.py"])
    built = run("release-build", [cargo, "build", "--locked", "--release", "--bins", "--examples"], timeout=900)
    if built:
        run("gui-callback", [python, "scripts/gui_smoke.py", str(examples / ("ui_smoke" + suffix)), str(output / "gui.ppm")], graphical=True)
        if platform.system() == "Darwin":
            run("native-input", [python, "scripts/gui_smoke.py", "--timeout", "180", str(examples / "input_smoke"), str(output / "input")], timeout=190, graphical=True)
            run("visual-pixels", [python, "scripts/visual_check.py", str(output / "input")])
            run("native-appearance", [python, "scripts/gui_smoke.py", "--timeout", "90", str(examples / "input_smoke"), str(output / "themes"), "--themes-only"], timeout=100, graphical=True)
            for appearance in ("light", "dark"):
                run(f"visual-{appearance}", [python, "scripts/visual_check.py", str(output / "themes" / appearance)])
        run("package", [python, "scripts/package.py", target, "--binary", str(root / "target/release" / ("pp-switch" + suffix))])
        run("update-helper", [str(examples / ("update_smoke" + suffix))], timeout=40)
        run("update-package", [str(examples / ("update_smoke" + suffix)), "--archive", str(package)])
        run("measure-empty", [python, "scripts/measure.py", str(examples / ("measure_ui" + suffix)), "--output", str(output / "metrics.json"), "--enforce"], graphical=True)
        run("measure-editor", [python, "scripts/measure.py", str(examples / ("measure_ui" + suffix)), "--editor", "--output", str(output / "editor-metrics.json"), "--enforce"], graphical=True)
    else:
        report["unrun"] = "GUI, input, package, update and measurements require a successful build."
    report["automated_pass"] = built and all(check["status"] == "passed" for check in report["checks"])
    report["visual_review"] = "pending: inspect newly captured images against design/UI.md; automation does not approve visuals"
    report["acceptance_complete"] = False
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print("REPORT " + str(output / "report.json"), flush=True)
    return 0 if report["automated_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
