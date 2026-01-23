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

pub(crate) use crate::prelude::*;

use crate::builder::plan::AnalyzedFieldReference;
use crate::builder::plan::AnalyzedLocalFieldReference;

pub use super::factory_bases::*;
pub use super::interface::*;
pub use crate::base::schema::*;
pub use crate::base::spec::*;
pub use crate::base::value::*;

// Disambiguate the ExportTargetBuildOutput type.
pub use super::factory_bases::TypedExportDataCollectionBuildOutput;
pub use super::registry::ExecutorFactoryRegistry;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptySpec {}

#[macro_export]
macro_rules! fields_value {
    ($($field:expr), +) => {
        $crate::base::value::FieldValues { fields: std::vec![ $(($field).into()),+ ] }
    };
}

pub struct SchemaBuilderFieldRef(AnalyzedLocalFieldReference);

impl SchemaBuilderFieldRef {
    pub fn to_field_ref(&self) -> AnalyzedFieldReference {
        AnalyzedFieldReference {
            local: self.0.clone(),
            scope_up_level: 0,
        }
    }
}
pub struct StructSchemaBuilder<'a> {
    base_fields_idx: Vec<u32>,
    target: &'a mut StructSchema,
}

impl<'a> StructSchemaBuilder<'a> {
    pub fn new(target: &'a mut StructSchema) -> Self {
        Self {
            base_fields_idx: Vec::new(),
            target,
        }
    }

    pub fn _set_description(&mut self, description: impl Into<Arc<str>>) {
        self.target.description = Some(description.into());
    }

    pub fn add_field(&mut self, field_schema: FieldSchema) -> SchemaBuilderFieldRef {
        let current_idx = self.target.fields.len() as u32;
        Arc::make_mut(&mut self.target.fields).push(field_schema);
        let mut fields_idx = self.base_fields_idx.clone();
        fields_idx.push(current_idx);
        SchemaBuilderFieldRef(AnalyzedLocalFieldReference { fields_idx })
    }

    pub fn _add_struct_field(
        &mut self,
        name: impl Into<FieldName>,
        nullable: bool,
        attrs: Arc<BTreeMap<String, serde_json::Value>>,
    ) -> (StructSchemaBuilder<'_>, SchemaBuilderFieldRef) {
        let field_schema = FieldSchema::new(
            name.into(),
            EnrichedValueType {
                typ: ValueType::Struct(StructSchema {
                    fields: Arc::new(Vec::new()),
                    description: None,
                }),
                nullable,
                attrs,
            },
        );
        let local_ref = self.add_field(field_schema);
        let struct_schema = match &mut Arc::make_mut(&mut self.target.fields)
            .last_mut()
            .unwrap()
            .value_type
            .typ
        {
            ValueType::Struct(s) => s,
            _ => unreachable!(),
        };
        (
            StructSchemaBuilder {
                base_fields_idx: local_ref.0.fields_idx.clone(),
                target: struct_schema,
            },
            local_ref,
        )
    }
}
