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
from performance import evaluate, MIB


class PublicationGates(unittest.TestCase):
    def test_cache_retaining_native_readiness_cannot_pass_cold_start_budget(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "metrics.json"
            data = {"samples": [{"startup_ms": 100, "idle_rss_bytes": 100, "peak_rss_bytes": 100}],
                    "startup_max_ms": 100, "idle_max_bytes": 100, "peak_max_bytes": 100,
                    "measured_threshold_pass": True, "native_action_readiness": True,
                    "cold_start_verified": False, "budget_pass": False}
            path.write_text(json.dumps(data))
            self.assertFalse(release.metrics(path)["budget_pass"])
            data["budget_pass"] = True
            path.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, "Inconsistent budget"):
                release.metrics(path)

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
            for filename, scenario in [("metrics.json", "empty-window"), ("editor-metrics.json", "provider-editor")]:
                # Budget failures must be retained, not silently made successful.
                samples = [{"startup_ms": 1001, "idle_rss_bytes": 100, "peak_rss_bytes": 100}] * 3
                system = "Darwin" if "apple" in target else "Windows" if "windows" in target else "Linux"
                data = {"scenario": scenario, "samples": samples,
                        "native_action_readiness": False, "cold_start_verified": False,
                        **evaluate(system, samples)}
                (artifact / filename).write_text(json.dumps(data))

    def test_current_routine_budget_is_separate_from_cold_start_and_validates_platform(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "metrics.json"
            samples = [{"startup_ms": 400, "idle_rss_bytes": 129 * MIB, "peak_rss_bytes": 130 * MIB}] * 3
            data = {"samples": samples, "native_action_readiness": False,
                    "cold_start_verified": False, **evaluate("Darwin", samples)}
            path.write_text(json.dumps(data))
            self.assertTrue(release.metrics(path, "aarch64-apple-darwin")["budget_pass"])
            self.assertFalse(release.metrics(path)["cold_start_verified"])
            with self.assertRaisesRegex(ValueError, "platform"):
                release.metrics(path, "x86_64-pc-windows-msvc")
            for key, value in [("budget_limits", {}), ("startup_median_ms", 1),
                               ("budget_pass", False), ("budget_version", 999)]:
                with self.subTest(key=key):
                    path.write_text(json.dumps({**data, key: value}))
                    with self.assertRaises(ValueError):
                        release.metrics(path)

    def test_old_reports_cannot_be_reused_as_current_release_measurements(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "artifacts"
            self.fixture(root)
            path = root / f"pp-switch-{release.TARGETS[0]}" / "metrics.json"
            path.write_text(json.dumps({"scenario": "empty-window",
                "samples": [{"startup_ms": 301, "idle_rss_bytes": 100, "peak_rss_bytes": 100}],
                "startup_max_ms": 301, "idle_max_bytes": 100, "peak_max_bytes": 100,
                "budget_pass": False}))
            with self.assertRaisesRegex(ValueError, "Current budget revision"):
                self.assemble(root, Path(directory) / "release")

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
            self.assertIn("Language and manual updates are in Settings", (out / "release-notes.md").read_text(encoding="utf-8"))
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
                    (first / "editor-metrics.json").unlink()
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

    def test_verified_draft_is_visible_before_publication(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "artifacts"
            self.fixture(root)
            out = Path(temp) / "release"
            self.assemble(root, out)
            assets = [p for p in out.iterdir() if p.name != "release-notes.md"]
            remote = {"assets": [{"name": p.name, "size": p.stat().st_size,
                "digest": f"sha256:{release.digest(p)}"} for p in assets]}

            def draft_lookup(command, **kwargs):
                # The REST release-by-tag endpoint returns 404 for drafts.
                if command[:2] == ["gh", "api"]:
                    raise RuntimeError("Not Found (HTTP 404)")
                self.assertEqual(command, ["gh", "release", "view", f"v{release.VERSION}",
                                           "--json", "assets"])
                return json.dumps(remote)

            with patch.dict(os.environ, RELEASE_TAG=f"v{release.VERSION}"), patch("release.subprocess.run") as run, patch("release.subprocess.check_output", side_effect=draft_lookup), contextlib.redirect_stdout(io.StringIO()):
                release.publish(out)
                self.assertEqual(run.call_count, 2)
                self.assertEqual(run.call_args.args[0],
                                 ["gh", "release", "edit", f"v{release.VERSION}", "--draft=false"])


if __name__ == "__main__":
    unittest.main()
