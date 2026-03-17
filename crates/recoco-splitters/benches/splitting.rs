// SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for text splitting operations.
//!
//! Compares separator-based and recursive splitting across three data tiers
//! (small ~1KB, medium ~100KB, large ~10MB) and multiple content types
//! (prose, Rust code, Python code, mixed markdown).
//!
//! Run with:
//!   cargo bench -p recoco-splitters --features rust,python,markdown
//!
//! Or for all languages:
//!   cargo bench -p recoco-splitters --features all

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use recoco_splitters::split::{
    RecursiveChunkConfig, RecursiveChunker, RecursiveSplitConfig, SeparatorSplitConfig,
    SeparatorSplitter,
};
use std::fs;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("benchmarks")
        .join("data")
}

fn load_fixture(tier: &str, name: &str) -> String {
    let path = data_dir().join(tier).join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to read benchmark fixture {}: {}. Run `python3 benchmarks/generate_data.py` first.",
            path.display(),
            e
        )
    })
}

struct Fixtures {
    prose: Vec<(String, String)>,   // (tier, content)
    rust: Vec<(String, String)>,
    python: Vec<(String, String)>,
    mixed: Vec<(String, String)>,
}

fn load_all_fixtures() -> Fixtures {
    let tiers = ["small", "medium", "large"];
    let mut prose = Vec::new();
    let mut rust = Vec::new();
    let mut python = Vec::new();
    let mut mixed = Vec::new();

    for tier in &tiers {
        prose.push((tier.to_string(), load_fixture(tier, "prose.txt")));
        rust.push((tier.to_string(), load_fixture(tier, "code_rust.rs")));
        python.push((tier.to_string(), load_fixture(tier, "code_python.py")));
        mixed.push((tier.to_string(), load_fixture(tier, "mixed.txt")));
    }

    Fixtures { prose, rust, python, mixed }
}

// ---------------------------------------------------------------------------
// Benchmark: Separator splitting
// ---------------------------------------------------------------------------

fn bench_separator_split(c: &mut Criterion) {
    let fixtures = load_all_fixtures();

    // Paragraph splitter (double newline)
    let para_splitter = SeparatorSplitter::new(SeparatorSplitConfig {
        separators_regex: vec![r"\n\n+".to_string()],
        keep_separator: None,
        include_empty: false,
        trim: true,
    })
    .unwrap();

    // Sentence splitter
    let sentence_splitter = SeparatorSplitter::new(SeparatorSplitConfig {
        separators_regex: vec![r"[.!?]\s+".to_string()],
        keep_separator: None,
        include_empty: false,
        trim: true,
    })
    .unwrap();

    // Line splitter
    let line_splitter = SeparatorSplitter::new(SeparatorSplitConfig {
        separators_regex: vec![r"\n".to_string()],
        keep_separator: None,
        include_empty: false,
        trim: true,
    })
    .unwrap();

    let mut group = c.benchmark_group("separator_split/paragraph");
    for (tier, content) in &fixtures.prose {
        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("prose", tier), content, |b, text| {
            b.iter(|| para_splitter.split(text));
        });
    }
    for (tier, content) in &fixtures.mixed {
        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("mixed", tier), content, |b, text| {
            b.iter(|| para_splitter.split(text));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("separator_split/sentence");
    for (tier, content) in &fixtures.prose {
        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("prose", tier), content, |b, text| {
            b.iter(|| sentence_splitter.split(text));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("separator_split/line");
    for (tier, content) in &fixtures.rust {
        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("rust_code", tier), content, |b, text| {
            b.iter(|| line_splitter.split(text));
        });
    }
    for (tier, content) in &fixtures.python {
        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(BenchmarkId::new("python_code", tier), content, |b, text| {
            b.iter(|| line_splitter.split(text));
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Recursive chunking
// ---------------------------------------------------------------------------

fn bench_recursive_chunk(c: &mut Criterion) {
    let fixtures = load_all_fixtures();
    let chunker = RecursiveChunker::new(RecursiveSplitConfig::default()).unwrap();

    // Default chunk sizes to benchmark
    let chunk_sizes: &[usize] = &[512, 1024, 2048];

    // Plain text (no language, regex-only path)
    let mut group = c.benchmark_group("recursive_chunk/prose");
    for (tier, content) in &fixtures.prose {
        for &chunk_size in chunk_sizes {
            let param = format!("{tier}/cs={chunk_size}");
            group.throughput(Throughput::Bytes(content.len() as u64));
            group.bench_with_input(BenchmarkId::new("no_lang", &param), content, |b, text| {
                b.iter(|| {
                    chunker.split(
                        text,
                        RecursiveChunkConfig {
                            chunk_size,
                            min_chunk_size: None,
                            chunk_overlap: Some(chunk_size / 10),
                            language: None,
                        },
                    )
                });
            });
        }
    }
    group.finish();

    // Rust code (tree-sitter path when feature enabled)
    let mut group = c.benchmark_group("recursive_chunk/rust");
    for (tier, content) in &fixtures.rust {
        for &chunk_size in chunk_sizes {
            let param = format!("{tier}/cs={chunk_size}");
            group.throughput(Throughput::Bytes(content.len() as u64));
            group.bench_with_input(
                BenchmarkId::new("lang=rust", &param),
                content,
                |b, text| {
                    b.iter(|| {
                        chunker.split(
                            text,
                            RecursiveChunkConfig {
                                chunk_size,
                                min_chunk_size: None,
                                chunk_overlap: Some(chunk_size / 10),
                                language: Some("rust".to_string()),
                            },
                        )
                    });
                },
            );
        }
    }
    group.finish();

    // Python code
    let mut group = c.benchmark_group("recursive_chunk/python");
    for (tier, content) in &fixtures.python {
        for &chunk_size in chunk_sizes {
            let param = format!("{tier}/cs={chunk_size}");
            group.throughput(Throughput::Bytes(content.len() as u64));
            group.bench_with_input(
                BenchmarkId::new("lang=python", &param),
                content,
                |b, text| {
                    b.iter(|| {
                        chunker.split(
                            text,
                            RecursiveChunkConfig {
                                chunk_size,
                                min_chunk_size: None,
                                chunk_overlap: Some(chunk_size / 10),
                                language: Some("python".to_string()),
                            },
                        )
                    });
                },
            );
        }
    }
    group.finish();

    // Markdown
    let mut group = c.benchmark_group("recursive_chunk/markdown");
    for (tier, content) in &fixtures.mixed {
        for &chunk_size in chunk_sizes {
            let param = format!("{tier}/cs={chunk_size}");
            group.throughput(Throughput::Bytes(content.len() as u64));
            group.bench_with_input(
                BenchmarkId::new("lang=markdown", &param),
                content,
                |b, text| {
                    b.iter(|| {
                        chunker.split(
                            text,
                            RecursiveChunkConfig {
                                chunk_size,
                                min_chunk_size: None,
                                chunk_overlap: Some(chunk_size / 10),
                                language: Some("markdown".to_string()),
                            },
                        )
                    });
                },
            );
        }
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Splitter construction
// ---------------------------------------------------------------------------

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");

    group.bench_function("SeparatorSplitter/simple", |b| {
        b.iter(|| {
            SeparatorSplitter::new(SeparatorSplitConfig {
                separators_regex: vec![r"\n\n+".to_string()],
                keep_separator: None,
                include_empty: false,
                trim: true,
            })
            .unwrap()
        });
    });

    group.bench_function("SeparatorSplitter/complex", |b| {
        b.iter(|| {
            SeparatorSplitter::new(SeparatorSplitConfig {
                separators_regex: vec![
                    r"\n\n+".to_string(),
                    r"\n".to_string(),
                    r"[.!?]\s+".to_string(),
                    r"[;:\-]\s+".to_string(),
                    r",\s+".to_string(),
                ],
                keep_separator: None,
                include_empty: false,
                trim: true,
            })
            .unwrap()
        });
    });

    group.bench_function("RecursiveChunker/default", |b| {
        b.iter(|| RecursiveChunker::new(RecursiveSplitConfig::default()).unwrap());
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Language detection
// ---------------------------------------------------------------------------

fn bench_language_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("language_detection");

    let filenames = [
        "main.rs",
        "app.py",
        "index.js",
        "Component.tsx",
        "styles.css",
        "README.md",
        "Cargo.toml",
        "query.sql",
        "unknown.xyz",
    ];

    group.bench_function("detect_batch", |b| {
        b.iter(|| {
            for name in &filenames {
                let _ = recoco_splitters::prog_langs::detect_language(name);
            }
        });
    });

    for name in &filenames {
        group.bench_with_input(BenchmarkId::new("detect_single", name), name, |b, fname| {
            b.iter(|| recoco_splitters::prog_langs::detect_language(fname));
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_separator_split,
    bench_recursive_chunk,
    bench_construction,
    bench_language_detection,
);
criterion_main!(benches);
