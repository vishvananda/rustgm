//! A dependency call owes the premises of the actual installed declaration.
use super::ImportedOriginals;
use crate::{
    body::{
        ir::{Body, Rvalue},
        regions::{Contract, Kind},
    },
    sema::{crates::SourceOriginal, declaration_inputs, model::Declarations},
    support::{constness::Constness, hash::HashSet},
    types::{
        Kind as Type, RegionId, TypeId,
        arguments::Argument,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        substitute,
    },
};
#[derive(Clone, Copy)]
pub(super) struct Request {
    callable: TypeId,
    effect: Constness,
}

pub(super) fn prepare(
    view: &SourceOriginal<'_>,
    body: &Body,
    work: &mut ImportedOriginals,
) -> Result<Vec<Request>, &'static str> {
    if !view.rows().descriptor().profile().bootstrap_bindings() {
        return Ok(Vec::new());
    }
    let rows = view.rows();
    let mut calls = Vec::new();
    for request in &view.raw().calls {
        work.charge(1)?;
        if rows.function_applications[request.target.0 as usize]
            .declaration
            .artifact
            .0
            != 0
        {
            let Rvalue::Call { callee, .. } = body.values[request.value as usize].kind else {
                return Err("source dependency premises require an actual reconstructed call");
            };
            calls.push(Request {
                callable: body.values[callee.0 as usize].ty,
                effect: request.effect,
            });
        }
    }
    Ok(calls)
}

pub(super) fn check(
    facts: &mut Declarations,
    requests: &[Request],
    contract: &Contract,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    if requests.is_empty() {
        return Ok(());
    }
    let recorded = Recorded::new(facts, contract, work)?;
    for &request in requests {
        check_call(facts, request, &recorded, work)?;
    }
    Ok(())
}

struct Recorded<'a> {
    environment: Option<&'a crate::types::parameters::environment::Environment>,
    predicates: HashSet<Predicate>,
    regions: HashSet<(RegionId, RegionId)>,
}
impl<'a> Recorded<'a> {
    fn new(
        facts: &Declarations,
        contract: &'a Contract,
        work: &mut ImportedOriginals,
    ) -> Result<Self, &'static str> {
        let builtins = contract
            .builtins
            .as_deref()
            .ok_or("source dependency premises require their actual caller environment")?;
        work.charge(
            contract
                .obligations
                .len()
                .saturating_add(builtins.predicates.len()),
        )?;
        let mut result = Self {
            environment: builtins.environment.as_deref(),
            predicates: builtins
                .predicates
                .iter()
                .map(|&(_, predicate)| predicate)
                .collect(),
            regions: HashSet::default(),
        };
        if let Some(environment) = result.environment {
            for clause in environment.trait_clauses()? {
                work.charge(1)?;
                if let Some(predicate) = unquantified(facts, clause) {
                    result.predicates.insert(predicate);
                }
            }
        }
        for obligation in &contract.obligations {
            match obligation.kind {
                Kind::Outlives(pair) => {
                    result.regions.insert(pair);
                }
                Kind::TypeOutlives { ty, shorter } => {
                    result.predicates.insert(Predicate {
                        ty,
                        bound: Bound::Outlives(shorter),
                    });
                }
                _ => {}
            }
        }
        Ok(result)
    }
}

fn unquantified(facts: &Declarations, clause: Clause) -> Option<Predicate> {
    match clause {
        Clause::Predicate(predicate) => Some(predicate),
        Clause::ForAll(id) => facts
            .types
            .quantified(id)
            .filter(|clause| clause.bound_regions == 0)
            .map(|clause| clause.predicate),
        Clause::UnstableFeature(_) => None,
    }
}

fn check_call(
    facts: &mut Declarations,
    request: Request,
    recorded: &Recorded<'_>,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    let Some(Type::FunctionItem {
        item, arguments, ..
    }) = facts.types.kind(request.callable)
    else {
        return Err("source dependency premises lost their applied callable");
    };
    let owner = crate::resolve::model::DefId(item.0);
    let (template, parameters) = facts
        .lifetimes
        .callables
        .registration(item)
        .ok_or("source dependency premises require their actual callable registration")?;
    let actuals = facts
        .types
        .arguments(arguments)
        .ok_or("source dependency premises require their actual generic arguments")?;
    work.charge(actuals.len().saturating_add(1))?;
    let mut early = Vec::with_capacity(template.parameters as usize);
    let mut bindings = Vec::with_capacity(parameters.len());
    for (index, actual) in actuals.enumerate() {
        if index < template.parameters as usize {
            let Argument::Lifetime(region) = actual else {
                return Err("source dependency premises changed an actual early region");
            };
            early.push(region);
        } else {
            let parameter = *parameters
                .get(index - template.parameters as usize)
                .ok_or("source dependency premises changed their complete arity")?;
            bindings.push((parameter, actual));
        }
    }
    region_bounds(facts, owner, &early, recorded, work)?;
    let mut fold = substitute::Bound::parameters_captured(&facts.types, &bindings, early)?;
    for input in
        declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), owner)?.iter()
    {
        work.charge(1)?;
        let clause = fold.clause(&mut facts.types, input.value)?;
        if let Clause::UnstableFeature(id) = clause {
            feature(facts, recorded, id, work)?;
            continue;
        }
        let goal = unquantified(facts, clause)
            .ok_or("source dependency call needs its quantified premise transport")?;
        let bound = match goal.bound {
            Bound::MaybeConstTrait(application) if request.effect == Constness::Ordinary => {
                Bound::Trait(application)
            }
            Bound::ConstTrait(_) | Bound::MaybeConstTrait(_) | Bound::ProjectionEqual(_) => {
                return Err("source dependency call needs its scoped premise transport");
            }
            bound => bound,
        };
        if !recorded.predicates.contains(&Predicate { bound, ..goal }) {
            return Err("source dependency call omitted its actual declared premise");
        }
    }
    work.charge(fold.steps)
}

fn feature(
    facts: &Declarations,
    recorded: &Recorded<'_>,
    feature: crate::types::parameters::features::FeatureId,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    let steps = std::cell::Cell::new(work.steps);
    let proved = facts.prove_feature(feature, recorded.environment, &|count, bytes| {
        steps.set(steps.get().saturating_add(count));
        if steps.get() > crate::support::crates::MAX_COMPONENTS || bytes != 0 {
            return Err("source dependency feature proof exceeded its envelope");
        }
        Ok(())
    })?;
    work.steps = steps.get();
    if !proved {
        return Err("source dependency feature premise has no actual caller evidence");
    }
    Ok(())
}

fn region_bounds(
    facts: &Declarations,
    owner: crate::resolve::model::DefId,
    early: &[RegionId],
    recorded: &Recorded<'_>,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    let Some(scheme) = facts.lifetimes.functions.get(&owner) else {
        return Ok(());
    };
    if scheme.bounds.is_empty() {
        return Ok(());
    }
    work.charge((scheme.total as usize).saturating_add(scheme.bounds.len()))?;
    let mut mapping = vec![None; scheme.total as usize];
    for ((_, slot), &actual) in scheme.early.iter().zip(early) {
        mapping[usize::from(*slot)] = Some(actual);
    }
    let replace = |region: RegionId| match region.bound_parts() {
        Some((0, slot)) => mapping
            .get(usize::from(slot))
            .copied()
            .flatten()
            .ok_or("source dependency lifetime bound has no actual early region"),
        _ => Ok(region),
    };
    for &(longer, shorter) in &scheme.bounds {
        let (longer, shorter) = (replace(longer)?, replace(shorter)?);
        if longer != shorter
            && longer != RegionId::STATIC
            && !recorded.regions.contains(&(longer, shorter))
        {
            return Err("source dependency call omitted its actual declared region premise");
        }
    }
    Ok(())
}
