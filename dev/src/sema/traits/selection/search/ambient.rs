//! Callable defining clauses share one scoped cursor and retained child protocol.
use super::{Action, Decision, Declarations, Frame, Search};
use crate::{
    support::hash::HashMap,
    types::{lifetime_proof::ScopeId, parameters::clauses::Clause},
};

#[derive(Default)]
pub(super) struct Cursor {
    pub index: usize,
    pub children: Vec<usize>,
    universes: HashMap<u32, ScopeId>,
    pub stopped: Option<Decision>,
    pub suspended: Option<usize>,
    publish: bool,
}
impl Cursor {
    pub(super) fn with_capacity(count: usize) -> Self {
        Self {
            children: Vec::with_capacity(count),
            ..Self::default()
        }
    }
}
impl super::mechanical::Pending {
    fn condition(&mut self) -> Result<(Option<Clause>, &mut Cursor), String> {
        match self {
            Self::Async(pending) => pending.condition(),
            Self::Scoped(pending) => Ok(pending.condition()),
            _ => Err("missing scoped callable conditions".into()),
        }
    }
}
impl Frame {
    fn condition(&mut self) -> Result<(Option<Clause>, &mut Cursor), String> {
        self.language
            .as_deref_mut()
            .ok_or("missing scoped callable frontier")?
            .condition()
    }
}
impl Search<'_> {
    pub(super) fn advance_callable_conditions(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        loop {
            let (clause, cursor) = frame.condition()?;
            if let Some(decision) = cursor.stopped {
                return Ok(Action::Complete(decision));
            }
            if std::mem::take(&mut cursor.publish) {
                return Ok(Action::Yield);
            }
            let Some(clause) = clause else {
                return match frame.language.as_deref() {
                    Some(super::mechanical::Pending::Async(_)) => {
                        self.finish_async_callable(facts, frame)
                    }
                    _ => self.finish_scoped_callable(facts, frame),
                };
            };
            let Clause::UnstableFeature(id) = clause else {
                return Ok(Action::Descend);
            };
            if !self.feature_condition(facts, &mut frame.inputs.context, id)? {
                return Ok(Action::Complete(Decision::Absent));
            }
            frame.condition()?.1.index += 1;
        }
    }

    pub(super) fn scoped_callable_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Frame, String> {
        let (clause, cursor) = frame
            .language
            .as_deref_mut()
            .ok_or("missing scoped callable frontier")?
            .condition()?;
        if let Some(slot) = cursor.suspended.take() {
            let mut child = self
                .query
                .suspended
                .get_mut(slot)
                .and_then(Option::take)
                .ok_or("missing retained callable condition")?;
            child.refresh(facts, &mut frame.inputs.context)?;
            return Ok(child);
        }
        let (types, payments) = facts.caller_input_store()?;
        self.clause_frame_types::<true>(
            types,
            &mut frame.inputs.context,
            super::bound_frames::ClauseRequest {
                clause: clause.ok_or("missing next callable condition")?,
                scope: frame.scope,
            },
            &mut cursor.universes,
            &|work, bytes| payments.charge(work, bytes),
        )
    }

    pub(super) fn accept_scoped_callable(
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let (_, cursor) = frame.condition()?;
        match decision {
            Decision::Proven(node) => {
                cursor.children.push(node);
                cursor.index += 1;
            }
            Decision::Absent | Decision::Deferred => cursor.stopped = Some(decision),
            Decision::Refuted(_) => return Err("negative callable child was not classified".into()),
        }
        Ok(())
    }

    pub(super) fn accept_partial_scoped_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        mut child: Frame,
    ) -> Result<(), String> {
        child.publish(facts, &mut frame.inputs.context)?;
        let slot = self.query.retain_partial(child)?;
        let (_, cursor) = frame.condition()?;
        cursor.suspended = Some(slot);
        cursor.publish = true;
        Ok(())
    }
}
