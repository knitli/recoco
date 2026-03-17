#!/usr/bin/env python3
"""Cross-language benchmark comparison: Recoco (Rust) vs CocoIndex (Python).

Reads Criterion JSON output from Rust benchmarks and JSON output from the
Python benchmark script, then produces a markdown comparison report.

Usage:
    # 1. Run Rust benchmarks (generates target/criterion/)
    cargo bench -p recoco-splitters --features rust,python,markdown

    # 2. Run Python benchmarks
    python3 benchmarks/python/bench_cocoindex.py --json > benchmarks/results/python.json

    # 3. Generate comparison
    python3 benchmarks/compare.py

    # Or run everything:
    python3 benchmarks/compare.py --run-all
"""

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
RESULTS_DIR = Path(__file__).parent / "results"
CRITERION_DIR = ROOT / "target" / "criterion"


# ---------------------------------------------------------------------------
# Parse Criterion JSON estimates
# ---------------------------------------------------------------------------

def parse_criterion_results() -> dict:
    """Walk target/criterion/ and extract benchmark timings."""
    results = {}

    if not CRITERION_DIR.exists():
        print(f"WARNING: {CRITERION_DIR} not found. Run Rust benchmarks first.", file=sys.stderr)
        return results

    for estimates_file in CRITERION_DIR.rglob("new/estimates.json"):
        # Path structure: target/criterion/<group>/<bench_id>/new/estimates.json
        parts = estimates_file.relative_to(CRITERION_DIR).parts
        if len(parts) < 3:
            continue
        bench_key = "/".join(parts[:-2])  # everything except "new/estimates.json"

        try:
            data = json.loads(estimates_file.read_text())
            # Criterion stores times in nanoseconds
            median_ns = data.get("median", {}).get("point_estimate", 0)
            mean_ns = data.get("mean", {}).get("point_estimate", 0)
            results[bench_key] = {
                "median_s": median_ns / 1e9,
                "mean_s": mean_ns / 1e9,
            }
        except (json.JSONDecodeError, KeyError) as e:
            print(f"WARNING: Could not parse {estimates_file}: {e}", file=sys.stderr)

    # Also try to get input bytes from benchmark_group metadata
    for sample_file in CRITERION_DIR.rglob("new/sample.json"):
        parts = sample_file.relative_to(CRITERION_DIR).parts
        bench_key = "/".join(parts[:-2])
        if bench_key in results:
            try:
                sample = json.loads(sample_file.read_text())
                throughput = sample.get("throughput")
                if throughput and isinstance(throughput, dict):
                    results[bench_key]["input_bytes"] = throughput.get("Bytes", 0)
            except (json.JSONDecodeError, KeyError):
                pass

    return results


def load_python_results() -> dict:
    """Load Python benchmark results from JSON file."""
    path = RESULTS_DIR / "python.json"
    if not path.exists():
        print(f"WARNING: {path} not found. Run Python benchmarks first.", file=sys.stderr)
        return {}

    data = json.loads(path.read_text())
    return data.get("benchmarks", {})


# ---------------------------------------------------------------------------
# Key matching between Rust and Python results
# ---------------------------------------------------------------------------

def normalize_key(key: str) -> str:
    """Normalize benchmark keys for cross-language matching.

    Rust criterion keys look like: separator_split/paragraph/prose/small
    Python keys look like:        separator_split/paragraph/prose/small
    """
    # Strip any leading/trailing slashes and whitespace
    return key.strip("/").strip()


def match_benchmarks(rust_results: dict, python_results: dict) -> list[dict]:
    """Match Rust and Python benchmarks by normalized key."""
    matches = []
    rust_norm = {normalize_key(k): (k, v) for k, v in rust_results.items()}
    python_norm = {normalize_key(k): (k, v) for k, v in python_results.items()}

    all_keys = sorted(set(rust_norm.keys()) | set(python_norm.keys()))

    for key in all_keys:
        entry = {"key": key}
        if key in rust_norm:
            rk, rv = rust_norm[key]
            entry["rust"] = rv
            entry["rust_key"] = rk
        if key in python_norm:
            pk, pv = python_norm[key]
            entry["python"] = pv
            entry["python_key"] = pk

        if "rust" in entry and "python" in entry:
            rust_median = entry["rust"]["median_s"]
            python_median = entry["python"]["median_s"]
            if rust_median > 0:
                entry["speedup"] = python_median / rust_median
            else:
                entry["speedup"] = float("inf")

        matches.append(entry)

    return matches


# ---------------------------------------------------------------------------
# Report generation
# ---------------------------------------------------------------------------

def format_time(seconds: float) -> str:
    """Format time for display."""
    if seconds < 1e-6:
        return f"{seconds * 1e9:.1f} ns"
    elif seconds < 1e-3:
        return f"{seconds * 1e6:.1f} us"
    elif seconds < 1:
        return f"{seconds * 1e3:.2f} ms"
    else:
        return f"{seconds:.3f} s"


def generate_report(matches: list[dict]) -> str:
    """Generate a markdown comparison report."""
    lines = []
    lines.append("# Benchmark Comparison: Recoco (Rust) vs CocoIndex/Python")
    lines.append("")
    lines.append("> Auto-generated by `benchmarks/compare.py`")
    lines.append("")

    # Summary
    paired = [m for m in matches if "rust" in m and "python" in m and "speedup" in m]
    if paired:
        speedups = [m["speedup"] for m in paired]
        avg_speedup = sum(speedups) / len(speedups)
        max_speedup = max(speedups)
        min_speedup = min(speedups)
        lines.append("## Summary")
        lines.append("")
        lines.append(f"- **Paired benchmarks**: {len(paired)}")
        lines.append(f"- **Average speedup**: {avg_speedup:.1f}x")
        lines.append(f"- **Range**: {min_speedup:.1f}x — {max_speedup:.1f}x")
        lines.append("")

    # Detailed table
    lines.append("## Detailed Results")
    lines.append("")
    lines.append("| Benchmark | Input | Recoco (Rust) | Python | Speedup |")
    lines.append("|-----------|-------|---------------|--------|---------|")

    for m in sorted(matches, key=lambda x: x["key"]):
        key = m["key"]
        input_bytes = ""
        if "rust" in m and "input_bytes" in m["rust"]:
            b = m["rust"]["input_bytes"]
            input_bytes = f"{b / 1024:.0f} KB" if b < 1e6 else f"{b / 1e6:.1f} MB"
        elif "python" in m and "input_bytes" in m["python"]:
            b = m["python"]["input_bytes"]
            input_bytes = f"{b / 1024:.0f} KB" if b < 1e6 else f"{b / 1e6:.1f} MB"

        rust_time = format_time(m["rust"]["median_s"]) if "rust" in m else "—"
        python_time = format_time(m["python"]["median_s"]) if "python" in m else "—"

        if "speedup" in m:
            speedup = f"**{m['speedup']:.1f}x**"
        else:
            speedup = "—"

        lines.append(f"| `{key}` | {input_bytes} | {rust_time} | {python_time} | {speedup} |")

    lines.append("")

    # Rust-only benchmarks
    rust_only = [m for m in matches if "rust" in m and "python" not in m]
    if rust_only:
        lines.append("## Rust-Only Benchmarks")
        lines.append("")
        lines.append("These benchmarks have no Python equivalent (e.g., construction, language detection).")
        lines.append("")
        lines.append("| Benchmark | Recoco (Rust) |")
        lines.append("|-----------|---------------|")
        for m in sorted(rust_only, key=lambda x: x["key"]):
            lines.append(f"| `{m['key']}` | {format_time(m['rust']['median_s'])} |")
        lines.append("")

    lines.append("## Methodology")
    lines.append("")
    lines.append("- **Rust**: [Criterion.rs](https://github.com/bheisler/criterion.rs) with statistical analysis")
    lines.append("- **Python**: Custom harness with auto-calibrated iteration count (minimum 2s per benchmark)")
    lines.append("- **Data**: Synthetic multi-tier fixtures (1 KB, 100 KB, 10 MB) in four content types")
    lines.append("- **Fairness**: Both sides process identical input data. Python benchmarks include")
    lines.append("  regex compilation in the measurement where Recoco includes it.")
    lines.append("")

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Run everything
# ---------------------------------------------------------------------------

def run_rust_benchmarks():
    """Run Rust criterion benchmarks."""
    print("Running Rust benchmarks (this may take a few minutes)...")
    result = subprocess.run(
        ["cargo", "bench", "-p", "recoco-splitters", "--features", "rust,python,markdown"],
        cwd=ROOT,
        capture_output=False,
    )
    if result.returncode != 0:
        print("WARNING: Rust benchmarks had non-zero exit code", file=sys.stderr)


def run_python_benchmarks():
    """Run Python benchmarks and save JSON output."""
    print("Running Python benchmarks...")
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        [sys.executable, str(Path(__file__).parent / "python" / "bench_cocoindex.py"), "--json"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"WARNING: Python benchmarks failed: {result.stderr}", file=sys.stderr)
        return

    (RESULTS_DIR / "python.json").write_text(result.stdout)
    print(f"Python results saved to {RESULTS_DIR / 'python.json'}")


def main():
    parser = argparse.ArgumentParser(description="Compare Recoco (Rust) vs CocoIndex (Python) benchmarks")
    parser.add_argument("--run-all", action="store_true", help="Run both Rust and Python benchmarks first")
    parser.add_argument("--run-python", action="store_true", help="Run Python benchmarks only")
    parser.add_argument("--run-rust", action="store_true", help="Run Rust benchmarks only")
    parser.add_argument("--output", type=str, default=None, help="Output file (default: stdout + benchmarks/RESULTS.md)")
    args = parser.parse_args()

    if args.run_all or args.run_rust:
        run_rust_benchmarks()
    if args.run_all or args.run_python:
        run_python_benchmarks()

    rust_results = parse_criterion_results()
    python_results = load_python_results()

    if not rust_results and not python_results:
        print("ERROR: No benchmark results found. Run benchmarks first:", file=sys.stderr)
        print("  cargo bench -p recoco-splitters --features rust,python,markdown", file=sys.stderr)
        print("  python3 benchmarks/python/bench_cocoindex.py --json > benchmarks/results/python.json", file=sys.stderr)
        sys.exit(1)

    matches = match_benchmarks(rust_results, python_results)
    report = generate_report(matches)

    # Write report
    output_path = Path(args.output) if args.output else RESULTS_DIR / "RESULTS.md"
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report)
    print(f"\nReport written to: {output_path}")

    # Also print to stdout
    print(report)


if __name__ == "__main__":
    main()
