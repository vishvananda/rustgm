//! Canonical implementation conditions descend on the existing iterative proof stack.
use super::{Context, Decision, Declarations, Frame, Predicate, Search, Trial, inference};
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, infer::Instantiation, lifetime_proof::ScopeId, parameters::clauses::Clause,
    substitute::Bound,
};

#[derive(Default)]
pub(in crate::sema::traits::selection) struct Conditions {
    clauses: Vec<Clause>,
    resolved: Option<(Predicate, GenericArgs)>,
    fold: Option<Bound>,
    groups: HashMap<u32, ScopeId>,
    cursor: usize,
    waiting: bool,
}
impl Conditions {
    pub(in crate::sema::traits::selection) fn push(&mut self, clause: Clause) {
        self.clauses.push(clause);
    }
    pub(super) fn waiting(&self) -> bool {
        self.waiting
    }
}
impl Trial {
    /// Universal children have isolated inference and cannot refine this parent.
    /// Resolve its complete application once, then share it across all conditions.
    pub(super) fn resolved_parent(
        &mut self,
        facts: &mut Declarations,
    ) -> Result<(Predicate, GenericArgs), crate::types::infer::Error> {
        if let Some(resolved) = self.instance.quantified.as_ref().and_then(|q| q.resolved) {
            return Ok(resolved);
        }
        let goal = self
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &self.goal)?;
        let arguments = self.instance.arguments(facts, &mut self.inputs.context)?;
        if let Some(conditions) = &mut self.instance.quantified {
            conditions.resolved = Some((goal, arguments));
        }
        Ok((goal, arguments))
    }
}
impl Search<'_> {
    pub(super) fn conditions(
        &mut self,
        facts: &mut Declarations,
        trial: &mut Trial,
        arguments: GenericArgs,
    ) -> Result<bool, String> {
        let Some(conditions) = &mut trial.instance.quantified else {
            return Ok(true);
        };
        if conditions.cursor == conditions.clauses.len() {
            return Ok(true);
        }
        if self.caller.retain_sized {
            return Err("quantified impl conditions require scoped sized requirements".into());
        }
        if conditions.fold.is_none() {
            let arguments = facts
                .types
                .arguments(arguments)
                .ok_or("missing impl arguments")?;
            let bindings: Vec<_> = trial
                .instance
                .bindings
                .iter()
                .map(|&(id, _)| id)
                .zip(arguments.skip(trial.instance.regions.len()))
                .collect();
            let fold = Bound::parameters(&facts.types, &bindings)?;
            trial
                .inputs
                .context
                .account_work(fold.steps)
                .map_err(inference)?;
            conditions.fold = Some(fold);
        }
        conditions.waiting = true;
        Ok(false)
    }

    pub(super) fn condition_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        if frame.normalizing() {
            return Ok(None);
        }
        let Some(trial) = &mut frame.trial else {
            return Ok(None);
        };
        let Some(conditions) = &mut trial.instance.quantified else {
            return Ok(None);
        };
        if !conditions.waiting {
            return Ok(None);
        }
        let fold = conditions
            .fold
            .as_mut()
            .ok_or("missing quantified impl substitution")?;
        let before = fold.steps;
        let Clause::ForAll(id) =
            fold.clause(&mut facts.types, conditions.clauses[conditions.cursor])?
        else {
            return Err("quantified impl condition lost its binder".into());
        };
        trial
            .inputs
            .context
            .account_work(fold.steps.saturating_sub(before))
            .map_err(inference)?;
        let value = facts
            .types
            .quantified(id)
            .ok_or("missing quantified impl condition")?;
        let first = self.query.allocations.reserve_scoped();
        let scopes = self.query.scopes.get_or_insert_with(Default::default);
        let scope = if let Some(&scope) = conditions.groups.get(&value.bound_regions) {
            scope
        } else {
            trial
                .inputs
                .context
                .account_work(value.bound_regions as usize)
                .map_err(inference)?;
            let scope = scopes.introduce(trial.scope, value.bound_regions, first)?;
            conditions.groups.insert(value.bound_regions, scope);
            scope
        };
        let (predicate, next, steps) = scopes.open(facts, scope, value.predicate, first)?;
        self.query.allocations.record(next);
        trial
            .inputs
            .context
            .account_work(steps)
            .map_err(inference)?;
        let mut context = Context::default();
        let mut substitution =
            Instantiation::new(&mut context, &facts.types, &[]).map_err(inference)?;
        let goal = context
            .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
            .map_err(inference)?;
        let mut child = Frame::new(facts, &mut context, &goal, next)?;
        trial
            .inputs
            .context
            .account_work(context.steps)
            .map_err(inference)?;
        child.scope = scope;
        Ok(Some(child))
    }

    pub(super) fn accept_condition(
        &mut self,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let trial = frame
            .trial
            .as_mut()
            .ok_or("missing quantified impl candidate")?;
        let conditions = trial
            .instance
            .quantified
            .as_mut()
            .ok_or("missing quantified impl conditions")?;
        match decision {
            Decision::Proven(node) => {
                trial.children.push(node);
                trial.end = trial
                    .end
                    .max(self.query.nodes[node].end)
                    .max(self.query.allocations.end());
                conditions.cursor += 1;
                conditions.waiting = false;
            }
            Decision::Absent | Decision::Deferred => {
                frame.deferred |= decision == Decision::Deferred;
                self.charge(&mut trial.inputs.context)?;
                frame.trial = None;
            }
        }
        Ok(())
    }
}
