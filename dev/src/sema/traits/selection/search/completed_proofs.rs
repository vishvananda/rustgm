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
    pub fn selection(&self, goal: Predicate, first: u32) -> Option<super::Selection> {
        Some(super::Selection::Proven(super::Evidence {
            provisional_size: false,
            source: *self.completed.get(&goal)?,
            proof: super::super::Obligations::new(first),
            sized: Vec::new(),
        }))
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
    pub fn push(&mut self, node: &Node, scope: ScopeId) {
        let independent = matches!(
            node.source,
            None | Some(Source::Builtin | Source::Implementation { .. })
        ) && (!matches!(node.source, Some(Source::Builtin))
            || node.goal.bound == crate::types::parameters::predicates::Bound::Sized);
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
                && node.children.iter().all(|&child| self.debt_free[child]),
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
            active.reuse.completed.insert(node.goal, source);
        }
        Ok(())
    }
    pub(super) fn reuse_completed(&mut self, frame: &Frame, goal: Predicate) -> Option<Decision> {
        if frame.scope != ScopeId::ROOT || self.universal.is_some() {
            return None;
        }
        let source = *self.reuse.as_ref()?.reuse.completed.get(&goal)?;
        let first = frame.available_region().max(self.allocations.floor());
        Some(Decision::Proven(self.push(
            Node {
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
            },
            frame.scope,
            first..first,
        )))
    }
}
