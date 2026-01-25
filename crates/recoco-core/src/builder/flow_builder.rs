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

use crate::{
    base::schema::EnrichedValueType, builder::plan::FieldDefFingerprint, prelude::*,
};
#[cfg(feature = "persistence")]
use crate::setup::ObjectSetupChange;

use recoco_utils::fingerprint::Fingerprinter;
use std::{collections::btree_map, ops::Deref};

use super::analyzer::{
    AnalyzerContext, CollectorBuilder, DataScopeBuilder, OpScope, ValueTypeBuilder,
    build_flow_instance_context,
};
use crate::lib_context::FlowContext;
use crate::{
    base::{
        schema::{CollectorSchema, FieldSchema},
        spec::{FieldName, NamedSpec},
    },
    lib_context::LibContext,
    ops::interface::FlowInstanceContext,
};

#[derive(Debug, Clone)]
pub struct OpScopeRef(Arc<OpScope>);

impl From<Arc<OpScope>> for OpScopeRef {
    fn from(scope: Arc<OpScope>) -> Self {
        Self(scope)
    }
}

impl Deref for OpScopeRef {
    type Target = Arc<OpScope>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for OpScopeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl OpScopeRef {
    pub fn add_collector(&mut self, name: String) -> Result<DataCollector> {
        let collector = DataCollector {
            name,
            scope: self.0.clone(),
            collector: Mutex::new(None),
        };
        Ok(collector)
    }
}

#[derive(Debug, Clone)]
pub struct DataType {
    schema: schema::EnrichedValueType,
}

impl From<schema::EnrichedValueType> for DataType {
    fn from(schema: schema::EnrichedValueType) -> Self {
        Self { schema }
    }
}

impl DataType {
    pub fn schema(&self) -> schema::EnrichedValueType {
        self.schema.clone()
    }
}

#[derive(Debug, Clone)]
pub struct DataSlice {
    scope: Arc<OpScope>,
    value: Arc<spec::ValueMapping>,
}

impl DataSlice {
    pub fn data_type(&self) -> Result<DataType> {
        Ok(DataType::from(self.value_type()?))
    }

    pub fn field(&self, field_name: &str) -> Result<Option<DataSlice>> {
        let value_mapping = match self.value.as_ref() {
            spec::ValueMapping::Field(spec::FieldMapping { scope, field_path }) => {
                let data_scope_builder = self.scope.data.lock().unwrap();
                let struct_schema = {
                    let (_, val_type, _) = data_scope_builder
                        .analyze_field_path(field_path, self.scope.base_value_def_fp.clone())?;
                    match &val_type.typ {
                        ValueTypeBuilder::Struct(struct_type) => struct_type,
                        _ => return Err(client_error!("expect struct type in field path")),
                    }
                };
                if struct_schema.find_field(field_name).is_none() {
                    return Ok(None);
                }
                spec::ValueMapping::Field(spec::FieldMapping {
                    scope: scope.clone(),
                    field_path: spec::FieldPath(
                        field_path
                            .iter()
                            .cloned()
                            .chain([field_name.to_string()])
                            .collect(),
                    ),
                })
            }

            spec::ValueMapping::Constant { .. } => {
                return Err(client_error!("field access not supported for literal",));
            }
        };
        Ok(Some(DataSlice {
            scope: self.scope.clone(),
            value: Arc::new(value_mapping),
        }))
    }
}

impl DataSlice {
    fn extract_value_mapping(&self) -> spec::ValueMapping {
        match self.value.as_ref() {
            spec::ValueMapping::Field(v) => spec::ValueMapping::Field(spec::FieldMapping {
                field_path: v.field_path.clone(),
                scope: v.scope.clone().or_else(|| Some(self.scope.name.clone())),
            }),
            v => v.clone(),
        }
    }

    fn value_type(&self) -> Result<schema::EnrichedValueType> {
        let result = match self.value.as_ref() {
            spec::ValueMapping::Constant(c) => c.schema.clone(),
            spec::ValueMapping::Field(v) => {
                let data_scope_builder = self.scope.data.lock().unwrap();
                let (_, val_type, _) = data_scope_builder
                    .analyze_field_path(&v.field_path, self.scope.base_value_def_fp.clone())?;
                EnrichedValueType::from_alternative(val_type)?
            }
        };
        Ok(result)
    }
}

impl std::fmt::Display for DataSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataSlice(")?;
        match self.value_type() {
            Ok(value_type) => write!(f, "{value_type}")?,
            Err(e) => write!(f, "<error: {}>", e)?,
        }
        write!(f, "; {} {}) ", self.scope, self.value)?;
        Ok(())
    }
}

pub struct DataCollector {
    name: String,
    scope: Arc<OpScope>,
    collector: Mutex<Option<CollectorBuilder>>,
}

impl std::fmt::Display for DataCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let collector = self.collector.lock().unwrap();
        write!(f, "DataCollector \"{}\" ({}", self.name, self.scope)?;
        if let Some(collector) = collector.as_ref() {
            write!(f, ": {}", collector.schema)?;
            if collector.is_used {
                write!(f, " (used)")?;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

pub struct FlowBuilder {
    lib_context: Arc<LibContext>,
    flow_inst_context: Arc<FlowInstanceContext>,

    root_op_scope: Arc<OpScope>,
    flow_instance_name: String,
    reactive_ops: Vec<NamedSpec<spec::ReactiveOpSpec>>,

    direct_input_fields: Vec<FieldSchema>,
    direct_output_value: Option<spec::ValueMapping>,

    import_ops: Vec<NamedSpec<spec::ImportOpSpec>>,
    export_ops: Vec<NamedSpec<spec::ExportOpSpec>>,

    declarations: Vec<spec::OpSpec>,

    next_generated_op_id: usize,
}

impl FlowBuilder {
    pub async fn new(name: &str) -> Result<Self> {
        let _span = info_span!("flow_builder.new", flow_name = %name).entered();
        let lib_context = get_lib_context().await?;
        let root_op_scope = OpScope::new(
            spec::ROOT_SCOPE_NAME.to_string(),
            None,
            Arc::new(Mutex::new(DataScopeBuilder::new())),
            FieldDefFingerprint::default(),
        );
        let flow_inst_context = build_flow_instance_context(name);
        let result = Self {
            lib_context,
            flow_inst_context,
            root_op_scope,
            flow_instance_name: name.to_string(),

            reactive_ops: vec![],

            import_ops: vec![],
            export_ops: vec![],

            direct_input_fields: vec![],
            direct_output_value: None,

            declarations: vec![],

            next_generated_op_id: 0,
        };
        Ok(result)
    }

    pub fn root_scope(&self) -> OpScopeRef {
        OpScopeRef(self.root_op_scope.clone())
    }

    pub async fn add_source(
        &mut self,
        kind: String,
        op_spec: serde_json::Map<String, serde_json::Value>,
        target_scope: Option<OpScopeRef>,
        name: String,
        refresh_options: Option<spec::SourceRefreshOptions>,
        execution_options: Option<spec::ExecutionOptions>,
    ) -> Result<DataSlice> {
        let _span = info_span!("flow_builder.add_source", flow_name = %self.flow_instance_name, source_name = %name, source_kind = %kind).entered();
        if let Some(target_scope) = target_scope
            && *target_scope != self.root_op_scope
        {
            return Err(client_error!("source can only be added to the root scope",));
        }
        let import_op = spec::NamedSpec {
            name,
            spec: spec::ImportOpSpec {
                source: spec::OpSpec {
                    kind,
                    spec: op_spec,
                },
                refresh_options: refresh_options.unwrap_or_default(),
                execution_options: execution_options.unwrap_or_default(),
            },
        };
        let analyzer_ctx = AnalyzerContext {
            lib_ctx: self.lib_context.clone(),
            flow_ctx: self.flow_inst_context.clone(),
        };
        let analyzed = analyzer_ctx
            .analyze_import_op(&self.root_op_scope, import_op.clone())
            .await?;
        std::mem::drop(analyzed);

        let result = Self::last_field_to_data_slice(&self.root_op_scope)?;
        self.import_ops.push(import_op);
        Ok(result)
    }

    pub fn constant(
        &self,
        value_type: schema::EnrichedValueType,
        value: serde_json::Value,
    ) -> Result<DataSlice> {
        let schema = value_type;
        let slice = DataSlice {
            scope: self.root_op_scope.clone(),
            value: Arc::new(spec::ValueMapping::Constant(spec::ConstantMapping {
                schema: schema.clone(),
                value,
            })),
        };
        Ok(slice)
    }

    pub fn add_direct_input(
        &mut self,
        name: String,
        value_type: schema::EnrichedValueType,
    ) -> Result<DataSlice> {
        {
            let mut root_data_scope = self.root_op_scope.data.lock().unwrap();
            root_data_scope.add_field(
                name.clone(),
                &value_type,
                FieldDefFingerprint {
                    source_op_names: HashSet::from([name.clone()]),
                    fingerprint: Fingerprinter::default()
                        .with("input")
                        .map_err(Error::from)?
                        .with(&name)
                        .map_err(Error::from)?
                        .into_fingerprint(),
                },
            )?;
        }
        let result = Self::last_field_to_data_slice(&self.root_op_scope)?;
        self.direct_input_fields.push(FieldSchema {
            name,
            value_type,
            description: None,
        });
        Ok(result)
    }

    pub fn set_direct_output(&mut self, data_slice: DataSlice) -> Result<()> {
        if data_slice.scope != self.root_op_scope {
            return Err(client_error!(
                "direct output must be value in the root scope",
            ));
        }
        self.direct_output_value = Some(data_slice.extract_value_mapping());
        Ok(())
    }

    pub fn for_each(
        &mut self,
        data_slice: DataSlice,
        execution_options: Option<spec::ExecutionOptions>,
    ) -> Result<OpScopeRef> {
        let parent_scope = &data_slice.scope;
        let field_path = match data_slice.value.as_ref() {
            spec::ValueMapping::Field(v) => &v.field_path,
            _ => return Err(client_error!("expect field path")),
        };
        let num_parent_layers = parent_scope.ancestors().count();
        let scope_name = format!(
            "{}_{}",
            field_path.last().map_or("", |s| s.as_str()),
            num_parent_layers
        );
        let (_, child_op_scope) =
            parent_scope.new_foreach_op_scope(scope_name.clone(), field_path)?;

        let reactive_op = spec::NamedSpec {
            name: format!(".for_each.{}", self.next_generated_op_id),
            spec: spec::ReactiveOpSpec::ForEach(spec::ForEachOpSpec {
                field_path: field_path.clone(),
                op_scope: spec::ReactiveOpScope {
                    name: scope_name,
                    ops: vec![],
                },
                execution_options: execution_options.unwrap_or_default(),
            }),
        };
        self.next_generated_op_id += 1;
        self.get_mut_reactive_ops(parent_scope)?.push(reactive_op);

        Ok(OpScopeRef(child_op_scope))
    }

    pub async fn transform(
        &mut self,
        kind: String,
        op_spec: serde_json::Map<String, serde_json::Value>,
        args: Vec<(DataSlice, Option<String>)>,
        target_scope: Option<OpScopeRef>,
        name: String,
    ) -> Result<DataSlice> {
        let _span = info_span!("flow_builder.transform", flow_name = %self.flow_instance_name, op_name = %name, op_kind = %kind).entered();
        let spec = spec::OpSpec {
            kind,
            spec: op_spec,
        };
        let op_scope = Self::minimum_common_scope(
            args.iter().map(|(ds, _)| &ds.scope),
            target_scope.as_ref().map(|s| &s.0),
        )?;

        let reactive_op = spec::NamedSpec {
            name,
            spec: spec::ReactiveOpSpec::Transform(spec::TransformOpSpec {
                inputs: args
                    .iter()
                    .map(|(ds, arg_name)| spec::OpArgBinding {
                        arg_name: spec::OpArgName(arg_name.clone()),
                        value: ds.extract_value_mapping(),
                    })
                    .collect(),
                op: spec,
                execution_options: Default::default(),
            }),
        };

        let analyzer_ctx = AnalyzerContext {
            lib_ctx: self.lib_context.clone(),
            flow_ctx: self.flow_inst_context.clone(),
        };
        let analyzed = analyzer_ctx
            .analyze_reactive_op(op_scope, &reactive_op)
            .await?;
        std::mem::drop(analyzed);

        self.get_mut_reactive_ops(op_scope)?.push(reactive_op);

        let result = Self::last_field_to_data_slice(op_scope)?;
        Ok(result)
    }

    pub async fn collect(
        &mut self,
        collector: &DataCollector,
        fields: Vec<(FieldName, DataSlice)>,
        auto_uuid_field: Option<FieldName>,
    ) -> Result<()> {
        let _span = info_span!("flow_builder.collect", flow_name = %self.flow_instance_name, collector_name = %collector.name).entered();
        let common_scope =
            Self::minimum_common_scope(fields.iter().map(|(_, ds)| &ds.scope), None)?;
        let name = format!(".collect.{}", self.next_generated_op_id);
        self.next_generated_op_id += 1;

        let reactive_op = spec::NamedSpec {
            name,
            spec: spec::ReactiveOpSpec::Collect(spec::CollectOpSpec {
                input: spec::StructMapping {
                    fields: fields
                        .iter()
                        .map(|(name, ds)| NamedSpec {
                            name: name.clone(),
                            spec: ds.extract_value_mapping(),
                        })
                        .collect(),
                },
                scope_name: collector.scope.name.clone(),
                collector_name: collector.name.clone(),
                auto_uuid_field: auto_uuid_field.clone(),
            }),
        };

        let analyzer_ctx = AnalyzerContext {
            lib_ctx: self.lib_context.clone(),
            flow_ctx: self.flow_inst_context.clone(),
        };
        let analyzed = analyzer_ctx
            .analyze_reactive_op(common_scope, &reactive_op)
            .await?;
        std::mem::drop(analyzed);

        self.get_mut_reactive_ops(common_scope)?.push(reactive_op);

        let collector_schema = CollectorSchema::from_fields(
            fields
                .into_iter()
                .map(|(name, ds)| {
                    Ok(FieldSchema {
                        name,
                        value_type: ds.value_type()?,
                        description: None,
                    })
                })
                .collect::<Result<Vec<FieldSchema>>>()?,
            auto_uuid_field,
        );
        {
            // TODO: Pass in the right field def fingerprint
            let mut collector = collector.collector.lock().unwrap();
            if let Some(collector) = collector.as_mut() {
                collector.collect(&collector_schema, FieldDefFingerprint::default())?;
            } else {
                *collector = Some(CollectorBuilder::new(
                    Arc::new(collector_schema),
                    FieldDefFingerprint::default(),
                ));
            }
        }

        Ok(())
    }

    pub fn export(
        &mut self,
        name: String,
        kind: String,
        op_spec: serde_json::Map<String, serde_json::Value>,
        attachments: Vec<spec::OpSpec>,
        index_options: spec::IndexOptions,
        input: &DataCollector,
        setup_by_user: bool,
    ) -> Result<()> {
        let _span = info_span!("flow_builder.export", flow_name = %self.flow_instance_name, export_name = %name, target_kind = %kind).entered();
        let spec = spec::OpSpec {
            kind,
            spec: op_spec,
        };

        if input.scope != self.root_op_scope {
            return Err(client_error!(
                "Export can only work on collectors belonging to the root scope.",
            ));
        }
        self.export_ops.push(spec::NamedSpec {
            name,
            spec: spec::ExportOpSpec {
                collector_name: input.name.clone(),
                target: spec,
                attachments,
                index_options,
                setup_by_user,
            },
        });
        Ok(())
    }

    pub fn declare(&mut self, op_spec: spec::OpSpec) -> Result<()> {
        self.declarations.push(op_spec);
        Ok(())
    }

    pub fn scope_field(&self, scope: OpScopeRef, field_name: &str) -> Result<Option<DataSlice>> {
        {
            let scope_builder = scope.0.data.lock().unwrap();
            if scope_builder.data.find_field(field_name).is_none() {
                return Err(client_error!("field {field_name} not found"));
            }
        }
        Ok(Some(DataSlice {
            scope: scope.0,
            value: Arc::new(spec::ValueMapping::Field(spec::FieldMapping {
                scope: None,
                field_path: spec::FieldPath(vec![field_name.to_string()]),
            })),
        }))
    }

    #[cfg(feature = "persistence")]
    pub async fn build_flow(&self) -> Result<Flow> {
        let _span =
            info_span!("flow_builder.build_flow", flow_name = %self.flow_instance_name).entered();
        let spec = spec::FlowInstanceSpec {
            name: self.flow_instance_name.clone(),
            import_ops: self.import_ops.clone(),
            reactive_ops: self.reactive_ops.clone(),
            export_ops: self.export_ops.clone(),
            declarations: self.declarations.clone(),
        };
        let flow_instance_ctx = self.flow_inst_context.clone();

        let flow_ctx = {
            let analyzed_flow =
                super::AnalyzedFlow::from_flow_instance(spec, flow_instance_ctx).await?;
            let persistence_ctx = self.lib_context.require_persistence_ctx()?;
            let flow_ctx = {
                let flow_setup_ctx = persistence_ctx.setup_ctx.read().await;
                FlowContext::new(
                    Arc::new(analyzed_flow),
                    flow_setup_ctx
                        .all_setup_states
                        .flows
                        .get(&self.flow_instance_name),
                )
                .await?
            };

            // Apply internal-only changes if any.
            {
                let mut flow_exec_ctx = flow_ctx.get_execution_ctx_for_setup().write().await;
                if flow_exec_ctx.setup_change.has_internal_changes()
                    && !flow_exec_ctx.setup_change.has_external_changes()
                {
                    let mut lib_setup_ctx = persistence_ctx.setup_ctx.write().await;
                    let mut output_buffer = Vec::<u8>::new();
                    setup::apply_changes_for_flow_ctx(
                        setup::FlowSetupChangeAction::Setup,
                        &flow_ctx,
                        &mut flow_exec_ctx,
                        &mut lib_setup_ctx,
                        &persistence_ctx.builtin_db_pool,
                        &mut output_buffer,
                    )
                    .await?;
                    trace!(
                        "Applied internal-only change for flow {}:\n{}",
                        self.flow_instance_name,
                        String::from_utf8_lossy(&output_buffer)
                    );
                }
            }

            Ok::<_, Error>(flow_ctx)
        }?;

        let mut flow_ctxs = self.lib_context.flows.lock().unwrap();
        let flow_ctx = match flow_ctxs.entry(self.flow_instance_name.clone()) {
            btree_map::Entry::Occupied(_) => {
                return Err(client_error!(
                    "flow instance name already exists: {}",
                    self.flow_instance_name
                ));
            }
            btree_map::Entry::Vacant(entry) => {
                let flow_ctx = Arc::new(flow_ctx);
                entry.insert(flow_ctx.clone());
                flow_ctx
            }
        };
        Ok(Flow(flow_ctx))
    }

    pub async fn build_transient_flow(&self) -> Result<TransientFlow> {
        if self.direct_input_fields.is_empty() {
            return Err(client_error!("expect at least one direct input"));
        }
        let direct_output_value = if let Some(direct_output_value) = &self.direct_output_value {
            direct_output_value
        } else {
            return Err(client_error!("expect direct output"));
        };
        let spec = spec::TransientFlowSpec {
            name: self.flow_instance_name.clone(),
            input_fields: self.direct_input_fields.clone(),
            reactive_ops: self.reactive_ops.clone(),
            output_value: direct_output_value.clone(),
        };

        let analyzed_flow = super::AnalyzedTransientFlow::from_transient_flow(spec).await?;

        Ok(TransientFlow(Arc::new(analyzed_flow)))
    }
}

impl std::fmt::Display for FlowBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Flow instance name: {}\n\n", self.flow_instance_name)?;
        for op in self.import_ops.iter() {
            write!(
                f,
                "Source op {}\n{}\n",
                op.name,
                serde_json::to_string_pretty(&op.spec).unwrap_or_default()
            )?;
        }
        for field in self.direct_input_fields.iter() {
            writeln!(f, "Direct input {}: {}", field.name, field.value_type)?;
        }
        if !self.direct_input_fields.is_empty() {
            writeln!(f)?;
        }
        for op in self.reactive_ops.iter() {
            write!(
                f,
                "Reactive op {}\n{}\n",
                op.name,
                serde_json::to_string_pretty(&op.spec).unwrap_or_default()
            )?;
        }
        for op in self.export_ops.iter() {
            write!(
                f,
                "Export op {}\n{}\n",
                op.name,
                serde_json::to_string_pretty(&op.spec).unwrap_or_default()
            )?;
        }
        if let Some(output) = &self.direct_output_value {
            write!(f, "Direct output: {output}\n\n")?;
        }
        Ok(())
    }
}

impl FlowBuilder {
    fn last_field_to_data_slice(op_scope: &Arc<OpScope>) -> Result<DataSlice> {
        let data_scope = op_scope.data.lock().unwrap();
        let last_field = data_scope.last_field().unwrap();
        let result = DataSlice {
            scope: op_scope.clone(),
            value: Arc::new(spec::ValueMapping::Field(spec::FieldMapping {
                scope: None,
                field_path: spec::FieldPath(vec![last_field.name.clone()]),
            })),
        };
        Ok(result)
    }

    fn minimum_common_scope<'a>(
        scopes: impl Iterator<Item = &'a Arc<OpScope>>,
        target_scope: Option<&'a Arc<OpScope>>,
    ) -> Result<&'a Arc<OpScope>> {
        let mut scope_iter = scopes;
        let mut common_scope = scope_iter
            .next()
            .ok_or_else(|| api_error!("expect at least one input"))?;
        for scope in scope_iter {
            if scope.is_op_scope_descendant(common_scope) {
                common_scope = scope;
            } else if !common_scope.is_op_scope_descendant(scope) {
                api_bail!(
                    "expect all arguments share the common scope, got {} and {} exclusive to each other",
                    common_scope,
                    scope
                );
            }
        }
        if let Some(target_scope) = target_scope {
            if !target_scope.is_op_scope_descendant(common_scope) {
                api_bail!(
                    "the field can only be attached to a scope or sub-scope of the input value. Target scope: {}, input scope: {}",
                    target_scope,
                    common_scope
                );
            }
            common_scope = target_scope;
        }
        Ok(common_scope)
    }

    fn get_mut_reactive_ops<'a>(
        &'a mut self,
        op_scope: &OpScope,
    ) -> Result<&'a mut Vec<spec::NamedSpec<spec::ReactiveOpSpec>>> {
        Self::get_mut_reactive_ops_internal(op_scope, &mut self.reactive_ops)
    }

    fn get_mut_reactive_ops_internal<'a>(
        op_scope: &OpScope,
        root_reactive_ops: &'a mut Vec<spec::NamedSpec<spec::ReactiveOpSpec>>,
    ) -> Result<&'a mut Vec<spec::NamedSpec<spec::ReactiveOpSpec>>> {
        let result = match &op_scope.parent {
            None => root_reactive_ops,
            Some((parent_op_scope, field_path)) => {
                let parent_reactive_ops =
                    Self::get_mut_reactive_ops_internal(parent_op_scope, root_reactive_ops)?;
                // Reuse the last foreach if matched, otherwise create a new one.
                match parent_reactive_ops.last() {
                    Some(spec::NamedSpec {
                        spec: spec::ReactiveOpSpec::ForEach(foreach_spec),
                        ..
                    }) if &foreach_spec.field_path == field_path
                        && foreach_spec.op_scope.name == op_scope.name => {}

                    _ => {
                        api_bail!("already out of op scope `{}`", op_scope.name);
                    }
                }
                match &mut parent_reactive_ops.last_mut().unwrap().spec {
                    spec::ReactiveOpSpec::ForEach(foreach_spec) => &mut foreach_spec.op_scope.ops,
                    _ => unreachable!(),
                }
            }
        };
        Ok(result)
    }
}

pub struct Flow(pub Arc<FlowContext>);

impl Flow {
    pub async fn run(&self) -> Result<()> {
        // Placeholder for run implementation
        Ok(())
    }
}

pub struct TransientFlow(pub Arc<super::AnalyzedTransientFlow>);
