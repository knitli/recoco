// Recoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://cocoindex.io)
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

use crate::error::{Error, Result};
use crate::retryable::{self, IsRetryable};
use crate::slow_warn::warn_if_slow;

const SLOW_REQUEST_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(30);

pub async fn request(
    req_builder: impl Fn(&reqwest::Client) -> reqwest::RequestBuilder,
) -> Result<reqwest::Response> {
    let resp = retryable::run(
        || async {
            let client = reqwest::Client::new();
            let request = req_builder(&client).build()?;
            let url = request.url().clone();
            let resp = warn_if_slow(
                &|| format!("HTTP request to {url}"),
                SLOW_REQUEST_THRESHOLD,
                client.execute(request),
            )
            .await?;
            let Err(err) = resp.error_for_status_ref() else {
                return Ok(resp);
            };

            let is_retryable = err.is_retryable();

            let mut error: Error = err.into();
            let body = resp.text().await?;
            if !body.is_empty() {
                error = error.context(format!("Error message body:\n{body}"));
            }

            Err(retryable::Error {
                error,
                is_retryable,
            })
        },
        &retryable::HEAVY_LOADED_OPTIONS,
    )
    .await?;
    Ok(resp)
}
