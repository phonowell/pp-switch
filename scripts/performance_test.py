"""Behavioral limits: platform overhead, launch outliers and sample sufficiency."""
import unittest
from performance import evaluate, MIB


class RoutineBudgets(unittest.TestCase):
    def samples(self, idle, peak, times=(100, 100, 100)):
        return [{"startup_ms": time, "idle_rss_bytes": idle * MIB,
                 "peak_rss_bytes": peak * MIB} for time in times]

    def test_platform_limits_are_inclusive_and_excess_memory_fails(self):
        for system, idle, peak in (("Darwin", 160, 192), ("Windows", 32, 48), ("Linux", 32, 48)):
            with self.subTest(system=system):
                self.assertTrue(evaluate(system, self.samples(idle, peak))["budget_pass"])
                for field in ("idle_rss_bytes", "peak_rss_bytes"):
                    samples = self.samples(idle, peak)
                    samples[0][field] += 1
                    self.assertFalse(evaluate(system, samples)["memory_budget_pass"])

    def test_cached_launch_requires_both_median_and_maximum(self):
        self.assertTrue(evaluate("Linux", self.samples(20, 30, (100, 500, 1000)))["budget_pass"])
        for times in ((501, 501, 501), (100, 100, 1001)):
            self.assertFalse(evaluate("Linux", self.samples(20, 30, times))["readiness_budget_pass"])

    def test_one_process_diagnostics_do_not_certify_a_budget(self):
        result = evaluate("Darwin", self.samples(55, 60, (200,)))
        self.assertTrue(result["measured_threshold_pass"])
        self.assertFalse(result["sample_count_pass"])
        self.assertFalse(result["budget_pass"])

    def test_unknown_platform_does_not_get_a_permissive_fallback(self):
        with self.assertRaisesRegex(ValueError, "Unsupported measurement platform"):
            evaluate("unknown", self.samples(20, 30))


if __name__ == "__main__":
    unittest.main()
