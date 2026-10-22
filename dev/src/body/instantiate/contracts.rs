//! Specialize typed obligations while separating caller and callee lifetime scopes.
#[cfg(test)]
#[path = "../../../tests/invariants/uncaptured_debts.rs"]
mod tests;
use super::{Bound, Declarations, Inputs};
use crate::{
    body::regions::{Contract, Kind},
    types::{Kind as Type, RegionId, arguments::Argument, instance::Instance, outlives::Queries},
};

pub(super) fn specialize(
    facts: &mut Declarations,
    original: &Contract,
    inputs: &mut Inputs,
    premises: super::Premises,
) -> Result<(Bound, Contract), &'static str> {
    let (bound, contract, _) = prepare(facts, original, inputs, premises, None, false, None)?;
    Ok((bound, contract))
}
pub(super) fn initializer(
    facts: &mut Declarations,
    original: &Contract,
    inputs: &mut Inputs,
) -> Result<(Bound, Contract), &'static str> {
    let close_static = !inputs.captures.is_empty()
        && inputs
            .captures
            .iter()
            .all(|&region| region == RegionId::STATIC);
    let (bound, contract, _) = prepare(
        facts,
        original,
        inputs,
        super::Premises::Required,
        None,
        close_static,
        None,
    )?;
    Ok((bound, contract))
}
pub(super) fn source_result(
    facts: &mut Declarations,
    original: &Contract,
    inputs: &mut Inputs,
    requested: crate::types::TypeId,
) -> Result<(Bound, Contract, crate::types::TypeId), &'static str> {
    let (bound, contract, imported) = prepare(
        facts,
        original,
        inputs,
        super::Premises::Required,
        Some(requested),
        false,
        None,
    )?;
    Ok((
        bound,
        contract,
        imported.ok_or("source result import was lost")?,
    ))
}
pub(super) fn rigid_initializer(
    facts: &mut Declarations,
    original: &Contract,
    inputs: &mut Inputs,
    parameters: Vec<crate::types::parameters::ParameterId>,
) -> Result<(Bound, Contract), &'static str> {
    let (bound, contract, _) = prepare(
        facts,
        original,
        inputs,
        super::Premises::Required,
        None,
        false,
        Some(parameters),
    )?;
    Ok((bound, contract))
}
fn prepare(
    facts: &mut Declarations,
    original: &Contract,
    inputs: &mut Inputs,
    premises: super::Premises,
    mut requested: Option<crate::types::TypeId>,
    close_static: bool,
    parameters: Option<Vec<crate::types::parameters::ParameterId>>,
) -> Result<(Bound, Contract, Option<crate::types::TypeId>), &'static str> {
    let mut contract = original.clone();
    if requested.is_some() && matches!(original.key, crate::body::ir::BodyKey::Anonymous(_)) {
        // Reprove the complete child without assuming its conditional conclusion.
        super::closures::formation(&mut contract);
    }
    super::caller_transport::contract(facts, &mut contract, inputs.caller)?;
    let regions = import_regions(facts, inputs, original.universals, &mut requested)?;
    let mut bound = if close_static {
        Bound::parameters_captured(&facts.types, &inputs.bindings, inputs.captures.clone())?
    } else {
        Bound::parameters(&facts.types, &inputs.bindings)?
    };
    if matches!(original.key, crate::body::ir::BodyKey::Closure(_)) {
        super::closures::formation(&mut contract);
    }
    if inputs.connections.is_some() && matches!(premises, super::Premises::Declared) {
        let crate::body::ir::BodyKey::Closure(identity) = original.key else {
            return Err("captured instance has a different source owner");
        };
        let conditions = facts
            .closures
            .conditions(identity)
            .ok_or("captured instance requires checked source execution conditions")?;
        contract.assumptions.extend_from_slice(conditions);
    }
    if let Some(pending) = &mut contract.builtins {
        let pending = std::sync::Arc::make_mut(pending);
        pending.environment = None;
        for (_, predicate) in &mut pending.predicates {
            *predicate = bound.predicate(&mut facts.types, *predicate)?;
            facts.aggregates.prepare(&mut facts.types, predicate.ty)?;
        }
    }
    connect_inputs(facts, inputs, &regions, &mut contract)?;
    contract.universals = original
        .universals
        .checked_add(
            u32::try_from(regions.universal_count()).map_err(|_| "too many instance lifetimes")?,
        )
        .ok_or("too many instance lifetimes")?;
    contract.parameters = parameters.unwrap_or_default();
    if let Some(signature) = &mut contract.signature {
        // Body signatures express their parameter roots inside the body binder.
        // Ordinary callable types visited below retain their nested binder.
        *signature = bound.signature(&mut facts.types, *signature, contract.universals)?;
    }
    for clause in &mut contract.type_assumptions {
        *clause = bound.clause(&mut facts.types, *clause)?;
    }
    for obligation in &mut contract.obligations {
        match &mut obligation.kind {
            Kind::Membership { captures, .. } => {
                *captures = bound.apply_arguments(&mut facts.types, *captures)?;
            }
            Kind::Coercion {
                actual, expected, ..
            } => {
                *actual = bound.apply(&mut facts.types, *actual)?;
                *expected = bound.apply(&mut facts.types, *expected)?;
            }
            Kind::ScopedWellFormed { ty, .. } => {
                *ty = bound.apply_inside(&mut facts.types, *ty, 1)?;
            }
            Kind::WellFormed(ty) | Kind::TypeOutlives { ty, .. } => {
                *ty = bound.apply(&mut facts.types, *ty)?;
            }
            _ => {}
        }
    }
    for evidence in &mut contract.proofs {
        evidence.proof.substitute(&mut facts.types, &mut bound)?;
    }
    if let Some(connections) = &mut inputs.connections
        && let Some(proof) = connections.proof.take()
    {
        super::closures::retain_proof(
            facts,
            &mut contract,
            proof,
            connections.slots.iter().copied(),
        )?;
    }
    argument_assumptions(facts, inputs, &mut contract)?;
    Ok((bound, contract, requested))
}

/// Selected argument formation is caller evidence, including erased arguments.
fn argument_assumptions(
    facts: &mut Declarations,
    inputs: &Inputs,
    contract: &mut Contract,
) -> Result<(), &'static str> {
    // Instantiation sites have checked argument well-formedness. Retain those
    // structural relations as assumptions in this separate proof universe,
    // including arguments absent from the signature's runtime parameters.
    for &(_, argument) in &inputs.bindings {
        if let Argument::Type(ty) = argument {
            facts
                .lifetimes
                .nominal_requirements
                .prepare(&mut facts.types, ty)?;
        }
    }
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let mut projections = Vec::new();
    for &(_, argument) in &inputs.bindings {
        if let Argument::Type(ty) = argument {
            let requirements = queries.symbolic_requirements(ty)?;
            if !requirements.scalars.parameters.is_empty() {
                return Err("instance contract retains generic argument requirements");
            }
            contract
                .assumptions
                .extend_from_slice(requirements.scalars.regions);
            projections.extend_from_slice(requirements.projections);
        }
    }
    let mut folds = crate::support::hash::HashMap::default();
    for (projection, shorter) in projections {
        let ty = if projection.depth == 0 {
            projection.ty
        } else {
            folds
                .entry(projection.depth)
                .or_insert_with(|| Bound::lower(projection.depth))
                .apply(&mut facts.types, projection.ty)?
        };
        contract.type_assumptions.push(
            crate::types::parameters::predicates::Predicate {
                ty,
                bound: crate::types::parameters::predicates::Bound::Outlives(shorter),
            }
            .into(),
        );
    }
    Ok(())
}

/// Import all source identities through one folder before specializing body facts.
fn import_regions(
    facts: &mut Declarations,
    inputs: &mut Inputs,
    universals: u32,
    requested: &mut Option<crate::types::TypeId>,
) -> Result<Bound, &'static str> {
    let mut regions = Bound::universals(universals);
    for capture in &mut inputs.captures {
        *capture = regions.region(*capture, 0)?;
    }
    if let Some(connections) = &mut inputs.connections {
        for (_, actual) in &mut connections.slots {
            *actual = regions.region(*actual, 0)?;
        }
        for (a, b) in &mut connections.equalities {
            *a = regions.region(*a, 0)?;
            *b = regions.region(*b, 0)?;
        }
    }
    for (_, argument) in &mut inputs.bindings {
        *argument = regions.apply_argument(&mut facts.types, *argument)?;
    }
    if let Some((expected, actual)) = &mut inputs.comparison {
        *expected = regions.apply(&mut facts.types, *expected)?;
        *actual = regions.apply(&mut facts.types, *actual)?;
    }
    if let Some(result) = requested {
        let before = regions.steps;
        *result = regions.apply(&mut facts.types, *result)?;
        facts.body_work.canonical_steps += regions.steps - before;
    }
    Ok(regions)
}

/// Reconnect imported argument lifetimes with the template's original early slots.
/// The call already supplies these exact identities; its late binders remain local.
fn early_identities(
    facts: &Declarations,
    instance: Instance,
    regions: &Bound,
    contract: &mut Contract,
) -> Result<(), &'static str> {
    if regions.universal_count() == 0 {
        return Ok(());
    }
    if matches!(facts.types.kind(instance.ty()), Some(Type::Closure { .. })) {
        // Closure captures carry the immediate enclosing body's complete prefix,
        // including late lifetimes; they are connected separately above.
        return Ok(());
    }
    let Some(Type::FunctionItem {
        item, arguments, ..
    }) = facts.types.kind(instance.ty())
    else {
        return Err("instance lifetime mapping requires a function item");
    };
    let callable = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("missing instance callable")?;
    if let Some(implementation) = facts
        .destructors
        .method(crate::resolve::model::DefId(item.0))
    {
        return destructor_identities(
            facts,
            implementation,
            arguments,
            callable.parameters,
            regions,
            contract,
        );
    }
    let scheme = facts
        .lifetimes
        .functions
        .get(&crate::resolve::model::DefId(item.0));
    if let Some(scheme) = scheme
        && scheme.captures as usize + scheme.early.len() != callable.parameters as usize
    {
        return Err("instance early lifetime slots disagree with its template");
    }
    for (index, argument) in facts
        .types
        .arguments(arguments)
        .ok_or("invalid instance lifetime arguments")?
        .take(callable.parameters as usize)
        .enumerate()
    {
        let Argument::Lifetime(actual) = argument else {
            return Err("instance early argument is not a lifetime");
        };
        let Some(actual) = regions.universal_image(actual) else {
            continue;
        };
        let slot = if let Some(scheme) = scheme {
            if index < scheme.captures as usize {
                u16::try_from(index).map_err(|_| "too many captured instance lifetimes")?
            } else {
                scheme
                    .early
                    .get(index - scheme.captures as usize)
                    .ok_or("missing instance early lifetime slot")?
                    .1
            }
        } else {
            // Tuple constructors retain their nominal lifetime declaration order.
            u16::try_from(index).map_err(|_| "too many constructor lifetimes")?
        };
        connect(contract, slot, actual)?;
    }
    Ok(())
}

fn destructor_identities(
    facts: &Declarations,
    implementation: &crate::sema::destructors::Implementation,
    arguments: crate::types::GenericArgs,
    early: u32,
    regions: &Bound,
    contract: &mut Contract,
) -> Result<(), &'static str> {
    // Drop callables use nominal argument order. Coverage retains the inverse
    // mapping to original impl slots, including slots absent from the callable.
    for (slot, &region) in implementation.coverage.regions.iter().enumerate() {
        let Some((0, index)) = region.bound_parts() else {
            return Err("invalid destructor instance capture mapping");
        };
        if u32::from(index) >= early {
            continue;
        }
        let Some(Argument::Lifetime(actual)) = facts
            .types
            .arguments(arguments)
            .and_then(|mut args| args.nth(usize::from(index)))
        else {
            return Err("missing destructor instance lifetime argument");
        };
        if let Some(actual) = regions.universal_image(actual) {
            connect(
                contract,
                u16::try_from(slot).map_err(|_| "too many destructor capture slots")?,
                actual,
            )?;
        }
    }
    Ok(())
}

fn connect(contract: &mut Contract, slot: u16, actual: RegionId) -> Result<(), &'static str> {
    if u32::from(slot) >= contract.universals {
        return Err("instance early lifetime is outside its body binder");
    }
    let formal = RegionId::bound(0, slot)?;
    if actual != formal {
        contract
            .assumptions
            .extend([(formal, actual), (actual, formal)]);
    }
    Ok(())
}

fn connect_inputs(
    facts: &Declarations,
    inputs: &Inputs,
    regions: &Bound,
    contract: &mut Contract,
) -> Result<(), &'static str> {
    if let Some(instance) = inputs.instance {
        early_identities(facts, instance, regions, contract)?;
    }
    for (index, &actual) in inputs.captures.iter().enumerate() {
        connect(
            contract,
            u16::try_from(index).map_err(|_| "too many inherited closure lifetimes")?,
            actual,
        )?;
    }
    if let Some(connections) = &inputs.connections {
        for &(slot, actual) in &connections.slots {
            if u32::from(slot) < connections.original {
                connect(contract, slot, actual)?;
            }
        }
        contract
            .obligations
            .extend(
                connections
                    .equalities
                    .iter()
                    .map(|&pair| crate::body::regions::Obligation {
                        source: contract.source,
                        kind: Kind::Equality(pair),
                    }),
            );
    }
    Ok(())
}
