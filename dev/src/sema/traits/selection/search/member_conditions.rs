//! Associated member input conditions remain on the parent's proof stack.
use super::{Action, Context, Decision, Declarations, Frame, Node, Search, inference};
use crate::support::hash::HashMap;
use crate::{
    sema::projections::families,
    types::{
        infer::Instantiation, lifetime_proof::ScopeId, parameters::clauses::Clause,
        projection::Application,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Member(Application, ScopeId),
    Nominal(crate::types::TypeId, u16, ScopeId),
}
#[derive(Clone, Copy)]
pub(super) enum Entry {
    Active,
    Completed(usize),
}
pub(super) enum Purpose {
    Member(Application),
    Select(crate::types::TypeId),
    Nominal(crate::types::TypeId, u16),
}
impl Purpose {
    pub(super) fn nominal(&self) -> bool {
        !matches!(self, Self::Member(_))
    }
}

pub(super) struct Formation {
    pub(super) purpose: Purpose,
    pub(super) requirements: families::Requirements,
    pub(super) nominals: Vec<(crate::types::TypeId, u16)>,
    pub(super) parent: Option<super::frames::Chosen>,
    pub(super) cursor: usize,
    pub(super) children: Vec<usize>,
    pub(super) universes: HashMap<u32, ScopeId>,
    pub(super) stopped: Option<Decision>,
    pub(super) registered: bool,
    pub(super) finished: bool,
}

impl Frame {
    pub(super) fn prepare_formation(
        &mut self,
        facts: &mut Declarations,
        application: Application,
    ) -> Result<(), String> {
        let Some(requirements) = families::requirements(facts, application)? else {
            return Ok(());
        };
        let parent = crate::types::parameters::predicates::Predicate {
            ty: application.subject,
            bound: crate::types::parameters::predicates::Bound::Trait(
                application.trait_application,
            ),
        };
        if self
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &self.goal)
            .map_err(inference)?
            != parent
        {
            return Err("family formation disagrees with its parent query".into());
        }
        self.inputs
            .context
            .account_work(requirements.steps)
            .map_err(inference)?;
        self.formation = Some(Box::new(Formation {
            purpose: Purpose::Member(application),
            nominals: Vec::new(),
            requirements,
            parent: None,
            cursor: 0,
            children: Vec::new(),
            universes: HashMap::default(),
            stopped: None,
            registered: false,
            finished: false,
        }));
        Ok(())
    }
    pub(super) fn forming(&self) -> bool {
        self.formation.as_ref().is_some_and(|formation| {
            (formation.purpose.nominal() || formation.parent.is_some()) && !formation.finished
        })
    }
}

impl Search<'_> {
    /// Consume parent input evidence once, then withhold completion until every
    /// member requirement has a retained proof. Failure cannot poison trait caches.
    pub(super) fn complete_formation(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<Option<Decision>, String> {
        if frame
            .formation
            .as_ref()
            .is_some_and(|f| f.purpose.nominal())
        {
            return self
                .complete_nominal_formation(facts, frame, decision)
                .map(Some);
        }
        if frame
            .formation
            .as_ref()
            .is_none_or(|formation| formation.parent.is_none())
        {
            if frame.formation.is_some() {
                // The parent answer is independent of this member's requirements.
                // Cache it before attaching the normalizer's prior input evidence;
                // neither a later family failure nor a composite proof belongs here.
                self.remember(frame, decision);
            }
            let decision = self.input_evidence(facts, frame, decision)?;
            let parent = match decision {
                Decision::Proven(parent) => super::frames::Chosen::Positive(parent),
                Decision::Refuted(parent) => super::frames::Chosen::Negative(parent),
                _ => return Ok(Some(decision)),
            };
            let Some(formation) = &mut frame.formation else {
                return Ok(Some(decision));
            };
            if let Some(key) = frame.key.take() {
                self.query.active.remove(&key);
            }
            let Purpose::Member(application) = formation.purpose else {
                return Err("missing member application".into());
            };
            let key = Key::Member(application, frame.scope);
            if self.query.families.contains_key(&key) {
                self.query.cycles += 1;
                return Ok(Some(Decision::Absent));
            }
            self.query.families.insert(key, Entry::Active);
            formation.registered = true;
            formation.parent = Some(parent);
            formation.children.push(parent.node());
            // No refinement from a family's proof may escape before all of its
            // conditions finish. Already selected parent inputs remain known.
            return Ok(None);
        }
        if let Some(formation) = &mut frame.formation
            && formation.registered
        {
            let Purpose::Member(application) = formation.purpose else {
                return Err("missing member application".into());
            };
            self.query
                .families
                .remove(&Key::Member(application, frame.scope));
            formation.registered = false;
        }
        Ok(Some(decision))
    }

    pub(super) fn advance_formation(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        if frame
            .formation
            .as_ref()
            .is_some_and(|f| f.purpose.nominal())
        {
            return self.advance_nominal_formation(facts, frame);
        }
        let formation = frame.formation.as_mut().ok_or("missing family formation")?;
        if let Some(decision) = formation.stopped {
            formation.finished = true;
            return Ok(Action::Complete(decision));
        }
        if formation.cursor < formation.requirements.clauses.len() {
            return Ok(Action::Descend);
        }
        let parent = formation
            .parent
            .ok_or("family formation has no parent proof")?;
        let proof = &self.query.nodes[parent.node()];
        let node = Node {
            debt_free: false,
            cycle_dependent: false,
            provisional_size: false,
            goal: proof.goal,
            source: proof.source,
            children: std::mem::take(&mut formation.children),
            first: proof.first,
            end: proof.end.max(self.query.allocations.end()),
            equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
            outlives: std::mem::take(&mut formation.requirements.regions),
            type_outlives: Vec::new(),
            sized: Vec::new(),
        };
        let selected = self.query.push(node, frame.scope, frame.first..frame.first);
        formation.finished = true;
        Ok(Action::Complete(match parent {
            super::frames::Chosen::Positive(_) => Decision::Proven(selected),
            super::frames::Chosen::Negative(_) => Decision::Refuted(selected),
        }))
    }

    pub(super) fn formation_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        if !frame.forming() {
            return Ok(None);
        }
        if frame
            .formation
            .as_ref()
            .is_some_and(|formation| formation.cursor >= formation.requirements.clauses.len())
        {
            return self.nominal_formation_child(facts, frame).map(Some);
        }
        let formation = frame.formation.as_mut().ok_or("missing family formation")?;
        let clause = *formation
            .requirements
            .clauses
            .get(formation.cursor)
            .ok_or("missing family condition")?;
        let mut first = self.query.allocations.reserve_scoped();
        let (predicate, scope) = match clause {
            Clause::Predicate(predicate) => (predicate, frame.scope),
            Clause::ForAll(id) => {
                if self.caller.retain_sized {
                    return Err(
                        "quantified family conditions require scoped sized requirements".into(),
                    );
                }
                let value = facts
                    .types
                    .quantified(id)
                    .ok_or("missing family condition binder")?;
                let scopes = self.query.scopes.get_or_insert_with(Default::default);
                let scope = if let Some(&scope) = formation.universes.get(&value.bound_regions) {
                    scope
                } else {
                    frame
                        .inputs
                        .context
                        .account_work(value.bound_regions as usize)
                        .map_err(inference)?;
                    let scope = scopes.introduce(frame.scope, value.bound_regions, first)?;
                    formation.universes.insert(value.bound_regions, scope);
                    scope
                };
                let (predicate, next, work) = scopes.open(facts, scope, value.predicate, first)?;
                frame.inputs.context.account_work(work).map_err(inference)?;
                first = next;
                self.query.allocations.record(next);
                (predicate, scope)
            }
        };
        let mut context = Context::default();
        let mut substitution =
            Instantiation::new(&mut context, &facts.types, &[]).map_err(inference)?;
        let goal = context
            .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
            .map_err(inference)?;
        let mut child = Frame::new(facts, &mut context, &goal, first)?;
        frame
            .inputs
            .context
            .account_work(context.steps)
            .map_err(inference)?;
        child.scope = scope;
        Ok(Some(child))
    }

    pub(super) fn accept_formation(frame: &mut Frame, decision: Decision) -> Result<(), String> {
        let formation = frame
            .formation
            .as_mut()
            .ok_or("missing family condition owner")?;
        match decision {
            Decision::Refuted(_) => return Err("negative child was not classified".into()),
            Decision::Proven(node) => {
                formation.children.push(node);
                formation.cursor += 1;
            }
            Decision::Absent | Decision::Deferred => formation.stopped = Some(decision),
        }
        Ok(())
    }
}
