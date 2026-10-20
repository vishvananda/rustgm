//! Declaration rules and caller-owned size queries have distinct lifetimes.
mod query;
use crate::resolve::unit::NodeKey;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        ItemId, Kind, Store, TypeId,
        parameters::environment::{BuiltinQuery, Environment},
        sizedness::Rules,
    },
};
pub(in crate::sema) use query::{Progress, Query};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub(in crate::sema) struct Requirement {
    pub body: NodeKey,
    pub source: NodeKey,
    pub ty: TypeId,
    pub environment: Option<Arc<Environment>>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::sema) struct Dispatch {
    pub predicate: crate::types::parameters::predicates::Predicate,
    pub owner: DefId,
    pub arguments: crate::types::GenericArgs,
}

#[derive(Default)]
pub(in crate::sema) struct Pending {
    pub active: HashSet<ItemId>,
    pub rules: Rules,
    queries: HashMap<ItemId, Option<Box<BuiltinQuery>>>,
    pub requirements: Vec<Requirement>,
    required: HashSet<(NodeKey, TypeId)>,
    pub dispatches: Vec<Dispatch>,
    dispatched: HashMap<crate::types::parameters::predicates::Predicate, usize>,
}
impl Pending {
    pub fn require_dispatch(&mut self, dispatch: Dispatch) -> Result<(), &'static str> {
        if let Some(&index) = self.dispatched.get(&dispatch.predicate) {
            return if self.dispatches[index] == dispatch {
                Ok(())
            } else {
                Err("provisional constant selection changed before field completion")
            };
        }
        if self.dispatches.len() >= 1_000_000 {
            return Err("deferred dispatch sizedness budget exceeded");
        }
        self.dispatched
            .insert(dispatch.predicate, self.dispatches.len());
        self.dispatches.push(dispatch);
        Ok(())
    }

    pub fn require(&mut self, requirement: Requirement) -> Result<(), &'static str> {
        if self.required.insert((requirement.body, requirement.ty)) {
            if self.requirements.len() >= 1_000_000 {
                return Err("deferred sizedness requirement budget exceeded");
            }
            self.requirements.push(requirement);
        }
        Ok(())
    }
}

pub(in crate::sema) fn prepare(
    facts: &mut Declarations,
    item: ItemId,
) -> Result<Option<ItemId>, &'static str> {
    let pending = facts
        .generics
        .size_rules
        .get_or_insert_with(Default::default);
    let environments = &facts.generics.environments;
    pending.rules.prepare(
        &mut facts.types,
        &facts.aggregates,
        item,
        |types, owner, ty| {
            let Some(environment) = environments.get(&DefId(owner.0)) else {
                return Ok(false);
            };
            assumed(
                types,
                ty,
                environment,
                pending.queries.entry(owner).or_default(),
            )
        },
    )
}

fn assumed(
    types: &Store,
    ty: TypeId,
    environment: &Arc<Environment>,
    query: &mut Option<Box<BuiltinQuery>>,
) -> Result<bool, &'static str> {
    if environment.assumed(ty).clone_ {
        return Ok(true);
    }
    match types.kind(ty) {
        Some(Kind::Parameter(id)) if environment.abilities(id).is_some_and(|a| a.clone_) => {
            return Ok(true);
        }
        Some(Kind::Projection(application))
            if environment.projection_promises(application.item).sized =>
        {
            return Ok(true);
        }
        _ => {}
    }
    environment.matching_sized(types, ty, 0, query)
}

/// Final rules are immutable. Each semantic query caches only their substitution,
/// keeping caller normalization and its evidence in the query's own scope.
#[derive(Default)]
struct Conditions {
    projections: HashMap<TypeId, TypeId>,
    steps: usize,
}
impl Conditions {
    fn apply(
        &mut self,
        types: &mut Store,
        ty: TypeId,
    ) -> Result<crate::types::sizedness::Condition, &'static str> {
        use crate::types::{
            arguments::Argument,
            sizedness::{Condition, Rule},
        };
        let Some(Kind::Adt(item, arguments)) = types.kind(ty) else {
            return Err("nominal size condition requires an aggregate");
        };
        Ok(
            match types
                .nominal_size_rule(item)
                .ok_or("missing sealed size rule")?
            {
                Rule::Sized => Condition::Sized,
                Rule::Unsized => Condition::Unsized,
                Rule::Argument(slot) => {
                    let Some(Argument::Type(ty)) = types
                        .arguments(arguments)
                        .and_then(|mut args| args.nth(slot as usize))
                    else {
                        return Err("missing nominal sizedness argument");
                    };
                    Condition::Type(ty)
                }
                Rule::Projection(template) => {
                    let result = if let Some(&result) = self.projections.get(&ty) {
                        result
                    } else {
                        let mut fold =
                            crate::types::substitute::Bound::joint(types, item, arguments)?;
                        let result = fold.apply(types, template)?;
                        self.steps = self.steps.saturating_add(fold.steps).saturating_add(1);
                        if self.steps > 1_000_000 {
                            return Err("projected size substitution budget exceeded");
                        }
                        self.projections.insert(ty, result);
                        result
                    };
                    Condition::Type(result)
                }
            },
        )
    }
}
