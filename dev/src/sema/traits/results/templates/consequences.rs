//! Same-result equations rewrite consequences without reversing parent applications.
use super::{Clause, Growth, HashMap, Index};
use crate::types::{
    Store, TypeId,
    arguments::Argument,
    parameters::predicates::Bound,
    substitute::{NormalizationFailure, Normalizer, Resolution, Step},
};

pub(super) fn normalize<const CHARGED: bool>(
    store: &mut Store,
    index: &mut Index,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<usize, String> {
    if index.equalities.is_empty() {
        return Ok(0);
    }
    let (equations, mut work) = equations::<CHARGED>(index, charge)?;
    if equations.is_empty() {
        return Ok(work);
    }
    let mut fold = Normalizer::default();
    for clauses in index.candidates.values_mut() {
        for clause in clauses {
            if CHARGED {
                charge(1, 0)?;
            }
            work = work.saturating_add(1);
            // A quantified clause keeps its original whole-binder identity.
            let Clause::Predicate(predicate) = clause else {
                continue;
            };
            let Some((id, effect)) = predicate.bound.trait_parts() else {
                continue;
            };
            let application = store
                .trait_application(id)
                .ok_or("invalid consequence trait application")?;
            let count = store
                .arguments(application.arguments)
                .ok_or("invalid consequence arguments")?
                .len();
            if count == 0 {
                continue;
            }
            if CHARGED {
                let growth = Growth::vector::<Argument>(0, 0, count);
                charge(count.saturating_add(growth.work), growth.bytes)?;
            }
            let mut arguments = store
                .arguments(application.arguments)
                .ok_or("missing consequence arguments")?
                .collect::<Vec<_>>();
            for argument in &mut arguments {
                *argument = if let Argument::Type(ty) = *argument {
                    let before = fold.steps();
                    let rewritten = rewrite::<CHARGED>(store, &mut fold, ty, &equations, charge);
                    work = work.saturating_add(fold.steps().saturating_sub(before));
                    match rewritten {
                        Ok(ty) => Argument::Type(ty),
                        Err(NormalizationFailure::Cycle) => {
                            fold = Normalizer::default();
                            Argument::Type(ty)
                        }
                        Err(error) => return Err(error.message().into()),
                    }
                } else {
                    *argument
                };
            }
            work = work.saturating_add(count);
            if work > 1_000_000 {
                return Err("result consequence work budget exceeded".into());
            }
            let arguments = store.intern_arguments(&arguments)?;
            let application =
                store.intern_trait_application(crate::types::traits::Application {
                    arguments,
                    ..application
                })?;
            predicate.bound = Bound::trait_application(application, effect);
        }
    }
    Ok(work)
}
fn equations<const CHARGED: bool>(
    index: &Index,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(HashMap<TypeId, Option<TypeId>>, usize), String> {
    let mut result = HashMap::default();
    let mut work = 0_usize;
    for clauses in index.equalities.values() {
        for clause in clauses {
            if CHARGED {
                charge(1, 0)?;
            }
            work = work.saturating_add(1);
            let Clause::Predicate(predicate) = *clause else {
                continue;
            };
            let Bound::ProjectionEqual(rhs) = predicate.bound else {
                continue;
            };
            if rhs == predicate.ty {
                continue;
            }
            if CHARGED {
                let growth = Growth::map::<TypeId, Option<TypeId>>(
                    result.len(),
                    result.capacity(),
                    usize::from(!result.contains_key(&predicate.ty)),
                );
                charge(growth.work, growth.bytes)?;
            }
            result
                .entry(predicate.ty)
                .and_modify(|prior| {
                    if *prior != Some(rhs) {
                        *prior = None;
                    }
                })
                .or_insert(Some(rhs));
        }
    }
    Ok((result, work))
}
fn rewrite<const CHARGED: bool>(
    store: &mut Store,
    fold: &mut Normalizer,
    ty: TypeId,
    equations: &HashMap<TypeId, Option<TypeId>>,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<TypeId, NormalizationFailure> {
    if CHARGED {
        charge(1, 0)?;
    }
    if store.contains_projections(ty) == Some(false) {
        return Ok(ty);
    }
    let mut step = fold.start_observed::<CHARGED>(store, ty, charge)?;
    loop {
        match step {
            Step::Complete(ty) => return Ok(ty),
            Step::Resolve(request) => {
                // Outer declaration equations cannot capture a nested callable's locals.
                let replacement = if request.depth == 0 {
                    let alias = store.intern(request.alias.kind())?;
                    equations.get(&alias).copied().flatten()
                } else {
                    None
                };
                let resolution = replacement.map_or(Resolution::Opaque, Resolution::Type);
                step = fold.resume_observed::<CHARGED>(store, request, resolution, charge)?;
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/consequence_folds.rs"]
mod tests;
#[cfg(test)]
pub(in crate::sema) use tests::controls;
