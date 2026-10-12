"""Real macOS cold/warm Apple URL events, explicitly targeting an isolated app."""
from pathlib import Path
import os
import plistlib
import signal
import shutil
import subprocess
import sys
import tempfile
import time

binary = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2]).resolve()
output.mkdir(parents=True, exist_ok=True)
lsregister = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
with tempfile.TemporaryDirectory(prefix="pp-protocol-") as temp:
    root = Path(temp)
    bundle = root / "ProtocolFixture.app"
    contents = bundle / "Contents"
    (contents / "MacOS").mkdir(parents=True)
    (contents / "Resources").mkdir()
    shutil.copy2(binary, contents / "MacOS" / "protocol_smoke")
    (contents / "Resources" / "fixture-path.txt").write_text(str(root))
    plist = {"CFBundleExecutable": "protocol_smoke", "CFBundleIdentifier": "io.github.phonowell.pp-switch.protocol-fixture",
             "CFBundleName": "ProtocolFixture", "CFBundlePackageType": "APPL", "NSHighResolutionCapable": True,
             "CFBundleURLTypes": [{"CFBundleURLSchemes": ["ccswitch", "ppswitch"], "CFBundleTypeRole": "Viewer", "LSHandlerRank": "Alternate"}]}
    (contents / "Info.plist").write_bytes(plistlib.dumps(plist))
    valid = "://v1/import?resource=provider&app=claude&name=Native%20link&endpoint=https%3A%2F%2Fnative.example.com&apiKey=native-fake-key"
    try:
        for step, link in enumerate(["ccswitch"+valid, "ppswitch"+valid, "ppswitch"+valid,
                                     "ccswitch://v2/import?resource=prompt&apiKey=never-log-me"], 1):
            command = ["open"] + (["-n"] if step == 1 else []) + ["-a", str(bundle), link]
            subprocess.run(command, check=True, timeout=10, capture_output=True)
            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                if (root / "error.txt").exists():
                    raise RuntimeError((root / "error.txt").read_text())
                if (root / "step.txt").exists() and (root / "step.txt").read_text() == str(step):
                    break
                time.sleep(0.05)
            else:
                raise RuntimeError(f"Native URL event {step} timed out")
        for image in root.glob("native-*.ppm"):
            shutil.copy2(image, output / image.name)
        print("PP-06: real cold ccswitch and warm ppswitch URL events, draft guard, malformed input and no writes passed.")
    finally:
        if (root / "pid.txt").exists():
            try:
                os.kill(int((root / "pid.txt").read_text()), signal.SIGTERM)
            except ProcessLookupError:
                pass
        # Remove only the disposable test bundle's LS entry; never set global defaults.
        subprocess.run([lsregister, "-u", str(bundle)], check=False, capture_output=True, timeout=10)
