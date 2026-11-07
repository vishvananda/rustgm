//! Both method contracts compare their clauses in the impl's normal forms.
//!
//! Rustc normalizes the trait method's predicates, substituted for the impl,
//! in the impl's param-env before it compares the override's predicates with
//! them. A trait bound that names `Self::Assoc` therefore states the impl's
//! associated definition, as the override repeating it or writing that type
//! does. Each clause root is normalized as the method signature is: through the
//! impl's checked definitions, then ordinary selection. An equality keeps its
//! projected side as written apart from its inputs, so it still states that
//! projection's result. A quantified clause is normalized inside its own binder.
#[cfg(test)]
#[path = "../../../../tests/invariants/override_projection_bounds.rs"]
mod tests;
use super::{Clause, Contract, Declarations, Definitions, Predicate};
use crate::types::{
    Kind, TypeId,
    arguments::Argument,
    lifetime_proof::{Obligations, ScopeId},
    parameters::{clauses::Quantified, environment::Environment, predicates::Bound},
};
use std::sync::Arc;

impl Definitions {
    /// Normalize the trait method's own bounds and the override's requirements,
    /// then re-form the environment from the normalized assumptions. Selection
    /// debts of both stay in `proof`, which the contract's region proof checks.
    pub(super) fn clauses(
        &self,
        facts: &mut Declarations,
        contract: &mut Contract,
        proof: &mut Obligations,
    ) -> Result<(), &'static str> {
        let environment = Arc::clone(&contract.environment);
        let mut changed = false;
        for clause in contract
            .assumptions
            .get_mut(contract.method_bounds..)
            .ok_or("method conformance lost its trait method bounds")?
        {
            changed |= self.clause(facts, &environment, proof, clause)?;
        }
        for clause in &mut contract.obligations {
            self.clause(facts, &environment, proof, clause)?;
        }
        if changed {
            contract.environment =
                super::mapping::environment(&facts.types, &contract.scope, &contract.assumptions)?
                    .0
                    .into();
        }
        Ok(())
    }

    fn clause(
        &self,
        facts: &mut Declarations,
        environment: &Arc<Environment>,
        proof: &mut Obligations,
        clause: &mut Clause,
    ) -> Result<bool, &'static str> {
        let (predicate, bound_regions) = match *clause {
            Clause::Predicate(predicate) => (predicate, None),
            Clause::ForAll(id) => {
                let value = facts
                    .types
                    .quantified(id)
                    .ok_or("missing quantified method clause")?;
                (value.predicate, Some(value.bound_regions))
            }
            Clause::UnstableFeature(_) => return Ok(false),
        };
        if !has_projected_inputs(facts, predicate)? {
            return Ok(false);
        }
        // A quantified body names its own regions at depth zero and the
        // contract's universals one binder deeper, as a fresh scope does.
        let scope = match bound_regions {
            Some(count) => proof.scope(ScopeId::ROOT, count)?,
            None => ScopeId::ROOT,
        };
        let mut session = crate::sema::projections::Session::new(
            Some(Arc::clone(environment)),
            proof,
            scope,
            crate::sema::traits::implementations::Availability::Declarations,
            |facts, request| self.resolve(facts, request),
        )?;
        let mut normalize =
            |facts: &mut Declarations, ty: TypeId| session.normalize(facts, ty).map(|(ty, _)| ty);
        let ty = if matches!(predicate.bound, Bound::ProjectionEqual(_)) {
            projected_inputs(facts, predicate.ty, &mut normalize)?
        } else {
            normalize(facts, predicate.ty)?
        };
        let bound = match predicate.bound {
            Bound::Trait(id) | Bound::ConstTrait(id) | Bound::MaybeConstTrait(id) => {
                let (_, constness) = predicate
                    .bound
                    .trait_parts()
                    .ok_or("missing method clause trait")?;
                Bound::trait_application(application(facts, id, &mut normalize)?, constness)
            }
            Bound::ProjectionEqual(result) => Bound::ProjectionEqual(normalize(facts, result)?),
            bound => bound,
        };
        let normalized = Predicate { ty, bound };
        if normalized == predicate {
            return Ok(false);
        }
        *clause = match bound_regions {
            None => Clause::Predicate(normalized),
            Some(bound_regions) => Clause::ForAll(facts.types.intern_quantified(Quantified {
                bound_regions,
                predicate: normalized,
            })?),
        };
        Ok(true)
    }
}

/// An equality's projected side is its statement, not an input: only the
/// projections inside its subject and arguments, or its result, need a form.
fn has_projected_inputs(facts: &Declarations, predicate: Predicate) -> Result<bool, &'static str> {
    let Bound::ProjectionEqual(result) = predicate.bound else {
        return predicate.contains_projections(&facts.types);
    };
    let Some(Kind::Projection(application)) = facts.types.kind(predicate.ty) else {
        return Err("associated equality lost its projected side");
    };
    let projected = |ty| facts.types.contains_projections(ty) == Some(true);
    let trait_inputs = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    Ok(projected(result)
        || trait_inputs.contains_projections(&facts.types)?
        || facts
            .types
            .arguments(application.arguments)
            .ok_or("invalid associated equality arguments")?
            .any(|argument| matches!(argument, Argument::Type(ty) if projected(ty))))
}

fn projected_inputs(
    facts: &mut Declarations,
    ty: TypeId,
    normalize: &mut impl FnMut(&mut Declarations, TypeId) -> Result<TypeId, &'static str>,
) -> Result<TypeId, &'static str> {
    let Some(Kind::Projection(mut application)) = facts.types.kind(ty) else {
        return Err("associated equality lost its projected side");
    };
    application.subject = normalize(facts, application.subject)?;
    application.trait_application =
        self::application(facts, application.trait_application, normalize)?;
    application.arguments = arguments(facts, application.arguments, normalize)?;
    facts.types.intern(Kind::Projection(application))
}

fn application(
    facts: &mut Declarations,
    id: crate::types::traits::ApplicationId,
    normalize: &mut impl FnMut(&mut Declarations, TypeId) -> Result<TypeId, &'static str>,
) -> Result<crate::types::traits::ApplicationId, &'static str> {
    let mut application = facts
        .types
        .trait_application(id)
        .ok_or("invalid method clause trait application")?;
    application.arguments = arguments(facts, application.arguments, normalize)?;
    facts.types.intern_trait_application(application)
}

fn arguments(
    facts: &mut Declarations,
    arguments: crate::types::GenericArgs,
    normalize: &mut impl FnMut(&mut Declarations, TypeId) -> Result<TypeId, &'static str>,
) -> Result<crate::types::GenericArgs, &'static str> {
    let mut values: Vec<_> = facts
        .types
        .arguments(arguments)
        .ok_or("invalid method clause arguments")?
        .collect();
    for value in &mut values {
        if let Argument::Type(ty) = value {
            *ty = normalize(facts, *ty)?;
        }
    }
    facts.types.intern_arguments(&values)
}
