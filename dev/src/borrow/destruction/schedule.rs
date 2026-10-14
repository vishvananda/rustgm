//! Borrow analysis consumes effects and access obligations, never native glue.
use super::{Event, Plan};
use crate::{borrow::ownership::initialization::Candidate, types::TypeId};

#[derive(Clone, Copy)]
pub(crate) struct Cleanup {
    pub candidate: Candidate,
    pub live_type: Option<TypeId>,
    pub access: bool,
}
#[derive(Clone, Copy)]
pub(crate) struct Schedule<'a> {
    pub events: &'a [Event],
    concrete: &'a [super::Cleanup],
    symbolic: &'a [Cleanup],
}
impl<'a> Schedule<'a> {
    pub(crate) fn concrete(plan: &'a Plan) -> Self {
        Self {
            events: &plan.events,
            concrete: &plan.cleanups,
            symbolic: &[],
        }
    }
    pub(crate) const fn symbolic(events: &'a [Event], cleanups: &'a [Cleanup]) -> Self {
        Self {
            events,
            concrete: &[],
            symbolic: cleanups,
        }
    }
    pub(crate) fn cleanups(self) -> impl Iterator<Item = Cleanup> + 'a {
        self.concrete
            .iter()
            .map(|c| Cleanup {
                candidate: c.candidate,
                live_type: c.live_type,
                access: c.node.is_some(),
            })
            .chain(self.symbolic.iter().copied())
    }
    pub(crate) fn cleanup(self, index: usize) -> Option<Cleanup> {
        if self.concrete.is_empty() {
            self.symbolic.get(index).copied()
        } else {
            self.concrete.get(index).map(|c| Cleanup {
                candidate: c.candidate,
                live_type: c.live_type,
                access: c.node.is_some(),
            })
        }
    }
}
