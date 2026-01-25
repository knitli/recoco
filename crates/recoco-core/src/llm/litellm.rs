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

use async_openai::Client as OpenAIClient;
use async_openai::config::OpenAIConfig;

pub use super::openai::Client;

impl Client {
    pub async fn new_litellm(
        address: Option<String>,
        api_key: Option<String>,
    ) -> anyhow::Result<Self> {
        let address = address.unwrap_or_else(|| "http://127.0.0.1:4000".to_string());

        let api_key = api_key.or_else(|| std::env::var("LITELLM_API_KEY").ok());

        let mut config = OpenAIConfig::new().with_api_base(address);
        if let Some(api_key) = api_key {
            config = config.with_api_key(api_key);
        }
        Ok(Client::from_parts(OpenAIClient::with_config(config)))
    }
}
