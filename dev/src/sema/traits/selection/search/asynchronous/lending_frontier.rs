//! Actual async trait clauses use the ordinary scoped clause and continuation engine.
use super::{Action, Clause, Decision, Declarations, Frame, Search};
impl Search<'_> {
    pub(super) fn advance_async_conditions(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        loop {
            let inputs = frame
                .async_callable()?
                .inputs
                .as_deref_mut()
                .ok_or("missing async inputs")?;
            if let Some(decision) = inputs.stopped {
                return Ok(Action::Complete(decision));
            }
            if std::mem::take(&mut inputs.publish) {
                return Ok(Action::Yield);
            }
            let Some(clause) = inputs.clause() else {
                return self.finish_async_callable(facts, frame);
            };
            let Clause::UnstableFeature(id) = clause else {
                return Ok(Action::Descend);
            };
            if !self.feature_condition(facts, &mut frame.inputs.context, id)? {
                return Ok(Action::Complete(Decision::Absent));
            }
            frame
                .async_callable()?
                .inputs
                .as_deref_mut()
                .ok_or("missing async inputs")?
                .cursor += 1;
        }
    }

    pub(in crate::sema::traits::selection::search) fn async_callable_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Frame, String> {
        let scope = frame.scope;
        let super::Language::Async(pending) = frame
            .language
            .as_deref_mut()
            .ok_or("missing async frontier")?
        else {
            return Err("different async frontier".into());
        };
        let inputs = pending
            .inputs
            .as_deref_mut()
            .ok_or("missing async conditions")?;
        if let Some(slot) = inputs.suspended.take() {
            let mut child = self
                .query
                .suspended
                .get_mut(slot)
                .and_then(Option::take)
                .ok_or("missing retained async child")?;
            child.refresh(facts, &mut frame.inputs.context)?;
            return Ok(child);
        }
        let (types, payments) = facts.caller_input_store()?;
        let clause = inputs.clause().ok_or("missing actual async clause")?;
        self.clause_frame_types::<true>(
            types,
            &mut frame.inputs.context,
            super::super::bound_frames::ClauseRequest { clause, scope },
            &mut inputs.universes,
            &|work, bytes| payments.charge(work, bytes),
        )
    }

    pub(in crate::sema::traits::selection::search) fn accept_async_callable(
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let inputs = frame
            .async_callable()?
            .inputs
            .as_deref_mut()
            .ok_or("missing async parent")?;
        match decision {
            Decision::Proven(node) => {
                inputs.children.push(node);
                inputs.cursor += 1;
            }
            Decision::Absent | Decision::Deferred => inputs.stopped = Some(decision),
            Decision::Refuted(_) => return Err("negative async child was not classified".into()),
        }
        Ok(())
    }

    pub(in crate::sema::traits::selection::search) fn accept_partial_async_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        mut child: Frame,
    ) -> Result<(), String> {
        child.publish(facts, &mut frame.inputs.context)?;
        let slot = self.query.retain_partial(child)?;
        let inputs = frame
            .async_callable()?
            .inputs
            .as_deref_mut()
            .ok_or("missing partial async parent")?;
        inputs.suspended = Some(slot);
        inputs.publish = true;
        Ok(())
    }
}
