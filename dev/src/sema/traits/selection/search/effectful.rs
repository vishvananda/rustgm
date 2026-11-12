//! A const closure or conditional const fn item proves its actual substituted
//! defining contract in the caller.
use super::ambient::Cursor;
use super::mechanical::Pending as Language;
use super::{Action, Decision, Declarations, Frame, Node, Predicate, Search, Source, inference};
use crate::{
    resolve::model::DefId,
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
    let (owner, fold, premise) = match facts.types.kind(instance.ty()) {
        Some(Kind::Closure {
            identity,
            arguments,
            ..
        }) => closure_inputs(facts, instance, identity, arguments)?,
        Some(Kind::FunctionItem {
            item, arguments, ..
        }) => item_inputs(facts, item, arguments)?,
        _ => {
            return Err(
                "const defining requirements need a complete closure or function item".into(),
            );
        }
    };
    conditions(facts, owner, fold, premise, effect)
}

/// A closure's defining contract is its owner's, through the owner's arguments
/// and the const trait premise of a defaulted owner.
fn closure_inputs(
    facts: &mut Declarations,
    instance: Instance,
    identity: crate::types::closure::ClosureId,
    arguments: crate::types::GenericArgs,
) -> Result<(DefId, Bound, Option<Clause>), String> {
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
    let fold = {
        let (types, payments) = facts.caller_input_store()?;
        Bound::parameters_captured_charged(
            types,
            &arguments.bindings,
            arguments.captures,
            &|work, bytes| payments.charge(work, bytes),
        )?
    };
    let premise = match facts
        .function_indices
        .get(&owner)
        .and_then(|&index| facts.functions.get(index))
        .and_then(|function| function.body)
    {
        Some(source) => facts.default_self_clause(owner, source)?,
        None => None,
    };
    Ok((owner, fold, premise))
}

/// A const fn item's contract is its own `[const]` predicates, and a const
/// trait member's also the trait's `Self: [const] Trait` premise. Both are
/// bound through the item's callable template, whose parameters include a
/// trait member's `Self`, as when its signature is instantiated.
fn item_inputs(
    facts: &mut Declarations,
    item: crate::types::ItemId,
    arguments: crate::types::GenericArgs,
) -> Result<(DefId, Bound, Option<Clause>), String> {
    let owner = DefId(item.0);
    let values = facts
        .types
        .arguments(arguments)
        .ok_or("missing const callable arguments")?
        .collect::<Vec<_>>();
    let (early, parameters) = match facts.lifetimes.callables.get(item) {
        Some(template) => (
            template.parameters as usize,
            facts
                .lifetimes
                .callables
                .generic_parameters(item)
                .ok_or("missing const callable parameters")?
                .to_vec(),
        ),
        None if values.is_empty() => (0, Vec::new()),
        None => return Err("const callable lacks its generic template".into()),
    };
    if values.len() != early.saturating_add(parameters.len()) {
        return Err("const callable argument count differs from its template".into());
    }
    let count = declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)?.count();
    let region_count = owner_bounds(facts, owner).len();
    facts.charge_caller_inputs(
        count
            .saturating_add(values.len())
            .saturating_add(region_count)
            .saturating_add(1),
        (count + 1) * size_of::<Clause>() * 2
            + size_of_val(owner_bounds(facts, owner))
            + values.len()
                * (size_of::<crate::types::arguments::Argument>() * 4 + size_of::<usize>()),
    )?;
    let regions = values[..early]
        .iter()
        .map(|argument| match argument {
            crate::types::arguments::Argument::Lifetime(region) => Ok(*region),
            _ => Err("const callable early argument is not a lifetime"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bindings = parameters
        .into_iter()
        .zip(values[early..].iter().copied())
        .collect::<Vec<_>>();
    let fold = {
        let (types, payments) = facts.caller_input_store()?;
        Bound::parameters_captured_charged(types, &bindings, regions, &|work, bytes| {
            payments.charge(work, bytes)
        })?
    };
    let premise = match facts.associated.trait_owner(owner) {
        Some(trait_definition)
            if facts.associated.constant_trait(trait_definition) == Some(true) =>
        {
            Some(facts.defining_self_clause(trait_definition)?)
        }
        _ => None,
    };
    Ok((owner, fold, premise))
}

fn conditions(
    facts: &mut Declarations,
    owner: DefId,
    mut fold: Bound,
    premise: Option<Clause>,
    effect: Constness,
) -> Result<Requirements, String> {
    let count = declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)?.count();
    // Ordinary formation and signature constraints retain their existing
    // producers, exactly as for an ordinary call. Const eligibility adds the
    // defining const conditions; it cannot certify ordinary formation.
    let mut clauses = Vec::with_capacity(count.saturating_add(1));
    for input in declaration_inputs::all(&facts.generics, facts.crates.as_deref(), owner)? {
        if const_condition(&facts.types, input.value)? {
            clauses.push(input.value);
        }
    }
    clauses.extend(premise);
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

fn const_condition(store: &crate::types::Store, clause: Clause) -> Result<bool, &'static str> {
    let predicate = match clause {
        Clause::UnstableFeature(_) => return Ok(true),
        Clause::Predicate(predicate) => predicate,
        Clause::ForAll(id) => {
            store
                .quantified(id)
                .ok_or("const defining condition lost its original binder")?
                .predicate
        }
    };
    Ok(predicate
        .bound
        .trait_parts()
        .is_some_and(|(_, effect)| effect != Constness::Ordinary))
}

fn owner_bounds(facts: &Declarations, owner: DefId) -> &[Pair] {
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
