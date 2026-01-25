// ReCoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex)
// Original code from CocoIndex is copyrighted by CocoIndex
// SPDX-FileCopyrightText: 2025-2026 CocoIndex (upstream)
// SPDX-FileContributor: CocoIndex Contributors
//
// All modifications from the upstream for ReCoco are copyrighted by Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// Both the upstream CocoIndex code and the ReCoco modifications are licensed under the Apache-2.0 License.
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(
    feature = "function-detect-lang",
    feature = "function-embed",
    feature = "function-extract-llm",
    feature = "function-json",
    feature = "function-split"
))]
use super::functions;
#[cfg(any(
    feature = "target-kuzu",
    feature = "target-neo4j",
    feature = "target-postgres",
    feature = "target-qdrant"
))]
use super::targets;
#[cfg(any(
    feature = "source-azure",
    feature = "source-gdrive",
    feature = "source-local-file",
    feature = "source-postgres",
    feature = "source-s3"
))]
use super::sources;
use super::{factory_bases::*, registry::ExecutorFactoryRegistry, sdk::ExecutorFactory};
use crate::prelude::*;
use recoco_utils::client_error;
use std::sync::{LazyLock, RwLock};

fn register_executor_factories(registry: &mut ExecutorFactoryRegistry) -> Result<()> {
    #[cfg(feature = "target-kuzu")]
    let reqwest_client = reqwest::Client::new();

    #[cfg(feature = "source-local-file")]
    sources::local_file::Factory.register(registry)?;
    #[cfg(feature = "source-gdrive")]
    sources::google_drive::Factory.register(registry)?;
    #[cfg(feature = "source-s3")]
    sources::amazon_s3::Factory.register(registry)?;
    #[cfg(feature = "source-azure")]
    sources::azure_blob::Factory.register(registry)?;
    #[cfg(feature = "source-postgres")]
    sources::postgres::Factory.register(registry)?;

    #[cfg(feature = "function-detect-lang")]
    functions::detect_program_lang::register(registry)?;
    #[cfg(feature = "function-embed")]
    functions::embed_text::register(registry)?;
    #[cfg(feature = "function-extract-llm")]
    functions::extract_by_llm::Factory.register(registry)?;
    #[cfg(feature = "function-json")]
    functions::parse_json::Factory.register(registry)?;
    #[cfg(feature = "function-split")]
    functions::split_by_separators::register(registry)?;
    #[cfg(feature = "function-split")]
    functions::split_recursively::register(registry)?;

    #[cfg(feature = "target-postgres")]
    targets::postgres::register(registry)?;
    #[cfg(feature = "target-qdrant")]
    targets::qdrant::register(registry)?;
    #[cfg(feature = "target-kuzu")]
    targets::kuzu::register(registry, reqwest_client)?;

    #[cfg(feature = "target-neo4j")]
    targets::neo4j::Factory::new().register(registry)?;

    Ok(())
}

static EXECUTOR_FACTORY_REGISTRY: LazyLock<RwLock<ExecutorFactoryRegistry>> = LazyLock::new(|| {
    let mut registry = ExecutorFactoryRegistry::new();
    register_executor_factories(&mut registry).expect("Failed to register executor factories");
    RwLock::new(registry)
});

pub fn get_optional_source_factory(
    kind: &str,
) -> Option<std::sync::Arc<dyn super::interface::SourceFactory + Send + Sync>> {
    let registry = EXECUTOR_FACTORY_REGISTRY.read().unwrap();
    registry.get_source(kind).cloned()
}

pub fn get_optional_function_factory(
    kind: &str,
) -> Option<std::sync::Arc<dyn super::interface::SimpleFunctionFactory + Send + Sync>> {
    let registry = EXECUTOR_FACTORY_REGISTRY.read().unwrap();
    registry.get_function(kind).cloned()
}

pub fn get_optional_target_factory(
    kind: &str,
) -> Option<std::sync::Arc<dyn super::interface::TargetFactory + Send + Sync>> {
    let registry = EXECUTOR_FACTORY_REGISTRY.read().unwrap();
    registry.get_target(kind).cloned()
}

pub fn get_optional_attachment_factory(
    kind: &str,
) -> Option<std::sync::Arc<dyn super::interface::TargetAttachmentFactory + Send + Sync>> {
    let registry = EXECUTOR_FACTORY_REGISTRY.read().unwrap();
    registry.get_target_attachment(kind).cloned()
}

pub fn get_source_factory(
    kind: &str,
) -> Result<std::sync::Arc<dyn super::interface::SourceFactory + Send + Sync>> {
    get_optional_source_factory(kind)
        .ok_or_else(|| client_error!("Source factory not found for op kind: {}", kind))
}

pub fn get_function_factory(
    kind: &str,
) -> Result<std::sync::Arc<dyn super::interface::SimpleFunctionFactory + Send + Sync>> {
    get_optional_function_factory(kind)
        .ok_or_else(|| client_error!("Function factory not found for op kind: {}", kind))
}

pub fn get_target_factory(
    kind: &str,
) -> Result<std::sync::Arc<dyn super::interface::TargetFactory + Send + Sync>> {
    get_optional_target_factory(kind)
        .ok_or_else(|| client_error!("Target factory not found for op kind: {}", kind))
}

pub fn get_attachment_factory(
    kind: &str,
) -> Result<std::sync::Arc<dyn super::interface::TargetAttachmentFactory + Send + Sync>> {
    get_optional_attachment_factory(kind)
        .ok_or_else(|| client_error!("Attachment factory not found for op kind: {}", kind))
}

pub fn register_factory(name: String, factory: ExecutorFactory) -> Result<()> {
    let mut registry = EXECUTOR_FACTORY_REGISTRY.write().unwrap();
    registry.register(name, factory)
}
