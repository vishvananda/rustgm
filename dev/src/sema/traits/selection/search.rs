//! Iterative candidate search; each child proof commits only its resolved goal.
mod frames;
#[cfg(test)]
#[path = "../../../../tests/invariants/proof_cache.rs"]
mod tests;
use super::{
    Caller, Context, Declarations, Evidence, Goal, Predicate, Selection, Source, inference,
};
use crate::types::{infer, outlives::Pair, substitute};
use frames::{Frame, Trial};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Absent,
    Deferred,
    Proven(usize),
}
enum Action {
    Continue,
    Descend,
    Complete(Decision),
}
struct Node {
    goal: Predicate,
    source: Option<Source>,
    children: Vec<usize>,
    first: u32,
    end: u32,
    equalities: Vec<Pair>,
    outlives: Vec<Pair>,
    type_outlives: Vec<(super::TypeId, super::RegionId)>,
    sized: Vec<super::TypeId>,
}
struct Search<'a> {
    caller: Caller<'a>,
    availability: super::Availability,
    nodes: Vec<Node>,
    active: HashSet<Predicate>,
    // This table belongs to one immutable caller, query mode and region scope.
    // A canonical key retains every type/const argument and binder identity.
    completed: HashMap<Predicate, Decision>,
    cycles: usize,
    reused: bool,
    shapes: substitute::Bound,
    steps: usize,
    builtins: crate::sema::nominals::Proof,
}

pub(super) fn prove(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
    availability: super::Availability,
) -> Result<Selection, String> {
    let first = Frame::new(facts, context, goal, caller.first_region)?;
    let mut search = Search::new(caller, availability);
    let decision = search.run(facts, first)?;
    context.account_work(search.steps).map_err(inference)?;
    match decision {
        Decision::Absent => Ok(Selection::Absent),
        Decision::Deferred => Ok(Selection::Deferred),
        Decision::Proven(root) => {
            context
                .equate_trait_predicate(&facts.types, goal, search.nodes[root].goal)
                .map_err(inference)?;
            search.evidence(root).map(Selection::Proven)
        }
    }
}

impl<'a> Search<'a> {
    fn new(caller: Caller<'a>, availability: super::Availability) -> Self {
        Self {
            builtins: match availability {
                super::Availability::Declarations => {
                    crate::sema::nominals::Proof::for_unvalidated_implementations(
                        caller.environment.cloned(),
                    )
                }
                super::Availability::Validated => {
                    crate::sema::nominals::Proof::new(caller.environment.cloned())
                }
            },
            caller,
            availability,
            nodes: Vec::new(),
            active: HashSet::new(),
            completed: HashMap::new(),
            cycles: 0,
            reused: false,
            shapes: substitute::Bound::shape(),
            steps: 0,
        }
    }
    fn charge(&mut self, context: &mut Context) -> Result<(), String> {
        self.steps = self
            .steps
            .saturating_add(1 + std::mem::take(&mut context.steps));
        if self.steps > 1_000_000 {
            return Err("trait selection work budget exceeded".into());
        }
        Ok(())
    }
    fn run(&mut self, facts: &mut Declarations, first: Frame) -> Result<Decision, String> {
        let mut stack = vec![first];
        while let Some(frame) = stack.last_mut() {
            let action = self.advance(facts, frame)?;
            self.charge(&mut frame.context)?;
            if let Some(trial) = &mut frame.trial {
                self.charge(&mut trial.context)?;
            }
            match action {
                Action::Continue => {}
                Action::Descend => {
                    let trial = frame.trial.as_mut().ok_or("missing descending candidate")?;
                    let index = trial.pending[trial.cursor];
                    let child = Frame::new(
                        facts,
                        &mut trial.context,
                        &trial.instance.goals[index],
                        trial.end,
                    )?;
                    stack.push(child);
                }
                Action::Complete(decision) => {
                    if let Some(key) = frame.key {
                        self.active.remove(&key);
                    }
                    let (canonical, cycles) = (frame.canonical, frame.cycles);
                    stack.pop();
                    let Some(parent) = stack.last_mut() else {
                        return Ok(decision);
                    };
                    self.remember(canonical, cycles, decision);
                    self.accept(facts, parent, decision)?;
                }
            }
        }
        Err("missing trait selection result".into())
    }
    fn remember(&mut self, canonical: Option<Predicate>, cycles: usize, decision: Decision) {
        match decision {
            Decision::Proven(index) => {
                let node = &self.nodes[index];
                // Reusing existential evidence requires fresh region rebasing.
                if node.first == node.end {
                    self.completed.insert(node.goal, decision);
                }
            }
            Decision::Absent if cycles == self.cycles => {
                // A cycle cutoff is relative to the active proof stack, not a
                // negative fact. Neither it nor any ancestor may cache failure.
                if let Some(goal) = canonical {
                    self.completed.insert(goal, decision);
                }
            }
            Decision::Absent | Decision::Deferred => {}
        }
    }
    fn advance(&mut self, facts: &mut Declarations, frame: &mut Frame) -> Result<Action, String> {
        if !frame.started {
            return self.start(facts, frame);
        }
        if let Some(trial) = &mut frame.trial {
            if trial.cursor < trial.pending.len() {
                return Ok(Action::Descend);
            }
            if !trial.next.is_empty() && trial.progress {
                trial.pending = std::mem::take(&mut trial.next);
                trial.cursor = 0;
                trial.progress = false;
                return Ok(Action::Continue);
            }
            let result = if trial.next.is_empty() {
                self.finish(facts, trial)?
            } else {
                Decision::Deferred
            };
            self.charge(&mut trial.context)?;
            frame.trial = None;
            match result {
                Decision::Absent => {}
                Decision::Deferred => frame.deferred = true,
                Decision::Proven(node) => {
                    if frame.selected.replace(node).is_some() {
                        return Ok(Action::Complete(Decision::Deferred));
                    }
                }
            }
            return Ok(Action::Continue);
        }
        if let Some(&owner) = frame.candidates.get(frame.cursor) {
            frame.cursor += 1;
            let mut trial = Trial::new(facts, frame, owner)?;
            let result =
                trial
                    .context
                    .equate_predicates(&facts.types, &trial.goal, &trial.instance.goal);
            self.charge(&mut trial.context)?;
            match result {
                Ok(()) => frame.trial = Some(trial),
                Err(infer::Error::Mismatch | infer::Error::Occurs) => {}
                Err(error) => return Err(inference(error)),
            }
            return Ok(Action::Continue);
        }
        Ok(Action::Complete(if frame.deferred {
            Decision::Deferred
        } else {
            frame.selected.map_or(Decision::Absent, Decision::Proven)
        }))
    }
    fn start(&mut self, facts: &mut Declarations, frame: &mut Frame) -> Result<Action, String> {
        use super::Assumed;
        frame.started = true;
        frame.cycles = self.cycles;
        match self
            .caller
            .assumptions
            .select(&facts.types, &mut frame.context, &frame.goal)
            .map_err(inference)?
        {
            Assumed::Proven(proof) => {
                let goal = frame
                    .context
                    .resolved_predicate(&mut facts.types, &frame.goal)
                    .map_err(inference)?;
                let index = self.nodes.len();
                self.nodes.push(Node {
                    goal,
                    source: Some(Source::Assumption(proof.assumption())),
                    children: Vec::new(),
                    first: frame.first,
                    end: frame.first,
                    equalities: std::mem::take(&mut frame.context.region_equalities),
                    outlives: Vec::new(),
                    type_outlives: Vec::new(),
                    sized: Vec::new(),
                });
                return Ok(Action::Complete(Decision::Proven(index)));
            }
            Assumed::Deferred => return Ok(Action::Complete(Decision::Deferred)),
            Assumed::Absent => {}
        }
        match frame
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => {
                frame.canonical = Some(goal);
                if let Some(&decision) = self.completed.get(&goal) {
                    self.reused |= matches!(decision, Decision::Proven(_));
                    return Ok(Action::Complete(decision));
                }
                let before = self.shapes.steps;
                let key = self.shapes.predicate(&mut facts.types, goal)?;
                frame
                    .context
                    .account_work(self.shapes.steps.saturating_sub(before))
                    .map_err(inference)?;
                if !self.active.insert(key) {
                    self.cycles += 1;
                    return Ok(Action::Complete(Decision::Absent));
                }
                frame.key = Some(key);
            }
            Err(infer::Error::Unresolved) => {}
            Err(error) => return Err(inference(error)),
        }
        frame.candidates = facts
            .generics
            .implementations
            .as_mut()
            .ok_or("missing impl registry")?
            .candidates(
                &mut facts.types,
                &mut frame.context,
                &frame.goal,
                self.availability,
            )
            .map_err(inference)?;
        Ok(Action::Continue)
    }
    fn accept(
        &mut self,
        facts: &Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let trial = frame
            .trial
            .as_mut()
            .ok_or("missing parent trait candidate")?;
        let index = *trial
            .pending
            .get(trial.cursor)
            .ok_or("missing pending trait condition")?;
        trial.cursor += 1;
        match decision {
            Decision::Absent => {
                self.charge(&mut trial.context)?;
                frame.trial = None;
            }
            Decision::Deferred => trial.next.push(index),
            Decision::Proven(node) => {
                trial
                    .context
                    .equate_trait_predicate(
                        &facts.types,
                        &trial.instance.goals[index],
                        self.nodes[node].goal,
                    )
                    .map_err(inference)?;
                if self.nodes[node].first != self.nodes[node].end {
                    trial.end = self.nodes[node].end;
                }
                trial.children.push(node);
                trial.progress = true;
                self.charge(&mut trial.context)?;
            }
        }
        Ok(())
    }
    fn evidence(mut self, root: usize) -> Result<Evidence, String> {
        let source = self.nodes[root]
            .source
            .take()
            .ok_or("missing selected trait evidence")?;
        let mut evidence = Evidence {
            source,
            regions: self.caller.first_region..self.nodes[root].end,
            equalities: Vec::new(),
            outlives: Vec::new(),
            type_outlives: Vec::new(),
            sized: Vec::new(),
        };
        let mut work = vec![root];
        let mut visited = self.reused.then(|| vec![false; self.nodes.len()]);
        while let Some(index) = work.pop() {
            if let Some(visited) = &mut visited {
                if visited[index] {
                    continue;
                }
                visited[index] = true;
            }
            let node = &mut self.nodes[index];
            work.extend(std::mem::take(&mut node.children));
            evidence.equalities.append(&mut node.equalities);
            evidence.outlives.append(&mut node.outlives);
            evidence.type_outlives.append(&mut node.type_outlives);
            evidence.sized.append(&mut node.sized);
        }
        Ok(evidence)
    }
}
