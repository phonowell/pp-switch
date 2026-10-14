import unittest

from test_runner import CLIENT_OVERRIDES, environment


class TestEnvironment(unittest.TestCase):
    def test_test_process_removes_overrides_without_mutating_parent(self):
        inherited = dict.fromkeys(CLIENT_OVERRIDES, "synthetic-override")
        inherited["PATH"] = "synthetic-path"
        cleaned = environment("target/debug/deps/behavior-0123456789abcdef", inherited)
        self.assertEqual(cleaned, {"PATH": "synthetic-path"})
        self.assertTrue(all(name in inherited for name in CLIENT_OVERRIDES))

    def test_normal_app_and_examples_preserve_environment(self):
        inherited = {"ANTHROPIC_BASE_URL": "https://fixture.example"}
        for binary in ("target/debug/pp-switch", "target/release/examples/ui_smoke",
                       "target/debug/pp-switch.exe"):
            self.assertEqual(environment(binary, inherited), inherited)

    def test_windows_test_binary_is_isolated(self):
        self.assertEqual(environment("target/debug/deps/behavior-0123456789abcdef.exe",
                                     {"ANTHROPIC_BASE_URL": "https://fixture.example"}), {})


if __name__ == "__main__":
    unittest.main()
