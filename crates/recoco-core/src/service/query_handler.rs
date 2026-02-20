// Recoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex)
// Original code from CocoIndex is copyrighted by CocoIndex
// SPDX-FileCopyrightText: 2025-2026 CocoIndex (upstream)
// SPDX-FileContributor: CocoIndex Contributors
//
// All modifications from the upstream for Recoco are copyrighted by Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. (Recoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// Both the upstream CocoIndex code and the Recoco modifications are licensed under the Apache-2.0 License.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    base::spec::{FieldName, VectorSimilarityMetric},
    prelude::*,
};

#[derive(Serialize, Deserialize, Default)]
pub struct QueryHandlerResultFields {
    embedding: Vec<String>,
    score: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct QueryHandlerSpec {
    #[serde(default)]
    result_fields: QueryHandlerResultFields,
}

#[derive(Serialize, Deserialize)]
pub struct QueryInput {
    pub query: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct QueryInfo {
    pub embedding: Option<serde_json::Value>,
    pub similarity_metric: Option<VectorSimilarityMetric>,
}

#[derive(Serialize, Deserialize)]
pub struct QueryOutput {
    pub results: Vec<Vec<(FieldName, serde_json::Value)>>,
    pub query_info: QueryInfo,
}

#[async_trait]
pub trait QueryHandler: Send + Sync {
    async fn query(
        &self,
        input: QueryInput,
        flow_ctx: &interface::FlowInstanceContext,
    ) -> Result<QueryOutput>;
}
