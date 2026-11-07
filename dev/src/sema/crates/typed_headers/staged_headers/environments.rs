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
    /// Version thirteen lifts the vacuous clause binder that parenthesized
    /// callable sugar opens; no other producer admits a quantified premise.
    fn vacuous_scopes(&self) -> bool {
        false
    }
}
struct Ordinary<'a>(&'a [Clause], bool);
impl sealed::Sealed for Ordinary<'_> {}
impl LeafClauses for Ordinary<'_> {
    fn clauses(&self) -> &[Clause] {
        self.0
    }
    fn vacuous_scopes(&self) -> bool {
        self.1
    }
    /// This consumer's row validation admits a conditional effect only on an
    /// actual callable role's own hierarchy (`Fn: [const] FnMut<Args>`).
    fn application(&self, bound: Bound) -> Result<ApplicationId, Error> {
        let (Bound::Trait(id) | Bound::MaybeConstTrait(id)) = bound else {
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
    (clauses, vacuous_scopes): (&[Clause], bool),
    previous_parameters: usize,
    previous_clauses: usize,
    budget: &mut impl Accounting,
) -> Result<Arc<Environment>, Error> {
    prepare_checked(
        facts,
        parent,
        parameters,
        &Ordinary(clauses, vacuous_scopes),
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
        if let Clause::UnstableFeature(id) = clause {
            facts
                .types
                .feature_name(*id)
                .ok_or_else(|| error("foreign imported feature premise"))?;
            continue;
        }
        roots = roots.saturating_add(assumption(facts, leaves, *clause, budget)?);
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
        super::super::predicate_remap::translation::feature_names::with_clauses(
            environment,
            &facts.types,
            assumptions,
            budget,
        )?,
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
/// One admitted leaf premise; returns its counted trait-input roots.
fn assumption(
    facts: &Declarations,
    leaves: &impl LeafClauses,
    clause: Clause,
    budget: &mut impl Accounting,
) -> Result<usize, Error> {
    let predicate = match clause {
        Clause::Predicate(predicate) => predicate,
        Clause::ForAll(id) if leaves.vacuous_scopes() => {
            facts
                .types
                .quantified(id)
                .filter(|scoped| scoped.bound_regions == 0)
                .ok_or_else(|| {
                    error("quantified source assumptions require a wider environment profile")
                })?
                .predicate
        }
        _ => {
            return Err(error(
                "quantified source assumptions require a wider environment profile",
            ));
        }
    };
    let id = if let Bound::ProjectionEqual(value) = predicate.bound
        && leaves.vacuous_scopes()
    {
        // A callable result binding: a leaf value for a declared result
        // whose own trait application is a leaf premise below.
        leaf(facts, value)?;
        let Some(Kind::Projection(projection)) = facts.types.kind(predicate.ty) else {
            return Err(error(
                "source result assumption lost its actual declared-result projection",
            ));
        };
        leaf(facts, projection.subject)?;
        projection.trait_application
    } else {
        leaf(facts, predicate.ty)?;
        leaves.application(predicate.bound)?
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or_else(|| error("source environment lost its actual trait application"))?;
    let arguments = facts
        .types
        .arguments(application.arguments)
        .ok_or_else(|| error("source environment lost its full trait arguments"))?;
    budget.charge(arguments.len().saturating_add(1))?;
    let roots = arguments.len().saturating_add(1);
    for argument in arguments {
        let Argument::Type(ty) = argument else {
            return Err(error(
                "source regional assumption requires a wider environment profile",
            ));
        };
        match facts.types.kind(ty) {
            // A callable role's argument tuple of leaves.
            Some(Kind::Tuple(elements)) if leaves.vacuous_scopes() && ty != TypeId::UNIT => {
                let elements = facts
                    .types
                    .list(elements)
                    .ok_or_else(|| error("source environment lost its tuple elements"))?;
                budget.charge(elements.len())?;
                for &element in elements {
                    leaf(facts, element)?;
                }
            }
            _ => leaf(facts, ty)?,
        }
    }
    Ok(roots)
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
