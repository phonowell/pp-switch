"""PP-11 revision 2: routine resource limits, separate from cold-start evidence."""
import statistics

BUDGET_VERSION = 2
MIB = 1024**2
MEMORY_MIB = {"Darwin": (160, 192), "Windows": (32, 48), "Linux": (32, 48)}


def limits(system):
    if system not in MEMORY_MIB:
        raise ValueError(f"Unsupported measurement platform: {system}")
    idle, peak = MEMORY_MIB[system]
    return {"idle_rss_bytes": idle * MIB, "peak_rss_bytes": peak * MIB,
            "startup_median_ms": 500, "startup_max_ms": 1000, "minimum_runs": 3}


def summarize(samples):
    return {"startup_median_ms": round(statistics.median(s["startup_ms"] for s in samples), 2),
            "startup_max_ms": max(s["startup_ms"] for s in samples),
            "idle_max_bytes": max(s["idle_rss_bytes"] for s in samples),
            "peak_max_bytes": max(s["peak_rss_bytes"] for s in samples)}


def evaluate(system, samples):
    budget = limits(system)
    measured = summarize(samples)
    memory_pass = (measured["idle_max_bytes"] <= budget["idle_rss_bytes"]
                   and measured["peak_max_bytes"] <= budget["peak_rss_bytes"])
    readiness_pass = (measured["startup_median_ms"] <= budget["startup_median_ms"]
                      and measured["startup_max_ms"] <= budget["startup_max_ms"])
    enough_samples = len(samples) >= budget["minimum_runs"]
    return {"budget_version": BUDGET_VERSION, "platform_system": system,
            "budget_limits": budget, **measured, "memory_budget_pass": memory_pass,
            "readiness_budget_pass": readiness_pass, "sample_count_pass": enough_samples,
            "measured_threshold_pass": memory_pass and readiness_pass,
            "budget_pass": memory_pass and readiness_pass and enough_samples}
