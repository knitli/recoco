// Recoco is a Rust-only fork of CocoIndex, by [CocoIndex](https://CocoIndex.io)
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
use crate::engine::profile::EngineProfile;
use crate::engine::stats::{ProcessingStats, ProgressReporter, UpdateStats};
use crate::prelude::*;

use crate::engine::component::Component;
use crate::engine::context::AppContext;

use crate::engine::environment::{AppRegistration, Environment};
use crate::state::stable_path::StablePath;
use tokio::sync::watch;

/// Options for updating an app.
#[derive(Debug, Clone, Default)]
pub struct AppUpdateOptions {
    /// If true, periodically report processing stats to stdout.
    pub report_to_stdout: bool,
    /// If true, reprocess everything and invalidate existing caches.
    pub full_reprocess: bool,
}

/// Options for dropping an app.
#[derive(Debug, Clone, Default)]
pub struct AppDropOptions {
    pub report_to_stdout: bool,
}

/// A handle to a running or completed update operation.
///
/// `UpdateHandle<R>` provides:
/// - **Stats observation**: call [`stats`](UpdateHandle::stats) for a point-in-time snapshot,
///   or [`watch_stats`](UpdateHandle::watch_stats) to receive a [`watch::Receiver`] that is
///   notified on every stats change during the run.
/// - **Result retrieval**: call [`result`](UpdateHandle::result) (or simply `.await`) to wait
///   for the update to finish and obtain the return value.
///
/// # Backward compatibility
///
/// `UpdateHandle<R>` implements [`std::future::IntoFuture`], so existing code that does
/// `let result = handle.await?` continues to work. When using [`App::update`], which
/// now returns `Result<UpdateHandle<_>>`, callers should write
/// `let result = app.update(...)?.await?;`.
pub struct UpdateHandle<R> {
    stats_rx: watch::Receiver<UpdateStats>,
    join_handle: tokio::task::JoinHandle<Result<R>>,
}

impl<R: Send + 'static> UpdateHandle<R> {
    /// Returns a point-in-time snapshot of the current processing statistics.
    pub fn stats(&self) -> UpdateStats {
        self.stats_rx.borrow().clone()
    }

    /// Returns a cloned [`watch::Receiver`] that receives a new [`UpdateStats`] snapshot
    /// every time any component's stats change.
    ///
    /// The receiver always holds the most recent value; call
    /// [`watch::Receiver::changed`] to wait for the next update.
    pub fn watch_stats(&self) -> watch::Receiver<UpdateStats> {
        self.stats_rx.clone()
    }

    /// Await the completion of the update and return the result.
    ///
    /// Equivalent to `handle.await` (see [`IntoFuture`](std::future::IntoFuture) impl).
    ///
    /// # Errors
    ///
    /// Returns an error if the update operation itself fails, or if the spawned task panics
    /// (in which case the error wraps a [`tokio::task::JoinError`]).
    pub async fn result(self) -> Result<R> {
        self.join_handle.await?
    }
}

impl<R: Send + 'static> std::future::IntoFuture for UpdateHandle<R> {
    type Output = Result<R>;
    type IntoFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.result().await })
    }
}

pub struct App<Prof: EngineProfile> {
    root_component: Component<Prof>,
}

impl<Prof: EngineProfile> App<Prof> {
    pub fn new(
        name: &str,
        env: Environment<Prof>,
        max_inflight_components: Option<usize>,
    ) -> Result<Self> {
        let app_reg = AppRegistration::new(name, &env)?;

        // TODO: This database initialization logic should happen lazily on first call to `update()`.
        let db = {
            let mut wtxn = env.db_env().write_txn()?;
            let db = env.db_env().create_database(&mut wtxn, Some(name))?;
            wtxn.commit()?;
            db
        };

        let app_ctx = AppContext::new(env, db, app_reg, max_inflight_components);
        let root_component = Component::new(app_ctx, StablePath::root());
        Ok(Self { root_component })
    }
}

impl<Prof: EngineProfile> App<Prof> {
    /// Start an update and return an [`UpdateHandle`] for tracking progress and awaiting the result.
    ///
    /// The handle is also [`IntoFuture`](std::future::IntoFuture), so
    /// `let result = app.update(...)?.await?` is the correct calling convention.
    ///
    /// # Errors
    ///
    /// Returns an error if the processor context cannot be created (e.g. database initialization
    /// failure).
    #[instrument(name = "app.update", skip_all, fields(app_name = %self.app_ctx().app_reg().name()))]
    pub fn update(
        &self,
        root_processor: Prof::ComponentProc,
        options: AppUpdateOptions,
    ) -> Result<UpdateHandle<Prof::FunctionData>>
    where
        Prof::FunctionData: Send + 'static,
    {
        let processing_stats = ProcessingStats::default();
        let stats_rx = processing_stats.subscribe();

        let context = self.root_component.new_processor_context_for_build(
            None,
            processing_stats.clone(),
            options.full_reprocess,
        )?;

        let root_component = self.root_component.clone();
        let join_handle = tokio::task::spawn(async move {
            let run_fut = async move {
                root_component
                    .run(root_processor, context)
                    .await?
                    .result(None)
                    .await
            };

            let result = if options.report_to_stdout {
                let reporter = ProgressReporter::new(processing_stats.clone());
                reporter.run_with_progress(run_fut).await
            } else {
                run_fut.await
            };
            processing_stats.notify_terminated();
            result
        });

        Ok(UpdateHandle {
            stats_rx,
            join_handle,
        })
    }

    /// Drop the app, reverting all target states and clearing the database.
    ///
    /// This method:
    /// 1. Deletes the root component (which cascades to delete all child components and their target states)
    /// 2. Waits for deletion to complete
    /// 3. Clears the app's database
    #[instrument(name = "app.drop", skip_all, fields(app_name = %self.app_ctx().app_reg().name()))]
    pub async fn drop_app(&self, options: AppDropOptions) -> Result<()> {
        let processing_stats = ProcessingStats::default();
        let providers = self
            .app_ctx()
            .env()
            .target_states_providers()
            .lock()
            .unwrap()
            .providers
            .clone();

        let context = self.root_component.new_processor_context_for_delete(
            providers,
            None,
            processing_stats.clone(),
        );

        let drop_fut = async {
            // Delete the root component
            let handle = self.root_component.clone().delete(context.clone())?;

            // Wait for the drop operation to complete
            handle.ready().await?;

            // Clear the database
            let db = *self.app_ctx().db();
            self.app_ctx()
                .env()
                .txn_batcher()
                .run(move |wtxn| Ok(db.clear(wtxn)?))
                .await?;

            info!("App dropped successfully");
            Ok(())
        };

        if options.report_to_stdout {
            let reporter = ProgressReporter::new(processing_stats);
            reporter.run_with_progress(drop_fut).await
        } else {
            drop_fut.await
        }
    }

    pub fn app_ctx(&self) -> &AppContext<Prof> {
        self.root_component.app_ctx()
    }
}
