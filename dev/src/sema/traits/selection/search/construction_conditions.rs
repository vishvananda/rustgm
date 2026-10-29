//! Immediate nominal contracts share formation-only proofs in one caller query.
use super::{Declarations, Frame, Search, inference};
use super::{
    frames::Chosen,
    member_conditions::{Entry, Formation, Key, Purpose},
};
use crate::{
    resolve::model::DefId,
    sema::{nominals, projections::families},
    support::{bindings::LangItem, hash::HashMap},
    types::{TypeId, lifetime_proof::ScopeId},
};
impl Frame {
    pub(super) fn formation_cacheable(&self) -> bool {
        self.formation.as_ref().is_none_or(|formation| {
            matches!(formation.purpose, Purpose::Select(_))
                && formation.finished
                && formation.stopped.is_none()
        })
    }
    pub(super) fn require_nominal_formed(
        &self,
        facts: &Declarations,
        ty: TypeId,
    ) -> Result<(), String> {
        if facts.types.contains_nominals(ty) != Some(false)
            && !self.formation.as_ref().is_some_and(|f| {
                matches!(f.purpose, Purpose::Select(subject) if subject == ty)
                    && f.finished
                    && f.stopped.is_none()
                    && f.parent.is_some()
            })
        {
            return Err("source structural rule requires completed nominal formation".into());
        }
        Ok(())
    }
}
impl Formation {
    pub(super) fn nominal_key(&self, scope: ScopeId) -> Result<Key, &'static str> {
        match self.purpose {
            Purpose::Select(ty) => Ok(Key::Nominal(ty, 0, scope)),
            Purpose::Nominal(ty, depth) => Ok(Key::Nominal(ty, depth, scope)),
            Purpose::Member(_) => Err("member proof has no nominal formation key"),
        }
    }
}
impl Search<'_> {
    pub(super) fn prepare_nominal_formation(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<bool, String> {
        if frame.formation.is_some() || facts.source_bindings.is_none() {
            return Ok(false);
        }
        // Actual source marker roles share formation independently of their
        // effect eligibility. Item arguments retain their formation duties;
        // their declaration signature was proved before forming the item.
        let source_rule = Self::source_auto_goal(facts, &frame.goal)?
            || matches!(
                Self::source_sizing_kind(facts, &frame.goal)?,
                Some(LangItem::Sized | LangItem::MetaSized | LangItem::PointeeSized)
            )
            || source_destruct_goal(facts, frame)?
            || Self::source_builtin_marker_goal(facts, frame)?
            || Self::source_refinement_marker_goal(facts, frame)?;
        if !source_rule {
            return Ok(false);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(crate::types::infer::Error::Unresolved) => return Ok(false),
            Err(error) => return Err(inference(error)),
        };
        if facts.types.contains_nominals(goal.ty) == Some(false) {
            return Ok(false);
        }
        self.begin_nominal_formation(facts, frame, goal.ty, 0, true)
    }
    fn begin_nominal_formation(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        ty: TypeId,
        depth: u16,
        select: bool,
    ) -> Result<bool, String> {
        let key = Key::Nominal(ty, depth, frame.scope);
        let cached = match self.query.families.get(&key) {
            Some(Entry::Active) => {
                self.query.cycles += 1;
                return Err(
                    "cyclic nominal formation requires a completed prerequisite proof".into(),
                );
            }
            Some(Entry::Completed(node)) => Some(*node),
            None => None,
        };
        let mut checked = nominals::Requirements::default();
        let mut nominals = Vec::new();
        let result = if cached.is_none() {
            nominals::immediate(facts, ty, depth, &mut checked, &mut nominals)
        } else {
            Ok(Vec::new())
        };
        frame
            .inputs
            .context
            .account_work(
                1usize
                    .saturating_add(checked.steps())
                    .saturating_add(nominals.len()),
            )
            .map_err(inference)?;
        let clauses = result?;
        if cached.is_none() {
            self.query.families.insert(key, Entry::Active);
        }
        frame.formation = Some(Box::new(Formation {
            purpose: if select {
                Purpose::Select(ty)
            } else {
                Purpose::Nominal(ty, depth)
            },
            requirements: families::Requirements {
                clauses,
                regions: Vec::new(),
                steps: 0,
            },
            nominals,
            parent: cached.map(Chosen::Positive),
            cursor: 0,
            children: Vec::new(),
            universes: HashMap::default(),
            stopped: None,
            registered: cached.is_none(),
            finished: cached.is_some() && select,
        }));
        Ok(cached.is_none())
    }
    #[cfg(test)]
    pub(super) fn nominal_formation_child_for_test(
        &mut self,
        facts: &mut Declarations,
        context: &mut super::Context,
        goal: &super::Goal,
        ty: TypeId,
        depth: u16,
    ) -> Result<Frame, String> {
        let mut child = Frame::new(facts, context, goal, 1)?;
        self.begin_nominal_formation(facts, &mut child, ty, depth, false)?;
        Ok(child)
    }
    pub(super) fn nominal_formation_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Frame, String> {
        let formation = frame
            .formation
            .as_ref()
            .ok_or("missing nominal condition owner")?;
        let &(ty, depth) = formation
            .nominals
            .get(formation.cursor - formation.requirements.clauses.len())
            .ok_or("missing immediate nominal dependency")?;
        // The inherited goal is diagnostic context only: this helper never starts
        // trait selection and its source=None result is consumed only by formation.
        let first = self.query.allocations.reserve_scoped();
        let mut child = Frame::new(facts, &mut frame.inputs.context, &frame.goal, first)?;
        child.scope = frame.scope;
        self.begin_nominal_formation(facts, &mut child, ty, depth, false)?;
        Ok(child)
    }
}
fn source_destruct_goal(facts: &Declarations, frame: &Frame) -> Result<bool, String> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some((item, arguments)) = frame.goal.trait_arguments() else {
        return Ok(false);
    };
    let Some(declaration) = catalog.language(LangItem::Destruct) else {
        return Ok(false);
    };
    if declaration.definition != DefId(item.0) {
        return Ok(false);
    }
    catalog.validated_language(LangItem::Destruct)?;
    Ok(arguments.is_empty())
}
