//! Only completed formation prefixes can support an actual selected trait answer.
use super::{Action, Decision, Declarations, Frame, Node, Search, inference};
use super::{
    frames::Chosen,
    member_conditions::{Entry, Purpose},
};
impl Search<'_> {
    pub(super) fn advance_nominal_formation(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let formation = frame
            .formation
            .as_mut()
            .ok_or("missing nominal formation")?;
        let key = formation.nominal_key(frame.scope)?;
        if let Some(decision) = formation.stopped {
            if formation.registered {
                self.query.families.remove(&key);
                formation.registered = false;
            }
            formation.finished = true;
            return match decision {
                Decision::Deferred => Ok(Action::Complete(decision)),
                _ => Err("source nominal formation prerequisite is unsatisfied".into()),
            };
        }
        if formation.cursor < formation.requirements.clauses.len() + formation.nominals.len() {
            return Ok(Action::Descend);
        }
        let selected = if let Some(parent) = formation.parent {
            if formation.registered {
                return Err("active formation reused an incomplete prefix".into());
            }
            parent.node()
        } else {
            if !formation.registered
                || !matches!(self.query.families.get(&key), Some(Entry::Active))
            {
                return Err("nominal formation lost its active owner".into());
            }
            let goal = frame
                .inputs
                .context
                .resolved_predicate(&mut facts.types, &frame.goal)
                .map_err(inference)?;
            let node = Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: None,
                children: std::mem::take(&mut formation.children),
                first: frame.first,
                end: self.query.allocations.end().max(frame.first),
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: std::mem::take(&mut formation.requirements.regions),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            };
            let selected = self.query.push(node, frame.scope, frame.first..frame.first);
            self.query.families.insert(key, Entry::Completed(selected));
            formation.registered = false;
            formation.parent = Some(Chosen::Positive(selected));
            selected
        };
        formation.finished = true;
        if matches!(formation.purpose, Purpose::Nominal(..)) {
            return Ok(Action::Complete(Decision::Proven(selected)));
        }
        // The actual trait still needs its own ordinary selection; formation-only
        // completion supplies no implementation, assumption or callable permission.
        self.candidates(facts, frame)
    }
    pub(super) fn complete_nominal_formation(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<Decision, String> {
        let formation = frame
            .formation
            .as_ref()
            .ok_or("missing nominal formation")?;
        if !formation.finished {
            return Err("nominal formation published before completion".into());
        }
        if let Some(stopped) = formation.stopped {
            return if stopped == Decision::Deferred && decision == stopped {
                Ok(stopped)
            } else {
                Err("nominal formation published an unsatisfied prerequisite".into())
            };
        }
        let prefix = formation
            .parent
            .ok_or("nominal formation has no prerequisite proof")?
            .node();
        if self.query.nodes[prefix].source.is_some()
            || !matches!(self.query.families.get(&formation.nominal_key(frame.scope)?), Some(Entry::Completed(node)) if *node == prefix)
        {
            return Err("nominal formation prefix lost its completed identity".into());
        }
        if matches!(formation.purpose, Purpose::Nominal(..)) {
            return if decision == Decision::Proven(prefix) {
                Ok(decision)
            } else {
                Err("formation-only helper cannot publish a trait selection".into())
            };
        }
        let goal = frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
            .map_err(inference)?;
        if !matches!(formation.purpose, Purpose::Select(ty) if ty == goal.ty) {
            return Err("nominal formation subject changed after preparation".into());
        }
        if decision == Decision::Absent && !self.query.nodes[prefix].debt_free {
            return Err(
                "conditional nominal formation cannot publish an absence without paying its debts"
                    .into(),
            );
        }
        let decision = match decision {
            Decision::Proven(node) => Decision::Proven(self.inherit_inputs(node, Some(prefix))),
            Decision::Refuted(node) => Decision::Refuted(self.inherit_inputs(node, Some(prefix))),
            decision => decision,
        };
        self.input_evidence(facts, frame, decision)
    }
}
