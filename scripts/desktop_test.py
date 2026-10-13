"""Shared-workstation launch guards: declined desktop checks never spawn a child."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class DesktopLaunchGuards(unittest.TestCase):
    def test_gui_wrapper_does_not_execute_without_desktop_opt_in(self):
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "launched"
            code = "from pathlib import Path; Path(" + repr(str(marker)) + ").write_text('bad')"
            result = subprocess.run(
                [sys.executable, str(ROOT / "scripts/gui_smoke.py"), sys.executable, "-c", code],
                env=dict(os.environ, CI=""), capture_output=True, text=True, timeout=10,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("SKIPPED", result.stderr)
            self.assertFalse(marker.exists())

    def test_url_and_measurement_wrappers_refuse_before_launch(self):
        for script in ("protocol_smoke.py", "measure.py"):
            with self.subTest(script=script), tempfile.TemporaryDirectory() as directory:
                result = subprocess.run(
                    [sys.executable, str(ROOT / "scripts" / script), "missing-binary", directory]
                    if script == "protocol_smoke.py" else
                    [sys.executable, str(ROOT / "scripts" / script), "missing-binary"],
                    env=dict(os.environ, CI=""), capture_output=True, text=True, timeout=10,
                )
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("SKIPPED", result.stderr)
                self.assertFalse((Path(directory) / "pid.txt").exists())
