//! Nested body states are parked in one flat scheduler with body-specific payloads.
mod completion;
mod incoming_fields;
pub(super) mod output;
use super::{Checker, Dependency, Error, NodeId, State, Work, closures, inline};
use crate::{
    sema::traits::selection::Source,
    types::infer::{Solved, arguments::Argument, refinement::Refinement},
};
use std::collections::HashMap;

enum Direction {
    Inward,
    Outward,
}
pub(super) struct InputRefresh {
    pub source: NodeId,
    pub pairs: Vec<(Argument, Argument)>,
    direction: Direction,
    prepared: bool,
}
impl InputRefresh {
    pub fn incoming(source: NodeId, pairs: Vec<(Argument, Argument)>) -> Self {
        Self {
            source,
            pairs,
            direction: Direction::Inward,
            prepared: false,
        }
    }
    pub fn outgoing(source: NodeId, pairs: Vec<(Argument, Argument)>) -> Self {
        Self {
            source,
            pairs,
            direction: Direction::Outward,
            prepared: true,
        }
    }
}

pub(super) enum Payload {
    Closure(closures::Metadata),
    Inline(inline::Metadata),
}

#[derive(Clone, Copy)]
enum Kind {
    Closure,
    Inline,
}

impl Payload {
    fn lifetimes_mut(&mut self) -> &mut closures::universes::Interface {
        match self {
            Self::Closure(value) => &mut value.lifetimes,
            Self::Inline(value) => &mut value.lifetimes,
        }
    }
    fn lifetimes(&self) -> &closures::universes::Interface {
        match self {
            Self::Closure(value) => &value.lifetimes,
            Self::Inline(value) => &value.lifetimes,
        }
    }
    const fn kind(&self) -> Kind {
        match self {
            Self::Closure(_) => Kind::Closure,
            Self::Inline(_) => Kind::Inline,
        }
    }
    pub fn closure(&self) -> Result<&closures::Metadata, &'static str> {
        match self {
            Self::Closure(value) => Ok(value),
            Self::Inline(_) => Err("inline body requested as a closure"),
        }
    }
    pub fn closure_mut(&mut self) -> Result<&mut closures::Metadata, &'static str> {
        match self {
            Self::Closure(value) => Ok(value),
            Self::Inline(_) => Err("inline body requested as a closure"),
        }
    }
    pub fn inline(&self) -> Result<&inline::Metadata, &'static str> {
        match self {
            Self::Inline(value) => Ok(value),
            Self::Closure(_) => Err("closure requested as an inline constant body"),
        }
    }
    pub fn inline_mut(&mut self) -> Result<&mut inline::Metadata, &'static str> {
        match self {
            Self::Inline(value) => Ok(value),
            Self::Closure(_) => Err("closure requested as an inline constant body"),
        }
    }
}

/// Parked states never own their scheduler, so source nesting cannot create a
/// recursive ownership tree. The active state owns this one shared frontier.
pub(super) struct Scheduler<P> {
    pub entries: Vec<Entry<P>>,
    pub stack: Vec<(Option<usize>, Box<State>)>,
    pub active: Option<usize>,
}

/// Only the isolated interface is exchanged with the parent inference context.
/// Body-specific control, lifetimes and publication remain in the payload owner.
pub(super) struct Entry<P> {
    pub source: NodeId,
    pub inputs: Refinement,
    pub state: Option<Box<State>>,
    pub payload: P,
}

impl<P> Default for Scheduler<P> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            stack: Vec::new(),
            active: None,
        }
    }
}

impl Checker<'_> {
    pub(super) fn park_nested(&mut self, child: State, entry: Entry<Payload>) -> Result<(), Error> {
        let mut scheduler = self.state.closures.take().unwrap_or_default();
        if scheduler.entries.len() >= 1_000_000 {
            return Err(self.error(entry.source, "nested body work limit exceeded"));
        }
        let index = scheduler.entries.len();
        scheduler.entries.push(entry);
        self.state.closure_children.push(index);
        self.state.retain_sparse();
        let parent = std::mem::replace(&mut self.state, child);
        scheduler.stack.push((scheduler.active, Box::new(parent)));
        scheduler.active = Some(index);
        self.state.closures = Some(scheduler);
        self.prepare_file(self.state.body.file);
        Ok(())
    }

    pub(super) fn finish_nested(
        &mut self,
        evidence: HashMap<usize, Source>,
    ) -> Result<Option<Dependency>, Error> {
        let Some(&index) = self.state.closure_children.get(self.state.closure_cursor) else {
            return self.finish_inference(evidence);
        };
        self.state.closure_cursor += 1;
        let kind = self
            .state
            .closures
            .as_ref()
            .and_then(|scheduler| scheduler.entries.get(index))
            .ok_or_else(|| self.error(self.state.body.node, "missing nested completion entry"))?
            .payload
            .kind();
        match kind {
            Kind::Closure => self.finish_closure_child(index, evidence),
            Kind::Inline => self.finish_inline_child(index, evidence),
        }
    }

    fn active_nested_kind(&self) -> Option<Kind> {
        let scheduler = self.state.closures.as_ref()?;
        scheduler
            .active
            .and_then(|index| scheduler.entries.get(index))
            .map(|entry| entry.payload.kind())
    }

    pub(super) fn publish_nested_progress(&mut self) -> Result<(), Error> {
        let kind = self.active_nested_kind();
        let Some(mut scheduler) = self.state.closures.take() else {
            return Ok(());
        };
        if let Some(index) = scheduler.active {
            let parent = scheduler
                .stack
                .last()
                .map(|(_, parent)| parent.body)
                .ok_or_else(|| {
                    self.error(self.state.body.node, "missing output interface parent")
                })?;
            self.prepare_nested_output(scheduler.entries[index].payload.lifetimes_mut(), parent)?;
        }
        let result = match kind {
            Some(Kind::Closure) => self.export_closure_inputs(&mut scheduler),
            Some(Kind::Inline) => self.export_inline_inputs(&mut scheduler),
            None => Ok(()),
        };
        self.state.closures = Some(scheduler);
        result
    }

    pub(super) fn publish_nested_body(&mut self, solved: &Solved) -> Result<(), Error> {
        match self.active_nested_kind() {
            Some(Kind::Closure) => self.publish_closure_body(solved),
            Some(Kind::Inline) => self.publish_inline_body(solved),
            None => Ok(()),
        }?;
        self.publish_active_output()
    }
}
