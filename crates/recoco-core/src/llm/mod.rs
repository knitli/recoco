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

use crate::prelude::*;

#[cfg(feature = "json-schema")]
use crate::base::json_schema::ToJsonSchemaOptions;

use infer::Infer;
#[cfg(feature = "json-schema")]
use schemars::Schema;
use std::borrow::Cow;

static INFER: LazyLock<Infer> = LazyLock::new(Infer::new);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LlmApiType {
    #[cfg(feature = "provider-ollama")]
    Ollama,
    #[cfg(feature = "provider-openai")]
    OpenAi,
    #[cfg(feature = "provider-gemini")]
    Gemini,
    #[cfg(feature = "provider-anthropic")]
    Anthropic,
    #[cfg(feature = "provider-openai")]
    LiteLlm,
    #[cfg(feature = "provider-openai")]
    OpenRouter,
    #[cfg(feature = "provider-voyage")]
    Voyage,
    #[cfg(feature = "provider-openai")]
    Vllm,
    #[cfg(feature = "provider-gemini")]
    VertexAi,
    #[cfg(feature = "provider-bedrock")]
    Bedrock,
    #[cfg(feature = "provider-azure")]
    AzureOpenAi,
}

#[cfg(feature = "provider-gemini")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAiConfig {
    pub project: String,
    pub region: Option<String>,
}

#[cfg(feature = "provider-openai")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiConfig {
    pub org_id: Option<String>,
    pub project_id: Option<String>,
}

#[cfg(feature = "provider-openai")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureOpenAiConfig {
    pub deployment_id: String,
    pub api_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum LlmApiConfig {
    #[cfg(feature = "provider-gemini")]
    VertexAi(VertexAiConfig),
    #[cfg(feature = "provider-openai")]
    OpenAi(OpenAiConfig),
    #[cfg(feature = "provider-azure")]
    AzureOpenAi(AzureOpenAiConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSpec {
    pub api_type: LlmApiType,
    pub address: Option<String>,
    pub model: String,
    pub api_key: Option<spec::AuthEntryReference<String>>,
    pub api_config: Option<LlmApiConfig>,
}

#[derive(Debug)]
pub enum OutputFormat<'a> {
    #[cfg(feature = "json-schema")]
    JsonSchema {
        name: Cow<'a, str>,
        schema: Cow<'a, Schema>,
    },
}

#[derive(Debug)]
pub struct LlmGenerateRequest<'a> {
    pub model: &'a str,
    pub system_prompt: Option<Cow<'a, str>>,
    pub user_prompt: Cow<'a, str>,
    pub image: Option<Cow<'a, [u8]>>,
    pub output_format: Option<OutputFormat<'a>>,
}

#[derive(Debug)]
pub enum GeneratedOutput {
    Json(serde_json::Value),
    Text(String),
}

#[derive(Debug)]
pub struct LlmGenerateResponse {
    pub output: GeneratedOutput,
}

#[async_trait]
pub trait LlmGenerationClient: Send + Sync {
    async fn generate<'req>(
        &self,
        request: LlmGenerateRequest<'req>,
    ) -> Result<LlmGenerateResponse>;

    #[cfg(feature = "json-schema")]
    fn json_schema_options(&self) -> ToJsonSchemaOptions;
}

#[derive(Debug)]
pub struct LlmEmbeddingRequest<'a> {
    pub model: &'a str,
    pub texts: Vec<Cow<'a, str>>,
    pub output_dimension: Option<u32>,
    pub task_type: Option<Cow<'a, str>>,
}

pub struct LlmEmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}

#[async_trait]
pub trait LlmEmbeddingClient: Send + Sync {
    async fn embed_text<'req>(
        &self,
        request: LlmEmbeddingRequest<'req>,
    ) -> Result<LlmEmbeddingResponse>;

    fn get_default_embedding_dimension(&self, model: &str) -> Option<u32>;

    fn behavior_version(&self) -> Option<u32> {
        Some(1)
    }
}

#[cfg(feature = "provider-anthropic")]
mod anthropic;
#[cfg(feature = "provider-bedrock")]
mod bedrock;
#[cfg(feature = "provider-gemini")]
mod gemini;
#[cfg(feature = "provider-openai")]
mod litellm;
#[cfg(feature = "provider-ollama")]
mod ollama;
#[cfg(any(feature = "provider-openai", feature = "provider-azure"))]
mod openai;
#[cfg(feature = "provider-openai")]
mod openrouter;
#[cfg(feature = "provider-openai")]
mod vllm;
#[cfg(feature = "provider-voyage")]
mod voyage;

pub async fn new_llm_generation_client(
    api_type: LlmApiType,
    address: Option<String>,
    api_key: Option<String>,
    api_config: Option<LlmApiConfig>,
) -> Result<Box<dyn LlmGenerationClient>> {
    let client = match api_type {
        #[cfg(feature = "provider-ollama")]
        LlmApiType::Ollama => {
            Box::new(ollama::Client::new(address).await?) as Box<dyn LlmGenerationClient>
        }
        #[cfg(feature = "provider-openai")]
        LlmApiType::OpenAi => Box::new(openai::Client::new(address, api_key, api_config)?)
            as Box<dyn LlmGenerationClient>,
        #[cfg(feature = "provider-gemini")]
        LlmApiType::Gemini => {
            Box::new(gemini::AiStudioClient::new(address, api_key)?) as Box<dyn LlmGenerationClient>
        }
        #[cfg(feature = "provider-gemini")]
        LlmApiType::VertexAi => {
            Box::new(gemini::VertexAiClient::new(address, api_key, api_config).await?)
                as Box<dyn LlmGenerationClient>
        }
        #[cfg(feature = "provider-anthropic")]
        LlmApiType::Anthropic => Box::new(anthropic::Client::new(address, api_key).await?)
            as Box<dyn LlmGenerationClient>,
        #[cfg(feature = "provider-bedrock")]
        LlmApiType::Bedrock => {
            Box::new(bedrock::Client::new(address).await?) as Box<dyn LlmGenerationClient>
        }
        #[cfg(feature = "provider-openai")]
        LlmApiType::LiteLlm => Box::new(litellm::Client::new_litellm(address, api_key).await?)
            as Box<dyn LlmGenerationClient>,
        #[cfg(feature = "provider-openai")]
        LlmApiType::OpenRouter => {
            Box::new(openrouter::Client::new_openrouter(address, api_key).await?)
                as Box<dyn LlmGenerationClient>
        }
        #[cfg(feature = "provider-azure")]
        LlmApiType::AzureOpenAi => {
            Box::new(openai::Client::new_azure(address, api_key, api_config).await?)
                as Box<dyn LlmGenerationClient>
        }
        #[cfg(feature = "provider-voyage")]
        LlmApiType::Voyage => {
            api_bail!("Voyage is not supported for generation")
        }
        #[cfg(feature = "provider-openai")]
        LlmApiType::Vllm => Box::new(vllm::Client::new_vllm(address, api_key).await?)
            as Box<dyn LlmGenerationClient>,
    };
    Ok(client)
}

pub async fn new_llm_embedding_client(
    api_type: LlmApiType,
    address: Option<String>,
    api_key: Option<String>,
    api_config: Option<LlmApiConfig>,
) -> Result<Box<dyn LlmEmbeddingClient>> {
    let client = match api_type {
        #[cfg(feature = "provider-ollama")]
        LlmApiType::Ollama => {
            Box::new(ollama::Client::new(address).await?) as Box<dyn LlmEmbeddingClient>
        }
        #[cfg(feature = "provider-openai")]
        LlmApiType::OpenRouter => {
            Box::new(openrouter::Client::new_openrouter(address, api_key).await?)
                as Box<dyn LlmEmbeddingClient>
        }
        #[cfg(feature = "provider-gemini")]
        LlmApiType::Gemini => {
            Box::new(gemini::AiStudioClient::new(address, api_key)?) as Box<dyn LlmEmbeddingClient>
        }
        #[cfg(feature = "provider-openai")]
        LlmApiType::OpenAi => Box::new(openai::Client::new(address, api_key, api_config)?)
            as Box<dyn LlmEmbeddingClient>,
        #[cfg(feature = "provider-voyage")]
        LlmApiType::Voyage => {
            Box::new(voyage::Client::new(address, api_key)?) as Box<dyn LlmEmbeddingClient>
        }
        #[cfg(feature = "provider-gemini")]
        LlmApiType::VertexAi => {
            Box::new(gemini::VertexAiClient::new(address, api_key, api_config).await?)
                as Box<dyn LlmEmbeddingClient>
        }
        #[cfg(feature = "provider-azure")]
        LlmApiType::AzureOpenAi => {
            Box::new(openai::Client::new_azure(address, api_key, api_config).await?)
                as Box<dyn LlmEmbeddingClient>
        }
        #[cfg(any(
            feature = "provider-openai",
            feature = "provider-anthropic",
            feature = "provider-bedrock"
        ))]
        LlmApiType::LiteLlm | LlmApiType::Vllm | LlmApiType::Anthropic | LlmApiType::Bedrock => {
            api_bail!("Embedding is not supported for API type {:?}", api_type)
        }
    };
    Ok(client)
}

pub fn detect_image_mime_type(bytes: &[u8]) -> Result<&'static str> {
    let infer = &*INFER;
    match infer.get(bytes) {
        Some(info) if info.mime_type().starts_with("image/") => Ok(info.mime_type()),
        _ => client_bail!("Unknown or unsupported image format"),
    }
}
