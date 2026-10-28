//! Closed source-call effects retain unmet facts; each execution selects afresh.
use super::{Catalog, calls::Effect};
use crate::{
    resolve::model::DefId,
    sema::{
        model::Declarations,
        traits::{assumptions::Assumptions, selection},
    },
    types::{
        Kind, Primitive, TypeId,
        infer::{Context, Instantiation},
        instance::Instance,
        parameters::{clauses::Clause, environment::Environment, predicates::Bound},
    },
};
use std::sync::Arc;
mod closed_inputs;
use closed_inputs::{closed_arguments, closed_clause, open_closed_clause};

/// Immutable materialization data, without caller permission or a selected proof.
pub(crate) struct Requirements {
    instance: Instance,
    definition: DefId,
    signature: TypeId,
    result: Primitive,
    environment: Arc<Environment>,
    goals: Vec<Clause>,
}
/// Fresh semantic success is still not an interpreter execution token.
/// The later owner binds it to one checked caller statement and consumes it.
pub(crate) struct Payment {
    requirements: Arc<Requirements>,
    caller: Option<Arc<Environment>>,
}
impl Catalog {
    pub(crate) fn requires_const_payment(&self, definition: DefId) -> Result<bool, &'static str> {
        match self.prepared_call_effect(definition)? {
            Some(Effect::Ordinary) => Ok(false),
            Some(Effect::Conditional) => Ok(true),
            None => Err("source const-call effects require completed declaration preparation"),
        }
    }
}
impl Requirements {
    pub(crate) fn signature(&self) -> TypeId {
        self.signature
    }
    pub(crate) fn result(&self) -> Primitive {
        self.result
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(crate::support::retained::vector(&self.goals))
    }
    pub(crate) fn matches(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<(), &'static str> {
        if instance != self.instance
            || !facts.is_const_function(self.definition)
            || !facts
                .source_bindings
                .as_deref()
                .ok_or("source const-call catalog disappeared")?
                .requires_const_payment(self.definition)?
            || !facts
                .generics
                .environments
                .get(&self.definition)
                .is_some_and(|env| Arc::ptr_eq(env, &self.environment))
        {
            return Err(
                "source const-call materialization belongs to another declaration or instance",
            );
        }
        Ok(())
    }
}
impl Payment {
    pub(crate) fn validate(
        &self,
        requirements: &Arc<Requirements>,
        caller: Option<&Arc<Environment>>,
    ) -> Result<(), &'static str> {
        let same = match (&self.caller, caller) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        };
        if !Arc::ptr_eq(&self.requirements, requirements) || !same {
            return Err("source const payment belongs to another use environment");
        }
        Ok(())
    }
}

/// Signature/profile validation is structural data, not a paid use witness.
pub(crate) fn profile(
    facts: &Declarations,
    instance: Instance,
) -> Result<(DefId, TypeId, crate::types::GenericArgs, Primitive), &'static str> {
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts.types.kind(instance.ty())
    else {
        return Err("source const-call target is not a function item");
    };
    let definition = DefId(item.0);
    // Intrinsic results retain their separate fresh-use witness and kernel.
    // A paid ordinary-function call must never execute a source placeholder.
    if facts
        .source_bindings
        .as_deref()
        .ok_or("source const-call catalog is absent")?
        .intrinsic_kind(definition)?
        .is_some()
    {
        return Err(
            "source conditional const-call payment for intrinsic targets is not implemented",
        );
    }
    if !facts
        .source_bindings
        .as_deref()
        .ok_or("source const-call catalog is absent")?
        .requires_const_payment(definition)?
        || !facts.is_const_function(definition)
    {
        return Err("source const payment requires an actual conditional const declaration");
    }
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("source const payment currently requires a closed callable signature");
    };
    let Some(Kind::Primitive(primitive)) = facts.types.kind(result) else {
        return Err("source const payment currently requires a primitive result");
    };
    if primitive == Primitive::Str
        || !closed_arguments(&facts.types, parameters)?
        || !facts.types.arguments_are_concrete(arguments)?
        || facts.types.needs_region_context(instance.ty()) != Some(false)
    {
        return Err(
            "source const payment for this open, lifetime-bearing or argument shape is not implemented",
        );
    }
    Ok((definition, signature, arguments, primitive))
}
pub(crate) fn prepare(
    facts: &mut Declarations,
    instance: Instance,
    budget: usize,
) -> Result<Arc<Requirements>, &'static str> {
    let (definition, signature, _, primitive) = profile(facts, instance)?;
    let environment = facts
        .generics
        .environments
        .get(&definition)
        .cloned()
        .ok_or("source const payment requires its frozen declaration environment")?;
    // Preflight both temporary copied clauses and retained goal capacity before
    // allocation. This declaration walk happens once per complete materialization.
    let count = facts
        .generics
        .declared_predicates_in_scope(definition)
        .count();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(count);
    let mut fold = closed_inputs::substitution(facts, instance, count, budget)?;
    let clauses = facts
        .generics
        .declared_predicates_in_scope(definition)
        .map(|p| p.value)
        .collect::<Vec<_>>();
    let mut goals = Vec::with_capacity(count);
    for clause in clauses {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) => {
                facts
                    .types
                    .quantified(id)
                    .ok_or("source const clause binder is missing")?
                    .predicate
            }
        };
        if !matches!(predicate.bound, Bound::MaybeConstTrait(_)) {
            continue;
        }
        closed_clause(&facts.types, clause)?;
        let copied = crate::sema::traits::effects::in_context(
            &mut facts.types,
            clause,
            crate::support::constness::Constness::Const,
        )?;
        let before = fold.steps;
        let mapped = fold.clause(&mut facts.types, copied);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(fold.steps.saturating_sub(before));
        let mapped = mapped?;
        let predicate = closed_clause(&facts.types, mapped)?;
        if !predicate.is_concrete(&facts.types)? || predicate.contains_projections(&facts.types)? {
            return Err("source const payment requires closed normalized obligations");
        }
        goals.push(mapped);
    }
    if goals.is_empty() {
        return Err("conditional source declaration has no retained effect obligations");
    }
    Ok(Arc::new(Requirements {
        instance,
        definition,
        signature,
        result: primitive,
        environment,
        goals,
    }))
}
pub(crate) fn pay(
    facts: &mut Declarations,
    requirements: &Arc<Requirements>,
    caller: Option<&Arc<Environment>>,
) -> Result<Payment, &'static str> {
    requirements.matches(facts, requirements.instance)?;
    let (assumptions, work) = Assumptions::from_environment(&facts.types, caller.map(Arc::as_ref))?;
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    for &clause in &requirements.goals {
        let predicate = open_closed_clause(facts, clause)?;
        let mut context = Context::default();
        let mut fold = Instantiation::new(&mut context, &facts.types, &[])
            .map_err(|_| "source const goal instantiation failed")?;
        let goal = context
            .instantiate_predicate(&mut facts.types, predicate, &mut fold)
            .map_err(|_| "source const goal instantiation failed")?;
        let selected = selection::select(
            facts,
            &mut context,
            &goal,
            selection::Caller {
                assumptions: &assumptions,
                environment: caller,
                first_region: 1,
                retain_sized: false,
            },
        );
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(context.steps);
        match selected
            .map_err(|_| "source const-call selection requires completed scoped prerequisites")?
        {
            selection::Selection::Proven(evidence)
                if evidence.proof.is_empty()
                    && evidence.sized.is_empty()
                    && !evidence.provisional_size => {}
            selection::Selection::Proven(_) => {
                return Err(
                    "source const-call payment of new lifetime or size debts is not implemented",
                );
            }
            selection::Selection::Deferred => {
                return Err("source const-call eligibility is still pending");
            }
            selection::Selection::Refuted(refutation) => {
                refutation.reject()?;
                return Err("source conditional const-call obligation is not satisfied");
            }
            selection::Selection::Absent => {
                return Err("source conditional const-call obligation is not satisfied");
            }
        }
    }
    Ok(Payment {
        requirements: Arc::clone(requirements),
        caller: caller.cloned(),
    })
}

#[cfg(test)]
#[path = "../../../tests/invariants/const_payment_facts.rs"]
mod tests;
