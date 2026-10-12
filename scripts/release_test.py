"""Behavioral publication gates, using disposable native artifact fixtures."""
import contextlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import release


class PublicationGates(unittest.TestCase):
    def fixture(self, root):
        for target in release.TARGETS:
            artifact = root / f"pp-switch-{target}"
            package = artifact / "package"
            package.mkdir(parents=True)
            name = f"pp-switch-v{release.VERSION}-{target}"
            archive = package / f"{name}.{'tar.gz' if 'linux' in target else 'zip'}"
            archive.write_bytes(b"disposable package")
            (package / f"{name}.json").write_text(json.dumps({"version": release.VERSION,
                "target": target, "installed_bytes": 100, "archive_bytes": archive.stat().st_size,
                "sha256": release.digest(archive)}))
            for filename, scenario in [("metrics.json", "empty-window"), ("import-metrics.json", "import-preview")]:
                # Budget failures must be retained, not silently made successful.
                data = {"scenario": scenario, "samples": [{"startup_ms": 301, "idle_rss_bytes": 100,
                    "peak_rss_bytes": 100}], "startup_max_ms": 301, "idle_max_bytes": 100,
                    "peak_max_bytes": 100, "budget_pass": False}
                (artifact / filename).write_text(json.dumps(data))

    def assemble(self, root, output):
        with patch.dict(os.environ, RELEASE_TAG=f"v{release.VERSION}", RELEASE_COMMIT="a"*40,
                        RELEASE_RUN_URL="https://github.com/phonowell/pp-switch/actions/runs/1"), contextlib.redirect_stdout(io.StringIO()):
            release.assemble(root, output)

    def test_complete_native_build_retains_failed_budgets_and_checksums(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "artifacts"
            self.fixture(root)
            out = Path(temp) / "release"
            self.assemble(root, out)
            evidence = json.loads((out / "verification.json").read_text(encoding="utf-8"))
            self.assertEqual(len(evidence["platforms"]), 4)
            self.assertTrue(all(not p["measurement"]["budget_pass"] for p in evidence["platforms"]))
            self.assertIn("**FAIL**", (out / "release-notes.md").read_text(encoding="utf-8"))
            self.assertIn("More → Register import links", (out / "release-notes.md").read_text(encoding="utf-8"))
            for line in (out / "SHA256SUMS.txt").read_text(encoding="utf-8").splitlines():
                digest, name = line.split("  ")
                self.assertEqual(digest, release.digest(out / name))

    def test_missing_target_tampered_archive_and_missing_measurement_block_release(self):
        for broken in ["missing-target", "tampered", "missing-metrics"]:
            with self.subTest(broken=broken), tempfile.TemporaryDirectory() as temp:
                root = Path(temp) / "artifacts"
                self.fixture(root)
                first = root / f"pp-switch-{release.TARGETS[0]}"
                if broken == "missing-target":
                    import shutil
                    shutil.rmtree(first)
                elif broken == "tampered":
                    next((first / "package").glob("*.zip")).write_bytes(b"tampered")
                else:
                    (first / "import-metrics.json").unlink()
                with self.assertRaises((ValueError, FileNotFoundError)):
                    self.assemble(root, Path(temp) / "release")

    def test_mismatched_tag_blocks_build_and_remote_digest_blocks_publication(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "artifacts"
            self.fixture(root)
            with patch.dict(os.environ, RELEASE_TAG="v9.9.9"):
                with self.assertRaises(ValueError):
                    release.assemble(root, Path(temp) / "invalid")
            out = Path(temp) / "release"
            self.assemble(root, out)
            assets = [p for p in out.iterdir() if p.name != "release-notes.md"]
            remote = {"assets": [{"name": p.name, "size": p.stat().st_size, "digest": "sha256:wrong"} for p in assets]}
            with patch.dict(os.environ, RELEASE_TAG=f"v{release.VERSION}"), patch("release.subprocess.run") as run, patch("release.subprocess.check_output", return_value=json.dumps(remote)):
                with self.assertRaises(ValueError):
                    release.publish(out)
                self.assertEqual(run.call_count, 1)  # draft created; publication never called


if __name__ == "__main__":
    unittest.main()
