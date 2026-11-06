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
    cleanups: Cleanups<'a>,
    selected: Option<&'a crate::borrow::ownership::Selected>,
}
#[derive(Clone, Copy)]
enum Cleanups<'a> {
    Concrete(&'a [super::Cleanup]),
    Symbolic(&'a [Cleanup]),
}
impl<'a> Schedule<'a> {
    pub(crate) fn concrete(plan: &'a Plan) -> Self {
        Self {
            events: &plan.events,
            cleanups: Cleanups::Concrete(&plan.cleanups),
            selected: None,
        }
    }
    pub(crate) const fn symbolic(events: &'a [Event], cleanups: &'a [Cleanup]) -> Self {
        Self {
            events,
            cleanups: Cleanups::Symbolic(cleanups),
            selected: None,
        }
    }
    pub(crate) fn with_selected(
        mut self,
        body: &crate::body::ir::Body,
        selected: Option<&'a crate::borrow::ownership::Selected>,
    ) -> Result<Self, &'static str> {
        if let Some(selected) = selected {
            selected.validate(body, self)?;
        }
        self.selected = selected;
        Ok(self)
    }
    pub(crate) const fn selected(self) -> Option<&'a crate::borrow::ownership::Selected> {
        self.selected
    }
    pub(crate) fn cleanups(self) -> impl Iterator<Item = Cleanup> + 'a {
        let (concrete, symbolic): (&[super::Cleanup], &[Cleanup]) = match self.cleanups {
            Cleanups::Concrete(cleanups) => (cleanups, &[]),
            Cleanups::Symbolic(cleanups) => (&[], cleanups),
        };
        concrete
            .iter()
            .map(|c| Cleanup {
                candidate: c.candidate,
                live_type: c.live_type,
                access: c.node.is_some(),
            })
            .chain(symbolic.iter().copied())
    }
    pub(crate) fn cleanup(self, index: usize) -> Option<Cleanup> {
        match self.cleanups {
            Cleanups::Symbolic(cleanups) => cleanups.get(index).copied(),
            Cleanups::Concrete(cleanups) => cleanups.get(index).map(|c| Cleanup {
                candidate: c.candidate,
                live_type: c.live_type,
                access: c.node.is_some(),
            }),
        }
    }
}
