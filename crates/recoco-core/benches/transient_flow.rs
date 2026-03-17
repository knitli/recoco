// SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: Apache-2.0

//! Criterion benchmarks for transient flow evaluation.
//!
//! Measures the end-to-end cost of building and evaluating flows through the
//! FlowBuilder → analyze → evaluate pipeline. This captures the framework
//! overhead that users actually pay.
//!
//! Run with:
//!   cargo bench -p recoco-core --bench transient_flow --features function-split

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::fs;
use std::path::PathBuf;

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
            "Missing fixture {}: {}. Run `python3 benchmarks/generate_data.py`.",
            path.display(),
            e
        )
    })
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// Benchmark: Transient flow — SplitBySeparators
// ---------------------------------------------------------------------------

#[cfg(feature = "function-split")]
fn bench_transient_split_by_separators(c: &mut Criterion) {
    use recoco_core::builder::FlowBuilder;
    use recoco_core::execution::evaluator::evaluate_transient_flow;
    use recoco_core::prelude::*;
    use serde_json::json;

    let rt = runtime();

    // One-time initialization
    rt.block_on(async {
        let _ = recoco_core::lib_context::init_lib_context(Some(
            recoco_core::settings::Settings::default(),
        ))
        .await;
    });

    let tiers = ["small", "medium"];
    let fixtures: Vec<(String, String)> = tiers
        .iter()
        .map(|t| (t.to_string(), load_fixture(t, "prose.txt")))
        .collect();

    // Benchmark: flow build + evaluate (amortized over evaluations)
    let mut group = c.benchmark_group("transient_flow/split_by_separators");

    for (tier, content) in &fixtures {
        group.throughput(Throughput::Bytes(content.len() as u64));

        // Build flow once, evaluate many times
        let flow = rt.block_on(async {
            let mut builder = FlowBuilder::new("bench_split").await.unwrap();
            let input = builder
                .add_direct_input(
                    "text_input".to_string(),
                    schema::make_output_type(schema::BasicValueType::Str),
                )
                .unwrap();

            let output = builder
                .transform(
                    "SplitBySeparators".to_string(),
                    json!({
                        "separators_regex": [r"\n\n+"],
                        "keep_separator": null,
                        "include_empty": false,
                        "trim": true
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                    vec![(input, Some("text".to_string()))],
                    None,
                    "splitter".to_string(),
                )
                .await
                .unwrap();

            builder.set_direct_output(output).unwrap();
            builder.build_transient_flow().await.unwrap()
        });

        group.bench_with_input(
            BenchmarkId::new("evaluate", tier),
            content,
            |b, text| {
                b.to_async(&rt).iter(|| async {
                    let input = value::Value::Basic(value::BasicValue::Str(text.clone().into()));
                    evaluate_transient_flow(&flow.0, &vec![input])
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();

    // Benchmark: full flow construction (build + analyze + plan)
    let mut group = c.benchmark_group("transient_flow/build_split_flow");
    group.sample_size(20); // Flow construction is slow, fewer samples

    group.bench_function("build_and_analyze", |b| {
        b.to_async(&rt).iter(|| async {
            let mut builder = FlowBuilder::new("bench_build").await.unwrap();
            let input = builder
                .add_direct_input(
                    "text_input".to_string(),
                    schema::make_output_type(schema::BasicValueType::Str),
                )
                .unwrap();

            let output = builder
                .transform(
                    "SplitBySeparators".to_string(),
                    json!({
                        "separators_regex": [r"\n\n+"],
                        "keep_separator": null,
                        "include_empty": false,
                        "trim": true
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                    vec![(input, Some("text".to_string()))],
                    None,
                    "splitter".to_string(),
                )
                .await
                .unwrap();

            builder.set_direct_output(output).unwrap();
            builder.build_transient_flow().await.unwrap()
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Transient flow — SplitRecursively
// ---------------------------------------------------------------------------

#[cfg(feature = "function-split")]
fn bench_transient_split_recursively(c: &mut Criterion) {
    use recoco_core::builder::FlowBuilder;
    use recoco_core::execution::evaluator::evaluate_transient_flow;
    use recoco_core::prelude::*;
    use serde_json::json;

    let rt = runtime();

    // Ensure init (idempotent)
    rt.block_on(async {
        let _ = recoco_core::lib_context::init_lib_context(Some(
            recoco_core::settings::Settings::default(),
        ))
        .await;
    });

    let tiers = ["small", "medium"];
    let fixtures: Vec<(String, String)> = tiers
        .iter()
        .map(|t| (t.to_string(), load_fixture(t, "code_rust.rs")))
        .collect();

    let mut group = c.benchmark_group("transient_flow/split_recursively");

    for (tier, content) in &fixtures {
        group.throughput(Throughput::Bytes(content.len() as u64));

        let flow = rt.block_on(async {
            let mut builder = FlowBuilder::new("bench_recursive").await.unwrap();
            let input = builder
                .add_direct_input(
                    "text_input".to_string(),
                    schema::make_output_type(schema::BasicValueType::Str),
                )
                .unwrap();

            let output = builder
                .transform(
                    "SplitRecursively".to_string(),
                    json!({
                        "chunk_size": 1024,
                        "chunk_overlap": 100,
                        "language": "rust"
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                    vec![(input, Some("text".to_string()))],
                    None,
                    "chunker".to_string(),
                )
                .await
                .unwrap();

            builder.set_direct_output(output).unwrap();
            builder.build_transient_flow().await.unwrap()
        });

        group.bench_with_input(
            BenchmarkId::new("evaluate", tier),
            content,
            |b, text| {
                b.to_async(&rt).iter(|| async {
                    let input = value::Value::Basic(value::BasicValue::Str(text.clone().into()));
                    evaluate_transient_flow(&flow.0, &vec![input])
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Value creation overhead
// ---------------------------------------------------------------------------

fn bench_value_creation(c: &mut Criterion) {
    use recoco_core::prelude::*;

    type Val = value::Value<value::ScopeValue>;

    let mut group = c.benchmark_group("value_creation");

    let small_text = "Hello, world!";
    let medium_text = "x".repeat(10_000);

    group.bench_function("basic_str/small", |b| {
        b.iter(|| Val::Basic(value::BasicValue::Str(small_text.into())));
    });

    group.bench_function("basic_str/10KB", |b| {
        let text = medium_text.clone();
        b.iter(|| Val::Basic(value::BasicValue::Str(text.clone().into())));
    });

    group.bench_function("basic_i64", |b| {
        b.iter(|| Val::Basic(value::BasicValue::Int64(42)));
    });

    group.bench_function("basic_f64", |b| {
        b.iter(|| Val::Basic(value::BasicValue::Float64(3.14159)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[cfg(feature = "function-split")]
criterion_group!(
    flow_benches,
    bench_transient_split_by_separators,
    bench_transient_split_recursively,
    bench_value_creation,
);

#[cfg(not(feature = "function-split"))]
criterion_group!(flow_benches, bench_value_creation,);

criterion_main!(flow_benches);
