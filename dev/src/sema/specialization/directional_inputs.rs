//! Child parameters stay rigid; only the parent's declared parameters become fresh.
use super::retention_work::{Budget, Counted};
use crate::{
    sema::{
        model::Declarations,
        traits::{assumptions::Assumptions, implementations::EntryKey},
    },
    types::{
        GenericArgs, RegionId,
        arguments::Argument as Canonical,
        infer::{self, Context, Instantiation, arguments::Argument, predicates::Predicate as Goal},
        parameters::{
            ParameterId,
            clauses::Clause,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        substitute,
    },
};
use std::sync::Arc;
pub(super) struct Inputs {
    pub child: EntryKey,
    pub parent: EntryKey,
    pub context: Counted<Context>,
    pub goals: Vec<Goal>,
    pub environment: Option<Arc<Environment>>,
    pub assumptions: Arc<Assumptions>,
    pub child_regions: Vec<RegionId>,
    pub parent_regions: Vec<RegionId>,
    pub parent_bindings: Vec<(ParameterId, Argument)>,
    pub next_region: u32,
}
impl Inputs {
    pub(super) fn new(
        facts: &mut Declarations,
        child: EntryKey,
        parent: EntryKey,
        budget: &Budget,
    ) -> Result<Option<Self>, String> {
        let registry = facts
            .generics
            .implementations
            .as_deref()
            .ok_or("specialization registry disappeared")?;
        let child_header = registry.checked_entry(child)?.predicate;
        let parent_header = registry.checked_entry(parent)?.predicate;
        let regions = Regions::prepare(facts, child, parent, budget)?;
        let ids = facts.generics.parameter_ids(&facts.types, parent.owner())?;
        let mut context = Counted::new(Context::default(), budget)?;
        let mut child_fold = Counted::new(substitute::Bound::new(regions.child.clone()), budget)?;
        let child_header = child_fold.predicate(&mut facts.types, child_header)?;
        let mut child_substitution = Instantiation::new(&mut context, &facts.types, &[])
            .map_err(|error| error.to_string())?;
        let child_goal = context
            .instantiate_predicate(&mut facts.types, child_header, &mut child_substitution)
            .map_err(|error| error.to_string())?;
        let mut parent_bindings = Vec::with_capacity(ids.len());
        for id in ids {
            parent_bindings.push((
                id,
                context
                    .parameter_variable(&facts.types, id)
                    .map_err(|error| error.to_string())?,
            ));
        }
        let mut parent_fold = Counted::new(substitute::Bound::new(regions.parent.clone()), budget)?;
        let parent_header = parent_fold.predicate(&mut facts.types, parent_header)?;
        let mut parent_substitution =
            Instantiation::new(&mut context, &facts.types, &parent_bindings)
                .map_err(|error| error.to_string())?;
        let parent_goal = context
            .instantiate_predicate(&mut facts.types, parent_header, &mut parent_substitution)
            .map_err(|error| error.to_string())?;
        match context.equate_predicate_shapes(&facts.types, &child_goal, &parent_goal) {
            Ok(()) => {}
            Err(infer::Error::Mismatch | infer::Error::Occurs) => {
                context.pay()?;
                child_fold.pay()?;
                parent_fold.pay()?;
                return Ok(None);
            }
            Err(error) => return Err(error.to_string()),
        }
        let goals = parent_goals(
            facts,
            parent,
            &mut context,
            &mut parent_fold,
            &mut parent_substitution,
            &parent_bindings,
            budget,
        )?;
        let environment = facts
            .generics
            .environments
            .get(&child.owner())
            .map(|environment| {
                budget.charge(1, size_of::<Environment>() + 2 * size_of::<usize>())?;
                environment
                    .substitute_declared_regions_charged(
                        &mut facts.types,
                        &mut child_fold,
                        &|steps, bytes| budget.charge(steps, bytes),
                    )
                    .map(Arc::new)
            })
            .transpose()?;
        let assumptions = Assumptions::from_environment_charged(
            &facts.types,
            environment.as_deref(),
            &|steps, bytes| budget.charge(steps, bytes),
        )?;
        context.pay()?;
        child_fold.pay()?;
        parent_fold.pay()?;
        Ok(Some(Self {
            child,
            parent,
            context,
            goals,
            environment,
            assumptions: Arc::new(assumptions),
            child_regions: regions.child,
            parent_regions: regions.parent,
            parent_bindings,
            next_region: regions.next,
        }))
    }
    pub(super) fn pay_context(&mut self) -> Result<(), &'static str> {
        self.context.pay()
    }
    pub(super) fn parent_arguments(
        &mut self,
        facts: &mut Declarations,
    ) -> Result<GenericArgs, String> {
        let mut arguments: Vec<_> = self
            .parent_regions
            .iter()
            .copied()
            .map(Canonical::Lifetime)
            .collect();
        for &(_, argument) in &self.parent_bindings {
            arguments.push(match argument {
                Argument::Type(term) => Canonical::Type(
                    self.context
                        .resolved(&mut facts.types, term)
                        .map_err(|error| error.to_string())?,
                ),
                Argument::Const(term) => Canonical::Const(
                    self.context
                        .resolved_const(&mut facts.types, term)
                        .map_err(|error| error.to_string())?,
                ),
                Argument::Lifetime(_) => {
                    return Err("impl parameter binding unexpectedly contains a lifetime".into());
                }
            });
        }
        facts.types.intern_arguments(&arguments).map_err(Into::into)
    }
}

struct Regions {
    child: Vec<RegionId>,
    parent: Vec<RegionId>,
    next: u32,
}
impl Regions {
    fn prepare(
        facts: &Declarations,
        child: EntryKey,
        parent: EntryKey,
        budget: &Budget,
    ) -> Result<Self, &'static str> {
        let child_schema = facts
            .associated
            .schema(child.owner())
            .ok_or("specialization child has no schema")?;
        let parent_schema = facts
            .associated
            .schema(parent.owner())
            .ok_or("specialization parent has no schema")?;
        let first_parent = child_schema
            .captures
            .checked_add(1)
            .ok_or("specialization child region overflow")?;
        let next_region = first_parent
            .checked_add(parent_schema.captures)
            .filter(|n| *n < 0x8000_0000)
            .ok_or("specialization parent region overflow")?;
        let parameter_count = facts
            .generics
            .parameters
            .get(&parent.owner())
            .map_or(0, Vec::len);
        budget.charge(
            1 + parameter_count,
            size_of::<Inputs>()
                + 8 * (child_schema.captures as usize + parent_schema.captures as usize)
                    * size_of::<RegionId>()
                + 4 * parameter_count
                    * (size_of::<ParameterId>()
                        + size_of::<(ParameterId, Argument)>()
                        + size_of::<Goal>())
                + 4 * size_of::<usize>()
                + size_of::<Assumptions>(),
        )?;
        let child: Vec<_> = (1..first_parent).map(RegionId).collect();
        let parent: Vec<_> = (first_parent..next_region).map(RegionId).collect();
        Ok(Self {
            child,
            parent,
            next: next_region,
        })
    }
}
fn parent_goals(
    facts: &mut Declarations,
    parent: EntryKey,
    context: &mut Context,
    parent_fold: &mut substitute::Bound,
    parent_substitution: &mut Instantiation,
    parent_bindings: &[(ParameterId, Argument)],
    budget: &Budget,
) -> Result<Vec<Goal>, String> {
    let mut requirements = Vec::new();
    for requirement in crate::sema::declaration_inputs::declared(
        &facts.generics,
        facts.crates.as_deref(),
        parent.owner(),
    )? {
        budget.charge(
            1,
            4 * (size_of::<crate::sema::declaration_inputs::Input>() + size_of::<Goal>()),
        )?;
        requirements.push(requirement);
    }
    let mut goals = Vec::with_capacity(requirements.len().saturating_add(parent_bindings.len()));
    for &(id, _) in parent_bindings {
        if matches!(
            facts.types.parameter(id).map(|parameter| parameter.kind),
            Some(crate::types::parameters::ParameterKind::Type { sized: true })
        ) {
            let ty = facts.types.intern(crate::types::Kind::Parameter(id))?;
            goals.push(
                context
                    .instantiate_predicate(
                        &mut facts.types,
                        Predicate {
                            ty,
                            bound: Bound::Sized,
                        },
                        parent_substitution,
                    )
                    .map_err(|error| error.to_string())?,
            );
        }
    }
    for located in requirements {
        let Clause::Predicate(predicate) = located.value else {
            return Err("higher-ranked specialization implication is not implemented".into());
        };
        let predicate = parent_fold.predicate(&mut facts.types, predicate)?;
        goals.push(
            context
                .instantiate_predicate(&mut facts.types, predicate, parent_substitution)
                .map_err(|error| error.to_string())?,
        );
    }
    Ok(goals)
}
