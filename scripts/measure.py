"""Measure the release measurement example; observer processes are separate."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument("binary", type=Path)
parser.add_argument("--runs", type=int, default=3)
parser.add_argument("--output", type=Path)
parser.add_argument("--enforce", action="store_true")
args = parser.parse_args()
if not 1 <= args.runs <= 20:
    raise SystemExit("runs must be between 1 and 20")

windows = os.name == "nt"
if windows:
    from ctypes import wintypes
    class Memory(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("faults", wintypes.DWORD)] + [(name, ctypes.c_size_t) for name in
            ["peak_rss", "rss", "peak_paged", "paged", "peak_nonpaged", "nonpaged", "pagefile", "peak_pagefile"]]
    memory_api = ctypes.windll.psapi.GetProcessMemoryInfo
    memory_api.argtypes = [wintypes.HANDLE, ctypes.POINTER(Memory), wintypes.DWORD]

def memory(process):
    if windows:
        info = Memory(); info.cb = ctypes.sizeof(info)
        if not memory_api(int(process._handle), ctypes.byref(info), info.cb):
            raise RuntimeError("GetProcessMemoryInfo failed")
        return info.rss, info.peak_rss
    if platform.system() == "Linux":
        fields = dict(line.split(":", 1) for line in Path(f"/proc/{process.pid}/status").read_text().splitlines() if ":" in line)
        return int(fields["VmRSS"].split()[0]) * 1024, int(fields["VmHWM"].split()[0]) * 1024
    rss = int(subprocess.check_output(["ps", "-o", "rss=", "-p", str(process.pid)]).strip()) * 1024
    return rss, rss

samples = []
for _ in range(args.runs):
    start = time.perf_counter()
    process = subprocess.Popen([str(args.binary.resolve())], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if process.stdout.readline().strip() != "READY":
        process.kill()
        raise SystemExit("Window did not enter its event loop")
    startup = (time.perf_counter() - start) * 1000
    idle = []; peak = 0; returncode = None
    while True:
        try:
            rss, high = memory(process); peak = max(peak, high)
            if time.perf_counter() - start >= 1:
                idle.append(rss)
        except (FileNotFoundError, ProcessLookupError, KeyError, subprocess.CalledProcessError):
            pass
        if windows:
            returncode = process.poll()
            if returncode is not None:
                break
        else:
            pid, status, usage = os.wait4(process.pid, os.WNOHANG)
            if pid:
                returncode = os.waitstatus_to_exitcode(status)
                process.returncode = returncode
                peak = max(peak, int(usage.ru_maxrss) * (1 if platform.system() == "Darwin" else 1024))
                break
        time.sleep(0.05)
    if returncode != 0 or not idle:
        raise SystemExit(f"Measurement process failed: {process.stderr.read()}")
    samples.append({"startup_ms": round(startup, 2), "idle_rss_bytes": int(statistics.median(idle)), "peak_rss_bytes": peak})
report = {"os": platform.platform(), "machine": platform.machine(), "method": "fresh processes; OS cache not cleared; READY on first event-loop timer; 50 ms RSS samples; idle after 1 s; OS high-water mark",
          "samples": samples, "startup_median_ms": round(statistics.median(s["startup_ms"] for s in samples), 2),
          "startup_max_ms": max(s["startup_ms"] for s in samples),
          "idle_max_bytes": max(s["idle_rss_bytes"] for s in samples), "peak_max_bytes": max(s["peak_rss_bytes"] for s in samples)}
report["budget_pass"] = report["startup_max_ms"] <= 300 and report["idle_max_bytes"] <= 20 * 1024 * 1024 and report["peak_max_bytes"] <= 30 * 1024 * 1024
text = json.dumps(report, indent=2) + "\n"
print(text)
if args.output:
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text)
if args.enforce and not report["budget_pass"]:
    raise SystemExit("PP-11 memory/startup budget failed; see measurements above")
