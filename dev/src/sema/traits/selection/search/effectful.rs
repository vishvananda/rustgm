//! A const closure proves its actual substituted defining contract in the caller.
use super::ambient::Cursor;
use super::mechanical::Pending as Language;
use super::{Action, Decision, Declarations, Frame, Node, Predicate, Search, Source, inference};
use crate::{
    sema::{closures::actual, declaration_inputs, projections::families::Requirements},
    support::constness::Constness,
    types::{
        Kind, instance::Instance, outlives::Pair, parameters::clauses::Clause, substitute::Bound,
    },
};
use std::ops::Range;

pub(super) struct Pending {
    matched: crate::sema::traits::callables::Matched,
    inputs: Box<Inputs>,
}
struct Inputs {
    goal: Predicate,
    output: crate::types::TypeId,
    requirements: Requirements,
    cursor: Cursor,
}
impl Pending {
    pub(super) fn regions(&self) -> Range<u32> {
        self.matched.regions.clone()
    }
    pub(super) fn retained_child(&self) -> Option<usize> {
        self.inputs.cursor.suspended
    }
    pub(super) fn condition(&mut self) -> (Option<Clause>, &mut Cursor) {
        let inputs = &mut self.inputs;
        (
            inputs
                .requirements
                .clauses
                .get(inputs.cursor.index)
                .copied(),
            &mut inputs.cursor,
        )
    }
}
impl Search<'_> {
    pub(super) fn begin_scoped_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        instance: Instance,
    ) -> Result<Action, String> {
        let Some(Language::Direct(matched)) = frame.language.as_deref() else {
            return Err("const closure lost its selected signature".into());
        };
        let resolved = frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
            .and_then(|goal| {
                frame
                    .inputs
                    .context
                    .resolved(&mut facts.types, matched.output)
                    .map(|output| (goal, output))
            });
        let (goal, output) = match resolved {
            Ok(value) => value,
            Err(
                crate::types::infer::Error::Unresolved | crate::types::infer::Error::Normalization,
            ) => {
                return Ok(
                    if self.query.refining && frame.scope == self.query.input_scope() {
                        Action::Yield
                    } else {
                        Action::Complete(Decision::Deferred)
                    },
                );
            }
            Err(error) => return Err(inference(error)),
        };
        if goal.ty != instance.ty() {
            return Err("const closure requirements changed their selected instance".into());
        }
        let effect = goal
            .bound
            .trait_parts()
            .ok_or("missing const callable effect")?
            .1;
        let requirements = requirements(facts, instance, effect)?;
        frame
            .inputs
            .context
            .account_work(requirements.steps)
            .map_err(inference)?;
        let count = requirements.clauses.len();
        facts.charge_caller_inputs(
            count.saturating_add(1),
            size_of::<Inputs>() + size_of::<Language>() + count * size_of::<usize>(),
        )?;
        let Some(Language::Direct(matched)) = frame.language.take().map(|value| *value) else {
            return Err("const closure equations changed during requirement preparation".into());
        };
        frame.language = Some(Box::new(Language::Scoped(Pending {
            matched,
            inputs: Box::new(Inputs {
                goal,
                output,
                requirements,
                cursor: Cursor::with_capacity(count),
            }),
        })));
        self.advance_callable_conditions(facts, frame)
    }

    pub(super) fn finish_scoped_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let Some(Language::Scoped(pending)) = frame.language.as_deref_mut() else {
            return Err("missing completed const callable frontier".into());
        };
        let inputs = &mut pending.inputs;
        if inputs.cursor.index != inputs.requirements.clauses.len()
            || inputs.cursor.stopped.is_some()
            || inputs.cursor.suspended.is_some()
        {
            return Err("const callable published before its defining conditions finished".into());
        }
        let growth = self.query.node_growth();
        facts.charge_caller_inputs(growth.work, growth.bytes)?;
        let regions = pending.matched.regions.clone();
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal: inputs.goal,
                source: Some(Source::Callable {
                    output: inputs.output,
                }),
                children: std::mem::take(&mut inputs.cursor.children),
                first: frame.first,
                end: regions.end.max(self.query.allocations.end()),
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: std::mem::take(&mut inputs.requirements.regions),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            regions,
        );
        Ok(Action::Complete(Decision::Proven(node)))
    }
}

fn requirements(
    facts: &mut Declarations,
    instance: Instance,
    effect: Constness,
) -> Result<Requirements, String> {
    let Some(Kind::Closure {
        identity,
        arguments,
        ..
    }) = facts.types.kind(instance.ty())
    else {
        return Err("const defining requirements need a complete closure instance".into());
    };
    let body = *facts
        .closures
        .identity(identity)
        .ok_or("missing const closure source")?;
    let owner = body
        .owner
        .ok_or("const closure has no defining contract owner")?;
    let count = declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)?.count();
    let region_count = owner_bounds(facts, owner).len();
    let argument_count = facts
        .types
        .arguments(arguments)
        .ok_or("missing const closure arguments")?
        .len();
    facts.charge_caller_inputs(
        count
            .saturating_add(argument_count)
            .saturating_add(region_count)
            .saturating_add(1),
        (count + 1) * size_of::<Clause>() * 2
            + size_of_val(owner_bounds(facts, owner))
            + argument_count
                * (size_of::<crate::types::arguments::Argument>() * 4 + size_of::<usize>()),
    )?;
    let arguments = actual::defining_arguments(facts, instance, body)?;
    let mut fold = {
        let (types, payments) = facts.caller_input_store()?;
        Bound::parameters_captured_charged(
            types,
            &arguments.bindings,
            arguments.captures,
            &|work, bytes| payments.charge(work, bytes),
        )?
    };
    let mut clauses = declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)?
        .map(|input| facts.member_use_clause(owner, input, effect))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(source) = facts
        .function_indices
        .get(&owner)
        .and_then(|&index| facts.functions.get(index))
        .and_then(|function| function.body)
        && let Some(clause) = facts.default_self_clause(owner, source)?
    {
        clauses.push(clause);
    }
    for clause in &mut clauses {
        let original = crate::sema::traits::effects::in_context(&mut facts.types, *clause, effect)?;
        let before = fold.steps;
        let result = fold.clause(&mut facts.types, original);
        facts.charge_caller_inputs(fold.steps.saturating_sub(before), 0)?;
        *clause = result?;
    }
    let mut regions = owner_bounds(facts, owner).to_vec();
    for (left, right) in &mut regions {
        *left = fold.region(*left, 0)?;
        *right = fold.region(*right, 0)?;
    }
    facts.charge_caller_inputs(regions.len().saturating_mul(2), 0)?;
    Ok(Requirements {
        clauses,
        regions,
        steps: fold.steps,
    })
}

fn owner_bounds(facts: &Declarations, owner: crate::resolve::model::DefId) -> &[Pair] {
    facts
        .lifetimes
        .functions
        .get(&owner)
        .map(|scheme| scheme.bounds.as_slice())
        .or_else(|| {
            facts
                .associated
                .schema(owner)
                .map(|schema| schema.bounds.as_slice())
        })
        .map_or(&[], |bounds| bounds)
}
