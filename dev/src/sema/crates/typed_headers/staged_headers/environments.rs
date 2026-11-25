//! Reuse empty extensions; source trait identities never become builtin abilities.
use super::{Abilities, Clause, Declarations, Environment, Error, Headers, ParameterId, error, s};
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
    /// Body profiles retain the actual clause binder from callable sugar.
    /// Earlier ordinary metadata permits only vacuous clause scopes.
    fn callable_scopes(&self) -> bool {
        false
    }
    fn regional_arguments(&self) -> bool {
        false
    }
    fn const_arguments(&self) -> bool {
        false
    }
}
struct Ordinary<'a>(&'a [Clause], Option<s::Profile>);
impl sealed::Sealed for Ordinary<'_> {}
impl LeafClauses for Ordinary<'_> {
    fn clauses(&self) -> &[Clause] {
        self.0
    }
    fn callable_scopes(&self) -> bool {
        self.1.is_some_and(s::Profile::body_nodes)
    }
    fn regional_arguments(&self) -> bool {
        self.1.is_some_and(s::Profile::declaration_policies)
    }
    fn const_arguments(&self) -> bool {
        self.1.is_some_and(s::Profile::declaration_policies)
    }
    /// Version thirteen admits conditional bounds on callable roles; version
    /// fourteen admits conditional and unconditional const trait bounds.
    fn application(&self, bound: Bound) -> Result<ApplicationId, Error> {
        match bound {
            Bound::Trait(id) | Bound::MaybeConstTrait(id) => Ok(id),
            Bound::ConstTrait(id) if self.1.is_some_and(s::Profile::declaration_policies) => Ok(id),
            _ => Err(error(
                "source assumption is outside the ordinary actual trait profile",
            )),
        }
    }
}

pub(in crate::sema::crates::typed_headers) fn prepare(
    facts: &Declarations,
    parent: Option<&Arc<Environment>>,
    parameters: (&[(ParameterId, Abilities)], &[ParameterId]),
    (clauses, profile): (&[Clause], Option<s::Profile>),
    previous_parameters: (usize, usize),
    previous_clauses: usize,
    budget: &mut impl Accounting,
) -> Result<Arc<Environment>, Error> {
    prepare_checked(
        facts,
        parent,
        parameters,
        &Ordinary(clauses, profile),
        previous_parameters,
        previous_clauses,
        budget,
    )
}

pub(in crate::sema::crates::typed_headers) fn prepare_checked(
    facts: &Declarations,
    parent: Option<&Arc<Environment>>,
    parameters: (&[(ParameterId, Abilities)], &[ParameterId]),
    leaves: &impl LeafClauses,
    previous_parameters: (usize, usize),
    previous_clauses: usize,
    budget: &mut impl Accounting,
) -> Result<Arc<Environment>, Error> {
    let (parameters, constants) = parameters;
    let same_parameters = previous_parameters == (parameters.len(), constants.len());
    let clauses = leaves.clauses();
    let own = &clauses[previous_clauses..];
    if same_parameters
        && own.is_empty()
        && let Some(parent) = parent
    {
        budget.charge(1)?;
        return Ok(Arc::clone(parent));
    }
    let mut roots = 0_usize;
    let mut copied = 0_usize;
    let (assumptions, inherited) = if same_parameters {
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
    budget.charge(
        parameters
            .len()
            .saturating_add(constants.len())
            .saturating_add(copied),
    )?;
    budget.reserve(Environment::trait_leaf_allocation_bound(
        parameters.len(),
        constants.len(),
        clauses.len(),
        roots,
    ))?;
    let environment = if let Some(parent) = inherited {
        parent.as_ref().clone()
    } else {
        Environment::new(&facts.types, parameters)
            .and_then(|environment| environment.with_const_parameters(&facts.types, constants))
            .map_err(error)?
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
    let (predicate, scoped) = match clause {
        Clause::Predicate(predicate) => (predicate, false),
        Clause::ForAll(id) if leaves.callable_scopes() => facts
            .types
            .quantified(id)
            .filter(|scoped| scoped.bound_regions == 0 || leaves.regional_arguments())
            .map(|scoped| (scoped.predicate, scoped.bound_regions != 0))
            .ok_or_else(|| {
                error("quantified source assumptions require a wider environment profile")
            })?,
        _ => {
            return Err(error(
                "quantified source assumptions require a wider environment profile",
            ));
        }
    };
    if leaves.regional_arguments() {
        return v14_assumption(facts, leaves, predicate, budget);
    }
    let (id, projected_subject) = if let Bound::ProjectionEqual(value) = predicate.bound
        && leaves.callable_scopes()
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
        (projection.trait_application, 0)
    } else {
        let roots = leaf_or_projection(facts, leaves, predicate.ty, budget)?;
        (leaves.application(predicate.bound)?, roots)
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
    let mut roots = arguments
        .len()
        .saturating_add(1)
        .saturating_add(projected_subject);
    for argument in arguments {
        let ty = match argument {
            Argument::Type(ty) => ty,
            Argument::Lifetime(_) if leaves.regional_arguments() => continue,
            Argument::Const(constant) if leaves.const_arguments() => {
                facts
                    .types
                    .const_type(constant)
                    .ok_or_else(|| error("source assumption lost its checked const argument"))?;
                continue;
            }
            _ => {
                return Err(error(
                    "source regional assumption requires a wider environment profile",
                ));
            }
        };
        match facts.types.kind(ty) {
            // Each callable argument element keeps its own checked projected inputs.
            Some(Kind::Tuple(elements)) if leaves.callable_scopes() && ty != TypeId::UNIT => {
                let elements = facts
                    .types
                    .list(elements)
                    .ok_or_else(|| error("source environment lost its tuple elements"))?;
                budget.charge(elements.len())?;
                for &element in elements {
                    if scoped
                        && let Some(Kind::Reference { referent, .. }) = facts.types.kind(element)
                    {
                        // The decoder checked this reference in the same clause binder.
                        // Retain the whole quantified clause and count its extra root.
                        leaf(facts, referent)?;
                        roots = roots.saturating_add(2);
                    } else {
                        roots = roots
                            .saturating_add(leaf_or_projection(facts, leaves, element, budget)?);
                    }
                }
            }
            _ => {
                roots = roots.saturating_add(leaf_or_projection(facts, leaves, ty, budget)?);
            }
        }
    }
    Ok(roots)
}

/// A v14 declaration projection keeps its actual subject and input roots.
fn leaf_or_projection(
    facts: &Declarations,
    leaves: &impl LeafClauses,
    mut ty: TypeId,
    budget: &mut impl Accounting,
) -> Result<usize, Error> {
    let mut additional_roots = 0;
    if leaves.regional_arguments()
        && let Some(Kind::Pattern { base, .. }) = facts.types.kind(ty)
    {
        budget.charge(1)?;
        additional_roots += 1;
        ty = base;
    }
    let Some(Kind::Projection(projection)) =
        facts.types.kind(ty).filter(|_| leaves.regional_arguments())
    else {
        leaf(facts, ty)?;
        return Ok(additional_roots);
    };
    leaf(facts, projection.subject)?;
    let application = facts
        .types
        .trait_application(projection.trait_application)
        .ok_or_else(|| error("source projected trait argument lost its actual declaring trait"))?;
    additional_roots += 1;
    for arguments in [application.arguments, projection.arguments] {
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or_else(|| error("source projected trait argument lost its complete inputs"))?;
        additional_roots += arguments.len();
        budget.charge(arguments.len().saturating_add(1))?;
        for argument in arguments {
            match argument {
                Argument::Type(ty) => leaf(facts, ty)?,
                Argument::Lifetime(_) => {}
                Argument::Const(constant) if leaves.const_arguments() => {
                    facts.types.const_type(constant).ok_or_else(|| {
                        error("source projected trait argument lost its const input")
                    })?;
                }
                Argument::Const(_) => {
                    return Err(error(
                        "projected environment argument requires its actual scope",
                    ));
                }
            }
        }
    }
    Ok(additional_roots)
}

fn v14_assumption(
    facts: &Declarations,
    leaves: &impl LeafClauses,
    predicate: crate::types::parameters::predicates::Predicate,
    budget: &mut impl Accounting,
) -> Result<usize, Error> {
    if matches!(predicate.bound, Bound::Outlives(_)) {
        return v14_roots(facts, predicate.ty, budget);
    }
    if let Bound::ProjectionEqual(value) = predicate.bound {
        if !matches!(facts.types.kind(predicate.ty), Some(Kind::Projection(_))) {
            return Err(error(
                "source result assumption lost its actual declared-result projection",
            ));
        }
        return Ok(v14_roots(facts, predicate.ty, budget)?
            .saturating_add(v14_roots(facts, value, budget)?));
    }
    let mut roots = v14_roots(facts, predicate.ty, budget)?;
    let id = leaves.application(predicate.bound)?;
    let application = facts
        .types
        .trait_application(id)
        .ok_or_else(|| error("source environment lost its actual trait application"))?;
    let mut worklist = Vec::new();
    let mut seen = crate::support::hash::HashSet::default();
    roots = roots.saturating_add(v14_push_args(
        facts,
        application.arguments,
        &mut worklist,
        budget,
    )?);
    v14_drain(facts, &mut worklist, &mut seen, &mut roots, budget)?;
    Ok(roots)
}

fn v14_roots(
    facts: &Declarations,
    root: TypeId,
    budget: &mut impl Accounting,
) -> Result<usize, Error> {
    let mut roots = 0_usize;
    let mut worklist = vec![root];
    let mut seen = crate::support::hash::HashSet::default();
    v14_drain(facts, &mut worklist, &mut seen, &mut roots, budget)?;
    Ok(roots)
}

fn v14_drain(
    facts: &Declarations,
    worklist: &mut Vec<TypeId>,
    seen: &mut crate::support::hash::HashSet<TypeId>,
    roots: &mut usize,
    budget: &mut impl Accounting,
) -> Result<(), Error> {
    while let Some(ty) = worklist.pop() {
        if !seen.insert(ty) {
            continue;
        }
        budget.charge(1)?;
        *roots = roots.saturating_add(1);
        match facts.types.kind(ty) {
            Some(Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) | Kind::Foreign(_)) => {}
            Some(
                Kind::Slice(inner)
                | Kind::Array { element: inner, .. }
                | Kind::Reference {
                    referent: inner, ..
                }
                | Kind::Pointer { pointee: inner, .. }
                | Kind::Pattern { base: inner, .. },
            ) => worklist.push(inner),
            Some(Kind::ArrayTerm { element, length }) => {
                facts
                    .types
                    .const_type(length)
                    .ok_or_else(|| error("source assumption lost its checked const length"))?;
                worklist.push(element);
            }
            Some(Kind::Tuple(elements)) => {
                let elements = facts
                    .types
                    .list(elements)
                    .ok_or_else(|| error("source environment lost its tuple elements"))?;
                worklist.extend(elements.iter().copied());
            }
            Some(Kind::Function {
                parameters, result, ..
            }) => {
                let parameters = facts
                    .types
                    .list(parameters)
                    .ok_or_else(|| error("source environment lost its function parameters"))?;
                worklist.extend(parameters.iter().copied());
                worklist.push(result);
            }
            Some(
                Kind::Adt(_, arguments)
                | Kind::Opaque(_, arguments)
                | Kind::Future { arguments, .. }
                | Kind::Dynamic { arguments, .. },
            ) => {
                *roots = roots.saturating_add(v14_push_args(facts, arguments, worklist, budget)?);
            }
            Some(Kind::FunctionItem {
                arguments,
                signature,
                ..
            }) => {
                *roots = roots.saturating_add(v14_push_args(facts, arguments, worklist, budget)?);
                worklist.push(signature);
            }
            Some(Kind::Closure {
                arguments,
                signature,
                environment,
                ..
            }) => {
                *roots = roots.saturating_add(v14_push_args(facts, arguments, worklist, budget)?);
                worklist.push(signature);
                worklist.push(environment);
            }
            Some(Kind::Projection(projection)) => {
                worklist.push(projection.subject);
                let application = facts
                    .types
                    .trait_application(projection.trait_application)
                    .ok_or_else(|| {
                        error("source projected trait argument lost its actual declaring trait")
                    })?;
                *roots = roots.saturating_add(v14_push_args(
                    facts,
                    application.arguments,
                    worklist,
                    budget,
                )?);
                *roots = roots.saturating_add(v14_push_args(
                    facts,
                    projection.arguments,
                    worklist,
                    budget,
                )?);
            }
            None => {
                return Err(error("source trait assumption has an unknown actual type"));
            }
        }
    }
    Ok(())
}

fn v14_push_args(
    facts: &Declarations,
    arguments: crate::types::GenericArgs,
    worklist: &mut Vec<TypeId>,
    budget: &mut impl Accounting,
) -> Result<usize, Error> {
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or_else(|| error("source environment lost its full trait arguments"))?;
    let count = arguments.len();
    budget.charge(count.saturating_add(1))?;
    for argument in arguments {
        match argument {
            Argument::Type(ty) => worklist.push(ty),
            Argument::Lifetime(_) => {}
            Argument::Const(constant) => {
                facts
                    .types
                    .const_type(constant)
                    .ok_or_else(|| error("source assumption lost its checked const argument"))?;
            }
        }
    }
    Ok(count)
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
