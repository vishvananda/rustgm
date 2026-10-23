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
mod originals;
pub(crate) use originals::{anonymous as original_anonymous, initializer as original_initializer};

pub(crate) fn resolve(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
) -> Result<(), String> {
    resolve_with_original(facts, body, contract, None)
}

fn resolve_with_original(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    mut original: Option<originals::Parameters>,
) -> Result<(), String> {
    let dispatch = body.values.iter().any(|value| {
        matches!(value.kind, Rvalue::AssociatedConstant(_) | Rvalue::SourceAssociated(_))
            || matches!(value.kind, Rvalue::Function(id) if facts.associated.trait_owner(id).is_some())
    });
    if !dispatch {
        return finish(facts, body, contract);
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
            let selected = dispatcher.select(facts, value.ty)?;
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
        rewrite(facts, body, contract, &mut fold)?;
        if matches!(
            body.owner,
            super::ir::Owner::Initializer { .. } | super::ir::Owner::AnonymousConstant
        ) {
            for original in originals {
                let selected = fold.apply(&mut facts.types, original)?;
                facts
                    .generics
                    .constant_callable_targets
                    .insert(original, selected);
            }
        }
    }
    let proof = dispatcher.finish();
    if proof.has_quantified() {
        contract
            .proofs
            .push(crate::body::regions::Proof::new(body.source, proof));
        return finish(facts, body, contract);
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
    finish(facts, body, contract)
}

fn finish(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
) -> Result<(), String> {
    super::invocation_entries::select(facts, body, contract).map_err(String::from)?;
    super::copy_requirements::prepare(facts, body, contract)
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
