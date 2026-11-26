//! Version thirteen's associated results are structure only: every consumer
//! re-derives their projections, bounds and normal forms from these rows.
use super::{
    Argument, BinderKind, HeaderIndex, HeaderRef, ParameterKind, Profile, Region, SourceRows, Type,
};
use crate::support::crates::generics::Work;

pub(super) fn validate(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    let families = profile.declaration_policies();
    for header in &rows.associated_types {
        declared(rows, headers, header, profile, work)?;
    }
    for projection in &rows.projections {
        super::operation_contracts::projection_contract(rows, headers, projection, families, work)?;
    }
    for definition in rows.associated_definitions() {
        work.charge(1)?;
        let scopes = headers.scopes();
        let own = scopes.declared_slots(definition.declaration, work)?;
        if own.start != own.end {
            let parent = scopes
                .parent(definition.declaration, work)?
                .ok_or("associated result definition lost its actual impl scope")?;
            let Some(HeaderRef::Implementation(index)) = headers.header(parent, work)? else {
                return Err("associated result definition has a foreign actual impl scope");
            };
            let captures = rows.implementations[index as usize].region_captures;
            if !families {
                return Err("associated result definition families require their wider profile");
            }
            let mut region_offset = 0_u32;
            let mut seen_non_region = false;
            for (index, &slot) in (own.start..own.end).zip(own.slice(&rows.slots)?) {
                work.charge(1)?;
                if slot.definition.is_none()
                    || slot.default.is_some()
                    || super::const_defaults::find(
                        &rows.cold().const_defaults,
                        super::SlotRef(index),
                    )
                    .is_some()
                {
                    return Err("associated definition changed its actual owned generic slots");
                }
                match slot.kind {
                    ParameterKind::Region => {
                        if seen_non_region
                            || Some(slot.index) != captures.checked_add(region_offset)
                        {
                            return Err(
                                "associated definition changed its actual owned lifetime slots",
                            );
                        }
                        region_offset = region_offset
                            .checked_add(1)
                            .ok_or("associated definition lifetime count overflow")?;
                    }
                    ParameterKind::Type { .. } | ParameterKind::Const { .. } => {
                        seen_non_region = true;
                    }
                }
            }
        }
    }
    for row in &rows.cold().associated_definition_inputs {
        work.charge(1)?;
        let declaration = rows.associated_definitions()[row.index as usize].declaration;
        let own = headers.scopes().declared_slots(declaration, work)?;
        if !families || own.start == own.end {
            return Err(
                "associated definition premises require their exact version-fourteen family",
            );
        }
        for input in row.inputs.slice(&rows.clauses)? {
            work.charge(1)?;
            let valid = match input.requirement {
                super::Requirement::TypeOutlives { .. } => input.binder.is_none(),
                super::Requirement::Trait { .. }
                | super::Requirement::ProjectionEqual { .. }
                | super::Requirement::Bootstrap { .. } => true,
                super::Requirement::Outlives(_) | super::Requirement::UnstableFeature(_) => false,
            };
            if !valid {
                return Err("associated definition premises require their actual family inputs");
            }
        }
    }
    Ok(())
}

/// The canonical projection applies the actual trait and member formals to
/// their own slots. Its result bounds and formation inputs remain separate.
fn declared(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &super::AssociatedTypeHeader,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(4)?;
    let Some(HeaderRef::Trait(index)) = headers.header(header.parent, work)? else {
        return Err("associated result has no actual parent trait");
    };
    let parent = &rows.traits[index as usize];
    super::operation_contracts::sizing(rows, header, profile.bootstrap_bindings(), work)?;
    let families = profile.declaration_policies();
    if !families && (header.inputs.start != header.inputs.end || header.declared_inputs != 0) {
        return Err("associated own-family inputs require their complete wider profile");
    }
    let scopes = headers.scopes();
    let own = scopes.declared_slots(header.scope, work)?;
    let complete = scopes.complete_slots(parent.scope, work)?;
    if !families
        && (own.start != own.end
            || scopes.complete_slots(header.scope, work)? != complete
            || parent.region_captures != 0
            || scopes.complete_type_slots(parent.scope, work)? != complete)
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
    let formals = (declared.start..declared.end).filter(|&slot| slot != self_slot.0);
    identity_application(rows, header, application.arguments, formals, work)?;
    identity_application(rows, header, projection.arguments, own.start..own.end, work)
}

fn identity_application(
    rows: &SourceRows,
    header: &super::AssociatedTypeHeader,
    arguments: super::Range,
    mut formals: impl Iterator<Item = u32>,
    work: &mut Work,
) -> Result<(), &'static str> {
    let error = "associated canonical projection is not its declaration's identity application";
    for &argument in arguments.slice(&rows.arguments)? {
        work.charge(1)?;
        let index = formals.next().ok_or(error)?;
        let formal = rows.slots.get(index as usize).ok_or(error)?;
        let same = match (argument, formal.kind) {
            (Argument::Type(ty), ParameterKind::Type { .. }) => {
                matches!(rows.types.get(ty.0 as usize), Some(Type::Parameter(slot)) if slot.0 == index)
            }
            (Argument::Region(region), ParameterKind::Region) => {
                matches!(rows.regions.get(region.0 as usize),
                    Some(Region::Bound { binder, depth: 0, slot }) if *slot == formal.index
                        && rows.binders.get(binder.0 as usize).is_some_and(|frame|
                            frame.owner == header.declaration && frame.kind == BinderKind::DeclarationHeader))
            }
            _ => false,
        };
        if !same {
            return Err(error);
        }
    }
    if formals.next().is_some() {
        return Err("associated canonical projection omitted a declared formal");
    }
    Ok(())
}
