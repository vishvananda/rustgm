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
//! a fresh search would. A node that the search itself built needs no entry
//! of its own when it proves the same goal or is a nominal formation prefix;
//! a replay registers that formation as the search did.
//! The self-check mode runs the fresh search on every hit and compares.
use super::{
    Decision, Declarations, Frame, Node, Predicate, Query, Search, Source,
    frames::Sources,
    member_conditions::{Entry as Formed, Key as Family},
};
use crate::resolve::model::DefId;
use crate::sema::compiler_items::SelectionState;
use crate::sema::query_census::{self, Answer as Event};
use crate::sema::traits::implementations::{Availability, Purpose};
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, Store, TypeId,
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
    /// A node without a source: a nominal formation prefix, whose children
    /// prove the formed type's requirements.
    Formation,
}
impl Proof {
    fn of(source: Option<&Source>) -> Option<Self> {
        match source {
            None => Some(Self::Formation),
            Some(Source::Builtin) => Some(Self::Builtin),
            Some(&Source::Implementation { owner, arguments }) => {
                Some(Self::Implementation { owner, arguments })
            }
            Some(_) => None,
        }
    }
    const fn source(self) -> Option<Source> {
        match self {
            Self::Builtin => Some(Source::Builtin),
            Self::Implementation { owner, arguments } => {
                Some(Source::Implementation { owner, arguments })
            }
            Self::Formation => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Step {
    goal: Predicate,
    shape: ShapeKey,
    proof: Proof,
    children: Box<[usize]>,
    /// The root-scope nominal formations, as type and depth, that the search
    /// completed with this node, in a fixed order.
    formed: Box<[(TypeId, u16)]>,
}
struct CompletedFormation {
    node: usize,
    keys: Vec<(TypeId, u16)>,
}
#[derive(Debug, PartialEq, Eq)]
enum Entry {
    Absent(ShapeKey),
    /// Children precede their parents; the last step is the goal's own node.
    /// `scoped` records that the search reserved a scoped region frontier.
    Proven {
        steps: Box<[Step]>,
        scoped: bool,
    },
}
impl Entry {
    /// A proof through an active goal must keep the search's cycle cutoff, and
    /// one through a node this query completed must share that node. A
    /// formation this query holds would likewise be shared or cut off.
    fn intersects(&self, query: &Query) -> bool {
        match self {
            Self::Absent(shape) => query.active.contains(shape),
            Self::Proven { steps, .. } => steps.iter().any(|step| {
                query.active.contains(&step.shape)
                    || query.completed(ScopeId::ROOT, step.goal).is_some()
                    || step.formed.iter().any(|&(ty, depth)| {
                        query
                            .families
                            .contains_key(&Family::Nominal(ty, depth, ScopeId::ROOT))
                    })
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
        self.proven(key)
            .filter(|step| step.proof == proof)
            .map(|step| step.shape)
    }

    /// The last step of the proof entered for `key`.
    fn proven(&self, key: Key) -> Option<&Step> {
        let Some(Entry::Proven { steps, .. }) = self.entries.get(&key) else {
            return None;
        };
        steps.last().filter(|step| step.goal == key.goal)
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
    /// The query's first node index that this frame's own search built.
    nodes: usize,
    reservations: u32,
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
            nodes: 0,
            reservations: 0,
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
        mark.nodes = self.query.nodes.len();
        mark.reservations = self.query.allocations.reservations();
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
            Decision::Absent if self.query.allocations.reservations() == mark.reservations => {
                Some(Entry::Absent(shape))
            }
            Decision::Proven(index) => self.capture(answers, index, key, (shape, mark)),
            Decision::Absent | Decision::Deferred | Decision::Refuted(_) => None,
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

    /// The selected proof below `root`, when every node is a debt-free
    /// source, builtin or formation proof. A node of another goal needs its own
    /// entry with the same proof, unless it is a formation prefix that this
    /// frame's own search built; a node of the same goal needs one only when an
    /// earlier search built it. Its steps follow the first-visit postorder of
    /// the children, so the entry does not depend on which nodes this query had
    /// completed before.
    fn capture(
        &self,
        answers: &Answers,
        root: usize,
        key: Key,
        (shape, mark): (ShapeKey, Mark),
    ) -> Option<Entry> {
        let nodes = &self.query.nodes;
        // Child flags reach the root when a node is pushed, so the root's
        // flags cover the whole proof.
        let top = nodes.get(root)?;
        if !top.debt_free
            || top.provisional_size
            || top.cycle_dependent
            || top.goal != key.goal
            || top.source.is_none()
        {
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
        let mut formed = self.formed(&order)?;
        let steps: Box<[Step]> = order
            .iter()
            .map(|&index| {
                let node = &nodes[index];
                let proof = Proof::of(node.source.as_ref())?;
                let own = index >= mark.nodes;
                if proof == Proof::Formation && !own {
                    return None;
                }
                let goal = Key {
                    goal: node.goal,
                    ..key
                };
                let shape = if index == root || (own && node.goal == key.goal) {
                    shape
                } else if proof == Proof::Formation {
                    answers.proven(goal)?.shape
                } else {
                    answers.shape(goal, proof)?
                };
                let children = node
                    .children
                    .iter()
                    .map(|child| order.iter().position(|index| index == child))
                    .collect::<Option<_>>()?;
                let formed = formed
                    .iter_mut()
                    .find(|formation| formation.node == index)
                    .map(|formation| std::mem::take(&mut formation.keys))
                    .unwrap_or_default();
                Some(Step {
                    goal: node.goal,
                    shape,
                    proof,
                    children,
                    formed: formed.into_boxed_slice(),
                })
            })
            .collect::<Option<_>>()?;
        let scoped = self.query.allocations.reservations() != mark.reservations;
        // A replay reserves the frontier after placing its root; only a
        // builtin root's place does not depend on that order.
        if scoped && steps.last().is_none_or(|step| step.proof != Proof::Builtin) {
            return None;
        }
        Some(Entry::Proven { steps, scoped })
    }

    /// The root-scope nominal formations that this query completed with each
    /// formation node of `order`, in a fixed order, or `None` when one of them
    /// is registered under a key a replay does not restore.
    fn formed(&self, order: &[usize]) -> Option<Vec<CompletedFormation>> {
        let nodes = &self.query.nodes;
        let mut formed: Vec<_> = order
            .iter()
            .copied()
            .filter(|&node| nodes[node].source.is_none())
            .map(|node| CompletedFormation {
                node,
                keys: Vec::new(),
            })
            .collect();
        if formed.is_empty() {
            return Some(formed);
        }
        for (&family, &entry) in &self.query.families {
            let Formed::Completed(index) = entry else {
                continue;
            };
            let Some(formation) = formed.iter_mut().find(|formation| formation.node == index)
            else {
                continue;
            };
            match family {
                Family::Nominal(ty, depth, scope) if scope == ScopeId::ROOT => {
                    formation.keys.push((ty, depth));
                }
                _ => return None,
            }
        }
        for formation in &mut formed {
            if formation.keys.is_empty() {
                return None;
            }
            formation
                .keys
                .sort_unstable_by_key(|&(ty, depth)| (ty.0, depth));
        }
        Some(formed)
    }

    /// Push an entry's proof as this query's own completed nodes, at the
    /// region a fresh search would have started that proof's root, and
    /// register its formations and scoped frontier as that search did.
    fn replay(&mut self, frame: &Frame, entry: &Entry) -> Decision {
        let Entry::Proven { steps, scoped } = entry else {
            return Decision::Absent;
        };
        let first = match steps.last().map(|step| step.proof) {
            Some(Proof::Builtin) => frame.first,
            _ => frame.available_region().max(self.query.allocations.floor()),
        };
        let query = &mut self.query;
        if *scoped {
            // The search allocated no region, so its reservation met the
            // frontier this one meets.
            query.allocations.reserve_scoped();
        }
        let start = query.nodes.len();
        let mut edges = 0;
        let mut formation = false;
        for step in steps {
            edges += step.children.len();
            let index = query.push(
                Node {
                    debt_free: false,
                    cycle_dependent: false,
                    provisional_size: false,
                    goal: step.goal,
                    source: step.proof.source(),
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
            for &(ty, depth) in &step.formed {
                query.families.insert(
                    Family::Nominal(ty, depth, ScopeId::ROOT),
                    Formed::Completed(index),
                );
            }
            if step.proof == Proof::Formation {
                formation = true;
            } else {
                query.remember(ScopeId::ROOT, step.goal, Decision::Proven(index));
            }
        }
        // A step with two parents is shared, as a completed node is, and a
        // formation prefix joins its goal's proof as inherited inputs do.
        query.reused |= formation || edges >= steps.len();
        Decision::Proven(query.nodes.len() - 1)
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/answer_table.rs"]
mod tests;
