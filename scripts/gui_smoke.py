"""Bound GUI verification; capture a stack sample if a macOS runner hangs."""
import argparse
import os
import platform
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--allow-desktop-input", action="store_true", help="explicitly allow foreground GUI/input on an idle or isolated desktop")
parser.add_argument("--timeout", type=float, default=30)
parser.add_argument("command", nargs=argparse.REMAINDER)
args = parser.parse_args()
if not args.command or not 1 <= args.timeout <= 300:
    parser.error("provide a command and a timeout between 1 and 300 seconds")
if not args.allow_desktop_input and os.environ.get("CI") != "true":
    raise SystemExit("SKIPPED: desktop activation/input requires --allow-desktop-input or a dedicated CI session")
if Path(args.command[0]).stem in {"input_smoke", "ui_smoke", "measure_ui"}:
    args.command.append("--allow-desktop-input")
process = subprocess.Popen(args.command)
try:
    raise SystemExit(process.wait(timeout=args.timeout))
except subprocess.TimeoutExpired:
    if platform.system() == "Darwin":
        Path("target").mkdir(exist_ok=True)
        subprocess.run(["sample", str(process.pid), "1", "-file", "target/gui-hang.txt"], timeout=10, check=False)
    process.kill()
    process.wait()
    raise SystemExit(f"GUI smoke exceeded {args.timeout:g} seconds; inspect target/gui-hang.txt on macOS")
