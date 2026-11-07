//! Version thirteen's associated results are structure only: every consumer
//! re-derives their projections, bounds and normal forms from these rows.
use super::{Argument, HeaderIndex, HeaderRef, ParameterKind, SourceRows, Type};
use crate::support::crates::generics::Work;

pub(super) fn validate(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    for header in &rows.associated_types {
        declared(rows, headers, header, work)?;
    }
    for projection in &rows.projections {
        super::operation_contracts::projection_contract(rows, headers, projection, work)?;
    }
    for definition in rows.associated_definitions() {
        work.charge(1)?;
        let scopes = headers.scopes();
        let own = scopes.declared_slots(definition.declaration, work)?;
        if own.start != own.end {
            return Err("associated result definition families require their wider profile");
        }
    }
    Ok(())
}

/// A trait's declared result: no own family inputs or region formals, the
/// implicit sizing promise only, and the canonical projection
/// `<Self as Trait<P..>>::Member` over the trait's own type formals.
fn declared(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::AssociatedTypeHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(4)?;
    let Some(HeaderRef::Trait(index)) = headers.header(header.parent, work)? else {
        return Err("associated result has no actual parent trait");
    };
    let parent = &rows.traits[index as usize];
    super::operation_contracts::sizing(rows, header, work)?;
    if header.inputs.start != header.inputs.end || header.declared_inputs != 0 {
        return Err("associated own-family inputs require their complete wider profile");
    }
    let scopes = headers.scopes();
    let own = scopes.declared_slots(header.scope, work)?;
    let complete = scopes.complete_slots(parent.scope, work)?;
    if own.start != own.end
        || scopes.complete_slots(header.scope, work)? != complete
        || parent.region_captures != 0
        || scopes.complete_type_slots(parent.scope, work)? != complete
    {
        return Err("associated result generic/region family requires its wider explicit profile");
    }
    let Some(Type::Projection(reference)) = rows.types.get(header.projection.0 as usize) else {
        return Err("associated result canonical header lost its actual projection");
    };
    let projection = rows
        .projections
        .get(reference.0 as usize)
        .ok_or("associated canonical projection is unknown")?;
    let application = rows
        .trait_applications
        .get(projection.trait_application.0 as usize)
        .ok_or("associated canonical projection lost its trait application")?;
    if projection.declaration.artifact.0 != 0
        || projection.declaration.declaration != header.declaration
        || projection.subject != parent.self_type
        || application.subject != parent.self_type
    {
        return Err("associated canonical projection changed its actual member/Self identity");
    }
    let Some(Type::Parameter(self_slot)) = rows.types.get(parent.self_type.0 as usize) else {
        return Err("associated canonical projection lost its trait Self formal");
    };
    let declared = scopes.declared_slots(parent.scope, work)?;
    let mut formals = (declared.start..declared.end).filter(|&slot| slot != self_slot.0);
    for argument in application.arguments.slice(&rows.arguments)? {
        work.charge(1)?;
        let formal = formals.next();
        let Argument::Type(ty) = *argument else {
            return Err("associated canonical projection has a non-type trait argument");
        };
        if !matches!(rows.types.get(ty.0 as usize), Some(Type::Parameter(slot)) if Some(slot.0) == formal)
            || formal
                .and_then(|slot| rows.slots.get(slot as usize))
                .is_none_or(|slot| !matches!(slot.kind, ParameterKind::Type { .. }))
        {
            return Err("associated canonical projection is not its trait's identity application");
        }
    }
    if formals.next().is_some() {
        return Err("associated canonical projection omitted a trait formal");
    }
    Ok(())
}
