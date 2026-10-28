//! `Iter::clone` copies its stored shared view through an ordinarily checked original.
use super::{
    ir::{Body, LocalKind, Owner, Place, ProjectionKind, Rvalue, ValueList},
    provided::value,
    regions::Contract,
    templates::{Template, Templates},
};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{
        cloning::Original,
        members::{FieldId, Name, Selection, VariantId},
        model::Declarations,
    },
    types::{Kind, TypeId},
};
use std::sync::Arc;
pub(super) fn append(
    report: &Report,
    facts: &mut Declarations,
    templates: &mut Templates,
    contracts: &mut Vec<Contract>,
) -> Result<(), Error> {
    let Some(original) = facts.cloning.as_deref().and_then(|r| r.original) else {
        return Ok(());
    };
    let source = facts
        .cloning
        .as_deref()
        .ok_or_else(|| Error::at(crate::resolve::unit::FileId(0), 0, "missing Clone registry"))?
        .canonical
        .origin;
    let error = |e| report.body_error(source, e);
    original.validate(facts).map_err(error)?;
    let environment = Arc::new(
        facts
            .source_parameter_environment(report, source, original.declaration.method)
            .map_err(error)?,
    );
    let contract = super::regions::callable(
        facts,
        original.declaration.method,
        original.declaration.method,
        source,
        original.signature,
    )
    .map_err(error)?;
    let mut body = build(facts, original).map_err(error)?;
    let mut query = crate::types::destruction::NeedsDrop::new(
        facts.destructors.implementations().iter().map(|i| i.item),
    )
    .scoped(Arc::clone(&environment));
    super::build::factory_owned_values(facts, &mut body, &mut query).map_err(error)?;
    let destruction = super::drops::requirements(facts, &body, &mut query).map_err(error)?;
    contracts.push(contract);
    templates.bodies.push(Template {
        body,
        destruction,
        environment,
    });
    Ok(())
}
fn build(facts: &mut Declarations, original: Original) -> Result<Body, &'static str> {
    original.validate(facts)?;
    let source = facts
        .cloning
        .as_deref()
        .ok_or("missing Clone registry")?
        .canonical
        .origin;
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(original.signature)
    else {
        return Err("invalid Clone signature");
    };
    let [receiver] = facts
        .types
        .list(parameters)
        .ok_or("missing Clone receiver")?
    else {
        return Err("invalid Clone arity");
    };
    let receiver = *receiver;
    let Some(Kind::Reference {
        referent,
        region,
        mutable: false,
    }) = facts.types.kind(receiver)
    else {
        return Err("Clone receiver is not shared");
    };
    if referent != original.subject {
        return Err("Clone receiver differs from selected Self");
    }
    let (variant, field, field_type) = view_field(facts, original)?;
    let mut body = Body::new(
        Owner::Function {
            definition: original.declaration.method,
            signature: original.signature,
            execution: None,
        },
        source,
        result,
    );
    body.storage = Some(super::storage::Facts::new(super::scopes::Facts::synthetic(
        source.node,
    )));
    let input =
        super::collection_factories::local(&mut body, receiver, LocalKind::Parameter(0), false)?;
    let root = super::collection_factories::projection(
        &mut body,
        Place::Local(input),
        original.subject,
        ProjectionKind::Dereference {
            region,
            mutable: false,
        },
    )?;
    let field = super::collection_factories::projection(
        &mut body,
        root,
        field_type,
        ProjectionKind::Field(Selection::Nominal(field)),
    )?;
    let copied = value(&mut body, field_type, Rvalue::Read(field))?;
    body.arguments.push(copied);
    let returned = value(
        &mut body,
        result,
        Rvalue::Aggregate {
            variant: Some(variant),
            fields: ValueList { start: 0, end: 1 },
        },
    )?;
    super::build::finish_factory_block(&mut body, 0, returned)?;
    Ok(body)
}

fn view_field(
    facts: &mut Declarations,
    original: Original,
) -> Result<(VariantId, FieldId, TypeId), &'static str> {
    facts
        .aggregates
        .prepare(&mut facts.types, original.subject)?;
    let Some(Kind::Adt(item, _)) = facts.types.kind(original.subject) else {
        return Err("Clone Self is not canonical Iter");
    };
    let [variant] = facts.members.variants(item) else {
        return Err("Clone Self is not a record");
    };
    let variant = *variant;
    let name = facts.members.names.intern("value")?;
    let field = facts
        .members
        .field(variant, Name::Named(name))
        .ok_or("missing canonical iterator view field")?;
    let shape = facts.aggregates.shape(&facts.types, original.subject)?;
    let mut shapes = shape.variants();
    let fields = shapes.next().ok_or("missing iterator shape")?.fields;
    if shapes.next().is_some() {
        return Err("invalid iterator variant shape");
    }
    let [field_type] = facts.types.list(fields).ok_or("missing iterator fields")? else {
        return Err("invalid iterator field count");
    };
    let field_type = *field_type;
    let Some(Kind::Reference {
        referent: slice,
        mutable: false,
        region: source_region,
    }) = facts.types.kind(field_type)
    else {
        return Err("iterator Clone field is not a shared reference");
    };
    if facts.types.kind(slice) != Some(Kind::Slice(original.parameter))
        || source_region != crate::types::RegionId::bound(0, 0)?
    {
        return Err("iterator Clone field differs from registered source view");
    }
    Ok((variant, field, field_type))
}
