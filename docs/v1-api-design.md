# Recoco v1.0.0 API Design Plan

## Table of Contents

1. [Current API Surface (main)](#1-current-api-surface-main)
2. [v1.0.0 Engine Architecture](#2-v100-engine-architecture)
3. [Gap Analysis: Current API vs Use Cases](#3-gap-analysis-current-api-vs-use-cases)
4. [CocoIndex v1 Python API Reference](#4-cocoindex-v1-python-api-reference)
5. [Proposed Recoco v1 Rust API](#5-proposed-recoco-v1-rust-api)
6. [Migration Path](#6-migration-path)
7. [Appendix A: Comparison with Issue #1667 (Approved Revision)](#appendix-a-comparison-with-issue-1667-approved-revision)
8. [Appendix B: Code Hash Constant Design](#appendix-b-code-hash-constant-design)

---

## 1. Current API Surface (main)

### Overview

The current `main` branch exposes a **FlowBuilder-centric** API with two execution modes (transient and persisted). The API is functional but verbose, with several ergonomic pain points.

### Public Modules

| Module | Purpose |
|--------|---------|
| `recoco::lib_context` | Global init (`init_lib_context`), settings, runtime access |
| `recoco::builder` | `FlowBuilder`, `DataSlice`, `DataCollector`, `OpScopeRef` |
| `recoco::execution::evaluator` | `evaluate_transient_flow()` |
| `recoco::base::schema` | `BasicValueType`, `EnrichedValueType`, `FieldSchema`, `TableSchema` |
| `recoco::base::value` | `Value`, `BasicValue`, `ScopeValue`, `FieldValues`, `KeyValue` |
| `recoco::ops::sdk` | `SimpleFunctionFactoryBase`, `SimpleFunctionExecutor`, `SourceFactoryBase`, etc. |
| `recoco::ops::registry` | `register_factory()`, `get_*_factory()` |
| `recoco::prelude` | Common re-exports |

### FlowBuilder API

```rust
FlowBuilder::new(name) -> Result<Self>
builder.add_direct_input(name, value_type) -> Result<DataSlice>
builder.add_source(kind, spec_json, scope, name, refresh, exec) -> Result<DataSlice>
builder.transform(kind, spec_json, args, scope, name) -> Result<DataSlice>
builder.set_direct_output(data_slice) -> Result<()>
builder.for_each(data_slice, exec_opts) -> Result<OpScopeRef>
builder.collect(collector, fields, auto_uuid) -> Result<()>
builder.export(name, kind, spec_json, attachments, index, input, setup) -> Result<()>
builder.build_transient_flow() -> Result<TransientFlow>
builder.build_flow() -> Result<Flow>  // persisted
```

### Key Pain Points

1. **JSON-based configuration**: Operations configured via `json!({}).as_object().unwrap().clone()` — no type safety, verbose, error-prone.
2. **String-based operation names**: `"SplitBySeparators"` — typos caught only at runtime.
3. **Verbose value wrapping**: `Value::Basic(BasicValue::Str(s.into()))` for every input.
4. **No schema inference**: Must manually specify `EnrichedValueType` for everything.
5. **Two-trait custom op pattern**: `SimpleFunctionFactoryBase` + `SimpleFunctionExecutor` + `Arc::new()` + `ExecutorFactory::SimpleFunction()` wrapping.
6. **No streaming/incremental in transient mode**: Transient flows evaluate to a single `Value`.
7. **Global singleton init**: `init_lib_context()` must be called once, manages hidden global state.

### Available Operations (Feature-Gated)

| Category | Operations |
|----------|-----------|
| Sources | `LocalFile`, `Postgres`, `S3`, `Azure`, `GDrive` |
| Functions | `SplitBySeparators`, `SplitRecursively`, `SentenceTransformerEmbed`, `ExtractByLlm`, `ParseJson`, `DetectProgrammingLanguage` |
| Targets | `Postgres`, `Qdrant`, `Neo4j`, `Kuzu` |

---

## 2. v1.0.0 Engine Architecture

### Fundamental Shift

The v1.0.0 branch replaces the **declarative FlowBuilder/DataSlice graph** with an **imperative component tree** model. This is a ground-up rewrite of the execution engine.

### Core Concepts

| v0 (main) | v1.0.0 |
|-----------|--------|
| `FlowBuilder` → DAG → `AnalyzedFlow` | `App` → `Component` tree → LMDB state |
| `DataSlice` (lazy graph node) | Direct function calls with memoization |
| `evaluate_transient_flow()` | `app.update()` |
| PostgreSQL for state tracking | LMDB (embedded) for state tracking |
| `init_lib_context()` (global) | `Environment` (explicit) |
| `SourceExecutor` trait | `localfs::walk_dir()` + user code |
| `SimpleFunctionExecutor` trait | `@coco.function` / plain functions |
| `TargetFactory` trait | `declare_row()` / `declare_file()` |
| `DataCollector` + `export()` | `mount_table_target()` + `declare_row()` |

### v1.0.0 Engine Types (from `recoco-core`)

```
recoco-core::engine
├── app.rs          — App<Prof>, UpdateHandle<R>, AppUpdateOptions
├── component.rs    — Component<Prof>, ComponentProcessor trait, mount/run
├── context.rs      — AppContext, ComponentProcessorContext, FnCallContext
├── environment.rs  — Environment<Prof>, EnvironmentSettings
├── execution.rs    — submit, memoization read/write, cleanup
├── function.rs     — FnCallMemoGuard, reserve_memoization()
├── profile.rs      — EngineProfile trait (type-level parameterization)
├── runtime.rs      — Tokio runtime management
├── stats.rs        — ProcessingStats, UpdateStats, ProgressReporter
├── target_state.rs — TargetHandler, TargetActionSink, TargetStateProvider
├── txn_batcher.rs  — LMDB write transaction batching
└── logic_registry  — Logic fingerprint tracking
```

### `EngineProfile` — The Type Backbone

The engine is generic over `EngineProfile`, which defines associated types for:

```rust
pub trait EngineProfile: Debug + Clone + PartialEq + Eq + Hash + Default + 'static {
    type HostRuntimeCtx: Clone + Send + Sync + 'static;
    type ComponentProc: ComponentProcessor<Self>;
    type FunctionData: Clone + Send + Sync + Persist + 'static;
    type TargetHdl: TargetHandler<Self>;
    type TargetStateTrackingRecord: Send + Persist + 'static;
    type TargetAction: Send + 'static;
    type TargetActionSink: TargetActionSink<Self>;
    type TargetStateValue: Send + 'static;
}
```

This is currently very abstract — upstream uses it to plug Python types via PyO3. For a pure Rust library, we need a **concrete profile** with Rust-native types.

### What v1.0.0 Has / Doesn't Have

**Has:**
- Complete LMDB-backed engine with component memoization
- Stable path tracking for incremental updates
- Target state reconciliation system
- Transaction batching
- Progress/stats reporting with `UpdateHandle`
- `inspect` module for database inspection

**Doesn't Have (must port from main or build new):**
- No sources (no `LocalFile`, `Postgres`, `S3`, etc.)
- No functions (no `Split`, `Embed`, `ExtractLlm`, etc.)
- No targets (no `Postgres`, `Qdrant`, `Neo4j`, etc.)
- No high-level user-facing API (only raw engine primitives)
- No `FlowBuilder` equivalent
- No concrete `EngineProfile` implementation

---

## 3. Gap Analysis: Current API vs Use Cases

### CocoIndex Target Use Cases

Based on cocoindex.io documentation and examples:

| Use Case | Description | Current API Support |
|----------|-------------|-------------------|
| **Text Embedding / RAG** | Read files → split → embed → store in vector DB | Partially — can build flow, but very verbose |
| **Knowledge Graph** | Extract entities → store in graph DB | Partially — Neo4j/Kuzu targets exist |
| **Codebase Indexing** | Parse code → detect lang → chunk → embed | Partially — detect-lang + split exist |
| **Custom ETL** | Arbitrary source → transform → target | Yes, via custom ops, but boilerplate-heavy |
| **Live/Incremental Updates** | Watch for changes, re-process only deltas | Only with persisted flows (requires Postgres) |
| **LLM Extraction** | Use LLM to extract structured data | Yes, via `ExtractByLlm` |
| **Transient/One-shot** | Single-run processing without persistence | Yes, via `build_transient_flow()` |
| **Multi-codebase Summarization** | Process multiple repos, aggregate results | Not ergonomically — needs manual orchestration |

### Can Our Docs Examples Actually Work?

| Example | Works? | Issues |
|---------|--------|--------|
| `transient.rs` | Yes | Verbose but functional |
| `file_processing.rs` | Yes | Manual file I/O, KTable pattern matching |
| `custom_op.rs` | Yes | Heavy boilerplate (2 traits + registration) |
| `detect_lang.rs` | Yes | Pattern matching for results |

### Key Gaps

1. **No composable pipeline without FlowBuilder**: You can't just call `split(text)` — you must build a whole flow.
2. **No direct function invocation**: Every operation must go through the graph builder.
3. **No context/resource sharing**: No equivalent to `ContextKey` / `use_context`.
4. **No incremental without Postgres**: Persisted flows require external PostgreSQL.
5. **No `walk_dir` or file walking**: Must use `add_source("LocalFile", ...)` inside a flow.
6. **No streaming results**: Transient flows return a single `Value`, not a stream.
7. **No proc macros for custom ops**: All boilerplate is manual.

---

## 4. CocoIndex v1 Python API Reference

For context, this is what the upstream Python v1 API looks like:

```python
# 1. Define shared resources
@coco.lifespan
def coco_lifespan(builder: coco.EnvironmentBuilder):
    builder.provide(EMBEDDER, SentenceTransformerEmbedder("model"))
    yield

# 2. Define processing functions
@coco.function(memo=True)
async def process_file(file, table):
    text = file.read_text()
    chunks = splitter.split(text, chunk_size=1000)
    await coco_aio.map(process_chunk, chunks, table)

# 3. Define app entry point
@coco.function
async def app_main(sourcedir: pathlib.Path):
    db = coco.use_context(PG_DB)
    table = await db.mount_table_target("embeddings", schema)
    files = localfs.walk_dir(sourcedir, recursive=True)
    await coco_aio.mount_each(process_file, files.items(), table)

# 4. Create and run
app = coco.App(coco.AppConfig(name="MyApp"), app_main, sourcedir=Path("./data"))
app.update(report_to_stdout=True)
```

### Key Patterns to Translate to Rust

1. **`@coco.function(memo=True)`** → Memoized function with fingerprint-based caching
2. **`coco.App`** → Top-level execution unit
3. **`mount_each(fn, items)`** → Parallel component mounting
4. **`use_context(KEY)`** → Dependency injection
5. **`declare_row()`** → Target state declaration
6. **`walk_dir()`** → Source enumeration
7. **`@coco.lifespan`** → Resource lifecycle management

---

## 5. Proposed Recoco v1 Rust API

### Design Principles

1. **Explicit over implicit** — No hidden global state. Pass context explicitly.
2. **Trait-based, not string-based** — Type-safe operations via traits and generics.
3. **Ergonomic but honest** — Use builders and Into conversions where helpful, but don't hide Rust's ownership model.
4. **Incremental by default** — LMDB-backed state tracking is the core value proposition.
5. **Feature-gated everything** — Maintain the modular compilation approach.
6. **Proc macros for boilerplate** — `#[recoco::function]` to reduce custom op ceremony.

### 5.1 Core Types

#### Environment & App

```rust
use recoco::prelude::*;

// Environment manages LMDB state and shared resources
let env = Environment::open(EnvironmentSettings {
    db_path: PathBuf::from(".recoco"),
    ..Default::default()
})?;

// App is the top-level execution unit
let app = App::new("my_pipeline", &env)?;

// Run the pipeline
let handle = app.update(root_processor, AppUpdateOptions::default())?;
let stats = handle.await?;
```

#### Context & Resource Sharing

```rust
// Define typed context keys
static EMBEDDER: ContextKey<SentenceTransformerEmbedder> = ContextKey::new("embedder");
static PG_DB: ContextKey<PgDatabase> = ContextKey::new("pg_db");

// Provide resources during environment setup
env.provide(&EMBEDDER, SentenceTransformerEmbedder::new("model-name")?);
env.provide(&PG_DB, PgDatabase::connect("postgres://...").await?);

// Access resources in processing functions
fn process_item(ctx: &Ctx, text: &str) -> Result<()> {
    let embedder = ctx.use_resource(&EMBEDDER);
    let embedding = embedder.embed(text).await?;
    // ...
}
```

#### Ctx Methods

```rust
impl Ctx {
    /// Get a shared resource by typed key.
    pub fn use_resource<T: Send + Sync + 'static>(&self, key: &ContextKey<T>) -> &T;

    /// Create a named sub-scope. Scopes track what changed between runs.
    pub async fn scope<T>(&self, key: &impl Display, f: impl FnOnce(Ctx) -> Future<Output = Result<T>>) -> Result<T>;

    /// Memoized computation. Skips the closure if the key hasn't changed since last run.
    pub async fn memo<K: Serialize, T: Persist>(&self, key: &K, f: impl FnOnce() -> Future<Output = Result<T>>) -> Result<T>;

    /// Batch-process items with per-item memoization.
    /// Cache hits return stored values. Cache misses are collected and passed to `f`
    /// as a single batch. Results stored back and merged in original order.
    pub async fn batch<Item, K, T>(
        &self,
        items: impl IntoIterator<Item = Item>,
        key_fn: impl Fn(&Item) -> K,
        f: impl FnOnce(Vec<Item>) -> Future<Output = Result<Vec<T>>>,
    ) -> Result<Vec<T>>
    where
        K: Serialize,
        T: Persist;

    /// Run a processor concurrently for each item, creating a child scope per item.
    pub async fn mount_each<Item, K, T>(
        &self,
        items: impl IntoIterator<Item = Item>,
        key_fn: impl Fn(&Item) -> K,
        f: impl Fn(Ctx, Item) -> Future<Output = Result<T>>,
    ) -> Result<Vec<T>>
    where
        K: Serialize + Display;

    /// Run a closure concurrently for each item within the current scope (no child scopes).
    pub async fn map<Item, T>(
        &self,
        items: impl IntoIterator<Item = Item>,
        f: impl Fn(Item) -> Future<Output = Result<T>>,
    ) -> Result<Vec<T>>;
}
```

### 5.2 Processing Functions

#### The `#[recoco::function]` Proc Macro

A unified macro with parameters that control behavior. The framework applies features in a fixed ordering: **caller → memo → batching → runner → core_fn**.

```rust
/// Base: tracked for incremental updates, emits code hash constant
#[recoco::function]
async fn process_file(ctx: &Ctx, file: FileEntry, table: &TableTarget) -> Result<()> {
    let text = file.read_text().await?;
    let chunks = RecursiveSplitter::default()
        .split(&text, SplitOptions { chunk_size: 1000, chunk_overlap: 200, ..Default::default() });

    ctx.map(process_chunk, chunks.iter(), &file.path, table).await?;
    Ok(())
}
// Emits: pub const __RECOCO_FN_HASH_PROCESS_FILE: u64 = 0x...;

/// Memoized: skips re-execution when inputs unchanged.
/// All non-ctx parameters become the cache key. Code hash included automatically.
#[recoco::function(memo)]
async fn embed_text(ctx: &Ctx, text: &str) -> Result<Vec<f32>> {
    let embedder = ctx.use_resource(&EMBEDDER);
    embedder.embed(text).await
}

/// Batching without caching: first non-ctx param is items collection.
/// Body receives all items every time — no per-item cache probing.
#[recoco::function(batching)]
async fn embed_all(ctx: &Ctx, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    let embedder = ctx.use_resource(&EMBEDDER);
    embedder.embed_batch(&texts).await
}

/// Memo + batching: per-item memoization with batch processing.
/// Cache hits return stored values. Cache misses are collected and passed
/// to the body as a single batch. Code hash included in per-item keys.
#[recoco::function(memo, batching)]
async fn extract_files(ctx: &Ctx, files: Vec<FileEntry>) -> Result<Vec<Info>> {
    let client = ctx.use_resource(&LLM_CLIENT).clone();
    // `files` here is only cache misses — unchanged files are served from cache
    let mut results = Vec::with_capacity(files.len());
    for file in &files {
        results.push(client.analyze(&file.content_str()?).await?);
    }
    Ok(results)
}

/// Manual version bust when you need to invalidate cache without changing the body
#[recoco::function(memo, version = 2)]
async fn analyze(ctx: &Ctx, file: &FileEntry) -> Result<Info> { /* ... */ }
```

The macro generates:
- A `__RECOCO_FN_HASH_<NAME>` compile-time constant (hash of function body tokens)
- `ComponentProcessor` impl with fingerprint computation
- Memoization wrapping (when `memo`)
- Batch collection wrapping (when `batching`)
- Proper error propagation

**Expansion example for `#[recoco::function(memo, batching)]`:**

```rust
pub const __RECOCO_FN_HASH_EXTRACT_FILES: u64 = 0x...;

async fn extract_files(ctx: &Ctx, files: Vec<FileEntry>) -> Result<Vec<Info>> {
    ctx.batch(
        files,
        |item| (__RECOCO_FN_HASH_EXTRACT_FILES, item.clone()),
        move |files| async move { /* original body */ }
    ).await
}
```

Extra parameters beyond the items collection are cloned into the closure and included in per-item cache keys:

```rust
#[recoco::function(memo, batching)]
async fn extract(ctx: &Ctx, files: Vec<FileEntry>, model: String) -> Result<Vec<Info>> {
    // `model` is included in each item's key: (hash, &model, item.clone())
    // ...
}
```

#### Manual Implementation (Without Proc Macro)

For users who need full control:

```rust
struct ProcessFile;

impl ComponentProcessor<RecocoProfile> for ProcessFile {
    fn process(
        &self,
        ctx: &RuntimeCtx,
        comp_ctx: &ComponentProcessorContext<RecocoProfile>,
    ) -> Result<impl Future<Output = Result<FunctionData>> + Send + 'static> {
        // ... manual implementation
    }

    fn memo_key_fingerprint(&self) -> Option<Fingerprint> {
        None // or Some(fp) for memoization
    }

    fn processor_info(&self) -> &ComponentProcessorInfo {
        &ComponentProcessorInfo::new("ProcessFile".into())
    }
}
```

### 5.3 Sources

Sources become regular Rust functions/iterators rather than trait-based registries:

```rust
use recoco::sources::local_file;

// Walk a directory — returns an iterator of FileEntry
let files = local_file::walk_dir(
    "./data",
    WalkOptions {
        recursive: true,
        patterns: vec!["*.md", "*.txt"],
        exclude: vec![".*/**"],
        ..Default::default()
    },
)?;

// Process files with component mounting (parallel, incremental)
ctx.mount_each(process_file, files, &target_table).await?;
```

```rust
use recoco::sources::postgres as pg_source;

// Read from Postgres
let rows = pg_source::query(
    &db,
    "SELECT id, content FROM documents",
    QueryOptions::default(),
).await?;

ctx.mount_each(process_row, rows, &target_table).await?;
```

### 5.4 Functions / Transforms

Functions become regular Rust types with direct invocation:

```rust
use recoco::functions::{RecursiveSplitter, SplitOptions, Chunk};

// Direct invocation — no flow builder needed
let splitter = RecursiveSplitter::default();
let chunks: Vec<Chunk> = splitter.split(&text, SplitOptions {
    language: Some(Language::Markdown),
    chunk_size: 2000,
    chunk_overlap: 500,
    ..Default::default()
});

// Embedding
use recoco::functions::SentenceTransformerEmbed;

let embedder = SentenceTransformerEmbed::new("sentence-transformers/all-MiniLM-L6-v2")?;
let embedding: Vec<f32> = embedder.embed(&text).await?;

// LLM Extraction
use recoco::functions::ExtractByLlm;

#[derive(Deserialize)]
struct DocInfo {
    title: String,
    topics: Vec<String>,
}

let extractor = ExtractByLlm::<DocInfo>::new(LlmConfig { model: "gpt-4".into(), ..Default::default() })?;
let info: DocInfo = extractor.extract(&text).await?;
```

### 5.5 Targets

Targets use a declare-what-you-want pattern:

```rust
use recoco::targets::postgres::{PgTarget, TableSchema};

// Mount a table target
let table = PgTarget::mount_table(
    &db,
    "doc_embeddings",
    TableSchema::builder()
        .field("id", FieldType::Uuid)
        .field("filename", FieldType::Text)
        .field("text", FieldType::Text)
        .field("embedding", FieldType::Vector(384))
        .primary_key(&["id"])
        .vector_index("embedding", VectorSimilarityMetric::Cosine)
        .build()?,
).await?;

// Declare rows (engine handles insert/update/delete)
table.declare_row(DocEmbedding {
    id: generate_id(&chunk.text),
    filename: filename.to_string(),
    text: chunk.text.clone(),
    embedding,
})?;
```

```rust
use recoco::targets::qdrant::{QdrantTarget, PointStruct};

let collection = QdrantTarget::mount_collection(
    &client,
    "my_collection",
    CollectionConfig { vector_size: 384, ..Default::default() },
).await?;

collection.declare_point(PointStruct {
    id: point_id,
    vector: embedding,
    payload: json!({ "text": chunk.text }),
})?;
```

### 5.6 Complete Example: Text Embedding Pipeline

```rust
use recoco::prelude::*;
use recoco::sources::local_file::{self, FileEntry, WalkOptions};
use recoco::functions::{RecursiveSplitter, SplitOptions};
use recoco::targets::postgres::{PgTarget, TableSchema};
use serde::{Serialize, Deserialize};

// Typed context keys for shared resources
static EMBEDDER: ContextKey<MyEmbedder> = ContextKey::new("embedder");
static PG_DB: ContextKey<PgDatabase> = ContextKey::new("pg_db");

#[derive(Serialize, Deserialize)]
struct DocEmbedding {
    id: u64,
    filename: String,
    text: String,
    embedding: Vec<f32>,
}

#[recoco::function(memo)]
async fn process_file(ctx: &Ctx, file: FileEntry, table: &TableTarget) -> Result<()> {
    let text = file.read_text().await?;
    let splitter = RecursiveSplitter::default();
    let chunks = splitter.split(&text, SplitOptions {
        language: Some(Language::Markdown),
        chunk_size: 2000,
        chunk_overlap: 500,
        ..Default::default()
    });

    let embedder = ctx.use_resource(&EMBEDDER);
    let mut id_gen = IdGenerator::new();

    for chunk in &chunks {
        let embedding = embedder.embed(&chunk.text).await?;
        table.declare_row(DocEmbedding {
            id: id_gen.next_id(&chunk.text).await?,
            filename: file.path().to_string(),
            text: chunk.text.clone(),
            embedding,
        })?;
    }
    Ok(())
}

#[recoco::function]
async fn pipeline_main(ctx: &Ctx, source_dir: PathBuf) -> Result<()> {
    let db = ctx.use_resource(&PG_DB);
    let table = PgTarget::mount_table(
        db,
        "doc_embeddings",
        TableSchema::builder()
            .field("id", FieldType::Int64)
            .field("filename", FieldType::Text)
            .field("text", FieldType::Text)
            .field("embedding", FieldType::Vector(384))
            .primary_key(&["id"])
            .vector_index("embedding", VectorSimilarityMetric::Cosine)
            .build()?,
    ).await?;

    let files = local_file::walk_dir(
        &source_dir,
        WalkOptions { recursive: true, patterns: vec!["*.md"], ..Default::default() },
    )?;

    ctx.mount_each(process_file, files, &table).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup environment with LMDB state tracking
    let env = Environment::open(EnvironmentSettings {
        db_path: ".recoco".into(),
        ..Default::default()
    })?;

    // Provide shared resources
    env.provide(&EMBEDDER, MyEmbedder::new("model-name")?);
    env.provide(&PG_DB, PgDatabase::connect("postgres://localhost/mydb").await?);

    // Create and run app
    let app = App::new("text_embedding", &env)?;
    let handle = app.update(
        pipeline_main.with_args(PathBuf::from("./markdown_files")),
        AppUpdateOptions { report_to_stdout: true, ..Default::default() },
    )?;

    handle.await?;
    Ok(())
}
```

### 5.7 Complete Example: Custom Operation (Before/After)

**Before (current main API):**

```rust
// ~60 lines for a simple string reverse
pub struct ReverseStringExecutor;
#[async_trait]
impl SimpleFunctionExecutor for ReverseStringExecutor {
    async fn evaluate(&self, input: Vec<value::Value>) -> Result<value::Value> {
        let s = input[0].as_str()?;
        Ok(value::Value::Basic(value::BasicValue::Str(s.chars().rev().collect::<String>().into())))
    }
}

pub struct ReverseStringFactory;
#[derive(Deserialize, Serialize)]
pub struct EmptySpec {}
#[async_trait]
impl SimpleFunctionFactoryBase for ReverseStringFactory {
    type Spec = EmptySpec;
    type ResolvedArgs = ();
    fn name(&self) -> &str { "ReverseString" }
    async fn analyze<'a>(&'a self, spec: &'a Self::Spec, args_resolver: &mut OpArgsResolver<'a>, context: &FlowInstanceContext) -> Result<SimpleFunctionAnalysisOutput<Self::ResolvedArgs>> {
        // ... 15 more lines
    }
    async fn build_executor(self: Arc<Self>, spec: Self::Spec, resolved_args: Self::ResolvedArgs, context: Arc<FlowInstanceContext>) -> Result<impl SimpleFunctionExecutor> {
        Ok(ReverseStringExecutor)
    }
}

// Registration
recoco::ops::register_factory("ReverseString".to_string(),
    ExecutorFactory::SimpleFunction(Arc::new(ReverseStringFactory)))?;
```

**After (proposed v1 API):**

```rust
// ~10 lines for the same operation
#[recoco::function(memo)]
fn reverse_string(_ctx: &Ctx, text: &str) -> Result<String> {
    Ok(text.chars().rev().collect())
}

// Usage — direct call, no registration needed
let reversed = reverse_string(&ctx, "hello").await?;
```

### 5.8 Transient Mode (No Persistence)

For one-shot processing without LMDB state:

```rust
use recoco::functions::{RecursiveSplitter, SplitOptions};

// Direct function calls — no App/Environment needed
let splitter = RecursiveSplitter::default();
let chunks = splitter.split(&text, SplitOptions::default());

// Or for more complex pipelines without persistence:
let result = recoco::eval(|ctx| async move {
    let text = "Hello, world!";
    let chunks = RecursiveSplitter::default().split(text, SplitOptions::default());
    Ok(chunks)
}).await?;
```

### 5.9 Module Organization

```
recoco (umbrella crate)
├── recoco-core (engine)
│   ├── engine/
│   │   ├── app.rs         — App, AppConfig, AppUpdateOptions, UpdateHandle
│   │   ├── component.rs   — Component, ComponentProcessor trait
│   │   ├── context.rs     — Ctx, ContextKey, use_resource()
│   │   ├── environment.rs — Environment, EnvironmentSettings
│   │   ├── function.rs    — Memoization, FnCallMemoGuard
│   │   ├── mount.rs       — mount(), mount_each(), use_mount()
│   │   ├── profile.rs     — RecocoProfile (concrete EngineProfile)
│   │   └── ...
│   ├── state/             — LMDB persistence, stable paths
│   └── inspect/           — Database inspection tools
│
├── recoco-macros (proc macros)
│   └── function.rs        — #[recoco::function], #[recoco::function(memo=true)]
│
├── recoco-sources (feature-gated)
│   ├── local_file.rs      — walk_dir(), FileEntry
│   ├── postgres.rs        — query(), PostgresSource
│   ├── s3.rs              — S3Source
│   ├── azure.rs           — AzureBlobSource
│   └── gdrive.rs          — GDriveSource
│
├── recoco-functions (feature-gated)
│   ├── split.rs           — RecursiveSplitter, SplitBySeparators
│   ├── embed.rs           — SentenceTransformerEmbed
│   ├── extract_llm.rs     — ExtractByLlm
│   ├── json.rs            — ParseJson
│   └── detect_lang.rs     — detect_programming_language()
│
├── recoco-targets (feature-gated)
│   ├── postgres.rs        — PgTarget, TableSchema, declare_row()
│   ├── qdrant.rs          — QdrantTarget, declare_point()
│   ├── neo4j.rs           — Neo4jTarget
│   ├── kuzu.rs            — KuzuTarget
│   └── local_fs.rs        — declare_file()
│
├── recoco-utils            — Fingerprinting, error handling, batching
└── recoco-splitters        — Text splitting algorithms
```

### 5.10 The Concrete `RecocoProfile`

The v1.0.0 engine is generic over `EngineProfile`. We need a concrete implementation:

```rust
/// The concrete engine profile for pure-Rust Recoco.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct RecocoProfile;

impl EngineProfile for RecocoProfile {
    /// Shared resources (embedders, DB connections, etc.)
    type HostRuntimeCtx = ResourceRegistry;

    /// User's processing function
    type ComponentProc = Box<dyn DynComponentProcessor>;

    /// Serializable return value from functions
    type FunctionData = FunctionOutput;

    /// Target reconciliation handler
    type TargetHdl = Box<dyn DynTargetHandler>;

    /// Tracking record stored in LMDB
    type TargetStateTrackingRecord = TrackingRecord;

    /// Action to apply to a target
    type TargetAction = TargetMutation;

    /// Batched action sink
    type TargetActionSink = Box<dyn DynTargetActionSink>;

    /// Desired target state value
    type TargetStateValue = TargetValue;
}
```

### 5.11 API Design Decisions

#### Why Not Just Wrap the FlowBuilder?

The FlowBuilder pattern from main is fundamentally a **declarative graph builder**. The v1 engine is **imperative with component tracking**. Trying to bridge them would:
- Add unnecessary indirection
- Lose the simplicity of direct function calls
- Make memoization awkward (you'd need to express it in the graph)
- Prevent natural Rust control flow (if/else, loops, match)

#### Why Proc Macros?

Without proc macros, implementing `ComponentProcessor` requires ~30 lines of boilerplate per function. The `#[recoco::function]` macro:
- Generates the `ComponentProcessor` impl
- Computes fingerprints from function arguments
- Handles `memo` for memoization
- Wraps return values in `FunctionData`
- Is **optional** — manual impl always available

#### Why `ContextKey` Instead of Global State?

- Type-safe resource access
- No hidden globals
- Testable — swap resources in tests
- Composable — different apps can have different resources

#### Why Direct Function Calls for Operations?

The current API forces everything through the FlowBuilder graph. In v1:
- `RecursiveSplitter::split()` is a direct call — use it anywhere
- `embedder.embed()` is a direct call — compose naturally
- Operations are just Rust structs with methods — no registry needed
- The engine handles memoization at the component level, not the operation level

### 5.12 Feature Flags

```toml
[features]
default = ["engine"]

# Core engine (LMDB, component tracking, memoization)
engine = ["dep:heed", "dep:tokio", ...]

# Proc macros
macros = ["dep:recoco-macros"]

# Sources
source-local-file = ["dep:walkdir", "dep:notify"]
source-postgres = ["dep:sqlx"]
source-s3 = ["dep:aws-sdk-s3"]
source-azure = ["dep:azure_storage_blobs"]
source-gdrive = ["dep:google-drive3"]

# Functions
function-split = ["dep:recoco-splitters"]
function-embed = ["dep:ort", "dep:tokenizers"]
function-extract-llm = ["dep:reqwest"]
function-json = ["dep:serde_json"]
function-detect-lang = ["dep:recoco-splitters"]

# Targets
target-postgres = ["dep:sqlx"]
target-qdrant = ["dep:qdrant-client"]
target-neo4j = ["dep:neo4rs"]
target-kuzu = ["dep:kuzu"]
target-local-fs = []

# Bundles
all-sources = ["source-local-file", "source-postgres", "source-s3", "source-azure", "source-gdrive"]
all-functions = ["function-split", "function-embed", "function-extract-llm", "function-json", "function-detect-lang"]
all-targets = ["target-postgres", "target-qdrant", "target-neo4j", "target-kuzu", "target-local-fs"]
full = ["engine", "macros", "all-sources", "all-functions", "all-targets"]
```

---

## 6. Migration Path

### Phase 1: Concrete Profile & Core API

1. Implement `RecocoProfile` with concrete Rust types
2. Build `Ctx` (context) type with `use_resource()` and `ContextKey`
3. Build `Environment` convenience wrapper around raw engine
4. Build `App` convenience wrapper
5. Implement `mount()`, `mount_each()`, `use_mount()` helpers

### Phase 2: Proc Macros

1. Create `recoco-macros` crate
2. Implement `#[recoco::function]` — generates `ComponentProcessor` impl
3. Implement `#[recoco::function(memo)]` — adds fingerprint computation
4. Handle argument extraction and serialization

### Phase 3: Port Operations

1. Port sources as standalone functions/types (not registry-based)
   - `local_file::walk_dir()` → returns `impl Iterator<Item = FileEntry>`
   - `postgres::query()` → returns rows
2. Port functions as standalone types with direct methods
   - `RecursiveSplitter::split()` → `Vec<Chunk>`
   - `SentenceTransformerEmbed::embed()` → `Vec<f32>`
3. Port targets as mount-and-declare types
   - `PgTarget::mount_table()` → `TableTarget`
   - `table.declare_row()` → engine handles reconciliation

### Phase 4: Transient/Direct Mode

1. `recoco::eval()` for one-shot pipelines without LMDB
2. Direct function calls without any engine context
3. Maintain backward compatibility with standalone usage of functions

### Phase 5: Query Handlers & Graph Mappings

1. Query handler registration on apps (search endpoints)
2. Graph DB mapping types for Neo4j/Kuzu (Node, Relationship, NodeFromFields)
3. `transform_flow` equivalent — reusable transforms shared between indexing and querying

### Phase 6: FlowBuilder Compatibility Layer (Optional)

If needed for gradual migration:
1. Thin `FlowBuilder` that translates to component mounting under the hood
2. `evaluate_transient_flow()` that uses `recoco::eval()`

---

## Appendix A: Comparison with Issue #1667 (Approved Revision)

Upstream [issue #1667](https://github.com/cocoindex-io/cocoindex/issues/1667) proposes an ergonomic Rust SDK. The original proposal used `#[cocoindex::cached]` and `#[cocoindex::component]` macros. After feedback from **georgeh0** (CocoIndex team member), the proposal was **revised to a unified `#[cocoindex::function]` macro with parameters** — and that revision was approved.

### What georgeh0 Asked For

1. **Unified macro**: `#[cocoindex::function]` with parameters (`memo`, `batching`) instead of separate macros, so the framework controls application ordering (caller → memo → batching → runner → core_fn)
2. **Caller-side component scoping**: Components are decided by the caller (via `ctx.scope()`, `ctx.mount_each()`), not decorated on the function definition

### Approved Upstream Design (Revised #1667)

| Aspect | Approved #1667 | Our Proposal | Difference |
|--------|---------------|--------------|------------|
| Entry point | `App::builder("name").db_path().provide().build()` | `Environment::open()` + `App::new("name", &env)` | We separate env from app — more flexible for multi-app |
| Context | `&Ctx` parameter | `&Ctx` parameter | Same |
| Base function | `#[cocoindex::function]` — tracked, code hash emitted | `#[recoco::function]` — tracked | **We adopt code hashing** |
| Memoized | `#[cocoindex::function(memo)]` | `#[recoco::function(memo)]` | Same |
| Batching | `#[cocoindex::function(batching)]` | `#[recoco::function(batching)]` | **We adopt this** |
| Memo+batch | `#[cocoindex::function(memo, batching)]` | `#[recoco::function(memo, batching)]` | **We adopt this** |
| Code hash | `__COCO_FN_HASH_<NAME>` compile-time constant | `__RECOCO_FN_HASH_<NAME>` | **We adopt this pattern** |
| Version bust | `#[cocoindex::function(memo, version = 2)]` | `#[recoco::function(memo, version = 2)]` | **We adopt this** |
| Resource access | `ctx.get::<T>()` — type-based lookup | `ctx.use_resource(&KEY)` — key-based | **We diverge** (see rationale below) |
| Scoping | `ctx.scope(key, \|ctx\| ...)` | `ctx.mount(key, processor)` | Equivalent |
| Parallel items | `ctx.mount_each(items, key_fn, f)` | `ctx.mount_each(fn, items, ...)` | Same concept, signature differs |
| Batch ctx method | `ctx.batch(items, key_fn, f)` | `ctx.batch(items, key_fn, f)` | **We adopt this** |
| File walking | `cocoindex::fs::walk(&dir, &patterns)` | `local_file::walk_dir(path, opts)` | Both standalone |
| File output | `ctx.write_file(path, content)` | `table.declare_row()`, `declare_file()` | We use target abstraction |
| Stats | `RunStats { processed, skipped, written, deleted, elapsed }` | `UpdateStats` (from engine) | Similar |

### Key Design Decisions Where We Diverge

**Resource access: `ContextKey` vs `ctx.get::<T>()`**

The approved #1667 uses type-based lookup (`ctx.get::<T>()`) which panics if not provided. We use `ContextKey<T>` because:
- Multiple instances of the same type are common (e.g., two `PgDatabase` connections)
- Keys are named and discoverable in code
- Compile-time checking — `ContextKey` is a static, not a runtime type query
- Testable — swap resources by providing different values for the same key

**Targets: `ctx.write_file()` vs `declare_row()`/`declare_file()`**

The approved #1667 only shows `ctx.write_file()` for output. We use the richer target abstraction because:
- Vector DBs, graph DBs, and relational DBs need structured declaration
- The engine handles insert/update/delete reconciliation
- `declare_row()` pattern integrates with LMDB state tracking

**Entry point: `App::builder()` vs `Environment` + `App`**

The approved #1667 bundles everything into `App::builder()`. We separate `Environment` from `App` because:
- Multiple apps can share one LMDB environment
- Resources are environment-scoped, not app-scoped
- Aligns with the upstream engine's actual architecture (`Environment<Prof>` + `App<Prof>`)

### Upstream Priority Assessment

- **Issue #1372**: badmonster0 (CocoIndex founder) confirmed "we do plan to support Rust natively"
- **Issue #1667**: georgeh0 (team member) approved the revised API design
- **Reality**: CocoIndex is not published to crates.io, no Rust SDK work has started, and the team appears focused on the Python SDK. Recoco has an opportunity to ship this first.

## Appendix B: Code Hash Constant Design

The `#[recoco::function]` proc macro emits a compile-time constant for cache invalidation:

```rust
// User writes:
#[recoco::function]
async fn analyze(ctx: &Ctx, file: &FileEntry) -> Result<Info> {
    // body
}

// Macro emits:
pub const __RECOCO_FN_HASH_ANALYZE: u64 = 0x...; // hash of the function body tokens

async fn analyze(ctx: &Ctx, file: &FileEntry) -> Result<Info> {
    // body
}
```

The hash is computed from the token stream of the function body at compile time. When `memo` is enabled, this hash is automatically prepended to the cache key. For bare `#[recoco::function]`, the constant is available for manual inclusion in `ctx.memo()` keys:

```rust
#[recoco::function]
async fn analyze(ctx: &Ctx, file: &FileEntry) -> Result<Info> {
    let client = ctx.use_resource(&LLM_CLIENT).clone();
    ctx.memo(&(__RECOCO_FN_HASH_ANALYZE, file.fingerprint()), move || async move {
        client.call(&file.content_str()?).await
    }).await
}
```

This gives users the best of both worlds:
- **`#[recoco::function(memo)]`**: Fully automatic — all args become the key, code hash included
- **`#[recoco::function]` + manual `ctx.memo()`**: Selective — user controls what's in the key, but gets automatic invalidation when the function body changes
