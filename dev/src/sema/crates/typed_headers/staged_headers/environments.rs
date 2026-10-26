//! Reuse empty extensions; source trait identities never become builtin abilities.
use super::{Abilities, Clause, Declarations, Environment, Error, Headers, ParameterId, error};
use crate::types::{
    Kind, TypeId,
    arguments::Argument,
    parameters::{environment::Retained, predicates::Bound},
    traits::ApplicationId,
};
use crate::{sema::crates::typed_headers::recipe_cursor::Budget, support::retained::Allocation};
use std::sync::Arc;
pub(in crate::sema::crates::typed_headers) trait Accounting:
    Budget
{
    fn observe(&mut self, allocation: Allocation);
}
impl Accounting for Headers {
    fn observe(&mut self, allocation: Allocation) {
        self.observed.include(allocation);
    }
}

pub(in crate::sema::crates::typed_headers) mod sealed {
    pub trait Sealed {}
}
/// Borrowed clauses whose actual producer fixes the accepted leaf effects.
pub(in crate::sema::crates::typed_headers) trait LeafClauses:
    sealed::Sealed
{
    fn clauses(&self) -> &[Clause];
    fn application(&self, bound: Bound) -> Result<ApplicationId, Error>;
}
struct Ordinary<'a>(&'a [Clause]);
impl sealed::Sealed for Ordinary<'_> {}
impl LeafClauses for Ordinary<'_> {
    fn clauses(&self) -> &[Clause] {
        self.0
    }
    fn application(&self, bound: Bound) -> Result<ApplicationId, Error> {
        let Bound::Trait(id) = bound else {
            return Err(error(
                "source assumption is outside the ordinary actual trait profile",
            ));
        };
        Ok(id)
    }
}

pub(in crate::sema::crates::typed_headers) fn prepare(
    facts: &Declarations,
    parent: Option<&Arc<Environment>>,
    parameters: &[(ParameterId, Abilities)],
    clauses: &[Clause],
    previous_parameters: usize,
    previous_clauses: usize,
    budget: &mut impl Accounting,
) -> Result<Arc<Environment>, Error> {
    prepare_checked(
        facts,
        parent,
        parameters,
        &Ordinary(clauses),
        previous_parameters,
        previous_clauses,
        budget,
    )
}

pub(in crate::sema::crates::typed_headers) fn prepare_checked(
    facts: &Declarations,
    parent: Option<&Arc<Environment>>,
    parameters: &[(ParameterId, Abilities)],
    leaves: &impl LeafClauses,
    previous_parameters: usize,
    previous_clauses: usize,
    budget: &mut impl Accounting,
) -> Result<Arc<Environment>, Error> {
    let clauses = leaves.clauses();
    let own = &clauses[previous_clauses..];
    if previous_parameters == parameters.len()
        && own.is_empty()
        && let Some(parent) = parent
    {
        budget.charge(1)?;
        return Ok(Arc::clone(parent));
    }
    let mut roots = 0_usize;
    let mut copied = 0_usize;
    let (assumptions, inherited) = if previous_parameters == parameters.len() {
        if let Some(parent) = parent {
            copied = previous_clauses;
            (own, Some(parent))
        } else {
            (clauses, None)
        }
    } else {
        (clauses, None)
    };
    for clause in assumptions {
        budget.charge(1)?;
        let Clause::Predicate(predicate) = clause else {
            return Err(error(
                "quantified source assumptions require a wider environment profile",
            ));
        };
        leaf(facts, predicate.ty)?;
        let id = leaves.application(predicate.bound)?;
        let application = facts
            .types
            .trait_application(id)
            .ok_or_else(|| error("source environment lost its actual trait application"))?;
        let arguments = facts
            .types
            .arguments(application.arguments)
            .ok_or_else(|| error("source environment lost its full trait arguments"))?;
        budget.charge(arguments.len().saturating_add(1))?;
        roots = roots.saturating_add(arguments.len().saturating_add(1));
        for argument in arguments {
            let Argument::Type(ty) = argument else {
                return Err(error(
                    "source regional assumption requires a wider environment profile",
                ));
            };
            leaf(facts, ty)?;
        }
    }
    budget.charge(parameters.len().saturating_add(copied))?;
    budget.reserve(Environment::trait_leaf_allocation_bound(
        parameters.len(),
        clauses.len(),
        roots,
    ))?;
    let environment = if let Some(parent) = inherited {
        parent.as_ref().clone()
    } else {
        Environment::new(&facts.types, parameters).map_err(error)?
    };
    let environment = Arc::new(
        environment
            .with_clauses(&facts.types, assumptions)
            .map_err(error)?,
    );
    let retained = if let Some(parent) = inherited {
        environment.retain_owned_from(parent, &mut Retained::default())
    } else {
        let empty = Environment::new(&facts.types, &[]).map_err(error)?;
        environment.retain_owned_from(&empty, &mut Retained::default())
    };
    budget.charge(retained.visits)?;
    budget.observe(retained);
    budget.observe(Allocation {
        bytes: 2 * size_of::<usize>(),
        visits: 1,
    });
    Ok(environment)
}
fn leaf(facts: &Declarations, ty: TypeId) -> Result<(), Error> {
    if ty != TypeId::UNIT
        && !matches!(
            facts.types.kind(ty),
            Some(Kind::Parameter(_) | Kind::Primitive(_))
        )
    {
        return Err(error(
            "source trait assumption has an unsupported non-leaf actual type",
        ));
    }
    Ok(())
}
