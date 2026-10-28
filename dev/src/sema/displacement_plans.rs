//! Single-field offsets retain actual source members and an ordinary source callable.
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, ItemContext, Namespace},
        package::Error,
        unit::NodeKey,
    },
    sema::{
        bodies::SourceCallUse,
        iteration::Call,
        members::{FieldId, Name, VariantId},
        model::Declarations,
    },
    syntax::{
        ast::{Kind, NodeId},
        token_tree::Span,
    },
    types::{ItemId, Kind as Type, TypeId},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Plan {
    pub node: NodeKey,
    pub source_use: SourceCallUse,
    pub container: TypeId,
    pub field: FieldId,
    pub call: Call,
    pub module: crate::resolve::package::ModuleId,
}

/// Fixed scratch rejects wider paths before allocating or traversing their tail.
pub(crate) fn components(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
    fields: NodeId,
) -> Result<[(NodeId, Span); 2], Error> {
    let mut parts = [(fields, Span { start: 0, end: 0 }); 2];
    let mut count = 0;
    let ast = &report.package.unit(key.file).configured.ast;
    crate::syntax::offset_fields::visit(ast, fields, |node, span| {
        super::header_attributes::charge_source_contract_query(facts, 1).map_err(|_| {
            crate::syntax::token_tree::error(span.start, "offset field work budget exceeded")
        })?;
        let slot = parts.get_mut(count).ok_or_else(|| {
            crate::syntax::token_tree::error(span.start, "nested offset fields are not implemented")
        })?;
        *slot = (node, span);
        count += 1;
        Ok(())
    })
    .map_err(|error| Error::at(key.file, error.offset, error.message))?;
    if count == 1 {
        parts[1] = parts[0];
    }
    Ok(parts)
}

pub(crate) fn variant(
    report: &Report,
    facts: &Declarations,
    key: NodeKey,
    owner: ItemId,
    part: (NodeId, Span),
    module: crate::resolve::package::ModuleId,
) -> Result<VariantId, Error> {
    let error = |message| Declarations::error(report, key, message);
    if !report.package.requested_offset_of_enum().map_err(error)? {
        return Err(error(
            "enum offset requires the crate's offset_of_enum feature",
        ));
    }
    super::header_attributes::charge_source_contract_query(facts, 4)?;
    let ast = &report.package.unit(key.file).configured.ast;
    let name_node = match ast.node(part.0).kind {
        Kind::Path {
            absolute: false,
            segments,
        } => match ast.list(segments) {
            [name] => *name,
            _ => return Err(error("offset variant must be unqualified")),
        },
        Kind::Name(..) => part.0,
        _ => return Err(error("offset variant requires its actual name")),
    };
    let Kind::Name(span, parsed) = ast.node(name_node).kind else {
        return Err(error("offset variant has no actual source name"));
    };
    if span != part.1 {
        return Err(error("offset variant source name changed"));
    }
    let spelling = report
        .package
        .unit(key.file)
        .spelling(span)
        .map_err(|e| error(e.message))?;
    let name = report
        .package
        .names
        .lookup_paid(spelling, &mut |growth| {
            super::header_attributes::charge_source_retention(facts, growth.work, growth.bytes)
                .map_err(|_| "offset variant name budget exhausted")
        })
        .map_err(error)?
        .ok_or_else(|| error("unknown offset enum variant"))?;
    if parsed.is_some_and(|parsed| parsed != name) {
        return Err(error("offset variant changed its canonical source name"));
    }
    let definition = report
        .resolution
        .definitions
        .get(owner.0 as usize)
        .ok_or_else(|| error("offset nominal owner disappeared"))?;
    let scope = definition
        .members
        .ok_or_else(|| error("offset enum has no member scope"))?;
    let context = report.package.item_context(key.file, span.start)?;
    let adjusted = report
        .adjust_item(DefId(owner.0), context, module)
        .map_err(error)?;
    let binding = report
        .resolution
        .bindings
        .get_key((scope, name, Namespace::Type, adjusted.context))
        .filter(|binding| !binding.ambiguous)
        .ok_or_else(|| error("unknown offset enum variant"))?;
    let definition = &report.resolution.definitions[binding.target.0 as usize];
    let variant = definition
        .node
        .and_then(|node| facts.members.at_node(node))
        .ok_or_else(|| error("offset variant has no actual typed member"))?;
    if definition.kind != DefKind::Variant
        || facts
            .members
            .variants
            .get(variant.0 as usize)
            .is_none_or(|v| v.owner != owner)
    {
        return Err(error("offset variant belongs to another nominal owner"));
    }
    Ok(variant)
}

/// Reauthenticate actual source and selection before the CFG emits an intrinsic call.
pub(crate) fn checked(
    report: &Report,
    facts: &Declarations,
    owner: NodeKey,
    node: NodeKey,
    id: super::bodies::OffsetPlanId,
) -> Result<Plan, Error> {
    let error = |message| Declarations::error(report, node, message);
    super::header_attributes::charge_source_contract_query(facts, 8)?;
    let plan = id.current(facts, owner, node).map_err(error)?;
    validate(report, facts, plan)?;
    Ok(plan)
}
pub(crate) fn validate(report: &Report, facts: &Declarations, plan: Plan) -> Result<(), Error> {
    let node = plan.node;
    let error = |message| Declarations::error(report, node, message);
    crate::resolve::require_offset(report, node)?;
    let ast = &report.package.unit(node.file).configured.ast;
    let Kind::OffsetOf {
        container, fields, ..
    } = ast.node(node.node).kind
    else {
        return Err(error("offset source operation changed"));
    };
    if facts
        .node(NodeKey {
            node: container,
            ..node
        })
        .and_then(|fact| fact.ty)
        != Some(plan.container)
    {
        return Err(error(
            "offset container differs from its actual typed source",
        ));
    }
    let Some(Type::Adt(item, _)) = facts.types.kind(plan.container) else {
        return Err(error(
            "offset container must be a nominal type without autoderef",
        ));
    };
    let identity = facts
        .members
        .identity(plan.field)
        .ok_or_else(|| error("offset field identity disappeared"))?;
    if identity.owner.0 != item.0 {
        return Err(error("offset field belongs to another nominal owner"));
    }
    let parts = components(report, facts, node, fields)?;
    let module = plan.module;
    check_owner(report, facts, plan)?;
    let selected_variant = if report.resolution.definitions[item.0 as usize].kind == DefKind::Enum {
        if parts[0].0 == parts[1].0 {
            return Err(error("enum offset requires a variant and field"));
        }
        variant(report, facts, node, item, parts[1], module)?
    } else {
        if parts[0].0 != parts[1].0 {
            return Err(error("nested offset fields are not implemented"));
        }
        let [variant] = facts.members.variants(item) else {
            return Err(error("offset container has no unique storage variant"));
        };
        *variant
    };
    if facts.members.variant(plan.field) != Some(selected_variant) {
        return Err(error("offset field changed its actual variant"));
    }
    let variant = facts
        .members
        .variants
        .get(selected_variant.0 as usize)
        .ok_or_else(|| error("offset storage variant disappeared"))?;
    if facts
        .members
        .variants(variant.owner)
        .get(identity.variant as usize)
        != Some(&selected_variant)
        || variant.start.checked_add(identity.field) != Some(plan.field.0)
        || plan.field.0 >= variant.end
    {
        return Err(error(
            "offset member coordinates differ from their actual typed lists",
        ));
    }
    check_field(report, facts, plan, selected_variant, parts[0].1)?;
    facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("offset source catalog disappeared"))?
        .require_offset_callable(report, facts, plan.call.callable, plan.container)
        .map_err(error)?;
    if !matches!(facts.types.kind(plan.call.callable), Some(Type::FunctionItem {signature,..}) if signature == plan.call.signature)
    {
        return Err(error("offset callable signature changed"));
    }
    Ok(())
}
fn check_owner(report: &Report, facts: &Declarations, plan: Plan) -> Result<(), Error> {
    let error = |message| Declarations::error(report, plan.node, message);
    let owner = plan.source_use.owner(facts).map_err(error)?;
    let scope = report
        .resolution
        .node_scopes
        .get(&(plan.module, owner.node))
        .and_then(|scope| report.resolution.scopes.get(scope.0 as usize));
    if owner.file != plan.node.file || scope.is_none_or(|scope| scope.module != plan.module) {
        return Err(error(
            "offset plan changed its actual source body or module",
        ));
    }
    Ok(())
}
fn check_field(
    report: &Report,
    facts: &Declarations,
    plan: Plan,
    selected_variant: VariantId,
    span: Span,
) -> Result<(), Error> {
    let node = plan.node;
    let error = |message| Declarations::error(report, node, message);
    let field = facts
        .members
        .fields
        .get(plan.field.0 as usize)
        .ok_or_else(|| error("offset field disappeared"))?;
    let spelling = report
        .package
        .unit(node.file)
        .spelling(span)
        .map_err(|e| error(e.message))?;
    super::header_attributes::charge_source_contract_query(
        facts,
        spelling.len().saturating_add(1),
    )?;
    let name_matches = match field.name {
        Name::Named(name) => {
            let spelling = spelling.strip_prefix("r#").unwrap_or(spelling);
            if spelling.is_ascii() {
                facts.members.names.get(name) == spelling
            } else {
                let normalized =
                    crate::support::names::Names::normalize_paid(spelling, &mut |growth| {
                        super::header_attributes::charge_source_retention(
                            facts,
                            growth.work,
                            growth.bytes,
                        )
                        .map_err(|_| "offset member normalization budget exhausted")
                    })
                    .map_err(error)?;
                facts.members.names.get(name) == normalized
            }
        }
        Name::Index(index) => {
            spelling.parse::<u32>() == Ok(index)
                && (spelling.len() == 1 || !spelling.starts_with('0'))
                && spelling.bytes().all(|byte| byte.is_ascii_digit())
        }
    };
    let context = report.package.item_context(node.file, span.start)?;
    let adjusted = report
        .adjust_item(
            facts
                .members
                .identity(plan.field)
                .ok_or_else(|| error("offset field identity disappeared"))?
                .owner,
            context,
            plan.module,
        )
        .map_err(error)?;
    let field_context = if matches!(field.name, Name::Index(_)) {
        ItemContext::ROOT
    } else {
        adjusted.context
    };
    let private = match field.access {
        crate::resolve::model::Access::ExternalPrivate => true,
        crate::resolve::model::Access::Within(module) => {
            !report.package.is_descendant(adjusted.privacy, module)
        }
        crate::resolve::model::Access::Public => false,
    };
    if !name_matches
        || private
        || facts
            .members
            .field_in(selected_variant, field.name, field_context)
            != Some(plan.field)
    {
        return Err(error(
            "offset field differs from its contextual source selection or privacy",
        ));
    }
    Ok(())
}
