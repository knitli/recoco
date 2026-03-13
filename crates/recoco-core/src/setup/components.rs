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

use super::{CombinedState, ResourceSetupChange, SetupChangeType, StateChange};
use crate::prelude::*;

pub trait State<Key>: Debug + Send + Sync {
    fn key(&self) -> Key;
}

#[async_trait]
pub trait SetupOperator: 'static + Send + Sync {
    type Key: Debug + Hash + Eq + Clone + Send + Sync;
    type State: State<Self::Key>;
    type SetupState: Send + Sync + IntoIterator<Item = Self::State>;
    type Context: Sync;

    fn describe_key(&self, key: &Self::Key) -> String;

    fn describe_state(&self, state: &Self::State) -> String;

    fn is_up_to_date(&self, current: &Self::State, desired: &Self::State) -> bool;

    async fn create(&self, state: &Self::State, context: &Self::Context) -> Result<()>;

    async fn delete(&self, key: &Self::Key, context: &Self::Context) -> Result<()>;

    async fn update(&self, state: &Self::State, context: &Self::Context) -> Result<()> {
        self.delete(&state.key(), context).await?;
        self.create(state, context).await
    }
}

#[derive(Debug)]
struct CompositeStateUpsert<S> {
    state: S,
    already_exists: bool,
}
#[derive(Debug)]
pub struct SetupChange<D: SetupOperator> {
    desc: D,
    keys_to_delete: IndexSet<D::Key>,
    states_to_upsert: Vec<CompositeStateUpsert<D::State>>,
}

impl<D: SetupOperator> SetupChange<D> {
    pub fn create(
        desc: D,
        desired: Option<D::SetupState>,
        existing: CombinedState<D::SetupState>,
    ) -> Result<Self> {
        let existing_component_states = CombinedState {
            current: existing.current.map(|s| {
                s.into_iter()
                    .map(|s| (s.key(), s))
                    .collect::<IndexMap<_, _>>()
            }),
            staging: existing
                .staging
                .into_iter()
                .map(|s| match s {
                    StateChange::Delete => StateChange::Delete,
                    StateChange::Upsert(s) => {
                        StateChange::Upsert(s.into_iter().map(|s| (s.key(), s)).collect())
                    }
                })
                .collect(),
            legacy_state_key: existing.legacy_state_key,
        };
        let mut keys_to_delete = IndexSet::new();
        let mut states_to_upsert = vec![];

        // Collect all existing component keys
        for c in existing_component_states.possible_versions() {
            keys_to_delete.extend(c.keys().cloned());
        }

        if let Some(desired_state) = desired {
            for desired_comp_state in desired_state {
                let key = desired_comp_state.key();

                // Remove keys that should be kept from deletion list
                keys_to_delete.shift_remove(&key);

                // Add components that need to be updated
                let is_up_to_date = existing_component_states.always_exists()
                    && existing_component_states.possible_versions().all(|v| {
                        v.get(&key)
                            .is_some_and(|s| desc.is_up_to_date(s, &desired_comp_state))
                    });
                if !is_up_to_date {
                    let already_exists = existing_component_states
                        .possible_versions()
                        .any(|v| v.contains_key(&key));
                    states_to_upsert.push(CompositeStateUpsert {
                        state: desired_comp_state,
                        already_exists,
                    });
                }
            }
        }

        Ok(Self {
            desc,
            keys_to_delete,
            states_to_upsert,
        })
    }
}

impl<D: SetupOperator + Send + Sync> ResourceSetupChange for SetupChange<D> {
    fn describe_changes(&self) -> Vec<setup::ChangeDescription> {
        let mut result = vec![];

        for key in &self.keys_to_delete {
            result.push(setup::ChangeDescription::Action(format!(
                "Delete {}",
                self.desc.describe_key(key)
            )));
        }

        for state in &self.states_to_upsert {
            result.push(setup::ChangeDescription::Action(format!(
                "{} {}",
                if state.already_exists {
                    "Update"
                } else {
                    "Create"
                },
                self.desc.describe_state(&state.state)
            )));
        }

        result
    }

    fn change_type(&self) -> SetupChangeType {
        if self.keys_to_delete.is_empty() && self.states_to_upsert.is_empty() {
            SetupChangeType::NoChange
        } else if self.keys_to_delete.is_empty() {
            SetupChangeType::Create
        } else if self.states_to_upsert.is_empty() {
            SetupChangeType::Delete
        } else {
            SetupChangeType::Update
        }
    }
}

/// Maximum number of component operations (deletes or upserts) that may run concurrently.
/// Keeping this bounded prevents overwhelming a database connection pool or
/// network layer when a large number of components change at once.
const COMPONENT_CONCURRENCY_LIMIT: usize = 16;

pub async fn apply_component_changes<D: SetupOperator>(
    changes: Vec<&SetupChange<D>>,
    context: &D::Context,
) -> Result<()> {
    let total_deletes: usize = changes.iter().map(|c| c.keys_to_delete.len()).sum();
    let total_upserts: usize = changes.iter().map(|c| c.states_to_upsert.len()).sum();

    // First delete components that need to be removed (bounded concurrency)
    let mut delete_futures = Vec::with_capacity(total_deletes);
    for change in changes.iter() {
        for key in &change.keys_to_delete {
            delete_futures.push(change.desc.delete(key, context));
        }
    }
    futures::stream::iter(delete_futures)
        .buffer_unordered(COMPONENT_CONCURRENCY_LIMIT)
        .try_collect::<Vec<_>>()
        .await?;

    // Then upsert components that need to be updated (bounded concurrency)
    let mut upsert_futures = Vec::with_capacity(total_upserts);
    for change in changes.iter() {
        for state in &change.states_to_upsert {
            if state.already_exists {
                upsert_futures.push(change.desc.update(&state.state, context));
            } else {
                upsert_futures.push(change.desc.create(&state.state, context));
            }
        }
    }
    futures::stream::iter(upsert_futures)
        .buffer_unordered(COMPONENT_CONCURRENCY_LIMIT)
        .try_collect::<Vec<_>>()
        .await?;

    Ok(())
}

impl<A: ResourceSetupChange, B: ResourceSetupChange> ResourceSetupChange for (A, B) {
    fn describe_changes(&self) -> Vec<setup::ChangeDescription> {
        let mut result = vec![];
        result.extend(self.0.describe_changes());
        result.extend(self.1.describe_changes());
        result
    }

    fn change_type(&self) -> SetupChangeType {
        match (self.0.change_type(), self.1.change_type()) {
            (SetupChangeType::Invalid, _) | (_, SetupChangeType::Invalid) => {
                SetupChangeType::Invalid
            }
            (SetupChangeType::NoChange, b) => b,
            (a, _) => a,
        }
    }
}
