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

use std::any::Any;

use futures::future::BoxFuture;
use sqlx::PgConnection;

use recoco_utils::batching::{Batcher, BatchingOptions, Runner};

use crate::prelude::*;

/// Type-erased body for a PostgreSQL write transaction.
///
/// Runs inside a shared `Transaction`, returns a boxed output value.
type TxnBody = Box<
    dyn for<'t> FnOnce(&'t mut PgConnection) -> BoxFuture<'t, Result<Box<dyn Any + Send>>> + Send,
>;

struct PgTxnRunner {
    pool: sqlx::PgPool,
}

#[async_trait]
impl Runner for PgTxnRunner {
    type Input = TxnBody;
    type Output = Box<dyn Any + Send>;

    async fn run(
        &self,
        inputs: Vec<TxnBody>,
    ) -> Result<impl ExactSizeIterator<Item = Box<dyn Any + Send>>> {
        let mut outputs = Vec::with_capacity(inputs.len());
        let mut txn = self.pool.begin().await?;
        for body in inputs {
            outputs.push(body(&mut *txn).await?);
        }
        txn.commit().await?;
        Ok(outputs.into_iter())
    }
}

/// Batches PostgreSQL write transactions: multiple callers' closures run
/// sequentially inside a single `BEGIN` → `COMMIT` cycle.
///
/// Leverages [`Batcher`] for FIFO scheduling: the first caller executes
/// immediately (inline), while concurrent callers queue up and are flushed
/// together once the current batch commits.
///
/// If any closure in a batch returns `Err`, the whole batch is rolled back
/// (the `Transaction` is dropped without committing), and every caller in the
/// batch receives an error.
pub struct PgTxnBatcher {
    inner: Batcher<PgTxnRunner>,
}

impl PgTxnBatcher {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            inner: Batcher::new(PgTxnRunner { pool }, BatchingOptions::default()),
        }
    }

    /// Run `body` inside a batched write transaction.
    ///
    /// The body receives an exclusive `&mut PgConnection` shared with other
    /// concurrent callers. If the body returns `Ok(value)`, `value` is
    /// returned once the transaction commits. If it returns `Err`, the
    /// transaction is aborted and every caller in the current batch receives
    /// an error.
    pub async fn run<T: Send + 'static>(
        &self,
        body: impl for<'t> FnOnce(&'t mut PgConnection) -> BoxFuture<'t, Result<T>> + Send + 'static,
    ) -> Result<T> {
        let output = self
            .inner
            .run(Box::new(
                move |conn: &'_ mut PgConnection| -> BoxFuture<'_, Result<Box<dyn Any + Send>>> {
                    let fut = body(conn);
                    Box::pin(async move {
                        let result = fut.await?;
                        Ok(Box::new(result) as Box<dyn Any + Send>)
                    })
                },
            ))
            .await?;
        output
            .downcast::<T>()
            .map(|b| *b)
            .map_err(|_| internal_error!("PgTxnBatcher: output type mismatch"))
    }
}
