//! Retain the exact candidate and proof frontier while a header is prepared.
use super::{Action, Caller, Decision, Declarations, Dependency, Frame, Node, Predicate, Search};
use crate::{sema::traits::implementations::Availability, types::shape_key::ShapeKey};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Progress {
    Ready(Decision),
    Pending(Dependency),
    Inferred,
}

pub(super) struct Query {
    pub reuse: Option<super::completed_proofs::Active>,
    pub refining: bool,
    pub inferred: bool,
    pub suspended: Vec<Option<Frame>>,
    pub scopes: Option<Box<super::scopes::Scopes>>,
    pub universal: Option<Box<super::input_universe::InputUniverse>>,
    pub allocations: super::allocations::Allocations,
    pub availability: Availability,
    pub nodes: Vec<Node>,
    pub active: HashSet<ShapeKey>,
    pub auto_active: HashMap<Predicate, (crate::types::lifetime_proof::ScopeId, usize)>,
    pub next_auto_segment: usize,
    pub auto_provisional:
        HashMap<usize, HashMap<(crate::types::lifetime_proof::ScopeId, Predicate), Decision>>,
    pub families: HashSet<super::member_conditions::Key>,
    // Completed answers belong to one immutable caller, mode and region scope.
    pub completed: HashMap<Predicate, Decision>,
    pub cycles: usize,
    pub reused: bool,
    pub steps: usize,
    #[cfg(test)]
    pub storage_steps: usize,
    pub builtins: crate::sema::nominals::Proof,
    pub stack: Vec<Frame>,
    waiting: Option<Dependency>,
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
                reuse: None,
                refining: false,
                inferred: false,
                suspended: Vec::new(),
                scopes: None,
                universal: None,
                allocations: super::allocations::Allocations::new(first),
                availability,
                builtins,
                nodes: Vec::new(),
                active: HashSet::new(),
                auto_active: HashMap::new(),
                next_auto_segment: 0,
                auto_provisional: HashMap::new(),
                families: HashSet::new(),
                completed: HashMap::new(),
                cycles: 0,
                reused: false,
                steps: 0,
                #[cfg(test)]
                storage_steps: 0,
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
            Progress::Inferred => Err("non-refining selection returned partial inputs".into()),
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
            if !owner.ready(facts) {
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
            self.charge(&mut frame.inputs.context)?;
            if let Some(trial) = &mut frame.trial {
                self.charge(&mut trial.inputs.context)?;
            }
            match action {
                Action::Continue => {}
                Action::Wait(owner) => {
                    self.query.stack = stack;
                    self.query.waiting = Some(owner);
                    return Ok(Progress::Pending(owner));
                }
                Action::Yield => {
                    self.query.allocations.reserve_scoped();
                    self.publish_trial(facts, frame)?;
                    let mut child = stack.pop().ok_or("missing partial selection frame")?;
                    if let Some(parent) = stack.last_mut() {
                        self.accept_partial(facts, parent, child)?;
                    } else {
                        child.inference = super::frames::Inference::Committed;
                        stack.push(child);
                        self.query.stack = stack;
                        self.query.inferred = true;
                        return Ok(Progress::Inferred);
                    }
                }
                Action::Descend => {
                    let mut child = if let Some(child) = self.formation_child(facts, frame)? {
                        child
                    } else if let Some(child) = self.storage_child(facts, frame)? {
                        child
                    } else if let Some(child) = self.auto_child(facts, frame)? {
                        child
                    } else if let Some(child) = self.callable_child(facts, frame)? {
                        child
                    } else if let Some(child) = self.condition_child(facts, frame)? {
                        child
                    } else {
                        frame.child(
                            facts,
                            self.query.allocations.floor(),
                            &mut self.query.suspended,
                        )?
                    };
                    self.query.prepare_auto_child(frame, &mut child);
                    stack.push(child);
                }
                Action::Complete(decision) => {
                    let Some(decision) = self.complete_formation(facts, frame, decision)? else {
                        continue;
                    };
                    if let Some(key) = frame.key {
                        self.query.active.remove(&key);
                        if let Some(goal) = frame.canonical {
                            self.query.auto_active.remove(&goal);
                        }
                    }
                    if !matches!(decision, Decision::Proven(_)) {
                        self.query.invalidate_auto(frame);
                    }
                    let completed = stack.pop().ok_or("missing completed selection frame")?;
                    let Some(parent) = stack.last_mut() else {
                        if self.query.universal.is_some() {
                            stack.push(completed);
                            self.query.stack = stack;
                        }
                        self.query.decision = Some(decision);
                        return Ok(Progress::Ready(decision));
                    };
                    if completed.formation.is_none() {
                        self.remember(&completed, decision);
                        self.query.export_completed(facts, decision)?;
                    }
                    self.accept(facts, parent, decision)?;
                }
            }
        }
        Err("missing trait selection result".into())
    }

    pub(super) fn resume_at(&mut self, first: u32) -> Result<(), String> {
        if self.query.waiting.is_none() && !self.query.inferred {
            return Err("trait selection is not waiting for a header".into());
        }
        let previous = self.query.allocations.end();
        self.query.allocations.resume_at(first)?;
        if let Some(universal) = &mut self.query.universal
            && first > previous
        {
            universal
                .sources
                .push(crate::types::substitute::ExportSource::Caller(
                    previous..first,
                ));
        }
        Ok(())
    }

    pub(super) fn publish_inputs(
        &mut self,
        facts: &mut Declarations,
        caller: &mut super::Context,
    ) -> Result<bool, String> {
        let frame = self.query.stack.first_mut().ok_or("missing partial root")?;
        let changed = if let Some(universal) = &mut self.query.universal {
            universal.publish(
                facts,
                caller,
                frame,
                &mut self.query.allocations,
                self.query
                    .scopes
                    .as_mut()
                    .ok_or("missing universal input scopes")?,
            )?
        } else {
            frame.publish(facts, caller)?
        };
        caller
            .account_work(std::mem::take(&mut self.query.steps))
            .map_err(super::inference)?;
        Ok(changed)
    }

    pub(super) fn refresh_inputs(
        &mut self,
        facts: &mut Declarations,
        caller: &mut super::Context,
    ) -> Result<(), String> {
        if self.query.inferred {
            let frame = self.query.stack.first_mut().ok_or("missing partial root")?;
            frame.refresh(facts, caller)?;
            self.query.inferred = false;
        }
        Ok(())
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
                return Ok(Action::Wait(Dependency::Header(owner)));
            }
        }
        frame.pending_headers = None;
        if matches!(
            frame.goal.builtin_bound(),
            Some(crate::types::parameters::predicates::Bound::Auto(_))
        ) && !frame.explicit_auto
        {
            self.complete_builtin(facts, frame)
        } else {
            self.candidates(facts, frame)
        }
    }
}
