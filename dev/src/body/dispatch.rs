//! Resolve concrete trait operands before promotion, dependency discovery and body proof.
use super::{
    ir::{Body, Rvalue},
    regions::{Contract, Kind, Obligation},
};
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{Kind as Type, RegionId, substitute::Bound},
};
mod callers;
mod non_type_enclosing;
mod originals;
pub(crate) use originals::{anonymous as original_anonymous, initializer as original_initializer};

/// Borrow the same actual member identity used by concrete operand dispatch.
pub(crate) fn trait_function(facts: &Declarations, value: &super::ir::Value) -> bool {
    matches!(value.kind, Rvalue::Function(id) if facts.associated.trait_owner(id).is_some())
}

pub(crate) fn resolve(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
) -> Result<(), String> {
    resolve_with_original(facts, body, contract, None, None)
}

pub(in crate::body) fn materialized(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    scope: Option<&mut super::copy_requirements::materialized::InstanceScope>,
) -> Result<(), String> {
    resolve_with_original(facts, body, contract, None, scope)
}

pub(crate) fn resolve_prepared(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    preparation: &mut dyn crate::sema::dispatch::Preparation,
) -> Result<(), crate::sema::dispatch::Failure> {
    resolve_using(facts, body, contract, None, None, Some(preparation))
}

fn resolve_with_original(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    original: Option<originals::Parameters>,
    scope: Option<&mut super::copy_requirements::materialized::InstanceScope>,
) -> Result<(), String> {
    resolve_using(facts, body, contract, original, scope, None)
        .map_err(crate::sema::dispatch::Failure::message)
}

fn resolve_using(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    mut original: Option<originals::Parameters>,
    mut scope: Option<&mut super::copy_requirements::materialized::InstanceScope>,
    mut preparation: Option<&mut dyn crate::sema::dispatch::Preparation>,
) -> Result<(), crate::sema::dispatch::Failure> {
    let dispatch = body.values.iter().any(|value| {
        matches!(
            value.kind,
            Rvalue::AssociatedConstant(_) | Rvalue::SourceAssociated(_)
        ) || trait_function(facts, value)
    });
    if !dispatch {
        return finish_using(facts, body, contract, scope.as_deref(), preparation);
    }
    let (mut caller, mut dispatcher) = callers::prepare(facts, body, contract)?;
    let mut replacements = HashMap::default();
    for value in &mut body.values {
        if let Rvalue::AssociatedConstant(instance) = value.kind
            && let Some(parameters) = &mut original
            && parameters.deferred(facts, instance)?
        {
            continue;
        }
        if let Rvalue::AssociatedConstant(instance) = value.kind
            && crate::sema::values::execution::BodyCaller::needed(facts, instance, value.ty)?
        {
            let witness = caller
                .as_mut()
                .ok_or("missing body source caller")?
                .prepare(facts, &mut dispatcher, instance, value.ty)?;
            value.kind = match witness {
                crate::sema::values::execution::PreparedConstant::Selected(selected) => {
                    facts.constant_targets.insert(instance, selected);
                    Rvalue::AssociatedConstant(selected)
                }
                crate::sema::values::execution::PreparedConstant::Source(source) => {
                    Rvalue::SourceAssociated(source)
                }
            };
            continue;
        }
        if let Rvalue::AssociatedConstant(instance) = &mut value.kind {
            let selected = dispatcher.constant(facts, *instance)?;
            facts.constant_targets.insert(*instance, selected);
            *instance = selected;
        }
        if let Rvalue::Function(id) = value.kind
            && facts.associated.trait_owner(id).is_some()
        {
            let selected = if let Some(service) = &mut preparation {
                dispatcher.select_prepared(facts, value.ty, &mut **service)?
            } else {
                dispatcher.select(facts, value.ty)?
            };
            if selected != value.ty {
                replacements.insert(value.ty, selected);
            }
        }
    }
    if let Some(parameters) = original {
        parameters.validate(facts)?;
    }
    if !replacements.is_empty() {
        let originals = replacements.keys().copied().collect::<Vec<_>>();
        let mut fold = Bound::selected_items(&facts.types, replacements)?;
        if let Some(scope) = scope.as_deref_mut() {
            scope.rewrite(facts, body, contract, &mut fold)?;
        }
        rewrite(facts, body, contract, &mut fold)?;
        retain_initializer_targets(facts, body, &originals, &mut fold)?;
    }
    let proof = dispatcher.finish();
    if proof.has_quantified() {
        contract
            .proofs
            .push(crate::body::regions::Proof::new(body.source, proof));
        return finish_using(facts, body, contract, scope.as_deref(), preparation);
    }
    let mut obligations = proof
        .regions
        .map(|id| Obligation {
            source: body.source.into(),
            kind: Kind::Variable {
                id: RegionId(id),
                needs_provenance: false,
            },
        })
        .collect::<Vec<_>>();
    obligations.append(&mut contract.obligations);
    obligations.extend(proof.equalities.into_iter().map(|pair| Obligation {
        source: body.source.into(),
        kind: Kind::Equality(pair),
    }));
    obligations.extend(proof.outlives.into_iter().map(|pair| Obligation {
        source: body.source.into(),
        kind: Kind::Outlives(pair),
    }));
    obligations.extend(proof.types.into_iter().map(|(ty, shorter)| Obligation {
        source: body.source.into(),
        kind: Kind::TypeOutlives { ty, shorter },
    }));
    contract.obligations = obligations;
    finish_using(facts, body, contract, scope.as_deref(), preparation)
}

fn finish_using(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    scope: Option<&super::copy_requirements::materialized::InstanceScope>,
    preparation: Option<&mut dyn crate::sema::dispatch::Preparation>,
) -> Result<(), crate::sema::dispatch::Failure> {
    super::invocation_entries::select(facts, body, contract)?;
    match preparation {
        Some(service) => {
            super::copy_requirements::prepare_selected(facts, body, contract, scope, service)
        }
        None => super::copy_requirements::prepare_scoped(facts, body, contract, scope)
            .map_err(Into::into),
    }
}
pub(super) fn copy_frontier(
    facts: &Declarations,
    contract: &Contract,
) -> Result<u32, &'static str> {
    callers::first_region(facts, contract, false)
}

fn rewrite(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    fold: &mut Bound,
) -> Result<(), &'static str> {
    if let super::ir::Owner::Function {
        execution: Some(signature),
        ..
    }
    | super::ir::Owner::Closure {
        execution: Some(signature),
        ..
    } = &mut body.owner
    {
        *signature = fold.apply(&mut facts.types, *signature)?;
    }
    body.result = fold.apply(&mut facts.types, body.result)?;
    for local in &mut body.locals {
        local.ty = fold.apply(&mut facts.types, local.ty)?;
    }
    for projection in &mut body.projections {
        projection.ty = fold.apply(&mut facts.types, projection.ty)?;
    }
    for value in &mut body.values {
        value.ty = fold.apply(&mut facts.types, value.ty)?;
        match &mut value.kind {
            Rvalue::Function(id) => {
                let Some(Type::FunctionItem { item, .. }) = facts.types.kind(value.ty) else {
                    return Err("dispatched operand is not a function item");
                };
                *id = DefId(item.0);
            }
            Rvalue::Call { signature, .. } | Rvalue::Invoke { signature, .. } => {
                *signature = fold.apply(&mut facts.types, *signature)?;
            }
            Rvalue::Intrinsic { instance, .. } => {
                let ty = fold.apply(&mut facts.types, instance.ty())?;
                *instance = crate::types::instance::Instance::new(&facts.types, ty)?;
            }
            Rvalue::Unsize { conversion, .. } => {
                *conversion = conversion.try_map(|ty| fold.apply(&mut facts.types, ty))?;
            }
            _ => {}
        }
        facts.aggregates.prepare(&mut facts.types, value.ty)?;
    }
    for obligation in &mut contract.obligations {
        match &mut obligation.kind {
            Kind::Coercion {
                actual, expected, ..
            } => {
                *actual = fold.apply(&mut facts.types, *actual)?;
                *expected = fold.apply(&mut facts.types, *expected)?;
            }
            Kind::ScopedWellFormed { ty, .. } => {
                *ty = fold.apply_inside(&mut facts.types, *ty, 1)?;
            }
            Kind::WellFormed(ty) | Kind::TypeOutlives { ty, .. } => {
                *ty = fold.apply(&mut facts.types, *ty)?;
            }
            _ => {}
        }
    }
    for evidence in &mut contract.proofs {
        evidence.proof.substitute(&mut facts.types, fold)?;
    }
    if let Some(pending) = &mut contract.builtins {
        for (_, predicate) in &mut std::sync::Arc::make_mut(pending).predicates {
            *predicate = fold.predicate(&mut facts.types, *predicate)?;
            facts.aggregates.prepare(&mut facts.types, predicate.ty)?;
        }
    }
    for clause in &mut contract.type_assumptions {
        *clause = fold.clause(&mut facts.types, *clause)?;
    }
    if let Some(signature) = &mut contract.signature {
        *signature = fold.apply(&mut facts.types, *signature)?;
    }
    Ok(())
}

// Initializer operands retain these exact selected identities for later constant demand.
fn retain_initializer_targets(
    facts: &mut Declarations,
    body: &Body,
    originals: &[crate::types::TypeId],
    fold: &mut Bound,
) -> Result<(), &'static str> {
    if matches!(
        body.owner,
        super::ir::Owner::Initializer { .. } | super::ir::Owner::AnonymousConstant
    ) {
        for &original in originals {
            let selected = fold.apply(&mut facts.types, original)?;
            facts
                .generics
                .constant_callable_targets
                .insert(original, selected);
        }
    }
    Ok(())
}
