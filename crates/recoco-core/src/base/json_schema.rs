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

use schemars::Schema;
use serde_json::{Map, Value as JsonValue};
use std::fmt::Write;
use utils::immutable::RefList;

pub struct ToJsonSchemaOptions {
    /// If true, mark all fields as required.
    /// Use union type (with `null`) for optional fields instead.
    /// Models like OpenAI will reject the schema if a field is not required.
    pub fields_always_required: bool,

    /// If true, the JSON schema supports the `format` keyword.
    pub supports_format: bool,

    /// If true, extract descriptions to a separate extra instruction.
    pub extract_descriptions: bool,

    /// If true, the top level must be a JSON object.
    pub top_level_must_be_object: bool,

    /// If true, include `additionalProperties: false` in object schemas.
    /// Some LLM APIs (e.g., Gemini) do not support this constraint and will error.
    pub supports_additional_properties: bool,
}

struct JsonSchemaBuilder {
    options: ToJsonSchemaOptions,
    extra_instructions_per_field: IndexMap<String, String>,
}

impl JsonSchemaBuilder {
    fn new(options: ToJsonSchemaOptions) -> Self {
        Self {
            options,
            extra_instructions_per_field: IndexMap::new(),
        }
    }

    fn add_description(
        &mut self,
        schema: &mut Schema,
        description: &str,
        field_path: RefList<'_, &'_ spec::FieldName>,
    ) {
        if self.options.extract_descriptions {
            let mut fields: Vec<_> = field_path.iter().map(|f| f.as_str()).collect();
            fields.reverse();
            let field_path_str = fields.join(".");

            let mut_description = self
                .extra_instructions_per_field
                .entry(field_path_str)
                .or_default();
            if !mut_description.is_empty() {
                mut_description.push_str("\n\n");
            }
            mut_description.push_str(description);
        } else {
            let obj = schema.ensure_object();
            let existing = obj
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_owned());
            let new_description = match existing {
                Some(existing) if !existing.is_empty() => format!("{existing}\n\n{description}"),
                _ => description.to_owned(),
            };
            obj.insert("description".to_owned(), JsonValue::String(new_description));
        }
    }

    fn for_basic_value_type(
        &mut self,
        mut schema: Schema,
        basic_type: &schema::BasicValueType,
        field_path: RefList<'_, &'_ spec::FieldName>,
    ) -> Schema {
        match basic_type {
            schema::BasicValueType::Str => {
                schema
                    .ensure_object()
                    .insert("type".to_owned(), JsonValue::String("string".to_owned()));
            }
            schema::BasicValueType::Bytes => {
                schema
                    .ensure_object()
                    .insert("type".to_owned(), JsonValue::String("string".to_owned()));
            }
            schema::BasicValueType::Bool => {
                schema
                    .ensure_object()
                    .insert("type".to_owned(), JsonValue::String("boolean".to_owned()));
            }
            schema::BasicValueType::Int64 => {
                schema
                    .ensure_object()
                    .insert("type".to_owned(), JsonValue::String("integer".to_owned()));
            }
            schema::BasicValueType::Float32 | schema::BasicValueType::Float64 => {
                schema
                    .ensure_object()
                    .insert("type".to_owned(), JsonValue::String("number".to_owned()));
            }
            schema::BasicValueType::Range => {
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("array".to_owned()));
                obj.insert("items".to_owned(), serde_json::json!({"type": "integer"}));
                obj.insert("minItems".to_owned(), JsonValue::Number(2.into()));
                obj.insert("maxItems".to_owned(), JsonValue::Number(2.into()));
                self.add_description(
                    &mut schema,
                    "A range represented by a list of two positions, start pos (inclusive), end pos (exclusive).",
                    field_path,
                );
            }
            schema::BasicValueType::Uuid => {
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("string".to_owned()));
                if self.options.supports_format {
                    obj.insert("format".to_owned(), JsonValue::String("uuid".to_owned()));
                }
                self.add_description(
                    &mut schema,
                    "A UUID, e.g. 123e4567-e89b-12d3-a456-426614174000",
                    field_path,
                );
            }
            schema::BasicValueType::Date => {
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("string".to_owned()));
                if self.options.supports_format {
                    obj.insert("format".to_owned(), JsonValue::String("date".to_owned()));
                }
                self.add_description(
                    &mut schema,
                    "A date in YYYY-MM-DD format, e.g. 2025-03-27",
                    field_path,
                );
            }
            schema::BasicValueType::Time => {
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("string".to_owned()));
                if self.options.supports_format {
                    obj.insert("format".to_owned(), JsonValue::String("time".to_owned()));
                }
                self.add_description(
                    &mut schema,
                    "A time in HH:MM:SS format, e.g. 13:32:12",
                    field_path,
                );
            }
            schema::BasicValueType::LocalDateTime => {
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("string".to_owned()));
                if self.options.supports_format {
                    obj.insert(
                        "format".to_owned(),
                        JsonValue::String("date-time".to_owned()),
                    );
                }
                self.add_description(
                    &mut schema,
                    "Date time without timezone offset in YYYY-MM-DDTHH:MM:SS format, e.g. 2025-03-27T13:32:12",
                    field_path,
                );
            }
            schema::BasicValueType::OffsetDateTime => {
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("string".to_owned()));
                if self.options.supports_format {
                    obj.insert(
                        "format".to_owned(),
                        JsonValue::String("date-time".to_owned()),
                    );
                }
                self.add_description(
                    &mut schema,
                    "Date time with timezone offset in RFC3339, e.g. 2025-03-27T13:32:12Z, 2025-03-27T07:32:12.313-06:00",
                    field_path,
                );
            }
            &schema::BasicValueType::TimeDelta => {
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("string".to_owned()));
                if self.options.supports_format {
                    obj.insert(
                        "format".to_owned(),
                        JsonValue::String("duration".to_owned()),
                    );
                }
                self.add_description(
                    &mut schema,
                    "A duration, e.g. 'PT1H2M3S' (ISO 8601) or '1 day 2 hours 3 seconds'",
                    field_path,
                );
            }
            schema::BasicValueType::Json => {
                // Can be any value. No type constraint.
            }
            schema::BasicValueType::Vector(s) => {
                let items_schema =
                    self.for_basic_value_type(Schema::default(), &s.element_type, field_path);
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("array".to_owned()));
                obj.insert(
                    "items".to_owned(),
                    serde_json::to_value(&items_schema).unwrap_or(JsonValue::Object(Map::new())),
                );
                if let Some(d) = s.dimension
                    && let Ok(d) = u32::try_from(d)
                {
                    obj.insert("minItems".to_owned(), JsonValue::Number(d.into()));
                    obj.insert("maxItems".to_owned(), JsonValue::Number(d.into()));
                }
            }
            schema::BasicValueType::Union(s) => {
                let one_of: Vec<JsonValue> = s
                    .types
                    .iter()
                    .map(|t| {
                        let inner_schema =
                            self.for_basic_value_type(Schema::default(), t, field_path);
                        serde_json::to_value(&inner_schema).unwrap_or(JsonValue::Object(Map::new()))
                    })
                    .collect();
                schema
                    .ensure_object()
                    .insert("oneOf".to_owned(), JsonValue::Array(one_of));
            }
        }
        schema
    }

    fn for_struct_schema(
        &mut self,
        mut schema: Schema,
        struct_schema: &schema::StructSchema,
        field_path: RefList<'_, &'_ spec::FieldName>,
    ) -> Schema {
        if let Some(description) = &struct_schema.description {
            self.add_description(&mut schema, description, field_path);
        }

        let mut properties = Map::new();
        let mut required: Vec<String> = Vec::new();

        for f in struct_schema.fields.iter() {
            let mut field_schema = Schema::default();
            // Set field description if available
            if let Some(description) = &f.description {
                self.add_description(&mut field_schema, description, field_path.prepend(&f.name));
            }
            let mut field_schema = self.for_enriched_value_type(
                field_schema,
                &f.value_type,
                field_path.prepend(&f.name),
            );

            if self.options.fields_always_required && f.value_type.nullable {
                // Add "null" to the type array
                let obj = field_schema.ensure_object();
                if let Some(type_val) = obj.get("type").cloned() {
                    let types = match type_val {
                        JsonValue::String(s) => JsonValue::Array(vec![
                            JsonValue::String(s),
                            JsonValue::String("null".to_owned()),
                        ]),
                        JsonValue::Array(mut arr) => {
                            arr.push(JsonValue::String("null".to_owned()));
                            JsonValue::Array(arr)
                        }
                        _ => type_val,
                    };
                    obj.insert("type".to_owned(), types);
                }
            }

            let field_json =
                serde_json::to_value(&field_schema).unwrap_or(JsonValue::Object(Map::new()));
            properties.insert(f.name.to_string(), field_json);

            if self.options.fields_always_required || !f.value_type.nullable {
                required.push(f.name.to_string());
            }
        }

        let obj = schema.ensure_object();
        obj.insert("type".to_owned(), JsonValue::String("object".to_owned()));
        obj.insert("properties".to_owned(), JsonValue::Object(properties));
        obj.insert(
            "required".to_owned(),
            JsonValue::Array(required.into_iter().map(JsonValue::String).collect()),
        );
        if self.options.supports_additional_properties {
            obj.insert("additionalProperties".to_owned(), JsonValue::Bool(false));
        }

        schema
    }

    fn for_value_type(
        &mut self,
        mut schema: Schema,
        value_type: &schema::ValueType,
        field_path: RefList<'_, &'_ spec::FieldName>,
    ) -> Schema {
        match value_type {
            schema::ValueType::Basic(b) => self.for_basic_value_type(schema, b, field_path),
            schema::ValueType::Struct(s) => self.for_struct_schema(schema, s, field_path),
            schema::ValueType::Table(c) => {
                let items_schema = self.for_struct_schema(Schema::default(), &c.row, field_path);
                let obj = schema.ensure_object();
                obj.insert("type".to_owned(), JsonValue::String("array".to_owned()));
                obj.insert(
                    "items".to_owned(),
                    serde_json::to_value(&items_schema).unwrap_or(JsonValue::Object(Map::new())),
                );
                schema
            }
        }
    }

    fn for_enriched_value_type(
        &mut self,
        schema: Schema,
        enriched_value_type: &schema::EnrichedValueType,
        field_path: RefList<'_, &'_ spec::FieldName>,
    ) -> Schema {
        self.for_value_type(schema, &enriched_value_type.typ, field_path)
    }

    fn build_extra_instructions(&self) -> Result<Option<String>> {
        if self.extra_instructions_per_field.is_empty() {
            return Ok(None);
        }

        let mut instructions = String::new();
        write!(&mut instructions, "Instructions for specific fields:\n\n")?;
        for (field_path, instruction) in self.extra_instructions_per_field.iter() {
            write!(
                &mut instructions,
                "- {}: {}\n\n",
                if field_path.is_empty() {
                    "(root object)"
                } else {
                    field_path.as_str()
                },
                instruction
            )?;
        }
        Ok(Some(instructions))
    }
}

pub struct ValueExtractor {
    value_type: schema::ValueType,
    object_wrapper_field_name: Option<String>,
}

impl ValueExtractor {
    pub fn extract_value(&self, json_value: serde_json::Value) -> Result<value::Value> {
        let unwrapped_json_value =
            if let Some(object_wrapper_field_name) = &self.object_wrapper_field_name {
                match json_value {
                    serde_json::Value::Object(mut o) => o
                        .remove(object_wrapper_field_name)
                        .unwrap_or(serde_json::Value::Null),
                    _ => {
                        client_bail!("Field `{}` not found", object_wrapper_field_name)
                    }
                }
            } else {
                json_value
            };
        let result = value::Value::from_json(unwrapped_json_value, &self.value_type)?;
        Ok(result)
    }
}

pub struct BuildJsonSchemaOutput {
    pub schema: Schema,
    pub extra_instructions: Option<String>,
    pub value_extractor: ValueExtractor,
}

pub fn build_json_schema(
    value_type: schema::EnrichedValueType,
    options: ToJsonSchemaOptions,
) -> Result<BuildJsonSchemaOutput> {
    let mut builder = JsonSchemaBuilder::new(options);
    let (schema, object_wrapper_field_name) = if builder.options.top_level_must_be_object
        && !matches!(value_type.typ, schema::ValueType::Struct(_))
    {
        let object_wrapper_field_name = "value".to_string();
        let wrapper_struct = schema::StructSchema {
            fields: Arc::new(vec![schema::FieldSchema {
                name: object_wrapper_field_name.clone(),
                value_type: value_type.clone(),
                description: None,
            }]),
            description: None,
        };
        (
            builder.for_struct_schema(Schema::default(), &wrapper_struct, RefList::Nil),
            Some(object_wrapper_field_name),
        )
    } else {
        (
            builder.for_enriched_value_type(Schema::default(), &value_type, RefList::Nil),
            None,
        )
    };
    Ok(BuildJsonSchemaOutput {
        schema,
        extra_instructions: builder.build_extra_instructions()?,
        value_extractor: ValueExtractor {
            value_type: value_type.typ,
            object_wrapper_field_name,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::schema::*;
    use expect_test::expect;
    use serde_json::json;
    use std::sync::Arc;

    fn create_test_options() -> ToJsonSchemaOptions {
        ToJsonSchemaOptions {
            fields_always_required: false,
            supports_format: true,
            extract_descriptions: false,
            top_level_must_be_object: false,
            supports_additional_properties: true,
        }
    }

    fn create_test_options_with_extracted_descriptions() -> ToJsonSchemaOptions {
        ToJsonSchemaOptions {
            fields_always_required: false,
            supports_format: true,
            extract_descriptions: true,
            top_level_must_be_object: false,
            supports_additional_properties: true,
        }
    }

    fn create_test_options_always_required() -> ToJsonSchemaOptions {
        ToJsonSchemaOptions {
            fields_always_required: true,
            supports_format: true,
            extract_descriptions: false,
            top_level_must_be_object: false,
            supports_additional_properties: true,
        }
    }

    fn create_test_options_top_level_object() -> ToJsonSchemaOptions {
        ToJsonSchemaOptions {
            fields_always_required: false,
            supports_format: true,
            extract_descriptions: false,
            top_level_must_be_object: true,
            supports_additional_properties: true,
        }
    }

    fn schema_to_json(schema: &Schema) -> serde_json::Value {
        serde_json::to_value(schema).unwrap()
    }

    #[test]
    fn test_basic_types_str() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Str),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "type": "string"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_bool() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Bool),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "type": "boolean"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_int64() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Int64),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "type": "integer"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_float32() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Float32),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "type": "number"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_float64() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Float64),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "type": "number"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_bytes() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Bytes),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "type": "string"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_range() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Range),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "description": "A range represented by a list of two positions, start pos (inclusive), end pos (exclusive).",
              "items": {
                "type": "integer"
              },
              "maxItems": 2,
              "minItems": 2,
              "type": "array"
            }"#]].assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_uuid() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Uuid),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "description": "A UUID, e.g. 123e4567-e89b-12d3-a456-426614174000",
              "format": "uuid",
              "type": "string"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_date() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Date),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "description": "A date in YYYY-MM-DD format, e.g. 2025-03-27",
              "format": "date",
              "type": "string"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_time() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Time),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "description": "A time in HH:MM:SS format, e.g. 13:32:12",
              "format": "time",
              "type": "string"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_local_date_time() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::LocalDateTime),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "description": "Date time without timezone offset in YYYY-MM-DDTHH:MM:SS format, e.g. 2025-03-27T13:32:12",
              "format": "date-time",
              "type": "string"
            }"#]].assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_offset_date_time() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::OffsetDateTime),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "description": "Date time with timezone offset in RFC3339, e.g. 2025-03-27T13:32:12Z, 2025-03-27T07:32:12.313-06:00",
              "format": "date-time",
              "type": "string"
            }"#]].assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_time_delta() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::TimeDelta),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "description": "A duration, e.g. 'PT1H2M3S' (ISO 8601) or '1 day 2 hours 3 seconds'",
              "format": "duration",
              "type": "string"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_json() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Json),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect!["{}"].assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_vector() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Vector(VectorTypeSchema {
                element_type: Box::new(BasicValueType::Str),
                dimension: Some(3),
            })),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "items": {
                "type": "string"
              },
              "maxItems": 3,
              "minItems": 3,
              "type": "array"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_basic_types_union() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Union(UnionTypeSchema {
                types: vec![BasicValueType::Str, BasicValueType::Int64],
            })),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "oneOf": [
                {
                  "type": "string"
                },
                {
                  "type": "integer"
                }
              ]
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_nullable_basic_type() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Str),
            nullable: true,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "type": "string"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_struct_type_simple() {
        let value_type = EnrichedValueType {
            typ: ValueType::Struct(StructSchema {
                fields: Arc::new(vec![
                    FieldSchema::new(
                        "name",
                        EnrichedValueType {
                            typ: ValueType::Basic(BasicValueType::Str),
                            nullable: false,
                            attrs: Arc::new(BTreeMap::new()),
                        },
                    ),
                    FieldSchema::new(
                        "age",
                        EnrichedValueType {
                            typ: ValueType::Basic(BasicValueType::Int64),
                            nullable: false,
                            attrs: Arc::new(BTreeMap::new()),
                        },
                    ),
                ]),
                description: None,
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "additionalProperties": false,
              "properties": {
                "age": {
                  "type": "integer"
                },
                "name": {
                  "type": "string"
                }
              },
              "required": [
                "name",
                "age"
              ],
              "type": "object"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_struct_type_with_optional_field() {
        let value_type = EnrichedValueType {
            typ: ValueType::Struct(StructSchema {
                fields: Arc::new(vec![
                    FieldSchema::new(
                        "name",
                        EnrichedValueType {
                            typ: ValueType::Basic(BasicValueType::Str),
                            nullable: false,
                            attrs: Arc::new(BTreeMap::new()),
                        },
                    ),
                    FieldSchema::new(
                        "age",
                        EnrichedValueType {
                            typ: ValueType::Basic(BasicValueType::Int64),
                            nullable: true,
                            attrs: Arc::new(BTreeMap::new()),
                        },
                    ),
                ]),
                description: None,
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "additionalProperties": false,
              "properties": {
                "age": {
                  "type": "integer"
                },
                "name": {
                  "type": "string"
                }
              },
              "required": [
                "name"
              ],
              "type": "object"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_struct_type_with_description() {
        let value_type = EnrichedValueType {
            typ: ValueType::Struct(StructSchema {
                fields: Arc::new(vec![FieldSchema::new(
                    "name",
                    EnrichedValueType {
                        typ: ValueType::Basic(BasicValueType::Str),
                        nullable: false,
                        attrs: Arc::new(BTreeMap::new()),
                    },
                )]),
                description: Some("A person".into()),
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "additionalProperties": false,
              "description": "A person",
              "properties": {
                "name": {
                  "type": "string"
                }
              },
              "required": [
                "name"
              ],
              "type": "object"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_struct_type_with_extracted_descriptions() {
        let value_type = EnrichedValueType {
            typ: ValueType::Struct(StructSchema {
                fields: Arc::new(vec![FieldSchema::new(
                    "name",
                    EnrichedValueType {
                        typ: ValueType::Basic(BasicValueType::Str),
                        nullable: false,
                        attrs: Arc::new(BTreeMap::new()),
                    },
                )]),
                description: Some("A person".into()),
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options_with_extracted_descriptions();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "additionalProperties": false,
              "properties": {
                "name": {
                  "type": "string"
                }
              },
              "required": [
                "name"
              ],
              "type": "object"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());

        // Check that description was extracted to extra instructions
        assert!(result.extra_instructions.is_some());
        let instructions = result.extra_instructions.unwrap();
        assert!(instructions.contains("A person"));
    }

    #[test]
    fn test_struct_type_always_required() {
        let value_type = EnrichedValueType {
            typ: ValueType::Struct(StructSchema {
                fields: Arc::new(vec![
                    FieldSchema::new(
                        "name",
                        EnrichedValueType {
                            typ: ValueType::Basic(BasicValueType::Str),
                            nullable: false,
                            attrs: Arc::new(BTreeMap::new()),
                        },
                    ),
                    FieldSchema::new(
                        "age",
                        EnrichedValueType {
                            typ: ValueType::Basic(BasicValueType::Int64),
                            nullable: true,
                            attrs: Arc::new(BTreeMap::new()),
                        },
                    ),
                ]),
                description: None,
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options_always_required();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "additionalProperties": false,
              "properties": {
                "age": {
                  "type": [
                    "integer",
                    "null"
                  ]
                },
                "name": {
                  "type": "string"
                }
              },
              "required": [
                "name",
                "age"
              ],
              "type": "object"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_table_type_utable() {
        let value_type = EnrichedValueType {
            typ: ValueType::Table(TableSchema {
                kind: TableKind::UTable,
                row: StructSchema {
                    fields: Arc::new(vec![
                        FieldSchema::new(
                            "id",
                            EnrichedValueType {
                                typ: ValueType::Basic(BasicValueType::Int64),
                                nullable: false,
                                attrs: Arc::new(BTreeMap::new()),
                            },
                        ),
                        FieldSchema::new(
                            "name",
                            EnrichedValueType {
                                typ: ValueType::Basic(BasicValueType::Str),
                                nullable: false,
                                attrs: Arc::new(BTreeMap::new()),
                            },
                        ),
                    ]),
                    description: None,
                },
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "items": {
                "additionalProperties": false,
                "properties": {
                  "id": {
                    "type": "integer"
                  },
                  "name": {
                    "type": "string"
                  }
                },
                "required": [
                  "id",
                  "name"
                ],
                "type": "object"
              },
              "type": "array"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_table_type_ktable() {
        let value_type = EnrichedValueType {
            typ: ValueType::Table(TableSchema {
                kind: TableKind::KTable(KTableInfo { num_key_parts: 1 }),
                row: StructSchema {
                    fields: Arc::new(vec![
                        FieldSchema::new(
                            "id",
                            EnrichedValueType {
                                typ: ValueType::Basic(BasicValueType::Int64),
                                nullable: false,
                                attrs: Arc::new(BTreeMap::new()),
                            },
                        ),
                        FieldSchema::new(
                            "name",
                            EnrichedValueType {
                                typ: ValueType::Basic(BasicValueType::Str),
                                nullable: false,
                                attrs: Arc::new(BTreeMap::new()),
                            },
                        ),
                    ]),
                    description: None,
                },
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "items": {
                "additionalProperties": false,
                "properties": {
                  "id": {
                    "type": "integer"
                  },
                  "name": {
                    "type": "string"
                  }
                },
                "required": [
                  "id",
                  "name"
                ],
                "type": "object"
              },
              "type": "array"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_table_type_ltable() {
        let value_type = EnrichedValueType {
            typ: ValueType::Table(TableSchema {
                kind: TableKind::LTable,
                row: StructSchema {
                    fields: Arc::new(vec![FieldSchema::new(
                        "value",
                        EnrichedValueType {
                            typ: ValueType::Basic(BasicValueType::Str),
                            nullable: false,
                            attrs: Arc::new(BTreeMap::new()),
                        },
                    )]),
                    description: None,
                },
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "items": {
                "additionalProperties": false,
                "properties": {
                  "value": {
                    "type": "string"
                  }
                },
                "required": [
                  "value"
                ],
                "type": "object"
              },
              "type": "array"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_top_level_must_be_object_with_basic_type() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Str),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options_top_level_object();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "additionalProperties": false,
              "properties": {
                "value": {
                  "type": "string"
                }
              },
              "required": [
                "value"
              ],
              "type": "object"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());

        // Check that value extractor has the wrapper field name
        assert_eq!(
            result.value_extractor.object_wrapper_field_name,
            Some("value".to_string())
        );
    }

    #[test]
    fn test_top_level_must_be_object_with_struct_type() {
        let value_type = EnrichedValueType {
            typ: ValueType::Struct(StructSchema {
                fields: Arc::new(vec![FieldSchema::new(
                    "name",
                    EnrichedValueType {
                        typ: ValueType::Basic(BasicValueType::Str),
                        nullable: false,
                        attrs: Arc::new(BTreeMap::new()),
                    },
                )]),
                description: None,
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options_top_level_object();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "additionalProperties": false,
              "properties": {
                "name": {
                  "type": "string"
                }
              },
              "required": [
                "name"
              ],
              "type": "object"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());

        // Check that value extractor has no wrapper field name since it's already a struct
        assert_eq!(result.value_extractor.object_wrapper_field_name, None);
    }

    #[test]
    fn test_nested_struct() {
        let value_type = EnrichedValueType {
            typ: ValueType::Struct(StructSchema {
                fields: Arc::new(vec![FieldSchema::new(
                    "person",
                    EnrichedValueType {
                        typ: ValueType::Struct(StructSchema {
                            fields: Arc::new(vec![
                                FieldSchema::new(
                                    "name",
                                    EnrichedValueType {
                                        typ: ValueType::Basic(BasicValueType::Str),
                                        nullable: false,
                                        attrs: Arc::new(BTreeMap::new()),
                                    },
                                ),
                                FieldSchema::new(
                                    "age",
                                    EnrichedValueType {
                                        typ: ValueType::Basic(BasicValueType::Int64),
                                        nullable: false,
                                        attrs: Arc::new(BTreeMap::new()),
                                    },
                                ),
                            ]),
                            description: None,
                        }),
                        nullable: false,
                        attrs: Arc::new(BTreeMap::new()),
                    },
                )]),
                description: None,
            }),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "additionalProperties": false,
              "properties": {
                "person": {
                  "additionalProperties": false,
                  "properties": {
                    "age": {
                      "type": "integer"
                    },
                    "name": {
                      "type": "string"
                    }
                  },
                  "required": [
                    "name",
                    "age"
                  ],
                  "type": "object"
                }
              },
              "required": [
                "person"
              ],
              "type": "object"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_value_extractor_basic_type() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Str),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options();
        let result = build_json_schema(value_type, options).unwrap();

        // Test extracting a string value
        let json_value = json!("hello world");
        let extracted = result.value_extractor.extract_value(json_value).unwrap();
        assert!(
            matches!(extracted, crate::base::value::Value::Basic(crate::base::value::BasicValue::Str(s)) if s.as_ref() == "hello world")
        );
    }

    #[test]
    fn test_value_extractor_with_wrapper() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Str),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = create_test_options_top_level_object();
        let result = build_json_schema(value_type, options).unwrap();

        // Test extracting a wrapped value
        let json_value = json!({"value": "hello world"});
        let extracted = result.value_extractor.extract_value(json_value).unwrap();
        assert!(
            matches!(extracted, crate::base::value::Value::Basic(crate::base::value::BasicValue::Str(s)) if s.as_ref() == "hello world")
        );
    }

    #[test]
    fn test_no_format_support() {
        let value_type = EnrichedValueType {
            typ: ValueType::Basic(BasicValueType::Uuid),
            nullable: false,
            attrs: Arc::new(BTreeMap::new()),
        };
        let options = ToJsonSchemaOptions {
            fields_always_required: false,
            supports_format: false,
            extract_descriptions: false,
            top_level_must_be_object: false,
            supports_additional_properties: true,
        };
        let result = build_json_schema(value_type, options).unwrap();
        let json_schema = schema_to_json(&result.schema);

        expect![[r#"
            {
              "description": "A UUID, e.g. 123e4567-e89b-12d3-a456-426614174000",
              "type": "string"
            }"#]]
        .assert_eq(&serde_json::to_string_pretty(&json_schema).unwrap());
    }

    #[test]
    fn test_description_concatenation() {
        // Create a struct with a field that has both field-level and type-level descriptions
        let struct_schema = StructSchema {
            description: Some(Arc::from("Test struct description")),
            fields: Arc::new(vec![FieldSchema {
                name: "uuid_field".to_string(),
                value_type: EnrichedValueType {
                    typ: ValueType::Basic(BasicValueType::Uuid),
                    nullable: false,
                    attrs: Default::default(),
                },
                description: Some(Arc::from("This is a field-level description for UUID")),
            }]),
        };

        let enriched_value_type = EnrichedValueType {
            typ: ValueType::Struct(struct_schema),
            nullable: false,
            attrs: Default::default(),
        };

        let options = ToJsonSchemaOptions {
            fields_always_required: false,
            supports_format: true,
            extract_descriptions: false, // We want to see the description in the schema
            top_level_must_be_object: false,
            supports_additional_properties: true,
        };

        let result = build_json_schema(enriched_value_type, options).unwrap();

        // Check if the description contains both field and type descriptions
        let schema_json = serde_json::to_value(&result.schema).unwrap();
        let description = schema_json
            .get("properties")
            .and_then(|p| p.get("uuid_field"))
            .and_then(|f| f.get("description"))
            .and_then(|d| d.as_str());

        assert_eq!(
            description,
            Some(
                "This is a field-level description for UUID\n\nA UUID, e.g. 123e4567-e89b-12d3-a456-426614174000"
            )
        );
    }
}
