# Benchmarks: Recoco (Rust) vs CocoIndex (Python)

Cross-language benchmark suite comparing Recoco's pure Rust API against
CocoIndex's Python SDK. Both use the **same underlying Rust splitting engine** —
the benchmark measures the overhead of the Python ↔ Rust FFI boundary (PyO3)
that CocoIndex users pay vs Recoco's direct Rust-to-Rust path.

## Quick Start

```bash
# 1. Generate test data (three tiers: 1KB, 100KB, 10MB)
python3 benchmarks/generate_data.py

# 2. Install CocoIndex for Python benchmarks
pip install cocoindex

# 3. Run Rust benchmarks (criterion)
cargo bench -p recoco-splitters --features rust,python,markdown
cargo bench -p recoco-core --bench transient_flow --features function-split

# 4. Run CocoIndex Python benchmarks
python3 benchmarks/python/bench_cocoindex.py

# 5. Generate comparison report
python3 benchmarks/python/bench_cocoindex.py --json > benchmarks/results/python.json
python3 benchmarks/compare.py
```

Or run everything at once:

```bash
python3 benchmarks/generate_data.py
python3 benchmarks/compare.py --run-all
```

## What's Benchmarked

Both sides execute the same operations through their respective transient flow
evaluation paths:

| Category | Operation | Recoco | CocoIndex |
|----------|-----------|--------|-----------|
| **Separator Split** | Paragraph (`\n\n+`) | `evaluate_transient_flow()` | `_engine.TransientFlow.evaluate_async()` |
| **Separator Split** | Sentence (`[.!?]\s+`) | `evaluate_transient_flow()` | `_engine.TransientFlow.evaluate_async()` |
| **Separator Split** | Line (`\n`) | `evaluate_transient_flow()` | `_engine.TransientFlow.evaluate_async()` |
| **Recursive Chunk** | Prose (no language) | `RecursiveChunker::split()` | `SplitRecursively` via engine |
| **Recursive Chunk** | Rust code (tree-sitter) | `RecursiveChunker::split()` | `SplitRecursively` via engine |
| **Recursive Chunk** | Python code (tree-sitter) | `RecursiveChunker::split()` | `SplitRecursively` via engine |
| **Recursive Chunk** | Markdown (tree-sitter) | `RecursiveChunker::split()` | `SplitRecursively` via engine |
| **Transient Flow** | Build + evaluate split | `FlowBuilder` + `evaluate_transient_flow()` | N/A (Rust-only) |
| **Value Creation** | String/int/float wrapping | `Value::Basic(...)` | N/A (Rust-only) |
| **Language Detection** | File extension lookup | `detect_language()` | N/A (Rust-only) |
| **Construction** | Splitter/chunker init | `::new()` | N/A (Rust-only) |

## Data Tiers

| Tier | Size | Purpose |
|------|------|---------|
| **small** | ~1 KB | Startup overhead, cache behavior |
| **medium** | ~100 KB | Typical document processing |
| **large** | ~10 MB | Throughput measurement, scaling |

Each tier includes four content types: prose, Rust code, Python code, and mixed markdown.

## What This Measures

The key insight: **CocoIndex and Recoco share the same Rust splitting engine.**
The performance difference comes from:

1. **FFI overhead** — CocoIndex serializes data across the Python ↔ Rust boundary
   via PyO3 for every `evaluate_async()` call
2. **Async bridge** — CocoIndex bridges Python's asyncio with Tokio via PyO3,
   adding event loop coordination overhead
3. **Value encoding/decoding** — CocoIndex converts between Python objects and
   Rust `Value` types on each call
4. **Framework overhead** — CocoIndex's flow builder and evaluation path includes
   Python-side orchestration

Recoco eliminates all of these — the splitting engine is called directly in Rust
with zero serialization or FFI cost.

## Interpreting Results

The comparison report (`benchmarks/results/RESULTS.md`) shows:

- **Median time** for each benchmark on both sides
- **Throughput** in MB/s where applicable
- **Speedup factor** (CocoIndex time / Recoco time)

For small inputs, the FFI overhead dominates (expect high speedup factors).
For large inputs, the actual computation dominates (expect smaller but still
meaningful speedup factors).
