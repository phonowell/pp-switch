"""Bound GUI verification; capture a stack sample if a macOS runner hangs."""
import platform
from pathlib import Path
import subprocess
import sys

process = subprocess.Popen(sys.argv[1:])
try:
    raise SystemExit(process.wait(timeout=30))
except subprocess.TimeoutExpired:
    if platform.system() == "Darwin":
        Path("target").mkdir(exist_ok=True)
        subprocess.run(["sample", str(process.pid), "1", "-file", "target/gui-hang.txt"], timeout=10, check=False)
    process.kill()
    process.wait()
    raise SystemExit("GUI smoke exceeded 30 seconds; inspect target/gui-hang.txt on macOS")
