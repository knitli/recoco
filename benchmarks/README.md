# Benchmarks: Recoco (Rust) vs CocoIndex (Python)

Cross-language benchmark suite comparing Recoco's pure Rust implementation against
CocoIndex's Python SDK for common text processing operations.

## Quick Start

```bash
# 1. Generate test data (three tiers: 1KB, 100KB, 10MB)
python3 benchmarks/generate_data.py

# 2. Run Rust benchmarks (criterion)
cargo bench -p recoco-splitters --features rust,python,markdown
cargo bench -p recoco-core --bench transient_flow --features function-split

# 3. Run Python benchmarks
python3 benchmarks/python/bench_cocoindex.py

# 4. Generate comparison report
python3 benchmarks/python/bench_cocoindex.py --json > benchmarks/results/python.json
python3 benchmarks/compare.py
```

Or run everything at once:

```bash
python3 benchmarks/generate_data.py
python3 benchmarks/compare.py --run-all
```

## What's Benchmarked

| Category | Operation | Recoco | Python |
|----------|-----------|--------|--------|
| **Separator Split** | Paragraph (`\n\n+`) | `SeparatorSplitter::split()` | `re.split()` |
| **Separator Split** | Sentence (`[.!?]\s+`) | `SeparatorSplitter::split()` | `re.split()` |
| **Separator Split** | Line (`\n`) | `SeparatorSplitter::split()` | `str.splitlines()` |
| **Recursive Chunk** | Prose (no language) | `RecursiveChunker::split()` | Recursive `re.split()` |
| **Recursive Chunk** | Rust code (tree-sitter) | `RecursiveChunker::split()` | Recursive `re.split()` |
| **Recursive Chunk** | Python code (tree-sitter) | `RecursiveChunker::split()` | Recursive `re.split()` |
| **Recursive Chunk** | Markdown (tree-sitter) | `RecursiveChunker::split()` | Recursive `re.split()` |
| **Transient Flow** | Build + evaluate split | `FlowBuilder` + `evaluate_transient_flow()` | N/A |
| **Value Creation** | String/int/float wrapping | `Value::Basic(...)` | N/A |
| **Language Detection** | File extension lookup | `detect_language()` | N/A |
| **Construction** | Splitter/chunker init | `::new()` | N/A |

## Data Tiers

| Tier | Size | Purpose |
|------|------|---------|
| **small** | ~1 KB | Startup overhead, cache behavior |
| **medium** | ~100 KB | Typical document processing |
| **large** | ~10 MB | Throughput measurement, scaling |

Each tier includes four content types: prose, Rust code, Python code, and mixed markdown.

## Python Setup

```bash
pip install -r benchmarks/python/requirements.txt
```

If `cocoindex` is not installed, the Python benchmarks fall back to stdlib
`re.split()` — which is actually what a Python user would use without CocoIndex's
Rust-backed splitting. This is a fair comparison since it measures the real
alternative a Python developer has.

## Interpreting Results

The comparison report (`benchmarks/results/RESULTS.md`) shows:

- **Median time** for each benchmark on both sides
- **Throughput** in MB/s where applicable
- **Speedup factor** (Python time / Rust time)

Note that the Python side includes the PyO3 bridge overhead that CocoIndex users
pay. For splitting operations, CocoIndex actually calls into Rust via PyO3, so
the Python overhead is primarily serialization/deserialization across the FFI
boundary. Recoco eliminates this entirely.
