//! Hidden Sized projections descend with the requesting frame's scope and evidence.
use super::{Action, Decision, Declarations, Dependency, Frame, Predicate, Search, inference};
use crate::{
    sema::nominals::sizes,
    types::{TypeId, infer::Instantiation, parameters::predicates::Bound},
};
use std::ops::ControlFlow;

struct Tail {
    subject: TypeId,
    bound: Bound,
    answer: Option<bool>,
}
pub(super) struct Pending {
    query: sizes::Query,
    tail: Option<Tail>,
    needed: Option<TypeId>,
    stopped: Option<Decision>,
    pub children: Vec<usize>,
    pub end: u32,
}

impl Search<'_> {
    pub(super) fn storage_condition(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        ty: TypeId,
    ) -> Result<ControlFlow<Action, (bool, bool)>, String> {
        let first = frame.available_region();
        let pending = frame.storage.get_or_insert_with(|| {
            Box::new(Pending {
                query: sizes::Query::with_placeholders(
                    self.caller.environment.cloned(),
                    self.query
                        .negative_sizing
                        .as_deref()
                        .map(|sizing| sizing.placeholders().clone()),
                ),
                tail: None,
                needed: None,
                stopped: None,
                children: Vec::new(),
                end: first,
            })
        });
        if let Some(decision) = pending.stopped {
            return Ok(ControlFlow::Break(Action::Complete(decision)));
        }
        if pending.tail.is_some() {
            return Err("size proof resumed as a source metadata-tail proof".into());
        }
        let query = &mut pending.query;
        let before = query.steps;
        let step = query.advance(facts, ty)?;
        #[cfg(test)]
        {
            self.query.storage_steps += query.steps - before;
        }
        frame
            .inputs
            .context
            .account_work(query.steps - before)
            .map_err(inference)?;
        Ok(match step {
            sizes::Step::Ready { sized, provisional } => {
                ControlFlow::Continue((sized, provisional))
            }
            sizes::Step::Fields(item) => ControlFlow::Break(Action::Wait(Dependency::Fields(item))),
            sizes::Step::Projection(ty) => {
                pending.needed = Some(ty);
                ControlFlow::Break(Action::Descend)
            }
        })
    }

    /// Unknown physical metadata does not disprove a scoped source sizing bound.
    /// Its actual terminal enters the same iterative, scoped proof stack.
    pub(super) fn source_tail_condition(
        frame: &mut Frame,
        subject: TypeId,
        bound: Bound,
    ) -> Result<ControlFlow<Action, bool>, String> {
        let first = frame.available_region();
        let pending = frame.storage.get_or_insert_with(|| {
            Box::new(Pending {
                // This query holds only a tail goal and is never advanced; a
                // change that advances it must thread the negative Sized set.
                query: sizes::Query::new(None),
                tail: Some(Tail {
                    subject,
                    bound,
                    answer: None,
                }),
                needed: Some(subject),
                stopped: None,
                children: Vec::new(),
                end: first,
            })
        });
        if let Some(decision) = pending.stopped {
            return Ok(ControlFlow::Break(Action::Complete(decision)));
        }
        match pending.tail {
            Some(Tail {
                subject: actual,
                bound: requested,
                answer,
            }) if actual == subject && requested == bound => {
                Ok(answer.map_or(ControlFlow::Break(Action::Descend), ControlFlow::Continue))
            }
            _ => Err("source metadata-tail proof changed its actual goal".into()),
        }
    }

    pub(super) fn storage_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        if frame.normalizing() || frame.forming() {
            return Ok(None);
        }
        let Some(pending) = &frame.storage else {
            return Ok(None);
        };
        let ty = pending.needed.ok_or("missing projected size subject")?;
        let bound = pending
            .tail
            .as_ref()
            .map_or(Bound::Sized, |tail| tail.bound);
        let first = pending.end.max(self.query.allocations.floor());
        let mut substitution =
            Instantiation::new(&mut frame.inputs.context, &facts.types, &[]).map_err(inference)?;
        let goal = frame
            .inputs
            .context
            .instantiate_predicate(&mut facts.types, Predicate { ty, bound }, &mut substitution)
            .map_err(inference)?;
        let mut child = Frame::new(facts, &mut frame.inputs.context, &goal, first)?;
        child.scope = frame.scope;
        Ok(Some(child))
    }

    pub(super) fn accept_storage(
        &mut self,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let pending = frame
            .storage
            .as_mut()
            .ok_or("missing size proof frontier")?;
        let ty = pending.needed.take().ok_or("missing size proof request")?;
        let (sized, provisional) = match decision {
            Decision::Refuted(_) => return Err("negative child was not classified".into()),
            Decision::Proven(index) => {
                let child = &self.query.nodes[index];
                pending.children.push(index);
                pending.end = pending.end.max(child.end);
                (true, child.provisional_size)
            }
            Decision::Absent => (false, false),
            Decision::Deferred => {
                pending.stopped = Some(decision);
                return Ok(());
            }
        };
        match &mut pending.tail {
            None => pending.query.answer_projection(ty, sized, provisional)?,
            Some(Tail {
                subject, answer, ..
            }) if *subject == ty && answer.is_none() => {
                *answer = Some(sized);
            }
            Some(_) => return Err("source tail completed a different proof".into()),
        }
        Ok(())
    }
}
