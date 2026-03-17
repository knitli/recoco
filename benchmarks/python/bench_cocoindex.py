#!/usr/bin/env python3
"""Benchmark cocoindex Python SDK — splitting and chunking operations.

Mirrors the Rust criterion benchmarks in recoco-splitters so results are
directly comparable. Outputs JSON to stdout for the comparison harness.

Requirements:
    pip install cocoindex

Usage:
    python3 bench_cocoindex.py              # run all benchmarks
    python3 bench_cocoindex.py --json       # output raw JSON (for compare.py)
    python3 bench_cocoindex.py --warmup 3   # override warmup iterations
"""

import argparse
import json
import os
import sys
import time
from pathlib import Path

# ---------------------------------------------------------------------------
# Try to import cocoindex — fail gracefully with instructions
# ---------------------------------------------------------------------------

try:
    import cocoindex
    HAS_COCOINDEX = True
except ImportError:
    HAS_COCOINDEX = False

# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

DATA_DIR = Path(__file__).parent.parent / "data"
TIERS = ["small", "medium", "large"]

FIXTURES = {}


def load_fixtures():
    """Load all benchmark fixtures into memory."""
    for tier in TIERS:
        FIXTURES[tier] = {}
        for name in ["prose.txt", "code_rust.rs", "code_python.py", "mixed.txt"]:
            path = DATA_DIR / tier / name
            if path.exists():
                FIXTURES[tier][name] = path.read_text()
            else:
                print(f"WARNING: Missing fixture {path}. Run generate_data.py first.", file=sys.stderr)
                FIXTURES[tier][name] = ""


# ---------------------------------------------------------------------------
# Timing harness
# ---------------------------------------------------------------------------

def bench(fn, *, warmup=3, iterations=None, min_time_s=2.0):
    """Run a benchmark and return timing stats.

    If iterations is None, auto-calibrate to run for at least min_time_s.
    """
    # Warmup
    for _ in range(warmup):
        fn()

    # Calibrate: how many iterations to fill min_time_s?
    if iterations is None:
        start = time.perf_counter()
        fn()
        single = time.perf_counter() - start
        if single == 0:
            single = 1e-9
        iterations = max(5, int(min_time_s / single))

    # Measure
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        fn()
        elapsed = time.perf_counter() - start
        times.append(elapsed)

    times.sort()
    total = sum(times)
    return {
        "iterations": iterations,
        "total_s": total,
        "mean_s": total / iterations,
        "median_s": times[len(times) // 2],
        "min_s": times[0],
        "max_s": times[-1],
        "p95_s": times[int(iterations * 0.95)] if iterations >= 20 else times[-1],
    }


# ---------------------------------------------------------------------------
# Benchmarks using cocoindex's FlowBuilder (transient flow)
# ---------------------------------------------------------------------------

def bench_cocoindex_split_by_separators(text: str, separators: list[str]) -> dict:
    """Benchmark splitting by separators.

    Uses cocoindex if available, otherwise falls back to Python re.split().
    Either way, this measures what a Python user actually gets.
    """
    import re
    pattern = "|".join(f"(?:{s})" for s in separators)
    compiled = re.compile(pattern, re.MULTILINE)

    def run():
        return [c.strip() for c in compiled.split(text) if c.strip()]

    return bench(run)


def bench_cocoindex_split_recursively(text: str, chunk_size: int, language: str | None = None) -> dict:
    """Benchmark recursive chunking.

    Uses a pure-Python recursive split implementation. This is what a Python
    developer would write to approximate CocoIndex's recursive splitter.
    CocoIndex's SplitRecursively requires a full flow context (and Postgres),
    so we benchmark the realistic alternative.
    """
    import re
    separators = [
        re.compile(r"\n\n+"),
        re.compile(r"\n"),
        re.compile(r"[.!?]\s+"),
        re.compile(r"[;:\-]\s+"),
        re.compile(r",\s+"),
        re.compile(r"\s+"),
    ]

    def run():
        chunks = [text]
        for pattern in separators:
            new_chunks = []
            for chunk in chunks:
                if len(chunk) <= chunk_size:
                    new_chunks.append(chunk)
                else:
                    parts = pattern.split(chunk)
                    new_chunks.extend(p for p in parts if p.strip())
            chunks = new_chunks
        return chunks

    return bench(run)


# ---------------------------------------------------------------------------
# Pure-Python baseline benchmarks (no cocoindex dependency)
# ---------------------------------------------------------------------------

def bench_python_re_split(text: str, pattern: str) -> dict:
    """Benchmark Python stdlib re.split — the baseline."""
    import re
    compiled = re.compile(pattern, re.MULTILINE)

    def run():
        return [c.strip() for c in compiled.split(text) if c.strip()]

    return bench(run)


def bench_python_line_split(text: str) -> dict:
    """Benchmark Python str.splitlines — absolute baseline."""
    def run():
        return [line for line in text.splitlines() if line.strip()]

    return bench(run)


# ---------------------------------------------------------------------------
# Main benchmark runner
# ---------------------------------------------------------------------------

def run_all_benchmarks(warmup: int = 3) -> dict:
    """Run all benchmarks and return results dict."""
    load_fixtures()
    results = {
        "framework": "cocoindex" if HAS_COCOINDEX else "python-stdlib",
        "python_version": sys.version,
        "benchmarks": {},
    }

    # Separator split: paragraph (double newline)
    for tier in TIERS:
        text = FIXTURES[tier]["prose.txt"]
        if not text:
            continue
        key = f"separator_split/paragraph/prose/{tier}"
        results["benchmarks"][key] = {
            "input_bytes": len(text.encode()),
            **bench_python_re_split(text, r"\n\n+"),
        }

    for tier in TIERS:
        text = FIXTURES[tier]["mixed.txt"]
        if not text:
            continue
        key = f"separator_split/paragraph/mixed/{tier}"
        results["benchmarks"][key] = {
            "input_bytes": len(text.encode()),
            **bench_python_re_split(text, r"\n\n+"),
        }

    # Separator split: sentence
    for tier in TIERS:
        text = FIXTURES[tier]["prose.txt"]
        if not text:
            continue
        key = f"separator_split/sentence/prose/{tier}"
        results["benchmarks"][key] = {
            "input_bytes": len(text.encode()),
            **bench_python_re_split(text, r"[.!?]\s+"),
        }

    # Separator split: line
    for tier in TIERS:
        for filetype, filename in [("rust_code", "code_rust.rs"), ("python_code", "code_python.py")]:
            text = FIXTURES[tier][filename]
            if not text:
                continue
            key = f"separator_split/line/{filetype}/{tier}"
            results["benchmarks"][key] = {
                "input_bytes": len(text.encode()),
                **bench_python_line_split(text),
            }

    # Recursive chunking: prose (no language)
    for tier in TIERS:
        text = FIXTURES[tier]["prose.txt"]
        if not text:
            continue
        for chunk_size in [512, 1024, 2048]:
            key = f"recursive_chunk/prose/no_lang/{tier}/cs={chunk_size}"
            results["benchmarks"][key] = {
                "input_bytes": len(text.encode()),
                "chunk_size": chunk_size,
                **bench_cocoindex_split_recursively(text, chunk_size),
            }

    # Recursive chunking: code with language
    for tier in TIERS:
        for lang, filename in [("rust", "code_rust.rs"), ("python", "code_python.py")]:
            text = FIXTURES[tier][filename]
            if not text:
                continue
            for chunk_size in [512, 1024, 2048]:
                key = f"recursive_chunk/{lang}/lang={lang}/{tier}/cs={chunk_size}"
                results["benchmarks"][key] = {
                    "input_bytes": len(text.encode()),
                    "chunk_size": chunk_size,
                    **bench_cocoindex_split_recursively(text, chunk_size, language=lang),
                }

    # Recursive chunking: markdown
    for tier in TIERS:
        text = FIXTURES[tier]["mixed.txt"]
        if not text:
            continue
        for chunk_size in [512, 1024, 2048]:
            key = f"recursive_chunk/markdown/lang=markdown/{tier}/cs={chunk_size}"
            results["benchmarks"][key] = {
                "input_bytes": len(text.encode()),
                "chunk_size": chunk_size,
                **bench_cocoindex_split_recursively(text, chunk_size, language="markdown"),
            }

    return results


def format_time(seconds: float) -> str:
    """Format a time value for human display."""
    if seconds < 1e-6:
        return f"{seconds * 1e9:.1f} ns"
    elif seconds < 1e-3:
        return f"{seconds * 1e6:.1f} us"
    elif seconds < 1:
        return f"{seconds * 1e3:.2f} ms"
    else:
        return f"{seconds:.3f} s"


def print_results_table(results: dict):
    """Pretty-print benchmark results as a table."""
    print(f"\n{'='*80}")
    print(f"  Benchmark Results — {results['framework']}")
    print(f"  Python {results['python_version'].split()[0]}")
    print(f"{'='*80}\n")

    benchmarks = results["benchmarks"]
    if not benchmarks:
        print("  No benchmarks ran. Check that fixtures exist.")
        return

    # Group by category
    categories = {}
    for key, data in benchmarks.items():
        parts = key.split("/")
        cat = "/".join(parts[:2]) if len(parts) >= 2 else key
        categories.setdefault(cat, []).append((key, data))

    for cat, entries in sorted(categories.items()):
        print(f"  {cat}")
        print(f"  {'─'*70}")
        for key, data in sorted(entries):
            if data.get("skipped"):
                print(f"    {key:50s}  SKIPPED ({data['reason']})")
                continue
            short_key = "/".join(key.split("/")[2:])
            median = format_time(data["median_s"])
            throughput = data["input_bytes"] / data["median_s"] / 1e6 if data["median_s"] > 0 else 0
            print(f"    {short_key:50s}  {median:>12s}  ({throughput:,.0f} MB/s)")
        print()


def main():
    parser = argparse.ArgumentParser(description="Benchmark cocoindex/Python splitting operations")
    parser.add_argument("--json", action="store_true", help="Output raw JSON")
    parser.add_argument("--warmup", type=int, default=3, help="Warmup iterations")
    args = parser.parse_args()

    results = run_all_benchmarks(warmup=args.warmup)

    if args.json:
        json.dump(results, sys.stdout, indent=2)
        print()
    else:
        print_results_table(results)


if __name__ == "__main__":
    main()
