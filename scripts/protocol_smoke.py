"""Real macOS cold/warm Apple URL events, explicitly targeting an isolated app."""
from pathlib import Path
import ctypes
import json
import os
import plistlib
import signal
import shutil
import subprocess
import sys
import tempfile
import time

def query_handlers():
    cf = ctypes.CDLL("/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation")
    ls = ctypes.CDLL("/System/Library/Frameworks/CoreServices.framework/CoreServices")
    cf.CFStringCreateWithCString.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_uint32]
    cf.CFStringCreateWithCString.restype = ctypes.c_void_p
    cf.CFStringGetCString.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_long, ctypes.c_uint32]
    cf.CFRelease.argtypes = [ctypes.c_void_p]
    ls.LSCopyDefaultHandlerForURLScheme.argtypes = [ctypes.c_void_p]
    ls.LSCopyDefaultHandlerForURLScheme.restype = ctypes.c_void_p
    result = {}
    for scheme in ("ccswitch", "ppswitch"):
        name = cf.CFStringCreateWithCString(None, scheme.encode(), 0x08000100)
        value = ls.LSCopyDefaultHandlerForURLScheme(name)
        cf.CFRelease(name)
        result[scheme] = None
        if value:
            buffer = ctypes.create_string_buffer(4096)
            try:
                if not cf.CFStringGetCString(value, buffer, len(buffer), 0x08000100):
                    raise RuntimeError("Cannot inspect CI URL association")
                result[scheme] = buffer.value.decode()
            finally:
                cf.CFRelease(value)
    return result

if sys.argv[1:] == ["--query-handlers"]:
    print(json.dumps(query_handlers()))
    raise SystemExit(0)

def handlers():
    # Launch Services caches preferences per client. Query in a fresh process
    # after a different app registers, rather than reading our pre-registration cache.
    return json.loads(subprocess.check_output([sys.executable, __file__, "--query-handlers"], timeout=10))

binary = Path(sys.argv[1]).resolve()
register = "--register-in-ci" in sys.argv[3:]
if register and os.environ.get("CI") != "true":
    raise SystemExit("Default-handler acceptance is restricted to disposable CI runners")
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
        if register:
            if any(handlers().values()):
                raise RuntimeError("Registration acceptance needs a clean CI user")
            (root / "register-only.txt").write_text("CI-only")
            subprocess.run(["open", "-W", "-n", "-a", str(bundle)], check=True, timeout=30, capture_output=True)
            if not (root / "registered.txt").exists():
                raise RuntimeError("Registration fixture did not finish")
            expected = plist["CFBundleIdentifier"]
            actual = handlers()
            if any(value != expected for value in actual.values()):
                raise RuntimeError(f"Absent cc-switch handler was not registered: {actual}")
            alternate = root / "AlternateFixture.app"
            shutil.copytree(bundle, alternate)
            alternate_plist = dict(plist, CFBundleIdentifier=expected+".alternate")
            (alternate / "Contents" / "Info.plist").write_bytes(plistlib.dumps(alternate_plist))
            (root / "registered.txt").unlink()
            subprocess.run(["open", "-W", "-n", "-a", str(alternate)], check=True, timeout=30, capture_output=True)
            if not (root / "registered.txt").exists() or any(value != expected for value in handlers().values()):
                raise RuntimeError("Registration replaced an installed handler")
            subprocess.run([lsregister, "-u", str(alternate)], check=True, capture_output=True, timeout=10)
            (root / "register-only.txt").unlink()
            print("PP-06: absent-handler registration and existing-handler preservation passed.")
        for step, link in enumerate(["ccswitch"+valid, "ppswitch"+valid, "ppswitch"+valid,
                                     "ccswitch://v2/import?resource=prompt&apiKey=never-log-me"], 1):
            command = ["open"] + (["-n"] if step == 1 else []) + ([] if register else ["-a", str(bundle)]) + [link]
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
