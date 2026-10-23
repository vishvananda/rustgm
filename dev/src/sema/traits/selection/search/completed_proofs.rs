//! Reuse completed, debt-free concrete proofs within one immutable caller and mode.
use super::{Decision, Declarations, Frame, Node, Predicate, Query, Source};
use crate::support::hash::HashMap;
use crate::types::lifetime_proof::ScopeId;

/// Owned by one fixed environment, implementation view and builtin-retention mode.
/// No failed search, provisional answer or caller-allocated witness enters this map.
#[derive(Default)]
pub(in crate::sema) struct Reuse {
    completed: HashMap<Predicate, Source>,
}
impl Reuse {
    pub(in crate::sema) fn needed(
        facts: &Declarations,
        predicate: Predicate,
    ) -> Result<bool, &'static str> {
        Ok(
            facts.types.contains_projected_sizes(predicate.ty) == Some(true)
                || (facts.types.contains_nominals(predicate.ty) == Some(true)
                    && crate::sema::compiler_items::sizing_predicate(facts, predicate)?),
        )
    }

    #[cfg(test)]
    pub(in crate::sema) fn observations(&self) -> (usize, usize) {
        (
            self.completed.len(),
            size_of::<Self>() + crate::support::retained::map(&self.completed),
        )
    }

    pub fn selection(
        &self,
        facts: &Declarations,
        goal: Predicate,
        first: u32,
    ) -> Result<Option<super::Selection>, String> {
        let Some(&source) = self.completed.get(&goal) else {
            return Ok(None);
        };
        validate_source(facts, goal, source)?;
        Ok(Some(super::Selection::Proven(super::Evidence {
            provisional_size: false,
            source,
            proof: super::super::Obligations::new(first),
            sized: Vec::new(),
        })))
    }
}

pub(super) struct Active {
    pub reuse: Reuse,
    debt_free: Vec<bool>,
}
impl Active {
    pub fn new(reuse: Reuse) -> Self {
        Self {
            reuse,
            debt_free: Vec::new(),
        }
    }
    pub fn push(&mut self, node: &Node, nodes: &[Node], scope: ScopeId, source_sizing: bool) {
        let mut children_free = true;
        let mut inherited = false;
        for &child in &node.children {
            let free = self.debt_free[child];
            children_free &= free;
            inherited |= free
                && nodes[child].goal == node.goal
                && matches!(nodes[child].source, Some(Source::Builtin));
        }
        let independent = matches!(
            node.source,
            None | Some(Source::Builtin | Source::Implementation { .. })
        ) && (!matches!(node.source, Some(Source::Builtin))
            || node.goal.bound == crate::types::parameters::predicates::Bound::Sized
            || source_sizing
            || inherited);
        self.debt_free.push(
            independent
                && scope == ScopeId::ROOT
                && !node.provisional_size
                && !node.cycle_dependent
                && node.first == node.end
                && node.equalities.is_empty()
                && node.outlives.is_empty()
                && node.type_outlives.is_empty()
                && node.sized.is_empty()
                && children_free,
        );
    }
}
impl Query {
    pub(super) fn export_completed(
        &mut self,
        facts: &Declarations,
        decision: Decision,
    ) -> Result<(), String> {
        if self.universal.is_some() {
            return Ok(());
        }
        let (Some(active), Decision::Proven(index)) = (&mut self.reuse, decision) else {
            return Ok(());
        };
        let node = &self.nodes[index];
        super::super::refutation::require_positive_source(facts, node.source)?;
        if active.debt_free[index]
            && node.goal.is_concrete(&facts.types)?
            && let Some(source) = node.source
        {
            validate_source(facts, node.goal, source)?;
            active.reuse.completed.insert(node.goal, source);
        }
        Ok(())
    }
    pub(super) fn reuse_completed(
        &mut self,
        facts: &Declarations,
        frame: &Frame,
        goal: Predicate,
    ) -> Result<Option<Decision>, String> {
        if frame.scope != ScopeId::ROOT || self.universal.is_some() {
            return Ok(None);
        }
        let Some(source) = self
            .reuse
            .as_ref()
            .and_then(|active| active.reuse.completed.get(&goal))
            .copied()
        else {
            return Ok(None);
        };
        validate_source(facts, goal, source)?;
        let first = frame.available_region().max(self.allocations.floor());
        let node = Node {
            debt_free: false,
            cycle_dependent: false,
            provisional_size: false,
            goal,
            source: Some(source),
            children: Vec::new(),
            first,
            end: first,
            equalities: Vec::new(),
            outlives: Vec::new(),
            type_outlives: Vec::new(),
            sized: Vec::new(),
        };
        let selected = if matches!(source, Source::Builtin)
            && crate::sema::compiler_items::sizing_predicate(facts, goal)?
        {
            self.push_source_sizing(facts, node, frame.scope, first..first)?
        } else {
            self.push(node, frame.scope, first..first)
        };
        Ok(Some(Decision::Proven(selected)))
    }
}

fn validate_source(facts: &Declarations, goal: Predicate, source: Source) -> Result<(), String> {
    super::super::refutation::require_positive_source(facts, Some(source))?;
    if matches!(source, Source::Builtin)
        && goal.bound != crate::types::parameters::predicates::Bound::Sized
        && !crate::sema::compiler_items::sizing_predicate(facts, goal)?
    {
        return Err("completed builtin proof lost its validated source sizing declaration".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/sharing_sizing.rs"]
mod sizing_tests;

#[cfg(test)]
#[path = "../../../../../tests/invariants/effect_capabilities.rs"]
mod const_capability_tests;
