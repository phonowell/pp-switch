"""Package a native release build; installed app-owned bytes must fit the spec."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import tarfile
import zipfile

parser = argparse.ArgumentParser()
parser.add_argument("target", help="Rust target triple")
parser.add_argument("--binary", type=Path)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
version = next(line.split('"')[1] for line in (root / "Cargo.toml").read_text().splitlines() if line.startswith("version ="))
windows = "windows" in args.target
macos = "apple" in args.target
binary = args.binary or root / "target" / args.target / "release" / ("pp-switch.exe" if windows else "pp-switch")
out = root / "target" / "package"
out.mkdir(parents=True, exist_ok=True)
name = f"pp-switch-v{version}-{args.target}"
stage = out / name
if stage.exists():
    shutil.rmtree(stage)
stage.mkdir()
if macos:
    app = stage / "pp-switch.app" / "Contents"
    (app / "MacOS").mkdir(parents=True)
    shutil.copy2(binary, app / "MacOS" / "pp-switch")
    with (app / "Info.plist").open("wb") as file:
        plistlib.dump({"CFBundleExecutable": "pp-switch", "CFBundleIdentifier": "io.github.phonowell.pp-switch",
                      "CFBundleName": "Pure Provider Switch", "CFBundleDisplayName": "Pure Provider Switch", "CFBundlePackageType": "APPL", "CFBundleVersion": version,
                      "CFBundleShortVersionString": version, "LSMinimumSystemVersion": "11.0",
                      "NSHighResolutionCapable": True, "CFBundleIconFile": "AppIcon.icns",
                      "CFBundleURLTypes": [{"CFBundleURLName": "Provider import",
                                            "CFBundleURLSchemes": ["ppswitch", "ccswitch"],
                                            "CFBundleTypeRole": "Viewer", "LSHandlerRank": "Alternate"}]}, file)
else:
    shutil.copy2(binary, stage / binary.name)
resources = app / "Resources" if macos else stage
resources.mkdir(exist_ok=True)
if macos:
    shutil.copy2(root / "assets" / "icon.icns", resources / "AppIcon.icns")
shutil.copy2(root / "LICENSE", resources / "LICENSE")
shutil.copy2(root / "THIRD_PARTY_NOTICES.txt", resources / "THIRD_PARTY_NOTICES.txt")
(resources / "README.txt").write_text(f"pp-switch {version} — Pure Provider Switch\n\n"
    "Switch only API base URLs and keys. Close the window to exit.\n"
    "Clients must already use a supported API-key configuration.\n"
    "Source, setup and limits: https://github.com/phonowell/pp-switch\n"
    "Secrets are stored in local user-only files; they are not encrypted.\n"
    "Based in part on the work of the FLTK project (https://www.fltk.org).\n"
    "macOS builds are not Apple Developer-signed or notarized.\n", encoding="utf-8")
files = [path for path in stage.rglob("*") if path.is_file()]
installed = sum(path.stat().st_size for path in files)
if installed > 5 * 1024 * 1024:
    raise SystemExit(f"PP-11 size budget failed: {installed} bytes > 5 MiB")
if macos or windows:
    archive = out / f"{name}.zip"
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zipped:
        for path in files:
            zipped.write(path, path.relative_to(stage))
else:
    archive = out / f"{name}.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        tar.add(stage, arcname=name)
report = {"version": version, "target": args.target, "installed_bytes": installed,
          "archive_bytes": archive.stat().st_size, "sha256": hashlib.sha256(archive.read_bytes()).hexdigest()}
(out / f"{name}.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))
