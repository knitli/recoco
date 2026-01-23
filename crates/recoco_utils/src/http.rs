// ReCoco is a Rust-only fork of CocoIndex, by [CocoIndex.io](https://cocoindex.io)
// Original code from CocoIndex is copyrighted by CocoIndex.io
// SPDX-FileCopyrightText: 2025-2026 CocoIndex.io (upstream)
// SPDX-FileContributor: CocoIndex Contributors
//
// All modifications from the upstream for ReCoco are copyrighted by Knitli Inc.
// SPDX-FileCopyrightText: 2026 Knitli Inc. (ReCoco)
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// Both the upstream CocoIndex code and the ReCoco modifications are licensed under the Apache-2.0 License.
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Result};
use crate::retryable::{self, IsRetryable};

pub async fn request(
    req_builder: impl Fn() -> reqwest::RequestBuilder,
) -> Result<reqwest::Response> {
    let resp = retryable::run(
        || async {
            let req = req_builder();
            let resp = req.send().await?;
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
