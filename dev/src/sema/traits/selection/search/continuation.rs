//! Retain the exact candidate and proof frontier while a header is prepared.
use super::{Action, Caller, Decision, Declarations, Frame, Node, Predicate, Search};
use crate::{
    resolve::model::DefId, sema::traits::implementations::Availability, types::shape_key::ShapeKey,
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Progress {
    Ready(Decision),
    Pending(DefId),
}

pub(super) struct Query {
    pub scopes: Option<Box<super::scopes::Scopes>>,
    pub allocations: super::allocations::Allocations,
    pub availability: Availability,
    pub nodes: Vec<Node>,
    pub active: HashSet<ShapeKey>,
    // Completed answers belong to one immutable caller, mode and region scope.
    pub completed: HashMap<Predicate, Decision>,
    pub cycles: usize,
    pub reused: bool,
    pub steps: usize,
    pub builtins: crate::sema::nominals::Proof,
    stack: Vec<Frame>,
    waiting: Option<DefId>,
    decision: Option<Decision>,
}

impl<'a> Search<'a> {
    pub(super) fn new(caller: Caller<'a>, availability: Availability) -> Self {
        let builtins = match availability {
            Availability::Declarations => {
                crate::sema::nominals::Proof::for_unvalidated_implementations(
                    caller.environment.cloned(),
                )
            }
            Availability::Validated => {
                crate::sema::nominals::Proof::new(caller.environment.cloned())
            }
        };
        let first = caller.first_region;
        Self {
            caller,
            query: Query {
                scopes: None,
                allocations: super::allocations::Allocations::new(first),
                availability,
                builtins,
                nodes: Vec::new(),
                active: HashSet::new(),
                completed: HashMap::new(),
                cycles: 0,
                reused: false,
                steps: 0,
                stack: Vec::new(),
                waiting: None,
                decision: None,
            },
        }
    }

    pub(super) fn run(
        &mut self,
        facts: &mut Declarations,
        first: Frame,
    ) -> Result<Decision, String> {
        match self.begin(facts, first)? {
            Progress::Ready(decision) => Ok(decision),
            Progress::Pending(_) => {
                Err("trait selection requires pending implementation headers".into())
            }
        }
    }

    pub(super) fn begin(
        &mut self,
        facts: &mut Declarations,
        first: Frame,
    ) -> Result<Progress, String> {
        if !self.query.stack.is_empty() || self.query.decision.is_some() {
            return Err("trait selection query was started twice".into());
        }
        self.query.stack.push(first);
        self.poll(facts)
    }

    pub(super) fn poll(&mut self, facts: &mut Declarations) -> Result<Progress, String> {
        if let Some(owner) = self.query.waiting {
            let registry = facts
                .generics
                .implementations
                .as_ref()
                .ok_or("pending trait selection lost its implementation inventory")?;
            if registry.header(owner).is_none() {
                return Ok(Progress::Pending(owner));
            }
            self.query.waiting = None;
        }
        if let Some(decision) = self.query.decision {
            return Ok(Progress::Ready(decision));
        }
        let mut stack = std::mem::take(&mut self.query.stack);
        while let Some(frame) = stack.last_mut() {
            let action = self.advance(facts, frame)?;
            self.charge(&mut frame.context)?;
            if let Some(trial) = &mut frame.trial {
                self.charge(&mut trial.context)?;
            }
            match action {
                Action::Continue => {}
                Action::Wait(owner) => {
                    self.query.stack = stack;
                    self.query.waiting = Some(owner);
                    return Ok(Progress::Pending(owner));
                }
                Action::Descend => {
                    let child = if let Some(child) = self.condition_child(facts, frame)? {
                        child
                    } else {
                        frame.child(facts, self.query.allocations.floor())?
                    };
                    stack.push(child);
                }
                Action::Complete(decision) => {
                    let decision = self.input_evidence(facts, frame, decision)?;
                    if let Some(key) = frame.key {
                        self.query.active.remove(&key);
                    }
                    let (canonical, cycles, scope) = (frame.canonical, frame.cycles, frame.scope);
                    stack.pop();
                    let Some(parent) = stack.last_mut() else {
                        self.query.decision = Some(decision);
                        return Ok(Progress::Ready(decision));
                    };
                    self.remember(canonical, cycles, decision, scope);
                    self.accept(facts, parent, decision)?;
                }
            }
        }
        Err("missing trait selection result".into())
    }

    pub(super) fn resume_at(&mut self, first: u32) -> Result<(), String> {
        if self.query.waiting.is_none() {
            return Err("trait selection is not waiting for a header".into());
        }
        self.query.allocations.resume_at(first).map_err(Into::into)
    }

    pub(super) fn await_header(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let registry = facts
            .generics
            .implementations
            .as_ref()
            .ok_or("pending trait selection lost its implementation inventory")?;
        let pending = frame
            .pending_headers
            .as_mut()
            .ok_or("missing pending trait candidates")?;
        for owner in pending.by_ref() {
            if registry.header(owner).is_none() {
                return Ok(Action::Wait(owner));
            }
        }
        frame.pending_headers = None;
        self.candidates(facts, frame)
    }
}
