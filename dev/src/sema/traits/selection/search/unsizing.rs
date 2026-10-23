//! Generated Unsize evidence owns every ordinary input and conversion obligation.
#[cfg(test)]
#[path = "../../../../../tests/invariants/coercion_formation.rs"]
mod formation_tests;
mod input_debts;
mod plan;
#[cfg(test)]
#[path = "../../../../../tests/invariants/coercion_frontier.rs"]
mod tests;
use super::mechanical::Pending as Mechanical;
use super::{Action, Decision, Declarations, Frame, Goal, Node, Search, Source, inference};
use crate::{
    resolve::model::DefId,
    support::{bindings::LangItem, constness::Constness},
    types::{
        ItemId,
        infer::{self, arguments::Argument},
    },
};

pub(super) struct Pending {
    goals: Vec<Goal>,
    cursor: u32,
    children: Vec<usize>,
    phase: Phase,
    pub end: u32,
}
impl Pending {
    fn index(&self) -> Result<usize, String> {
        usize::try_from(self.cursor).map_err(|_| "Unsize cursor exceeds host indexing".into())
    }
}

#[derive(Clone, Copy)]
enum Completion {
    Proven,
    Absent,
    Deferred,
}
enum Phase {
    Ready(Completion),
    Suspended {
        slot: usize,
        publish: bool,
        completion: Completion,
    },
}
impl Pending {
    fn completion(&self) -> Completion {
        match self.phase {
            Phase::Ready(completion) | Phase::Suspended { completion, .. } => completion,
        }
    }
    fn advance(&mut self) -> Result<(), String> {
        self.cursor = self.cursor.checked_add(1).ok_or("Unsize cursor overflow")?;
        Ok(())
    }
}

impl Frame {
    fn unsizing(&mut self) -> Result<&mut Pending, String> {
        match self.language.as_deref_mut() {
            Some(Mechanical::Unsize(pending)) => Ok(pending),
            _ => Err("missing source Unsize conditions".into()),
        }
    }
}
impl Search<'_> {
    pub(super) fn source_unsizing_kind(
        facts: &Declarations,
        goal: &Goal,
    ) -> Result<Option<LangItem>, String> {
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(None);
        };
        let role = crate::sema::compiler_items::unsizing_role(facts, DefId(item.0))?;
        if role.is_some()
            && (!matches!(arguments, [Argument::Type(_)])
                || goal.constness() != Some(Constness::Ordinary))
        {
            return Err("source unsizing requires one ordinary target type".into());
        }
        Ok(role)
    }
    pub(super) fn source_unsizing_result(
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if Self::source_unsizing_kind(facts, &frame.goal)? != Some(LangItem::Unsize)
            || frame.partial.is_some()
            || frame.selected.is_some()
        {
            return Ok(None);
        }
        let (_, arguments) = frame
            .goal
            .trait_arguments()
            .ok_or("source Unsize lost its inputs")?;
        let [Argument::Type(target)] = arguments else {
            return Err("source Unsize target changed kind".into());
        };
        let source = frame.goal.subject();
        let target = *target;
        let mut goals = Vec::new();
        let completion = match plan::requirements(
            facts,
            &mut frame.inputs.context,
            source,
            target,
            &mut goals,
        ) {
            Ok(plan::Outcome::Ready) => Completion::Proven,
            Ok(plan::Outcome::Absent) | Err(infer::Error::Mismatch | infer::Error::Occurs) => {
                Completion::Absent
            }
            Ok(plan::Outcome::Deferred)
            | Err(infer::Error::Unresolved | infer::Error::Normalization) => Completion::Deferred,
            Ok(plan::Outcome::Unsupported(message)) => return Err(message.into()),
            Err(error) => return Err(inference(error)),
        };
        let completion = if frame.deferred {
            Completion::Deferred
        } else {
            completion
        };
        if !matches!(completion, Completion::Proven) {
            goals.clear();
        }
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("source Unsize lost its catalog")?;
        let pointee = catalog
            .language(LangItem::PointeeSized)
            .ok_or("source Unsize requires actual PointeeSized")?;
        if catalog.sizing_kind(pointee.definition)? != Some(LangItem::PointeeSized) {
            return Err("source Unsize requires completed PointeeSized".into());
        }
        frame.inputs.context.account_work(2).map_err(inference)?;
        goals.push(Goal::trait_application(
            source,
            ItemId(pointee.definition.0),
            Vec::new(),
        ));
        goals.push(Goal::trait_application(
            target,
            ItemId(pointee.definition.0),
            Vec::new(),
        ));
        // Every generated goal was charged under the existing 1,000,000-step
        // inference budget. Check the retained cursor width independently too.
        u32::try_from(goals.len()).map_err(|_| "source Unsize goal count overflow")?;
        frame.language = Some(Box::new(Mechanical::Unsize(Pending {
            goals,
            cursor: 0,
            children: Vec::new(),
            phase: Phase::Ready(completion),
            end: frame.available_region(),
        })));
        Ok(Some(Action::Descend))
    }
    pub(super) fn unsizing_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        let first = frame.available_region().max(self.query.allocations.floor());
        let Some(Mechanical::Unsize(pending)) = frame.language.as_deref_mut() else {
            return Ok(None);
        };
        if let Phase::Suspended {
            slot, completion, ..
        } = pending.phase
        {
            pending.phase = Phase::Ready(completion);
            let mut child = self
                .query
                .suspended
                .get_mut(slot)
                .and_then(Option::take)
                .ok_or("missing retained Unsize child")?;
            child.refresh(facts, &mut frame.inputs.context)?;
            return Ok(Some(child));
        }
        let goal = pending
            .goals
            .get(pending.index()?)
            .ok_or("missing source Unsize child")?;
        let mut child = Frame::new(facts, &mut frame.inputs.context, goal, first)?;
        child.scope = frame.scope;
        child.inference = if frame.inference == super::frames::Inference::Disabled {
            super::frames::Inference::Disabled
        } else {
            super::frames::Inference::Selecting
        };
        Ok(Some(child))
    }
    pub(super) fn accept_unsizing(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let scope = frame.scope;
        let first = frame.first;
        let Some(Mechanical::Unsize(pending)) = frame.language.as_deref_mut() else {
            return Err("missing source Unsize parent".into());
        };
        let input_index = pending.index()?;
        let is_input = input_index
            >= pending
                .goals
                .len()
                .checked_sub(2)
                .ok_or("missing Unsize inputs")?;
        pending.end = pending.end.max(self.query.allocations.end());
        if let Decision::Proven(node) = decision {
            frame
                .inputs
                .context
                .equate_proved_predicate(
                    &mut facts.types,
                    pending
                        .goals
                        .get(pending.index()?)
                        .ok_or("missing source Unsize child goal")?,
                    self.query.nodes[node].goal,
                )
                .map_err(inference)?;
            pending.children.push(node);
            pending.advance()?;
            pending.end = pending.end.max(self.query.nodes[node].end);
        } else {
            let completion = if matches!(decision, Decision::Deferred) {
                Completion::Deferred
            } else {
                Completion::Absent
            };
            let inputs = pending
                .goals
                .len()
                .checked_sub(2)
                .ok_or("missing Unsize formation goals")?;
            if pending.index()? >= inputs {
                if !matches!(decision, Decision::Deferred) {
                    return Err("source Unsize input formation is unsatisfied".into());
                }
                pending.advance()?;
            } else {
                if let Decision::Refuted(node) = decision {
                    pending.children.push(node);
                }
                pending.cursor =
                    u32::try_from(inputs).map_err(|_| "Unsize formation cursor overflow")?;
            }
            pending.phase = Phase::Ready(completion);
        }

        if is_input {
            let goal = pending
                .goals
                .get(input_index)
                .ok_or("missing Unsize input goal")?;
            match frame
                .inputs
                .context
                .resolved_predicate(&mut facts.types, goal)
            {
                Ok(goal) => {
                    if let Some(prefix) = self.unsizing_input_debts(
                        facts,
                        &mut frame.inputs.context,
                        goal,
                        scope,
                        first,
                        pending.end,
                    )? {
                        pending.children.push(prefix);
                    }
                }
                Err(infer::Error::Unresolved | infer::Error::Normalization) => {}
                Err(error) => return Err(inference(error)),
            }
        }
        Ok(())
    }
    pub(super) fn accept_partial_unsizing(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        mut child: Frame,
    ) -> Result<(), String> {
        child.publish(facts, &mut frame.inputs.context)?;
        let end = self.query.allocations.end();
        let slot = self.query.retain_partial(child)?;
        let pending = frame.unsizing()?;
        pending.phase = Phase::Suspended {
            slot,
            publish: true,
            completion: pending.completion(),
        };
        pending.end = pending.end.max(end);
        Ok(())
    }
    pub(super) fn complete_unsizing(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let Some(Mechanical::Unsize(pending)) = frame.language.as_deref_mut() else {
            return Err("missing completed Unsize parent".into());
        };
        if let Phase::Suspended { publish, .. } = &mut pending.phase
            && std::mem::take(publish)
        {
            return Ok(Action::Yield);
        }
        if pending.index()? != pending.goals.len() {
            return Ok(Action::Descend);
        }
        match pending.completion() {
            Completion::Absent => return Ok(Action::Complete(Decision::Absent)),
            Completion::Deferred => return Ok(Action::Complete(Decision::Deferred)),
            Completion::Proven => {}
        }
        let children = std::mem::take(&mut pending.children);
        let end = pending.end;
        let goal = frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
            .map_err(inference)?;
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::Builtin),
                children,
                first: frame.first,
                end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            frame.first..frame.first,
        );
        Ok(Action::Complete(Decision::Proven(node)))
    }
    pub(super) fn validate_unsizing_absence(
        &mut self,
        facts: &Declarations,
        frame: &mut Frame,
    ) -> Result<(), String> {
        if Self::source_unsizing_kind(facts, &frame.goal)? != Some(LangItem::Unsize) {
            return Ok(());
        }
        let count = match frame.language.as_deref() {
            Some(Mechanical::Unsize(pending)) => pending.children.len(),
            _ => 0,
        };
        frame
            .inputs
            .context
            .account_work(count.saturating_add(3))
            .map_err(inference)?;
        self.charge(&mut frame.inputs.context)?;
        if !frame.inputs.context.region_equalities.is_empty()
            || frame.normalization.as_ref().is_some_and(|n| n.pending())
        {
            return Err("Unsize absence requires completed and paid input normalization".into());
        }
        let children = match frame.language.as_deref() {
            Some(Mechanical::Unsize(pending)) => pending.children.as_slice(),
            _ => &[],
        };
        let normalized = frame.normalized_inputs.as_ref().and_then(|n| n.proof);
        let normalization = frame.normalization.as_ref().and_then(|n| n.proof);
        for child in children
            .iter()
            .copied()
            .chain(normalized)
            .chain(normalization)
        {
            if !self
                .query
                .nodes
                .get(child)
                .ok_or("missing Unsize input prefix")?
                .debt_free
            {
                return Err("conditional Unsize input formation cannot publish an absence without paying its debts".into());
            }
        }
        Ok(())
    }
}
