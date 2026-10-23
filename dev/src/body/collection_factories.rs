//! Collection bodies enter ordinary original-template lifetime and ownership checks.
mod mutable_next;
mod next;
use super::{
    ir::{
        Action, Body, Local, LocalId, LocalKind, Owner, Place, Projection, ProjectionId,
        ProjectionKind, Rvalue, ValueList,
    },
    provided::{statement, value},
    regions::{Contract, Kind as Debt, Obligation},
    templates::{Template, Templates},
};
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{iteration::collections::Original, model::Declarations},
    types::{Kind, TypeId},
};
use std::sync::Arc;
pub(super) fn append(
    report: &Report,
    facts: &mut Declarations,
    templates: &mut Templates,
    contracts: &mut Vec<Contract>,
) -> Result<(), Error> {
    let originals = facts
        .iteration
        .as_deref()
        .and_then(|r| r.collections.as_deref())
        .map(|r| r.all_originals().copied().collect::<Vec<_>>())
        .unwrap_or_default();
    for original in originals {
        let source = facts
            .iteration
            .as_deref()
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing iteration registry",
                )
            })?
            .canonical
            .origin;
        let error = |e| report.body_error(source, e);
        original.validate(facts).map_err(error)?;
        let environment = Arc::new(
            facts
                .generics
                .parameter_environment(&facts.types, original.declaration.method)
                .map_err(error)?,
        );
        let mut contract = super::regions::callable(
            facts,
            original.declaration.method,
            original.declaration.method,
            source,
            original.signature,
        )
        .map_err(error)?;
        let mut body = build(facts, original, &mut contract).map_err(error)?;
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
    }
    Ok(())
}
fn build(
    facts: &mut Declarations,
    original: Original,
    contract: &mut Contract,
) -> Result<Body, &'static str> {
    original.validate(facts)?;
    let source = facts
        .iteration
        .as_deref()
        .ok_or("missing collection registry")?
        .canonical
        .origin;
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(original.signature)
    else {
        return Err("invalid collection signature");
    };
    let [receiver] = facts
        .types
        .list(parameters)
        .ok_or("missing collection receiver")?
    else {
        return Err("invalid collection arity");
    };
    let receiver = *receiver;
    facts
        .aggregates
        .prepare(&mut facts.types, original.iterator)?;
    facts.aggregates.prepare(&mut facts.types, result)?;
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
    local(&mut body, receiver, LocalKind::Parameter(0), false)?;
    if original.declaration.kind.next() {
        next::build(facts, original, &mut body)?;
    } else {
        let slice = slice_reference(facts, original)?;
        let input = value(&mut body, receiver, Rvalue::Read(Place::Local(LocalId(0))))?;
        let field = if receiver == slice {
            input
        } else {
            let Some(Kind::Reference {
                referent: array,
                mutable,
                ..
            }) = facts.types.kind(receiver)
            else {
                return Err("invalid collection array receiver");
            };
            if mutable != original.declaration.kind.mutable() {
                return Err("collection array receiver has different permission");
            }
            if !matches!(
                facts.types.kind(array),
                Some(Kind::Array { .. } | Kind::ArrayTerm { .. })
            ) {
                return Err("collection unsizing requires array extent");
            }
            contract.obligations.push(Obligation {
                source: source.into(),
                kind: Debt::Coercion {
                    actual: receiver,
                    expected: slice,
                    unsize: true,
                    reference: None,
                },
            });
            value(
                &mut body,
                slice,
                Rvalue::Unsize {
                    value: input,
                    conversion: crate::types::unsizing::Conversion::Slice(array),
                },
            )?
        };
        let variant = iterator_variant(facts, original)?;
        body.arguments.push(field);
        let returned = value(
            &mut body,
            result,
            Rvalue::Aggregate {
                variant: Some(variant),
                fields: ValueList { start: 0, end: 1 },
            },
        )?;
        super::build::finish_factory(&mut body, returned)?;
    }
    Ok(body)
}
pub(super) fn local(
    body: &mut Body,
    ty: TypeId,
    kind: LocalKind,
    mutable: bool,
) -> Result<LocalId, &'static str> {
    let id = LocalId(u32::try_from(body.locals.len()).map_err(|_| "too many collection locals")?);
    body.locals.push(Local {
        scope: super::scopes::Id(0),
        storage_parent: id.0.checked_sub(1).map(LocalId),
        ty,
        kind,
        definition: None,
        mutable,
        source: body.root_site(),
    });
    statement(body, Action::StorageLive(id));
    Ok(id)
}
pub(super) fn projection(
    body: &mut Body,
    base: Place,
    ty: TypeId,
    kind: ProjectionKind,
) -> Result<Place, &'static str> {
    let id = ProjectionId(
        u32::try_from(body.projections.len()).map_err(|_| "too many collection projections")?,
    );
    body.projections.push(Projection { base, kind, ty });
    statement(body, Action::Project(id));
    Ok(Place::Projection(id))
}
fn dereference(facts: &Declarations, body: &mut Body, place: Place) -> Result<Place, &'static str> {
    let Some(Kind::Reference {
        referent,
        region,
        mutable,
    }) = body.place_type(place).and_then(|t| facts.types.kind(t))
    else {
        return Err("collection dereference is not a reference");
    };
    projection(
        body,
        place,
        referent,
        ProjectionKind::Dereference { region, mutable },
    )
}
fn iterator_variant(
    facts: &Declarations,
    original: Original,
) -> Result<crate::sema::members::VariantId, &'static str> {
    let Some(Kind::Adt(owner, _)) = facts.types.kind(original.iterator) else {
        return Err("collection iterator is not nominal");
    };
    let [variant] = facts.members.variants(owner) else {
        return Err("collection iterator requires one record");
    };
    Ok(*variant)
}
fn slice_reference(facts: &Declarations, original: Original) -> Result<TypeId, &'static str> {
    let shape = facts.aggregates.shape(&facts.types, original.iterator)?;
    let mut variants = shape.variants();
    let variant = variants.next().ok_or("missing collection iterator shape")?;
    if variants.next().is_some() {
        return Err("invalid collection iterator shape");
    }
    let [field] = facts
        .types
        .list(variant.fields)
        .ok_or("missing collection iterator fields")?
    else {
        return Err("invalid collection iterator fields");
    };
    Ok(*field)
}
