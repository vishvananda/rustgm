//! The graph supplies the defining header after a packet has sealed its own roots.
use super::{Interface, TypedExports, TypedTargets, s};
use crate::support::{constness::Constness, crates::owner_catalog::DeclarationRef};

pub(super) fn check(
    targets: &mut TypedTargets<'_>,
    artifact: &Interface,
    rows: s::ValidatedView<'_>,
) -> Result<(), &'static str> {
    for application in &rows.function_applications {
        let address = application.declaration;
        if address.artifact.0 == 0 {
            continue;
        }
        let exports = targets.target(artifact, address, 2)?;
        let header = function(exports.rows, address.declaration)?;
        let slots = formals(exports, header.scope)?;
        let regions = header
            .regions
            .early
            .slice(&exports.rows.region_slots)?
            .len();
        let actuals = application.arguments.slice(&rows.arguments)?;
        let (early, actuals) = actuals
            .split_at_checked(regions)
            .ok_or("source dependency application omitted early regions")?;
        if early
            .iter()
            .any(|argument| !matches!(argument, s::Argument::Region(_)))
        {
            return Err("source dependency application changed an early region kind");
        }
        arguments(
            rows,
            exports.rows,
            actuals,
            slots
                .iter()
                .filter(|slot| slot.kind != s::ParameterKind::Region),
        )?;
    }
    for application in &rows.trait_applications {
        let address = application.declaration;
        if address.artifact.0 == 0 {
            continue;
        }
        let exports = targets.target(artifact, address, 1)?;
        let Some(s::HeaderRef::Trait(index)) = exports.rows.header(address.declaration)? else {
            return Err("source dependency application has no actual trait header");
        };
        let header = &exports.rows.traits[index as usize];
        arguments(
            rows,
            exports.rows,
            application.arguments.slice(&rows.arguments)?,
            formals(exports, header.scope)?.iter(),
        )?;
    }
    for original in &rows.originals {
        calls(targets, artifact, rows, original)?;
    }
    Ok(())
}

fn formals<'a>(
    exports: &'a TypedExports<'_>,
    scope: DeclarationRef,
) -> Result<&'a [s::ParameterSlot], &'static str> {
    exports
        .slots
        .get(&scope)
        .ok_or("source dependency application has no actual formal scope")?
        .slice(&exports.rows.slots)
}

fn function(
    rows: s::ValidatedView<'_>,
    declaration: DeclarationRef,
) -> Result<s::FunctionHeader, &'static str> {
    let Some(s::HeaderRef::Function(index)) = rows.header(declaration)? else {
        return Err("source dependency application has no actual callable header");
    };
    let header = rows.functions[index as usize];
    if header.parent.is_some() || header.regions.captures != 0 {
        return Err("source dependency call requires an actual free callable");
    }
    Ok(header)
}

fn arguments<'a>(
    caller: s::ValidatedView<'_>,
    target: s::ValidatedView<'_>,
    actuals: &[s::Argument],
    mut formals: impl Iterator<Item = &'a s::ParameterSlot>,
) -> Result<(), &'static str> {
    for argument in actuals {
        match (*argument, formals.next().map(|slot| slot.kind)) {
            (s::Argument::Type(_), Some(s::ParameterKind::Type { .. }))
            | (s::Argument::Region(_), Some(s::ParameterKind::Region)) => {}
            (s::Argument::Const(term), Some(s::ParameterKind::Const { ty }))
                if const_kind(caller, term)? == s::const_parameter_type(&target.types, ty)? => {}
            _ => return Err("source dependency application changed its actual formal kind"),
        }
    }
    if formals.next().is_some() {
        return Err("source dependency application omitted an actual formal");
    }
    Ok(())
}

fn const_kind(
    rows: s::ValidatedView<'_>,
    term: s::ConstRef,
) -> Result<crate::support::crates::Scalar, &'static str> {
    let ty = match rows.consts()[term.0 as usize] {
        s::ConstTerm::Value(value) => value.ty,
        s::ConstTerm::Variant(value) => value.ty,
        s::ConstTerm::Parameter(slot) => {
            let s::ParameterKind::Const { ty } = rows.slots[slot.0 as usize].kind else {
                return Err("source dependency const changed its actual formal kind");
            };
            ty
        }
    };
    s::const_parameter_type(&rows.types, ty)
}

fn calls(
    targets: &mut TypedTargets<'_>,
    artifact: &Interface,
    rows: s::ValidatedView<'_>,
    original: &s::Original,
) -> Result<(), &'static str> {
    for request in &original.calls {
        let address = rows.function_applications[request.target.0 as usize].declaration;
        if address.artifact.0 == 0 {
            continue;
        }
        let exports = targets.target(artifact, address, 2)?;
        let header = function(exports.rows, address.declaration)?;
        if original.context != s::CallContext::Runtime && header.constness != Constness::Const {
            return Err("source const call requires its actual const dependency target");
        }
        let s::Type::Function(signature) = rows.types[request.signature.0 as usize] else {
            return Err("source dependency call requires its actual callable signature");
        };
        let callable = header.callable;
        if signature.convention != callable.convention
            || signature.unsafe_ != callable.unsafe_
            || signature.variadic != callable.variadic
        {
            return Err("source dependency call changed its actual callable flags");
        }
        let late = header.regions.late.end - header.regions.late.start;
        match request.opening {
            None if late == 0 => {}
            Some(opening)
                if late != 0
                    && opening.binder == callable.binder
                    && opening.regions.slice(&original.openings)?.len() == late as usize => {}
            _ => return Err("source dependency call changed its actual callable binder"),
        }
    }
    Ok(())
}
