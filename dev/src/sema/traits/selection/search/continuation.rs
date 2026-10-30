//! Retain the exact candidate and proof frontier while a header is prepared.
use super::{Action, Caller, Decision, Declarations, Dependency, Frame, Node, Predicate, Search};
use crate::support::hash::{HashMap, HashSet};
use crate::{sema::traits::implementations::Availability, types::shape_key::ShapeKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Progress {
    Ready(Decision),
    Pending(Dependency),
    Inferred,
}

/// Input publication and observed hints belong to the same retained query.
/// Observing a hint alone cannot authorize exporting any caller input.
#[derive(Clone, Copy, Default)]
pub(super) enum Publication {
    #[default]
    Fresh,
    HintsObserved,
    Published,
    PublishedWithHints,
}
impl Publication {
    pub fn published(self) -> bool {
        matches!(self, Self::Published | Self::PublishedWithHints)
    }
    pub fn has_hints(self) -> bool {
        matches!(self, Self::HintsObserved | Self::PublishedWithHints)
    }
    pub fn observe(&mut self, hinted: bool) {
        if hinted {
            *self = if self.published() {
                Self::PublishedWithHints
            } else {
                Self::HintsObserved
            };
        }
    }
    pub fn publish(&mut self) {
        *self = if self.has_hints() {
            Self::PublishedWithHints
        } else {
            Self::Published
        };
    }
    pub fn resume(&mut self) {
        *self = if self.has_hints() {
            Self::HintsObserved
        } else {
            Self::Fresh
        };
    }
}

pub(super) struct Query {
    pub reuse: Option<super::completed_proofs::Active>,
    pub refining: bool,
    pub publication: Publication,
    pub suspended: Vec<Option<Frame>>,
    pub scopes: Option<Box<super::scopes::Scopes>>,
    pub universal: Option<Box<super::input_universe::InputUniverse>>,
    pub allocations: super::allocations::Allocations,
    pub availability: Availability,
    pub purpose: crate::sema::traits::implementations::Purpose,
    pub nodes: Vec<Node>,
    pub active: HashSet<ShapeKey>,
    pub auto_active: HashMap<Predicate, (crate::types::lifetime_proof::ScopeId, usize)>,
    pub next_auto_segment: usize,
    pub auto_provisional:
        HashMap<usize, HashMap<(crate::types::lifetime_proof::ScopeId, Predicate), Decision>>,
    pub families: HashMap<super::member_conditions::Key, super::member_conditions::Entry>,
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

#[cfg(test)]
impl Query {
    /// Exercise a later sibling using the same caller's completed prefix graph.
    pub(super) fn resume_sibling_for_test(&mut self) -> Result<(), &'static str> {
        if !self.stack.is_empty()
            || self.waiting.is_some()
            || self.universal.is_some()
            || !self.active.is_empty()
            || self.decision.is_none()
        {
            return Err("cannot reuse an unfinished test query");
        }
        self.decision = None;
        Ok(())
    }
}

impl<'a> Search<'a> {
    pub(super) fn new(caller: Caller<'a>, availability: Availability) -> Self {
        let builtins = match availability {
            Availability::Declarations => {
                crate::sema::nominals::Proof::for_unvalidated_implementations(
                    caller.environment.cloned(),
                )
            }
            Availability::Validated | Availability::Executable => {
                crate::sema::nominals::Proof::new(caller.environment.cloned())
            }
        };
        let first = caller.first_region;
        Self {
            caller,
            query: Query {
                reuse: None,
                refining: false,
                publication: Publication::Fresh,
                suspended: Vec::new(),
                scopes: None,
                universal: None,
                allocations: super::allocations::Allocations::new(first),
                availability,
                purpose: crate::sema::traits::implementations::Purpose::Use,
                builtins,
                nodes: Vec::new(),
                active: HashSet::default(),
                auto_active: HashMap::default(),
                next_auto_segment: 0,
                auto_provisional: HashMap::default(),
                families: HashMap::default(),
                completed: HashMap::default(),
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

    #[cfg(test)]
    pub(super) fn run(
        &mut self,
        facts: &mut Declarations,
        first: Frame,
    ) -> Result<Decision, String> {
        self.run_checked(facts, first)
            .map_err(super::super::Failure::message)
    }

    pub(super) fn run_checked(
        &mut self,
        facts: &mut Declarations,
        first: Frame,
    ) -> Result<Decision, super::super::Failure> {
        match self.begin(facts, first)? {
            Progress::Ready(decision) => Ok(decision),
            Progress::Pending(dependency) => Err(super::super::Failure::Pending(dependency)),
            Progress::Inferred => Err(super::super::Failure::Invalid(
                "non-refining selection returned partial inputs".into(),
            )),
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
            self.account_query_work(1)?;
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
            let action = self.advance(facts, frame);
            let charged = self.charge(&mut frame.inputs.context);
            let trial_charged = frame
                .trial
                .as_mut()
                .map_or(Ok(()), |trial| self.charge(&mut trial.inputs.context));
            // Preserve the semantic error while retaining every attempted charge.
            let action = action?;
            charged?;
            trial_charged?;
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
                        self.query.publication.publish();
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
                    } else if let Some(child) = self.language_child(facts, frame)? {
                        child
                    } else if let Some(child) = self.metadata_result_child(facts, frame)? {
                        child
                    } else if let Some(child) = self.inherited_result_child(facts, frame)? {
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
                    if completed.formation_cacheable() {
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
        if self.query.waiting.is_none() && !self.query.publication.published() {
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
        if self.query.publication.published() {
            let frame = self.query.stack.first_mut().ok_or("missing partial root")?;
            frame.refresh(facts, caller)?;
            self.query.publication.resume();
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
        while let Some(&owner) = pending.as_slice().first() {
            if registry.header(owner).is_none() {
                #[cfg(test)]
                super::copy_trace::record(|| {
                    format!(
                        "pending-header query={:p} scope={:?} owner={owner:?} canonical={:?}",
                        &self.query, frame.scope, frame.canonical
                    )
                });
                return Ok(Action::Wait(Dependency::Header(owner)));
            }
            if self.query.availability == Availability::Executable && !registry.is_validated(owner)
            {
                return Ok(Action::Wait(Dependency::ImplementationValidated(owner)));
            }
            let _ = pending.next();
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
