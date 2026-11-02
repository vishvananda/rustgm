//! A closure's copying marker requires each stored capture's actual source trait.
use super::mechanical::Pending as Language;
use super::{
    Action, Decision, Declarations, Frame, Goal, Node, Predicate, Search, Source, inference,
};
use crate::types::{Kind, infer::Instantiation};

pub(super) struct Pending {
    goals: Vec<Goal>,
    cursor: usize,
    children: Vec<usize>,
    stopped: Option<Decision>,
    suspended: Option<usize>,
    publish: bool,
}
impl Pending {
    pub(super) fn retained_child(&self) -> Option<usize> {
        self.suspended
    }
}
impl Frame {
    fn capture_markers(&mut self) -> Result<&mut Pending, String> {
        match self.language.as_deref_mut() {
            Some(Language::Captures(pending)) => Ok(pending),
            _ => Err("missing stored-capture marker frontier".into()),
        }
    }
}
impl Search<'_> {
    pub(super) fn start_capture_markers(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        goal: Predicate,
    ) -> Result<Action, String> {
        let Some(Kind::Closure {
            identity,
            environment,
            ..
        }) = facts.types.kind(goal.ty)
        else {
            return Err("capture marker requires an actual closure type".into());
        };
        if facts.types.closure_kind(identity).is_none() {
            return Ok(Action::Complete(Decision::Deferred));
        }
        let Some(Kind::Tuple(fields)) = facts.types.kind(environment) else {
            return Err("closure marker requires its typed capture tuple".into());
        };
        let count = facts
            .types
            .list(fields)
            .ok_or("missing stored capture types")?
            .len();
        frame
            .inputs
            .context
            .account_work(count.saturating_mul(2))
            .map_err(inference)?;
        if count == 0 {
            return Ok(self.complete_source_builtin(frame, goal, false));
        }
        let mut goals = Vec::new();
        goals
            .try_reserve_exact(count)
            .map_err(|_| "capture marker allocation failed")?;
        let mut substitution =
            Instantiation::new(&mut frame.inputs.context, &facts.types, &[]).map_err(inference)?;
        for index in 0..count {
            let ty = *facts
                .types
                .list(fields)
                .and_then(|f| f.get(index))
                .ok_or("stored capture type disappeared")?;
            goals.push(
                frame
                    .inputs
                    .context
                    .instantiate_predicate(
                        &mut facts.types,
                        Predicate {
                            ty,
                            bound: goal.bound,
                        },
                        &mut substitution,
                    )
                    .map_err(inference)?,
            );
        }
        let mut children = Vec::new();
        children
            .try_reserve_exact(count)
            .map_err(|_| "capture proof allocation failed")?;
        frame.language = Some(Box::new(Language::Captures(Pending {
            goals,
            cursor: 0,
            children,
            stopped: None,
            suspended: None,
            publish: false,
        })));
        Ok(Action::Descend)
    }
    pub(super) fn capture_marker_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        let first = frame.available_region().max(self.query.allocations.floor());
        let Some(Language::Captures(pending)) = frame.language.as_deref_mut() else {
            return Err("missing capture marker child frontier".into());
        };
        if let Some(slot) = pending.suspended.take() {
            let mut child = self
                .query
                .suspended
                .get_mut(slot)
                .and_then(Option::take)
                .ok_or("missing retained capture proof")?;
            child.refresh(facts, &mut frame.inputs.context)?;
            return Ok(Some(child));
        }
        let goal = pending
            .goals
            .get(pending.cursor)
            .ok_or("missing next capture goal")?;
        let mut child = Frame::new(facts, &mut frame.inputs.context, goal, first)?;
        child.scope = frame.scope;
        child.inference = if frame.inference == super::frames::Inference::Disabled {
            super::frames::Inference::Disabled
        } else {
            super::frames::Inference::Selecting
        };
        Ok(Some(child))
    }
    pub(super) fn accept_capture_marker(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        if let Decision::Proven(index) = decision {
            let Some(Language::Captures(pending)) = frame.language.as_deref_mut() else {
                return Err("missing completed capture marker frontier".into());
            };
            let goal = pending
                .goals
                .get(pending.cursor)
                .ok_or("missing current capture goal")?;
            frame
                .inputs
                .context
                .equate_proved_predicate(&mut facts.types, goal, self.query.nodes[index].goal)
                .map_err(inference)?;
            pending.children.push(index);
            pending.cursor += 1;
        } else {
            frame.capture_markers()?.stopped = Some(decision);
        }
        Ok(())
    }
    pub(super) fn accept_partial_capture_marker(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        mut child: Frame,
    ) -> Result<(), String> {
        child.publish(facts, &mut frame.inputs.context)?;
        let slot = self.query.retain_partial(child)?;
        let pending = frame.capture_markers()?;
        pending.suspended = Some(slot);
        pending.publish = true;
        Ok(())
    }
    pub(super) fn complete_capture_markers(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let pending = frame.capture_markers()?;
        if let Some(decision) = pending.stopped {
            return Ok(Action::Complete(decision));
        }
        if std::mem::take(&mut pending.publish) {
            return Ok(Action::Yield);
        }
        if pending.cursor != pending.goals.len() {
            return Ok(Action::Descend);
        }
        let children = std::mem::take(&mut pending.children);
        let end = children
            .iter()
            .map(|&i| self.query.nodes[i].end)
            .fold(frame.available_region(), u32::max);
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
}
