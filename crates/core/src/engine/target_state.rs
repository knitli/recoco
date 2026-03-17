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

//! Target state management for the incremental dataflow engine.
//!
//! This module provides the core abstractions for managing target state providers,
//! handlers, and the attachment provider mechanism. Attachment providers allow a
//! [`TargetHandler`] to expose additional, independently-tracked sub-handlers that
//! carry their own state through the engine's reconciliation loop.
//!
//! ## Attachment Providers
//!
//! An *attachment provider* is a child [`TargetStateProvider`] associated with a
//! named *attachment type* on a parent provider. When a target needs to track
//! additional state alongside its primary state (e.g. an index structure, a
//! secondary store, or a metadata sidecar), the handler can implement
//! [`TargetHandler::attachment`] to vend a child handler for each supported type.
//!
//! Attachment providers are registered lazily during a component build via
//! [`TargetStateProvider::register_attachment_provider`]. They inherit the
//! parent provider's [`TargetStateProviderGeneration`], and their path in the
//! provider registry is the parent path concatenated with the attachment type
//! name as a symbol key.
//!
//! ### Usage pattern
//!
//! ```ignore
//! // A handler that supports an "index" attachment.
//! impl TargetHandler<MyProfile> for MyHandler {
//!     fn attachment(&self, att_type: &str) -> Result<Option<MyHandler>> {
//!         match att_type {
//!             "index" => Ok(Some(MyIndexHandler::new())),
//!             _ => Ok(None),
//!         }
//!     }
//!     // ... reconcile() ...
//! }
//!
//! // During component processing, register the attachment:
//! let att_provider =
//!     parent_provider.register_attachment_provider(&comp_ctx, "index")?;
//! ```

use crate::prelude::*;

use crate::{
    engine::{context::ComponentProcessorContext, profile::EngineProfile},
    state::{
        stable_path::StableKey,
        target_state_path::{TargetStatePath, TargetStateProviderGeneration},
    },
};

use std::hash::Hash;

/// Definition of a child target created during action application.
pub struct ChildTargetDef<Prof: EngineProfile> {
    pub handler: Prof::TargetHdl,
}

/// Sink that receives and applies batched target actions.
///
/// Implementations are expected to spawn the application logic onto a separate
/// thread or task when needed so the caller is not blocked.
#[async_trait]
pub trait TargetActionSink<Prof: EngineProfile>: Send + Sync + Eq + Hash + 'static {
    // TODO: Add method to expose function info and arguments, for tracing purpose & no-change detection.

    /// Run the logic to apply the action.
    ///
    /// We expect the implementation of this method to spawn the logic to a separate thread or task when needed.
    async fn apply(
        &self,
        host_runtime_ctx: &Prof::HostRuntimeCtx,
        actions: Vec<Prof::TargetAction>,
    ) -> Result<Option<Vec<Option<ChildTargetDef<Prof>>>>>;
}

/// Describes how a change to a parent target's state invalidates its child targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildInvalidation {
    /// Child data is completely invalidated and must be recomputed from scratch.
    Destructive,
    /// Child data may be partially stale; a best-effort update is sufficient.
    Lossy,
}

/// Output produced by [`TargetHandler::reconcile`] for a single state transition.
pub struct TargetReconcileOutput<Prof: EngineProfile> {
    pub action: Prof::TargetAction,
    pub sink: Prof::TargetActionSink,
    pub tracking_record: Option<Prof::TargetStateTrackingRecord>,
    pub child_invalidation: Option<ChildInvalidation>,
}

/// Core trait for target handlers that reconcile desired state with the engine.
///
/// Implementors are responsible for comparing the desired state with any
/// previously-tracked state and producing the appropriate action (if any) to
/// drive the target toward the desired state.
///
/// ## Attachment support
///
/// Handlers can optionally expose named *attachment* sub-handlers by overriding
/// [`attachment`][TargetHandler::attachment]. The default implementation returns
/// `Ok(None)` for every attachment type, meaning the handler does not support
/// attachments. Attachment handlers are used to track additional state alongside
/// the primary target state (see the [module documentation][self] for details).
pub trait TargetHandler<Prof: EngineProfile>: Send + Sync + Sized + 'static {
    /// Reconcile the desired state for a single keyed entry.
    ///
    /// Returns `None` when no action is needed (the current state already
    /// matches the desired state), or `Some(output)` containing the action to
    /// apply.
    fn reconcile(
        &self,
        key: StableKey,
        desired_target_state: Option<Prof::TargetStateValue>,
        prev_possible_states: &[Prof::TargetStateTrackingRecord],
        prev_may_be_missing: bool,
    ) -> Result<Option<TargetReconcileOutput<Prof>>>;

    /// Return a child handler for the named attachment type, or `Ok(None)` if
    /// this handler does not support the given attachment type.
    ///
    /// The default implementation returns `Ok(None)` for all attachment types.
    /// Override this method to support attachment providers (see the
    /// [module documentation][self]).
    fn attachment(&self, _att_type: &str) -> Result<Option<Prof::TargetHdl>> {
        Ok(None)
    }
}

pub(crate) struct TargetStateProviderInner<Prof: EngineProfile> {
    parent_provider: Option<TargetStateProvider<Prof>>,
    stable_key: StableKey,
    target_state_path: TargetStatePath,
    handler: OnceLock<Prof::TargetHdl>,
    orphaned: OnceLock<()>,
    provider_generation: OnceLock<TargetStateProviderGeneration>,
    /// Cache of registered attachment sub-providers, keyed by attachment type name.
    attachments: Mutex<HashMap<Arc<str>, TargetStateProvider<Prof>>>,
}

/// A handle to a registered target state provider within the engine.
///
/// A `TargetStateProvider` is a node in the provider tree. The root nodes are
/// created via [`TargetStateProviderRegistry::register_root`]; child nodes are
/// created lazily via [`register_lazy`][TargetStateProviderRegistry::register_lazy]
/// or as attachment providers via
/// [`register_attachment_provider`][TargetStateProvider::register_attachment_provider].
///
/// Cloning is cheap — internally the provider is reference-counted.
#[derive(Clone)]
pub struct TargetStateProvider<Prof: EngineProfile> {
    pub(crate) inner: Arc<TargetStateProviderInner<Prof>>,
}

impl<Prof: EngineProfile> TargetStateProvider<Prof> {
    /// The stable path that uniquely identifies this provider in the registry.
    pub fn target_state_path(&self) -> &TargetStatePath {
        &self.inner.target_state_path
    }

    /// The handler associated with this provider, or `None` if it has not yet
    /// been fulfilled (see [`fulfill_handler`][Self::fulfill_handler]).
    pub fn handler(&self) -> Option<&Prof::TargetHdl> {
        self.inner.handler.get()
    }

    /// Set the handler for a lazily-registered provider.
    ///
    /// Returns an error if the handler has already been set.
    pub fn fulfill_handler(&self, handler: Prof::TargetHdl) -> Result<()> {
        self.inner
            .handler
            .set(handler)
            .map_err(|_| internal_error!("Handler is already fulfilled"))
    }

    /// Returns the chain of stable keys from the root provider down to this one,
    /// in root-first order.
    pub fn stable_key_chain(&self) -> Vec<StableKey> {
        let mut chain = vec![self.inner.stable_key.clone()];
        let mut current = self;
        while let Some(parent) = &current.inner.parent_provider {
            chain.push(parent.inner.stable_key.clone());
            current = parent;
        }
        chain.reverse();
        chain
    }

    /// Returns `true` if this provider has been marked as orphaned (i.e. the
    /// component that created it has been deleted).
    pub fn is_orphaned(&self) -> bool {
        self.inner.orphaned.get().is_some()
    }

    /// The generation metadata for this provider, or `None` if it has not yet
    /// been assigned.
    pub fn provider_generation(&self) -> Option<&TargetStateProviderGeneration> {
        self.inner.provider_generation.get()
    }

    /// Assign the generation metadata for this provider.
    ///
    /// Returns an error if the generation has already been set.
    pub fn set_provider_generation(&self, generation: TargetStateProviderGeneration) -> Result<()> {
        self.inner
            .provider_generation
            .set(generation)
            .map_err(|_| internal_error!("Provider generation already set"))
    }

    /// Register (or retrieve an already-registered) attachment sub-provider for
    /// the given attachment type name.
    ///
    /// The attachment type name is passed to [`TargetHandler::attachment`] on
    /// this provider's handler to obtain the child handler. If the handler does
    /// not support the given attachment type, an error is returned.
    ///
    /// Calling this method twice with the same `att_type` returns the same
    /// provider both times (idempotent).
    ///
    /// The new sub-provider's path is `self.target_state_path()` concatenated
    /// with `att_type` as a [`StableKey::Symbol`], and it inherits the parent's
    /// [`TargetStateProviderGeneration`].
    ///
    /// # Errors
    ///
    /// - If this provider does not yet have a handler (see
    ///   [`fulfill_handler`][Self::fulfill_handler]).
    /// - If the handler returns an error from [`attachment`][TargetHandler::attachment].
    /// - If the handler does not support the given `att_type`.
    /// - If this provider does not yet have a provider generation assigned (see
    ///   [`set_provider_generation`][Self::set_provider_generation]).
    pub fn register_attachment_provider(
        &self,
        comp_ctx: &ComponentProcessorContext<Prof>,
        att_type: &str,
    ) -> Result<TargetStateProvider<Prof>> {
        let mut attachments = self.inner.attachments.lock().unwrap();
        if let Some(existing) = attachments.get(att_type) {
            return Ok(existing.clone());
        }

        let handler = self
            .handler()
            .ok_or_else(|| client_error!("Cannot register attachment on unfulfilled provider"))?
            .attachment(att_type)?
            .ok_or_else(|| {
                client_error!("Handler does not support attachment type: {att_type:?}")
            })?;

        let symbol_key = StableKey::Symbol(att_type.into());
        let target_state_path = self.target_state_path().concat(&symbol_key);

        let provider_generation = self
            .provider_generation()
            .ok_or_else(|| {
                internal_error!(
                    "Parent provider generation must be set before registering attachment"
                )
            })?
            .clone();

        let provider = TargetStateProvider {
            inner: Arc::new(TargetStateProviderInner {
                parent_provider: Some(self.clone()),
                stable_key: symbol_key,
                target_state_path: target_state_path.clone(),
                handler: OnceLock::from(handler),
                orphaned: OnceLock::new(),
                provider_generation: OnceLock::from(provider_generation),
                attachments: Mutex::new(HashMap::new()),
            }),
        };

        comp_ctx.update_building_state(|building_state| {
            building_state
                .target_states
                .provider_registry
                .add(target_state_path, provider.clone())
        })?;

        attachments.insert(att_type.into(), provider.clone());
        Ok(provider)
    }
}

/// Registry of all [`TargetStateProvider`]s active within a component processing context.
///
/// The registry maps [`TargetStatePath`]s to their corresponding providers. New
/// providers are added via [`register_root`][Self::register_root] (for top-level
/// targets) and [`register_lazy`][Self::register_lazy] (for child targets whose
/// handlers are resolved later). Attachment sub-providers are added via
/// [`TargetStateProvider::register_attachment_provider`].
#[derive(Default)]
pub struct TargetStateProviderRegistry<Prof: EngineProfile> {
    pub(crate) providers: rpds::HashTrieMapSync<TargetStatePath, TargetStateProvider<Prof>>,
    pub(crate) curr_target_state_paths: Vec<TargetStatePath>,
}

impl<Prof: EngineProfile> TargetStateProviderRegistry<Prof> {
    /// Create a new registry pre-populated with an existing set of providers.
    pub fn new(
        providers: rpds::HashTrieMapSync<TargetStatePath, TargetStateProvider<Prof>>,
    ) -> Self {
        Self {
            providers,
            curr_target_state_paths: Vec::new(),
        }
    }

    /// Add a provider to the registry.
    ///
    /// Returns an error if a provider is already registered for `target_state_path`.
    pub fn add(
        &mut self,
        target_state_path: TargetStatePath,
        provider: TargetStateProvider<Prof>,
    ) -> Result<()> {
        if self.providers.contains_key(&target_state_path) {
            client_bail!(
                "Target state provider already registered for path: {:?}",
                target_state_path
            );
        }
        self.curr_target_state_paths.push(target_state_path.clone());
        self.providers.insert_mut(target_state_path, provider);
        Ok(())
    }

    /// Register a root-level target provider with a fully-resolved handler.
    ///
    /// The provider's path is derived from `name` using a stable fingerprint.
    pub fn register_root(
        &mut self,
        name: String,
        handler: Prof::TargetHdl,
    ) -> Result<TargetStateProvider<Prof>> {
        let target_state_path = TargetStatePath::new(
            utils::fingerprint::Fingerprinter::default()
                .with(&name)?
                .into_fingerprint(),
            None,
        );
        let provider = TargetStateProvider {
            inner: Arc::new(TargetStateProviderInner {
                parent_provider: None,
                stable_key: StableKey::Symbol(name.into()),
                target_state_path: target_state_path.clone(),
                handler: OnceLock::from(handler),
                orphaned: OnceLock::new(),
                provider_generation: OnceLock::new(),
                attachments: Mutex::new(HashMap::new()),
            }),
        };
        self.add(target_state_path, provider.clone())?;
        Ok(provider)
    }

    /// Register a lazily-resolved child provider under `parent_provider`.
    ///
    /// The new provider's handler is left unfulfilled; call
    /// [`TargetStateProvider::fulfill_handler`] before the provider is used for
    /// reconciliation.
    pub fn register_lazy(
        &mut self,
        parent_provider: &TargetStateProvider<Prof>,
        stable_key: StableKey,
    ) -> Result<TargetStateProvider<Prof>> {
        let target_state_path = parent_provider.target_state_path().concat(&stable_key);
        let provider = TargetStateProvider {
            inner: Arc::new(TargetStateProviderInner {
                parent_provider: Some(parent_provider.clone()),
                stable_key,
                target_state_path: target_state_path.clone(),
                handler: OnceLock::new(),
                orphaned: OnceLock::new(),
                provider_generation: OnceLock::new(),
                attachments: Mutex::new(HashMap::new()),
            }),
        };
        self.add(target_state_path, provider.clone())?;
        Ok(provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::component::{ComponentProcessor, ComponentProcessorInfo};
    use crate::engine::profile::{EngineProfile, Persist};
    use bytes::Bytes;
    use std::future::Future;

    // ── Minimal mock engine profile ──────────────────────────────────────────

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
    struct MockProfile;

    #[derive(Clone, Default)]
    struct MockFunctionData;

    impl Persist for MockFunctionData {
        fn to_bytes(&self) -> Result<Bytes> {
            Ok(Bytes::new())
        }
        fn from_bytes(_: &[u8]) -> Result<Self> {
            Ok(Self)
        }
    }

    struct MockTrackingRecord;

    impl Persist for MockTrackingRecord {
        fn to_bytes(&self) -> Result<Bytes> {
            Ok(Bytes::new())
        }
        fn from_bytes(_: &[u8]) -> Result<Self> {
            Ok(Self)
        }
    }

    /// A minimal handler that can optionally support an `"inner"` attachment.
    struct MockHandler {
        supports_attachment: bool,
    }

    impl TargetHandler<MockProfile> for MockHandler {
        fn reconcile(
            &self,
            _key: StableKey,
            _desired_target_state: Option<()>,
            _prev_possible_states: &[MockTrackingRecord],
            _prev_may_be_missing: bool,
        ) -> Result<Option<TargetReconcileOutput<MockProfile>>> {
            Ok(None)
        }

        fn attachment(&self, att_type: &str) -> Result<Option<MockHandler>> {
            if self.supports_attachment && att_type == "inner" {
                Ok(Some(MockHandler {
                    supports_attachment: false,
                }))
            } else {
                Ok(None)
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash)]
    struct MockSink;

    #[async_trait]
    impl TargetActionSink<MockProfile> for MockSink {
        async fn apply(
            &self,
            _host_runtime_ctx: &(),
            _actions: Vec<()>,
        ) -> Result<Option<Vec<Option<ChildTargetDef<MockProfile>>>>> {
            Ok(None)
        }
    }

    struct MockComponentProc;

    impl ComponentProcessor<MockProfile> for MockComponentProc {
        fn process(
            &self,
            _host_runtime_ctx: &(),
            _comp_ctx: &crate::engine::context::ComponentProcessorContext<MockProfile>,
        ) -> Result<impl Future<Output = Result<MockFunctionData>> + Send + 'static> {
            Ok(async { Ok(MockFunctionData) })
        }

        fn memo_key_fingerprint(&self) -> Option<utils::fingerprint::Fingerprint> {
            None
        }

        fn processor_info(&self) -> &ComponentProcessorInfo {
            static INFO: LazyLock<ComponentProcessorInfo> =
                LazyLock::new(|| ComponentProcessorInfo::new("mock".to_string()));
            &INFO
        }
    }

    impl EngineProfile for MockProfile {
        type HostRuntimeCtx = ();
        type ComponentProc = MockComponentProc;
        type FunctionData = MockFunctionData;
        type TargetHdl = MockHandler;
        type TargetStateTrackingRecord = MockTrackingRecord;
        type TargetAction = ();
        type TargetActionSink = MockSink;
        type TargetStateValue = ();
    }

    // ── Registry tests ───────────────────────────────────────────────────────

    #[test]
    fn register_root_creates_provider() {
        let mut registry = TargetStateProviderRegistry::<MockProfile>::default();
        let provider = registry
            .register_root(
                "target".to_string(),
                MockHandler {
                    supports_attachment: false,
                },
            )
            .unwrap();

        assert_eq!(provider.stable_key_chain().len(), 1);
        assert!(provider.handler().is_some());
        assert!(!provider.is_orphaned());
        assert_eq!(registry.curr_target_state_paths.len(), 1);
        assert_eq!(registry.providers.size(), 1);
    }

    #[test]
    fn register_root_duplicate_returns_error() {
        let mut registry = TargetStateProviderRegistry::<MockProfile>::default();
        registry
            .register_root(
                "target".to_string(),
                MockHandler {
                    supports_attachment: false,
                },
            )
            .unwrap();
        let result = registry.register_root(
            "target".to_string(),
            MockHandler {
                supports_attachment: false,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn register_lazy_creates_child_with_longer_path() {
        let mut registry = TargetStateProviderRegistry::<MockProfile>::default();
        let root = registry
            .register_root(
                "parent".to_string(),
                MockHandler {
                    supports_attachment: false,
                },
            )
            .unwrap();

        let child_key = StableKey::Symbol("child".into());
        let child = registry.register_lazy(&root, child_key).unwrap();

        assert!(
            child.target_state_path().as_slice().len() > root.target_state_path().as_slice().len()
        );
        assert_eq!(child.stable_key_chain().len(), 2);
        // Child handler is not yet fulfilled.
        assert!(child.handler().is_none());
        assert_eq!(registry.curr_target_state_paths.len(), 2);
    }

    // ── TargetHandler attachment tests ───────────────────────────────────────

    #[test]
    fn attachment_default_returns_none() {
        let handler = MockHandler {
            supports_attachment: false,
        };
        assert!(handler.attachment("any_type").unwrap().is_none());
    }

    #[test]
    fn attachment_override_returns_handler_for_supported_type() {
        let handler = MockHandler {
            supports_attachment: true,
        };
        assert!(handler.attachment("inner").unwrap().is_some());
    }

    #[test]
    fn attachment_override_returns_none_for_unsupported_type() {
        let handler = MockHandler {
            supports_attachment: true,
        };
        assert!(handler.attachment("unknown").unwrap().is_none());
    }
}
