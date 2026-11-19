//! Crate-local reuse for closed, ordinary implementation conditions.
use super::frames::{Inference, Sources};
use super::{Decision, Declarations, Frame, Node, Predicate, Query, Search, Source};
use crate::{
    resolve::model::DefId,
    sema::traits::implementations::{Availability, Purpose},
    support::{constness::Constness, hash::HashMap},
    types::{
        GenericArgs, Kind, Store, TypeId, lifetime_proof::ScopeId, parameters::predicates::Bound,
        shape_key::ShapeKey, traits::Application,
    },
};
use std::num::NonZeroU32;

/// Admit only small proof DAGs. A parent recipe preserves both child order and
/// the identity of shared nodes inside its original selected proof.
const NODES: u16 = 8;
const EDGES: usize = 16;
const ENTRIES: usize = 1024;
// At most 17 pending rows, checks against at most eight seen rows, and
// bounded sorting and edge remapping over the same tiny proof.
const CAPTURE_WORK: u16 = 256;
const CHILD_PROOF_WORK: u16 = 2048;

pub(super) enum Probe {
    Ordinary,
    Watch(NonZeroU32),
    Hit(usize),
}

struct Step {
    goal: Predicate,
    shape: ShapeKey,
    owner: DefId,
    arguments: GenericArgs,
    children: Vec<usize>,
}
struct Recipe {
    steps: Box<[Step]>,
}

#[derive(Default)]
pub(in crate::sema::traits::selection) struct ChildProofs {
    entries: HashMap<Predicate, Recipe>,
}

impl Query {
    pub(super) fn enable_child_proof_work(&mut self) {
        self.child_proof_work = CHILD_PROOF_WORK;
    }

    // A miss or capture can be skipped without spending the semantic budget.
    fn pay_child_proof_work(&mut self, work: u16) -> bool {
        let Some(remaining) = self.child_proof_work.checked_sub(work) else {
            self.child_proof_work = 0;
            return false;
        };
        self.child_proof_work = remaining;
        true
    }
}

impl Search<'_> {
    pub(super) fn probe_child_proof(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Probe {
        if frame.scope != ScopeId::ROOT
            || frame.inference != Inference::Selecting
            || frame.sources != Sources::All
            || frame.forming()
            || frame.normalizing()
            || frame.language.is_some()
            || frame.storage.is_some()
            || frame.autos.is_some()
            || frame.equality_hint.is_some()
            || frame.normalized_inputs.is_some()
            || !self.neutral_child_query(facts)
        {
            return Probe::Ordinary;
        }
        let Some(trial) = frame.trial.as_mut() else {
            return Probe::Ordinary;
        };
        let Some(&index) = trial.pending.get(trial.cursor) else {
            return Probe::Ordinary;
        };
        if trial.wakeups.contains(index)
            || trial
                .instance
                .quantified
                .as_deref()
                .is_some_and(super::conditions::Conditions::waiting)
        {
            return Probe::Ordinary;
        }
        let input = &trial.instance.goals[index];
        let Some((item, [])) = input.trait_arguments() else {
            return Probe::Ordinary;
        };
        if input.constness() != Some(Constness::Ordinary) {
            return Probe::Ordinary;
        }
        if !self.query.pay_child_proof_work(1) {
            return Probe::Ordinary;
        }
        let Ok(Some(ty)) = trial.inputs.context.settled_identity(input.subject()) else {
            return Probe::Ordinary;
        };
        if !closed(&facts.types, ty) {
            return Probe::Ordinary;
        }
        facts.generics.selection_memo.child_work.goals += 1;
        let Some(application) = facts.types.find_trait_application(Application {
            item,
            arguments: GenericArgs::EMPTY,
        }) else {
            return Probe::Ordinary;
        };
        let goal = Predicate {
            ty,
            bound: Bound::Trait(application),
        };
        if let Some(recipe) = facts
            .generics
            .selection_memo
            .children()
            .and_then(|memo| memo.entries.get(&goal))
        {
            // The check and memo access use optional work. Replayed nodes
            // still pay normal proof work, bounded by the recipe's size cap.
            if !self.query.pay_child_proof_work(NODES)
                || !self.query_work_fits(2 * (usize::from(NODES) + EDGES))
            {
                return Probe::Ordinary;
            }
            // A warm child may sit under different ancestors. Falling back
            // preserves ordinary active-stack cutoffs in an intersecting search.
            if recipe.steps.iter().all(|step| {
                !self.query.active.contains(&step.shape)
                    && self.query.completed(ScopeId::ROOT, step.goal).is_none()
            }) {
                let first = trial.end.max(self.query.allocations.floor());
                let node = recipe.replay(&mut self.query, first);
                facts.generics.selection_memo.child_work.hits += 1;
                return Probe::Hit(node);
            }
            return Probe::Ordinary;
        }
        u32::try_from(facts.types.interned_extent())
            .ok()
            .and_then(NonZeroU32::new)
            .map_or(Probe::Ordinary, Probe::Watch)
    }

    fn neutral_child_query(&self, facts: &Declarations) -> bool {
        self.query.child_proof_work != 0
            && facts.source_bindings.is_none()
            && self.query.purpose == Purpose::Use
            && self.query.availability == Availability::Validated
            && self.query.allocations.untouched()
            && self.query.scopes.is_none()
            && self.query.universal.is_none()
            && self.caller.nesting == super::super::Nesting::ROOT
            && self.caller.placeholders.is_none()
            && !self.caller.retain_sized
            && self.caller.assumptions.is_empty()
            && self
                .caller
                .environment
                .is_none_or(|environment| environment.is_neutral())
            && facts.types.nominal_sizes_sealed()
            && facts
                .generics
                .implementations
                .as_ref()
                .is_some_and(|registry| {
                    registry.headers_validated() && registry.selection_headers_ready()
                })
    }

    pub(super) fn remember_child_proof(
        &mut self,
        facts: &mut Declarations,
        frame: &Frame,
        decision: Decision,
    ) {
        let (Some(extent), Some(shape), Decision::Proven(index)) =
            (frame.child_proof_extent, frame.key, decision)
        else {
            return;
        };
        let node = &self.query.nodes[index];
        if extent.get() as usize != facts.types.interned_extent()
            || !self.neutral_child_query(facts)
            || frame.cycles != self.query.cycles
            || frame.candidates.as_slice().len() != 1
            || frame
                .selected
                .is_none_or(|selected| selected.node() != index)
            || frame.canonical != Some(node.goal)
            || frame.explicit_auto
            || frame.normalized_inputs.is_some()
            || frame.formation.is_some()
            || !node.debt_free
        {
            return;
        }
        let goal = node.goal;
        if !self.query.pay_child_proof_work(CAPTURE_WORK) {
            return;
        }
        let memo = facts.generics.selection_memo.child_proofs();
        if memo.entries.len() == ENTRIES || memo.entries.contains_key(&goal) {
            return;
        }
        if let Some(recipe) = memo.capture(&self.query, index, shape) {
            memo.entries.insert(goal, recipe);
        }
    }
}

impl ChildProofs {
    fn capture(&self, query: &Query, root: usize, shape: ShapeKey) -> Option<Recipe> {
        let mut pending = vec![root];
        let mut seen = Vec::<(usize, Step)>::new();
        let mut edges = 0;
        while let Some(index) = pending.pop() {
            if seen.iter().any(|&(known, _)| known == index) {
                continue;
            }
            let node = &query.nodes[index];
            edges += node.children.len();
            if seen.len() == usize::from(NODES) || edges > EDGES {
                return None;
            }
            let step = self.step(query, index, root, shape)?;
            seen.push((index, step));
            pending.extend(&node.children);
        }
        seen.sort_unstable_by_key(|&(index, _)| index);
        let ids: Vec<_> = seen.iter().map(|&(index, _)| index).collect();
        let steps = seen
            .into_iter()
            .map(|(_, mut step)| {
                for child in &mut step.children {
                    // Every child comes from the traversal above.
                    *child = ids.partition_point(|index| *index < *child);
                }
                step
            })
            .collect();
        Some(Recipe { steps })
    }

    fn step(&self, query: &Query, index: usize, root: usize, root_shape: ShapeKey) -> Option<Step> {
        let node = &query.nodes[index];
        let Some(Source::Implementation { owner, arguments }) = node.source else {
            return None;
        };
        let shape = if index == root {
            root_shape
        } else {
            // A completed descendant must independently satisfy the same
            // single-owner, no-scope and no-side-effect admission check.
            let earlier = self.entries.get(&node.goal)?.steps.last()?;
            if earlier.owner != owner || earlier.arguments != arguments {
                return None;
            }
            earlier.shape
        };
        Some(Step {
            goal: node.goal,
            shape,
            owner,
            arguments,
            children: node.children.clone(),
        })
    }
}

impl Recipe {
    fn replay(&self, query: &mut Query, first: u32) -> usize {
        let start = query.nodes.len();
        let mut edges = 0;
        for step in &self.steps {
            edges += step.children.len();
            let index = query.push(
                Node {
                    debt_free: false,
                    cycle_dependent: false,
                    provisional_size: false,
                    goal: step.goal,
                    source: Some(Source::Implementation {
                        owner: step.owner,
                        arguments: step.arguments,
                    }),
                    children: step.children.iter().map(|index| start + index).collect(),
                    first,
                    end: first,
                    equalities: Vec::new(),
                    outlives: Vec::new(),
                    type_outlives: Vec::new(),
                    sized: Vec::new(),
                },
                ScopeId::ROOT,
                first..first,
            );
            query.remember(ScopeId::ROOT, step.goal, Decision::Proven(index));
        }
        query.reused |= edges >= self.steps.len();
        query.nodes.len() - 1
    }
}

fn closed(types: &Store, ty: TypeId) -> bool {
    matches!(types.kind(ty), Some(Kind::Adt(..) | Kind::Primitive(_)))
        && types.contains_parameters(ty) == Some(false)
        && types.contains_regions(ty) == Some(false)
        && types.contains_opaques(ty) == Some(false)
        && types.contains_projections(ty) == Some(false)
        && types.contains_callables(ty) == Some(false)
        && types.contains_function_items(ty) == Some(false)
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/child_proof_reuse.rs"]
mod tests;
