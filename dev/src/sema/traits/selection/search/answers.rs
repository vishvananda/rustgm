//! One crate-wide table of trait answers for closed goals.
//!
//! A goal is closed when its inputs name no parameter, projection, opaque type
//! or caller region. Its answer then depends on the sealed implementation
//! registry, the query mode and the caller's premises, and a caller without a
//! closed premise answers it exactly as a neutral caller with the same
//! crate-wide projection promises does: a rigid parameter equals no closed
//! type. The first finished search of such a goal
//! records either its absence or its selected proof, and a later frame for the
//! same goal and mode in any query replays that answer instead of assembling
//! candidates. Only a debt-free proof whose every node has its own entry
//! enters: no region, lifetime or Sized debt, no provisional size, no cycle
//! cutoff, no automatic or caller-supplied node, and a search that interned no
//! type, learned no size and allocated no region, so a replay leaves the state
//! a fresh search would.
//! The self-check mode runs the fresh search on every hit and compares.
use super::{
    Decision, Declarations, Frame, Node, Predicate, Query, Search, Source, frames::Sources,
};
use crate::resolve::model::DefId;
use crate::sema::compiler_items::SelectionState;
use crate::sema::query_census::{self, Answer as Event};
use crate::sema::traits::implementations::{Availability, Purpose};
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, Store,
    lifetime_proof::ScopeId,
    parameters::{environment::Environment, premise_index::ClauseIndex},
    shape_key::ShapeKey,
};
use std::num::NonZeroU16;
use std::sync::Arc;

/// A proof with more nodes keeps the ordinary search.
const STEPS: usize = 64;

type Promises = HashMap<crate::types::ItemId, crate::types::projection::Builtins>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    goal: Predicate,
    mode: u8,
    /// The caller's crate-wide projection promises.
    promises: usize,
}

/// How a caller's premises bear on closed goals.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Class {
    /// A closed premise may answer a closed goal for this caller alone.
    Premised,
    /// The caller answers closed goals as a neutral caller with the same
    /// projection promises does.
    Neutral { promises: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Proof {
    Builtin,
    Implementation {
        owner: DefId,
        arguments: GenericArgs,
    },
}
impl Proof {
    fn of(source: Option<&Source>) -> Option<Self> {
        match *source? {
            Source::Builtin => Some(Self::Builtin),
            Source::Implementation { owner, arguments } => {
                Some(Self::Implementation { owner, arguments })
            }
            _ => None,
        }
    }
    const fn source(self) -> Source {
        match self {
            Self::Builtin => Source::Builtin,
            Self::Implementation { owner, arguments } => {
                Source::Implementation { owner, arguments }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Step {
    goal: Predicate,
    shape: ShapeKey,
    proof: Proof,
    children: Box<[usize]>,
}
#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Absent(ShapeKey),
    /// Children precede their parents; the last step is the goal's own node.
    Proven(Box<[Step]>),
}
impl Entry {
    /// A proof through an active goal must keep the search's cycle cutoff, and
    /// one through a node this query completed must share that node.
    fn intersects(&self, query: &Query) -> bool {
        match self {
            Self::Absent(shape) => query.active.contains(shape),
            Self::Proven(steps) => steps.iter().any(|step| {
                query.active.contains(&step.shape)
                    || query.completed(ScopeId::ROOT, step.goal).is_some()
            }),
        }
    }
}

struct Caller {
    environment: Option<Arc<Environment>>,
    index: Option<Arc<ClauseIndex>>,
    premised: bool,
}

#[derive(Default)]
pub(in crate::sema::traits::selection) struct Answers {
    entries: HashMap<Key, Entry>,
    /// Every projection promise set that a key names by address. Holding it
    /// keeps that address from naming another set while the entry lives.
    promises: Vec<Arc<Promises>>,
    /// The in-place state of the source catalog every entry was proved under,
    /// compared by content: a catalog never moves out of its `Declarations`,
    /// and its address would not tell an in-place write apart.
    catalog: Option<SelectionState>,
    /// The caller classified last; consecutive queries usually share it.
    caller: Option<Caller>,
    #[cfg(test)]
    pub(super) counts: Counts,
}
#[cfg(test)]
#[derive(Default)]
pub(super) struct Counts {
    pub hits: usize,
    pub checked: usize,
    pub recorded: usize,
}
impl Answers {
    /// Entries answer only for the catalog state they were proved under.
    fn current(&mut self, catalog: Option<SelectionState>) -> &mut Self {
        if self.catalog != catalog {
            self.entries.clear();
            self.promises.clear();
            self.catalog = catalog;
        }
        self
    }

    /// Enter `entry` unless its key has one, holding the promise set the key
    /// names; a key that names another set than `promises` is refused.
    fn insert(&mut self, key: Key, entry: Entry, promises: Option<&Arc<Promises>>) -> bool {
        if promises.map_or(0, |set| Arc::as_ptr(set).addr()) != key.promises {
            return false;
        }
        if let Some(set) = promises
            && !self.promises.iter().any(|held| Arc::ptr_eq(held, set))
        {
            self.promises.push(Arc::clone(set));
        }
        self.entries.entry(key).or_insert(entry);
        true
    }

    /// Whether the caller's clauses or its environment hold a closed premise.
    /// The classification is read once per caller identity.
    fn premised(
        &mut self,
        types: &Store,
        environment: Option<&Arc<Environment>>,
        index: Option<&Arc<ClauseIndex>>,
    ) -> Result<bool, &'static str> {
        if let Some(caller) = &self.caller
            && same(caller.environment.as_ref(), environment)
            && same(caller.index.as_ref(), index)
        {
            return Ok(caller.premised);
        }
        let premised = index.map_or(Ok(false), |index| index.has_closed_premises(types))?
            || environment.map_or(Ok(false), |environment| {
                environment.has_closed_premises(types)
            })?;
        self.caller = Some(Caller {
            environment: environment.cloned(),
            index: index.cloned(),
            premised,
        });
        Ok(premised)
    }

    /// The shape of a completed descendant, which must have its own entry
    /// with the same selected source.
    fn shape(&self, key: Key, proof: Proof) -> Option<ShapeKey> {
        let Some(Entry::Proven(steps)) = self.entries.get(&key) else {
            return None;
        };
        steps
            .last()
            .filter(|step| step.goal == key.goal && step.proof == proof)
            .map(|step| step.shape)
    }
}

fn same<T>(left: Option<&Arc<T>>, right: Option<&Arc<T>>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn catalog(facts: &Declarations) -> Option<SelectionState> {
    facts
        .source_bindings
        .as_deref()
        .map(crate::sema::compiler_items::Catalog::selection_state)
}

/// The marked frames of one query, in marking order.
#[derive(Default)]
pub(super) struct Marks(Vec<Mark>);

/// The state a marked frame's search must leave unchanged to be recorded.
#[derive(Clone, Copy)]
struct Mark {
    mode: u8,
    promises: usize,
    extent: usize,
    sizes: usize,
    regions: u32,
    /// A self-check compares this fresh search with the existing entry.
    checking: bool,
}

impl Search<'_> {
    /// The table key's mode, or why this frame or query is outside the table.
    fn answer_mode(&self, facts: &Declarations, frame: &Frame) -> Result<u8, Event> {
        let query = &self.query;
        if frame.scope != ScopeId::ROOT || query.scopes.is_some() || query.universal.is_some() {
            return Err(Event::Scoped);
        }
        if frame.sources != Sources::All || frame.explicit_auto {
            return Err(Event::Restricted);
        }
        if frame.normalized_inputs.is_some() || frame.formation.is_some() {
            return Err(Event::Normalizing);
        }
        if query.reuse.is_some() {
            return Err(Event::Reusing);
        }
        if query.purpose != Purpose::Use || self.caller.placeholders.is_some() {
            return Err(Event::Purpose);
        }
        if facts.generics.partial_epoch
            || !facts.types.nominal_sizes_sealed()
            || !facts
                .generics
                .implementations
                .as_ref()
                .is_some_and(|registry| {
                    registry.headers_validated() && registry.selection_headers_ready()
                })
        {
            return Err(Event::Unsealed);
        }
        let availability = match query.availability {
            Availability::Validated => 0,
            Availability::Executable => 1,
            Availability::Declarations => 2,
        };
        Ok(availability | (u8::from(self.caller.retain_sized) << 2))
    }

    fn caller_class(&mut self, facts: &mut Declarations) -> Result<Class, String> {
        let premised = if let Some(known) = self.query.answer_premised {
            known
        } else {
            let known = facts.generics.selection_memo.answers().premised(
                &facts.types,
                self.caller.environment,
                self.caller.assumptions.clause_index(),
            )?;
            self.query.answer_premised = Some(known);
            known
        };
        Ok(if premised {
            Class::Premised
        } else {
            Class::Neutral {
                promises: self
                    .caller
                    .environment
                    .map_or(0, |environment| environment.projection_promise_identity()),
            }
        })
    }

    /// Before a resolved, non-automatic goal is keyed for its own search:
    /// replay the table's answer, or mark the frame so that its finished
    /// search may enter the table.
    pub(super) fn answer(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        goal: Predicate,
    ) -> Result<Option<Decision>, String> {
        frame.answer = None;
        if !goal.is_concrete(&facts.types)? || goal.contains_opaques(&facts.types)? {
            return Ok(None);
        }
        let mode = match self.answer_mode(facts, frame) {
            Ok(mode) => mode,
            Err(event) => {
                query_census::answer(event);
                return Ok(None);
            }
        };
        let Class::Neutral { promises } = self.caller_class(facts)? else {
            query_census::answer(Event::CallerPremises);
            return Ok(None);
        };
        query_census::answer(Event::Lookup);
        let catalog = catalog(facts);
        let answers = facts.generics.selection_memo.answers().current(catalog);
        let key = Key {
            goal,
            mode,
            promises,
        };
        let checking = match answers.entries.get(&key) {
            None => false,
            Some(entry) if entry.intersects(&self.query) => {
                query_census::answer(Event::Intersecting);
                return Ok(None);
            }
            Some(_) if crate::support::sealed::self_check() => {
                query_census::answer(Event::Checked);
                #[cfg(test)]
                {
                    answers.counts.checked += 1;
                }
                true
            }
            Some(entry) => {
                query_census::answer(Event::Hit);
                let decision = self.replay(frame, entry);
                #[cfg(test)]
                {
                    answers.counts.hits += 1;
                }
                return Ok(Some(decision));
            }
        };
        let marks = &mut self.query.answer_marks.get_or_insert_with(Box::default).0;
        let Some(slot) = u16::try_from(marks.len() + 1)
            .ok()
            .and_then(NonZeroU16::new)
        else {
            return Ok(None);
        };
        marks.push(Mark {
            mode,
            promises,
            extent: 0,
            sizes: 0,
            regions: 0,
            checking,
        });
        frame.answer = Some(slot);
        Ok(None)
    }

    /// Once a marked frame holds its shape key, record what its own search
    /// may not change.
    pub(super) fn begin_answer(&mut self, facts: &Declarations, frame: &Frame) {
        let Some(mark) = frame.answer.and_then(|slot| {
            let marks = &mut self.query.answer_marks.as_deref_mut()?.0;
            marks.get_mut(usize::from(slot.get()) - 1)
        }) else {
            return;
        };
        mark.extent = facts.types.interned_extent();
        mark.sizes = self.query.builtins.learned_sizes();
        mark.regions = self.query.allocations.end();
    }

    /// After a marked frame's search: enter its answer, or under a self-check
    /// fail unless it equals the entry that a hit would have replayed.
    pub(super) fn remember_answer(
        &mut self,
        facts: &mut Declarations,
        frame: &Frame,
        decision: Decision,
    ) -> Result<(), String> {
        let Some(&mark) = frame.answer.and_then(|slot| {
            let marks = &self.query.answer_marks.as_deref()?.0;
            marks.get(usize::from(slot.get()) - 1)
        }) else {
            return Ok(());
        };
        let (Some(goal), Some(shape)) = (frame.canonical, frame.key) else {
            return Ok(());
        };
        let key = Key {
            goal,
            mode: mark.mode,
            promises: mark.promises,
        };
        let unsettled = if frame.cycles != self.query.cycles {
            Some(Event::Cyclic)
        } else if mark.extent != facts.types.interned_extent() {
            Some(Event::Interning)
        } else if mark.sizes != self.query.builtins.learned_sizes() {
            Some(Event::Sizing)
        } else if mark.regions != self.query.allocations.end() {
            Some(Event::Allocating)
        } else {
            None
        };
        let catalog = catalog(facts);
        let answers = facts.generics.selection_memo.answers().current(catalog);
        let fresh = match decision {
            Decision::Absent => Some(Entry::Absent(shape)),
            Decision::Proven(index) => self.capture(answers, index, key, shape),
            Decision::Deferred | Decision::Refuted(_) => None,
        };
        if mark.checking {
            if answers.entries.get(&key) != fresh.as_ref() {
                return Err(format!(
                    "self-check: the cached trait answer for {goal:?} differs from a fresh \
                     search: cached {:?}, fresh {fresh:?}",
                    answers.entries.get(&key)
                ));
            }
            return Ok(());
        }
        match (fresh, unsettled) {
            (Some(entry), None) => {
                let promises = self
                    .caller
                    .environment
                    .and_then(|environment| environment.projection_promise_set());
                if !answers.insert(key, entry, promises) {
                    return Err("trait answer key names another caller's promises".into());
                }
                query_census::answer(Event::Recorded);
                #[cfg(test)]
                {
                    answers.counts.recorded += 1;
                }
            }
            (_, Some(event)) => query_census::answer(event),
            (None, None) if matches!(decision, Decision::Proven(_)) => {
                query_census::answer(Event::Uncaptured);
            }
            (None, None) => query_census::answer(Event::Unproven),
        }
        Ok(())
    }

    /// The selected proof below `root`, when every node is a debt-free source
    /// or builtin proof and every descendant already has its own entry. Its
    /// steps follow the first-visit postorder of the children, so the entry
    /// does not depend on which nodes this query had completed before.
    fn capture(&self, answers: &Answers, root: usize, key: Key, shape: ShapeKey) -> Option<Entry> {
        let nodes = &self.query.nodes;
        // Child flags reach the root when a node is pushed, so the root's
        // flags cover the whole proof.
        let top = nodes.get(root)?;
        if !top.debt_free || top.provisional_size || top.cycle_dependent || top.goal != key.goal {
            return None;
        }
        let mut order = Vec::new();
        let mut entered = vec![root];
        let mut stack = vec![(root, 0)];
        while let Some((index, next)) = stack.last_mut() {
            if let Some(&child) = nodes.get(*index)?.children.get(*next) {
                *next += 1;
                if !entered.contains(&child) {
                    if entered.len() == STEPS {
                        return None;
                    }
                    entered.push(child);
                    stack.push((child, 0));
                }
            } else {
                order.push(*index);
                stack.pop();
            }
        }
        let steps = order
            .iter()
            .map(|&index| {
                let node = &nodes[index];
                let proof = Proof::of(node.source.as_ref())?;
                let shape = if index == root {
                    shape
                } else {
                    answers.shape(
                        Key {
                            goal: node.goal,
                            ..key
                        },
                        proof,
                    )?
                };
                let children = node
                    .children
                    .iter()
                    .map(|child| order.iter().position(|index| index == child))
                    .collect::<Option<_>>()?;
                Some(Step {
                    goal: node.goal,
                    shape,
                    proof,
                    children,
                })
            })
            .collect::<Option<_>>()?;
        Some(Entry::Proven(steps))
    }

    /// Push an entry's proof as this query's own completed nodes, at the
    /// region a fresh search would have started that proof's root.
    fn replay(&mut self, frame: &Frame, entry: &Entry) -> Decision {
        let Entry::Proven(steps) = entry else {
            return Decision::Absent;
        };
        let first = match steps.last().map(|step| step.proof) {
            Some(Proof::Builtin) => frame.first,
            _ => frame.available_region().max(self.query.allocations.floor()),
        };
        let query = &mut self.query;
        let start = query.nodes.len();
        let mut edges = 0;
        for step in steps {
            edges += step.children.len();
            let index = query.push(
                Node {
                    debt_free: false,
                    cycle_dependent: false,
                    provisional_size: false,
                    goal: step.goal,
                    source: Some(step.proof.source()),
                    children: step.children.iter().map(|child| start + child).collect(),
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
        // A step with two parents is shared, as a completed node is.
        query.reused |= edges >= steps.len();
        Decision::Proven(query.nodes.len() - 1)
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/answer_table.rs"]
mod tests;
