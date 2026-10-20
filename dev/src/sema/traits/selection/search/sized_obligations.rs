//! Hidden Sized projections descend with the requesting frame's scope and evidence.
use super::{Action, Decision, Declarations, Dependency, Frame, Predicate, Search, inference};
use crate::{
    sema::nominals::sizes,
    types::{TypeId, infer::Instantiation, parameters::predicates::Bound},
};
use std::ops::ControlFlow;

pub(super) struct Pending {
    query: sizes::Query,
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
                query: sizes::Query::new(self.caller.environment.cloned()),
                needed: None,
                stopped: None,
                children: Vec::new(),
                end: first,
            })
        });
        if let Some(decision) = pending.stopped {
            return Ok(ControlFlow::Break(Action::Complete(decision)));
        }
        let before = pending.query.steps;
        let step = pending.query.advance(facts, ty)?;
        #[cfg(test)]
        {
            self.query.storage_steps += pending.query.steps - before;
        }
        frame
            .inputs
            .context
            .account_work(pending.query.steps - before)
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
        let first = pending.end.max(self.query.allocations.floor());
        let mut substitution =
            Instantiation::new(&mut frame.inputs.context, &facts.types, &[]).map_err(inference)?;
        let goal = frame
            .inputs
            .context
            .instantiate_predicate(
                &mut facts.types,
                Predicate {
                    ty,
                    bound: Bound::Sized,
                },
                &mut substitution,
            )
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
        pending.query.answer_projection(ty, sized, provisional)?;
        Ok(())
    }
}
