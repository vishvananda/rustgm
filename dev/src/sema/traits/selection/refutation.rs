//! A refuted application is never positive evidence or a debt-free cached absence.
use super::{Declarations, Source};
use crate::{
    resolve::model::DefId,
    sema::traits::{assumptions::Assumptions, implementations::Polarity},
    types::{GenericArgs, parameters::environment::Environment},
};
use std::sync::Arc;

pub(in crate::sema) enum Refutation {
    DebtFree(Box<Closed>),
    Conditional(Box<Conditional>),
}
pub(in crate::sema) struct Closed {
    pub(super) goal: super::Predicate,
    pub(super) owner: DefId,
    pub(super) arguments: GenericArgs,
    pub(super) assumptions: Assumptions,
    pub(super) environment: Option<Arc<Environment>>,
    pub(super) availability: super::Availability,
    pub(super) first_region: u32,
}
pub(in crate::sema) struct Conditional {
    pub(super) identity: Closed,
    pub(super) proof: super::Obligations,
    pub(super) sized: Vec<crate::types::TypeId>,
}
impl Refutation {
    pub(super) fn validate(
        &self,
        goal: super::Predicate,
        caller: &super::Caller<'_>,
        availability: super::Availability,
    ) -> Result<(), &'static str> {
        let identity = match self {
            Self::DebtFree(identity) => identity.as_ref(),
            Self::Conditional(conditional) => &conditional.identity,
        };
        let same_environment = match (identity.environment.as_ref(), caller.environment) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if identity.goal != goal
            || identity.first_region != caller.first_region
            || identity.availability != availability
            || !identity.assumptions.same_snapshot(caller.assumptions)
            || !same_environment
        {
            return Err("negative result belongs to another goal or caller scope");
        }
        Ok(())
    }
    /// Only a completed debt-free negative can reject a positive application.
    /// Paying a conditional negative's scoped debts is a required continuation.
    pub(in crate::sema) fn reject(self) -> Result<(), &'static str> {
        match self {
            Self::DebtFree(closed) => {
                // Keep the exact application and immutable proof environment
                // alive until the owner consumes this one result.
                let Closed {
                    goal,
                    owner,
                    arguments,
                    assumptions,
                    environment,
                    availability,
                    first_region,
                } = *closed;
                let _identity = (
                    goal,
                    owner,
                    arguments,
                    assumptions,
                    environment,
                    availability,
                    first_region,
                );
                Ok(())
            }
            Self::Conditional(conditional) => {
                let Conditional {
                    identity,
                    proof,
                    sized,
                } = *conditional;
                let _unpaid = (identity, proof, sized);
                Err("conditional negative implementation requires scoped debt payment")
            }
        }
    }
}
pub(super) fn implementation_negative(facts: &Declarations, owner: DefId) -> Result<bool, String> {
    Ok(facts
        .generics
        .implementations
        .as_ref()
        .ok_or("missing implementation polarity registry")?
        .polarity(&facts.associated, owner)?
        == Polarity::Negative)
}
pub(super) fn require_positive_source(
    facts: &Declarations,
    source: Option<Source>,
) -> Result<(), String> {
    if let Some(Source::Implementation { owner, .. }) = source
        && implementation_negative(facts, owner)?
    {
        return Err("negative implementation cannot become positive trait evidence".into());
    }
    Ok(())
}
