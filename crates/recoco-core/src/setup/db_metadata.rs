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

use crate::prelude::*;

use super::{ResourceSetupChange, ResourceSetupInfo, SetupChangeType, StateChange};
use axum::http::StatusCode;
use sqlx::PgPool;
use utils::db::WriteAction;

const SETUP_METADATA_TABLE_NAME: &str = "cocoindex_setup_metadata";
pub const FLOW_VERSION_RESOURCE_TYPE: &str = "__FlowVersion";

/// Get the fully qualified metadata table name (schema.table)
fn get_qualified_metadata_table_name(schema: &str) -> String {
    format!("\"{}\".\"{}\"", schema, SETUP_METADATA_TABLE_NAME)
}

/// Get the qualified table name from lib context (for non-initialization calls)
async fn get_qualified_table_name_from_ctx() -> Result<String> {
    let lib_context = get_lib_context().await?;
    Ok(get_qualified_metadata_table_name(&lib_context.internal_schema))
}

/// Ensure the internal schema exists
async fn ensure_internal_schema_exists(pool: &PgPool, schema: &str) -> Result<()> {
    let query = format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema);
    sqlx::query(&query).execute(pool).await?;
    Ok(())
}

#[derive(sqlx::FromRow, Debug)]
pub struct SetupMetadataRecord {
    pub flow_name: String,
    // e.g. "Flow", "SourceTracking", "Target:{TargetType}"
    pub resource_type: String,
    pub key: serde_json::Value,
    pub state: Option<serde_json::Value>,
    pub staging_changes: sqlx::types::Json<Vec<StateChange<serde_json::Value>>>,
}

pub fn parse_flow_version(state: &Option<serde_json::Value>) -> Option<u64> {
    match state {
        Some(serde_json::Value::Number(n)) => n.as_u64(),
        _ => None,
    }
}

/// Returns None if metadata table doesn't exist.
pub async fn read_setup_metadata(
    pool: &PgPool,
    internal_schema: &str,
) -> Result<Option<Vec<SetupMetadataRecord>>> {
    let mut db_conn = pool.acquire().await?;
    let qualified_table_name = get_qualified_metadata_table_name(internal_schema);

    let query_str = format!(
        "SELECT flow_name, resource_type, key, state, staging_changes FROM {}",
        qualified_table_name
    );
    let metadata = sqlx::query_as(&query_str).fetch_all(&mut *db_conn).await;
    let result = match metadata {
        Ok(metadata) => Some(metadata),
        Err(err) => {
            let exists: Option<bool> = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = $1 AND tablename = $2)",
            )
            .bind(internal_schema)
            .bind(SETUP_METADATA_TABLE_NAME)
            .fetch_one(&mut *db_conn)
            .await?;
            if !exists.unwrap_or(false) {
                None
            } else {
                return Err(err.into());
            }
        }
    };
    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceTypeKey {
    pub resource_type: String,
    pub key: serde_json::Value,
}

impl ResourceTypeKey {
    pub fn new(resource_type: String, key: serde_json::Value) -> Self {
        Self { resource_type, key }
    }
}

static VERSION_RESOURCE_TYPE_ID: LazyLock<ResourceTypeKey> = LazyLock::new(|| ResourceTypeKey {
    resource_type: FLOW_VERSION_RESOURCE_TYPE.to_string(),
    key: serde_json::Value::Null,
});

async fn read_metadata_records_for_flow(
    flow_name: &str,
    db_executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
) -> Result<HashMap<ResourceTypeKey, SetupMetadataRecord>> {
    let qualified_table_name = get_qualified_table_name_from_ctx().await?;
    let query_str = format!(
        "SELECT flow_name, resource_type, key, state, staging_changes FROM {} WHERE flow_name = $1",
        qualified_table_name
    );
    let metadata: Vec<SetupMetadataRecord> = sqlx::query_as(&query_str)
        .bind(flow_name)
        .fetch_all(db_executor)
        .await?;
    let result = metadata
        .into_iter()
        .map(|m| {
            (
                ResourceTypeKey {
                    resource_type: m.resource_type.clone(),
                    key: m.key.clone(),
                },
                m,
            )
        })
        .collect();
    Ok(result)
}

async fn read_state(
    flow_name: &str,
    type_id: &ResourceTypeKey,
    db_executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
) -> Result<Option<serde_json::Value>> {
    let qualified_table_name = get_qualified_table_name_from_ctx().await?;
    let query_str = format!(
        "SELECT state FROM {} WHERE flow_name = $1 AND resource_type = $2 AND key = $3",
        qualified_table_name
    );
    let state: Option<serde_json::Value> = sqlx::query_scalar(&query_str)
        .bind(flow_name)
        .bind(&type_id.resource_type)
        .bind(&type_id.key)
        .fetch_optional(db_executor)
        .await?;
    Ok(state)
}

async fn upsert_staging_changes(
    flow_name: &str,
    type_id: &ResourceTypeKey,
    staging_changes: Vec<StateChange<serde_json::Value>>,
    db_executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    action: WriteAction,
) -> Result<()> {
    let qualified_table_name = get_qualified_table_name_from_ctx().await?;
    let query_str = match action {
        WriteAction::Insert => format!(
            "INSERT INTO {} (flow_name, resource_type, key, staging_changes) VALUES ($1, $2, $3, $4)",
            qualified_table_name
        ),
        WriteAction::Update => format!(
            "UPDATE {} SET staging_changes = $4 WHERE flow_name = $1 AND resource_type = $2 AND key = $3",
            qualified_table_name
        ),
    };
    sqlx::query(&query_str)
        .bind(flow_name)
        .bind(&type_id.resource_type)
        .bind(&type_id.key)
        .bind(sqlx::types::Json(staging_changes))
        .execute(db_executor)
        .await?;
    Ok(())
}

async fn upsert_state(
    flow_name: &str,
    type_id: &ResourceTypeKey,
    state: &serde_json::Value,
    action: WriteAction,
    db_executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
) -> Result<()> {
    let qualified_table_name = get_qualified_table_name_from_ctx().await?;
    let query_str = match action {
        WriteAction::Insert => format!(
            "INSERT INTO {} (flow_name, resource_type, key, state, staging_changes) VALUES ($1, $2, $3, $4, $5)",
            qualified_table_name
        ),
        WriteAction::Update => format!(
            "UPDATE {} SET state = $4, staging_changes = $5 WHERE flow_name = $1 AND resource_type = $2 AND key = $3",
            qualified_table_name
        ),
    };
    sqlx::query(&query_str)
        .bind(flow_name)
        .bind(&type_id.resource_type)
        .bind(&type_id.key)
        .bind(sqlx::types::Json(state))
        .bind(sqlx::types::Json(Vec::<serde_json::Value>::new()))
        .execute(db_executor)
        .await?;
    Ok(())
}

async fn delete_state(
    flow_name: &str,
    type_id: &ResourceTypeKey,
    db_executor: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
) -> Result<()> {
    let qualified_table_name = get_qualified_table_name_from_ctx().await?;
    let query_str = format!(
        "DELETE FROM {} WHERE flow_name = $1 AND resource_type = $2 AND key = $3",
        qualified_table_name
    );
    sqlx::query(&query_str)
        .bind(flow_name)
        .bind(&type_id.resource_type)
        .bind(&type_id.key)
        .execute(db_executor)
        .await?;
    Ok(())
}

pub struct StateUpdateInfo {
    pub desired_state: Option<serde_json::Value>,
    pub legacy_key: Option<ResourceTypeKey>,
}

impl StateUpdateInfo {
    pub fn new(
        desired_state: Option<&impl Serialize>,
        legacy_key: Option<ResourceTypeKey>,
    ) -> Result<Self> {
        Ok(Self {
            desired_state: desired_state
                .as_ref()
                .map(serde_json::to_value)
                .transpose()?,
            legacy_key,
        })
    }
}

pub async fn stage_changes_for_flow(
    flow_name: &str,
    seen_metadata_version: Option<u64>,
    resource_update_info: &HashMap<ResourceTypeKey, StateUpdateInfo>,
    pool: &PgPool,
) -> Result<u64> {
    let mut txn = pool.begin().await?;
    let mut existing_records = read_metadata_records_for_flow(flow_name, &mut *txn).await?;
    let latest_metadata_version = existing_records
        .get(&VERSION_RESOURCE_TYPE_ID)
        .and_then(|m| parse_flow_version(&m.state));
    if seen_metadata_version < latest_metadata_version {
        return Err(ApiError::new(
            "seen newer version in the metadata table",
            StatusCode::CONFLICT,
        ))?;
    }
    let new_metadata_version = seen_metadata_version.unwrap_or_default() + 1;
    upsert_state(
        flow_name,
        &VERSION_RESOURCE_TYPE_ID,
        &serde_json::Value::Number(new_metadata_version.into()),
        if latest_metadata_version.is_some() {
            WriteAction::Update
        } else {
            WriteAction::Insert
        },
        &mut *txn,
    )
    .await?;

    for (type_id, update_info) in resource_update_info {
        let existing = existing_records.remove(type_id);
        let change = match &update_info.desired_state {
            Some(desired_state) => StateChange::Upsert(desired_state.clone()),
            None => StateChange::Delete,
        };
        let mut new_staging_changes = vec![];
        if let Some(legacy_key) = &update_info.legacy_key
            && let Some(legacy_record) = existing_records.remove(legacy_key)
        {
            new_staging_changes.extend(legacy_record.staging_changes.0);
            delete_state(flow_name, legacy_key, &mut *txn).await?;
        }
        let (action, existing_staging_changes) = match existing {
            Some(existing) => {
                let existing_staging_changes = existing.staging_changes.0;
                if existing_staging_changes.iter().all(|c| c != &change) {
                    new_staging_changes.push(change);
                }
                (WriteAction::Update, existing_staging_changes)
            }
            None => {
                if update_info.desired_state.is_some() {
                    new_staging_changes.push(change);
                }
                (WriteAction::Insert, vec![])
            }
        };
        if !new_staging_changes.is_empty() {
            upsert_staging_changes(
                flow_name,
                type_id,
                [existing_staging_changes, new_staging_changes].concat(),
                &mut *txn,
                action,
            )
            .await?;
        }
    }
    txn.commit().await?;
    Ok(new_metadata_version)
}

pub async fn commit_changes_for_flow(
    flow_name: &str,
    curr_metadata_version: u64,
    state_updates: &HashMap<ResourceTypeKey, StateUpdateInfo>,
    delete_version: bool,
    pool: &PgPool,
) -> Result<()> {
    let mut txn = pool.begin().await?;
    let latest_metadata_version =
        parse_flow_version(&read_state(flow_name, &VERSION_RESOURCE_TYPE_ID, &mut *txn).await?);
    if latest_metadata_version != Some(curr_metadata_version) {
        return Err(ApiError::new(
            "seen newer version in the metadata table",
            StatusCode::CONFLICT,
        ))?;
    }
    for (type_id, update_info) in state_updates.iter() {
        match &update_info.desired_state {
            Some(desired_state) => {
                upsert_state(
                    flow_name,
                    type_id,
                    desired_state,
                    WriteAction::Update,
                    &mut *txn,
                )
                .await?;
            }
            None => {
                delete_state(flow_name, type_id, &mut *txn).await?;
            }
        }
    }
    if delete_version {
        delete_state(flow_name, &VERSION_RESOURCE_TYPE_ID, &mut *txn).await?;
    }
    txn.commit().await?;
    Ok(())
}

#[derive(Debug)]
pub struct MetadataTableSetup {
    pub metadata_table_missing: bool,
}

impl MetadataTableSetup {
    pub fn into_setup_info(self) -> ResourceSetupInfo<(), (), MetadataTableSetup> {
        ResourceSetupInfo {
            key: (),
            state: None,
            has_tracked_state_change: self.metadata_table_missing,
            description: "CocoIndex Metadata Table".to_string(),
            setup_change: Some(self),
            legacy_key: None,
        }
    }
}

impl ResourceSetupChange for MetadataTableSetup {
    fn describe_changes(&self) -> Vec<setup::ChangeDescription> {
        if self.metadata_table_missing {
            vec![setup::ChangeDescription::Action(format!(
                "Create the cocoindex metadata table {SETUP_METADATA_TABLE_NAME}"
            ))]
        } else {
            vec![]
        }
    }

    fn change_type(&self) -> SetupChangeType {
        if self.metadata_table_missing {
            SetupChangeType::Create
        } else {
            SetupChangeType::NoChange
        }
    }
}

impl MetadataTableSetup {
    pub async fn apply_change(&self) -> Result<()> {
        if !self.metadata_table_missing {
            return Ok(());
        }
        let lib_context = get_lib_context().await?;
        let pool = lib_context.require_builtin_db_pool()?;
        let schema = &lib_context.internal_schema;

        // Ensure the internal schema exists
        ensure_internal_schema_exists(pool, schema).await?;

        let qualified_table_name = get_qualified_metadata_table_name(schema);
        let query_str = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                flow_name TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                key JSONB NOT NULL,
                state JSONB,
                staging_changes JSONB NOT NULL,

                PRIMARY KEY (flow_name, resource_type, key)
            )
        ",
            qualified_table_name
        );
        sqlx::query(&query_str).execute(pool).await?;
        Ok(())
    }
}
