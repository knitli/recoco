#!/usr/bin/env python3
"""Benchmark CocoIndex Python SDK — actual CocoIndex operations via transient flows.

This benchmarks the REAL CocoIndex engine (same Rust core, accessed via PyO3)
against Recoco's pure Rust API. Both sides use their respective transient flow
evaluation paths, making this a true apples-to-apples comparison of:

  - CocoIndex: Python → PyO3 → Rust engine → PyO3 → Python
  - Recoco:    Rust → Rust engine (no FFI boundary)

Requirements:
    pip install cocoindex

Usage:
    python3 bench_cocoindex.py              # run all benchmarks
    python3 bench_cocoindex.py --json       # output raw JSON (for compare.py)
    python3 bench_cocoindex.py --warmup 3   # override warmup iterations
"""

import argparse
import asyncio
import json
import sys
import time
from pathlib import Path

# ---------------------------------------------------------------------------
# CocoIndex import
# ---------------------------------------------------------------------------

try:
    import cocoindex
    from cocoindex import _engine
    HAS_COCOINDEX = True
except ImportError:
    HAS_COCOINDEX = False
    print("ERROR: cocoindex is required. Install with: pip install cocoindex", file=sys.stderr)
    sys.exit(1)

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
    for _ in range(warmup):
        fn()

    if iterations is None:
        start = time.perf_counter()
        fn()
        single = time.perf_counter() - start
        if single == 0:
            single = 1e-9
        iterations = max(5, int(min_time_s / single))

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
# CocoIndex transient flow builders
# ---------------------------------------------------------------------------

_STR_TYPE = {"type": {"kind": "Str"}, "nullable": False, "attrs": {}}
_INT_TYPE = {"type": {"kind": "Int64"}, "nullable": False, "attrs": {}}


async def build_split_by_separators_flow(loop, separators: list[str]):
    """Build a transient SplitBySeparators flow."""
    builder = _engine.FlowBuilder("bench_sep_split", loop)
    input_slice = builder.add_direct_input("text_input", _STR_TYPE)
    output = builder.transform(
        "SplitBySeparators",
        {
            "separators_regex": separators,
            "include_empty": False,
            "trim": True,
        },
        [(input_slice, "text")],
        None,
        "splitter",
    )
    builder.set_direct_output(output)
    return await builder.build_transient_flow_async(loop)


async def build_split_recursively_flow(loop, chunk_size: int, chunk_overlap: int, language: str | None = None):
    """Build a transient SplitRecursively flow."""
    builder = _engine.FlowBuilder("bench_recursive", loop)
    input_slice = builder.add_direct_input("text_input", _STR_TYPE)

    args = [(input_slice, "text")]

    cs = builder.constant(_INT_TYPE, chunk_size)
    args.append((cs, "chunk_size"))

    co = builder.constant(_INT_TYPE, chunk_overlap)
    args.append((co, "chunk_overlap"))

    if language is not None:
        lang = builder.constant(_STR_TYPE, language)
        args.append((lang, "language"))

    output = builder.transform(
        "SplitRecursively",
        {},
        args,
        None,
        "chunker",
    )
    builder.set_direct_output(output)
    return await builder.build_transient_flow_async(loop)


async def bench_async(flow, text: str, *, warmup=3, iterations=None, min_time_s=2.0):
    """Benchmark a transient flow evaluation asynchronously."""
    for _ in range(warmup):
        await flow.evaluate_async([text])

    if iterations is None:
        start = time.perf_counter()
        await flow.evaluate_async([text])
        single = time.perf_counter() - start
        if single == 0:
            single = 1e-9
        iterations = max(5, int(min_time_s / single))

    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        await flow.evaluate_async([text])
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
# Benchmark runner
# ---------------------------------------------------------------------------

def run_all_benchmarks(warmup: int = 3) -> dict:
    """Run all benchmarks using CocoIndex's actual engine via transient flows."""
    load_fixtures()
    cocoindex.init()

    results = {
        "framework": "cocoindex",
        "cocoindex_version": cocoindex.__version__ if hasattr(cocoindex, "__version__") else "unknown",
        "python_version": sys.version,
        "benchmarks": {},
    }

    async def _run_all():
        loop = asyncio.get_running_loop()

        # -------------------------------------------------------------------
        # SplitBySeparators benchmarks
        # -------------------------------------------------------------------

        # Paragraph split (double newline)
        flow_para = await build_split_by_separators_flow(loop, [r"\n\n+"])
        for tier in TIERS:
            text = FIXTURES[tier]["prose.txt"]
            if not text:
                continue
            key = f"separator_split/paragraph/prose/{tier}"
            results["benchmarks"][key] = {
                "input_bytes": len(text.encode()),
                **await bench_async(flow_para, text, warmup=warmup),
            }

        for tier in TIERS:
            text = FIXTURES[tier]["mixed.txt"]
            if not text:
                continue
            key = f"separator_split/paragraph/mixed/{tier}"
            results["benchmarks"][key] = {
                "input_bytes": len(text.encode()),
                **await bench_async(flow_para, text, warmup=warmup),
            }

        # Sentence split
        flow_sentence = await build_split_by_separators_flow(loop, [r"[.!?]\s+"])
        for tier in TIERS:
            text = FIXTURES[tier]["prose.txt"]
            if not text:
                continue
            key = f"separator_split/sentence/prose/{tier}"
            results["benchmarks"][key] = {
                "input_bytes": len(text.encode()),
                **await bench_async(flow_sentence, text, warmup=warmup),
            }

        # Line split
        flow_line = await build_split_by_separators_flow(loop, [r"\n"])
        for tier in TIERS:
            for filetype, filename in [("rust_code", "code_rust.rs"), ("python_code", "code_python.py")]:
                text = FIXTURES[tier][filename]
                if not text:
                    continue
                key = f"separator_split/line/{filetype}/{tier}"
                results["benchmarks"][key] = {
                    "input_bytes": len(text.encode()),
                    **await bench_async(flow_line, text, warmup=warmup),
                }

        # -------------------------------------------------------------------
        # SplitRecursively benchmarks
        # -------------------------------------------------------------------

        chunk_sizes = [512, 1024, 2048]

        # Prose (no language — regex-only path)
        for chunk_size in chunk_sizes:
            flow = await build_split_recursively_flow(loop, chunk_size, chunk_size // 10)
            for tier in TIERS:
                text = FIXTURES[tier]["prose.txt"]
                if not text:
                    continue
                key = f"recursive_chunk/prose/no_lang/{tier}/cs={chunk_size}"
                results["benchmarks"][key] = {
                    "input_bytes": len(text.encode()),
                    "chunk_size": chunk_size,
                    **await bench_async(flow, text, warmup=warmup),
                }

        # Rust code (tree-sitter path)
        for chunk_size in chunk_sizes:
            flow = await build_split_recursively_flow(loop, chunk_size, chunk_size // 10, "rust")
            for tier in TIERS:
                text = FIXTURES[tier]["code_rust.rs"]
                if not text:
                    continue
                key = f"recursive_chunk/rust/lang=rust/{tier}/cs={chunk_size}"
                results["benchmarks"][key] = {
                    "input_bytes": len(text.encode()),
                    "chunk_size": chunk_size,
                    **await bench_async(flow, text, warmup=warmup),
                }

        # Python code (tree-sitter path)
        for chunk_size in chunk_sizes:
            flow = await build_split_recursively_flow(loop, chunk_size, chunk_size // 10, "python")
            for tier in TIERS:
                text = FIXTURES[tier]["code_python.py"]
                if not text:
                    continue
                key = f"recursive_chunk/python/lang=python/{tier}/cs={chunk_size}"
                results["benchmarks"][key] = {
                    "input_bytes": len(text.encode()),
                    "chunk_size": chunk_size,
                    **await bench_async(flow, text, warmup=warmup),
                }

        # Markdown (tree-sitter path)
        for chunk_size in chunk_sizes:
            flow = await build_split_recursively_flow(loop, chunk_size, chunk_size // 10, "markdown")
            for tier in TIERS:
                text = FIXTURES[tier]["mixed.txt"]
                if not text:
                    continue
                key = f"recursive_chunk/markdown/lang=markdown/{tier}/cs={chunk_size}"
                results["benchmarks"][key] = {
                    "input_bytes": len(text.encode()),
                    "chunk_size": chunk_size,
                    **await bench_async(flow, text, warmup=warmup),
                }

    asyncio.run(_run_all())
    return results


# ---------------------------------------------------------------------------
# Display
# ---------------------------------------------------------------------------

def format_time(seconds: float) -> str:
    if seconds < 1e-6:
        return f"{seconds * 1e9:.1f} ns"
    elif seconds < 1e-3:
        return f"{seconds * 1e6:.1f} us"
    elif seconds < 1:
        return f"{seconds * 1e3:.2f} ms"
    else:
        return f"{seconds:.3f} s"


def print_results_table(results: dict):
    print(f"\n{'='*80}")
    print(f"  CocoIndex Benchmark Results (via transient flow / engine API)")
    print(f"  CocoIndex {results.get('cocoindex_version', '?')}, Python {results['python_version'].split()[0]}")
    print(f"{'='*80}\n")

    benchmarks = results["benchmarks"]
    if not benchmarks:
        print("  No benchmarks ran. Check that fixtures exist.")
        return

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
    parser = argparse.ArgumentParser(description="Benchmark CocoIndex splitting via transient flows")
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
