//! An off-by-default shadow count of the trait queries one canonical cache
//! would answer. It reads the goals the search receives, interns nothing and
//! changes no decision; its rows are diagnostics, never proof facts.
mod callers;
#[cfg(test)]
#[path = "../../tests/invariants/query_census.rs"]
mod tests;
use super::traits::selection::Caller;
use crate::support::hash::{FxHasher, HashSet};
use crate::types::{
    Kind, RegionId, Store, TypeId,
    arguments::Argument as Canonical,
    infer::{Context, arguments::Argument, predicates::Predicate as Goal},
    parameters::{
        ParameterId, ParameterKind, environment::Environment, predicates::Bound,
        premise_index::ClauseIndex,
    },
};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Set once any thread counts; an ordinary compilation pays this one load.
static ACTIVE: AtomicBool = AtomicBool::new(false);
thread_local! {
    static STATE: RefCell<Option<Box<State>>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Declarations,
    /// Body typing, and the constant completion that follows it.
    Bodies,
    /// Everything after analysis: demanded instances and lowering.
    Instances,
}
impl Phase {
    pub const ALL: [Self; 3] = [Self::Declarations, Self::Bodies, Self::Instances];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Declarations => "declarations",
            Self::Bodies => "bodies",
            Self::Instances => "instances",
        }
    }
}
/// A root is one query a consumer asked; a frame is every goal a search
/// evaluated for it, the root's own included; a lookup is a frame that no
/// assumption, builtin rule or per-query proof answered before the
/// implementation candidates were assembled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Root,
    Frame,
    Lookup,
}
impl Level {
    pub const ALL: [Self; 3] = [Self::Root, Self::Frame, Self::Lookup];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Frame => "frame",
            Self::Lookup => "lookup",
        }
    }
}
/// Keys are distinct within their phase; `*_first` counts those that no
/// earlier phase had asked either.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Row {
    pub queries: usize,
    /// An input kept an inference variable or a constant argument.
    pub open: usize,
    /// Keyed queries that name a free region, which the key numbers.
    pub regional: usize,
    /// No parameter in any input and no caller clause for the trait head.
    pub independent: usize,
    pub independent_keys: usize,
    pub independent_first: usize,
    /// The key carries the caller's environment identity.
    pub dependent: usize,
    pub dependent_keys: usize,
    pub dependent_first: usize,
    /// Dependent keys again, where callers with equal premises share one
    /// identity.
    pub structural_keys: usize,
    pub structural_first: usize,
    /// Likewise, with every parameter named by its argument slot.
    pub renamed_keys: usize,
    pub renamed_first: usize,
}

/// What the crate-wide answer table did with one resolved goal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The goal is closed and its search may use the table.
    Lookup,
    Hit,
    /// The entry's proof meets an active goal or a completed node of this
    /// search, so the ordinary search keeps its cycle cutoffs and sharing.
    Intersecting,
    /// A self-check re-ran the search that a hit would have answered.
    Checked,
    Recorded,
    /// The finished search did not enter the table: it met a cycle cutoff,
    /// interned a type, learned a size or allocated a region; it proved
    /// more than a closed source proof; or it deferred or refuted the goal.
    Cyclic,
    Interning,
    Sizing,
    Allocating,
    Uncaptured,
    Unproven,
    /// The goal is closed but the caller has a closed premise.
    CallerPremises,
    /// The goal is closed but its frame or query is outside the table: a
    /// nested scope or universe, a restricted source set, a normalization or
    /// formation frame, a proof-reuse session, a non-use purpose or
    /// placeholder premises, or an unsealed registry or size table.
    Scoped,
    Restricted,
    Normalizing,
    Reusing,
    Purpose,
    Unsealed,
}
impl Answer {
    pub const ALL: [Self; 18] = [
        Self::Lookup,
        Self::Hit,
        Self::Intersecting,
        Self::Checked,
        Self::Recorded,
        Self::Cyclic,
        Self::Interning,
        Self::Sizing,
        Self::Allocating,
        Self::Uncaptured,
        Self::Unproven,
        Self::CallerPremises,
        Self::Scoped,
        Self::Restricted,
        Self::Normalizing,
        Self::Reusing,
        Self::Purpose,
        Self::Unsealed,
    ];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Lookup => "lookup",
            Self::Hit => "hit",
            Self::Intersecting => "intersecting",
            Self::Checked => "checked",
            Self::Recorded => "recorded",
            Self::Cyclic => "cyclic",
            Self::Interning => "interning",
            Self::Sizing => "sizing",
            Self::Allocating => "allocating",
            Self::Uncaptured => "uncaptured",
            Self::Unproven => "unproven",
            Self::CallerPremises => "caller-premises",
            Self::Scoped => "scoped",
            Self::Restricted => "restricted",
            Self::Normalizing => "normalizing",
            Self::Reusing => "reusing",
            Self::Purpose => "purpose",
            Self::Unsealed => "unsealed",
        }
    }
}

const EXACT: u64 = 1;
const REFERENCE: u64 = 2;
const ADT: u64 = 4;
const TUPLE: u64 = 5;
const SLICE: u64 = 6;
const ARRAY: u64 = 7;
const POINTER: u64 = 8;
const FIXED_REGION: u64 = 10;
const FREE_REGION: u64 = 11;
const CONSTANT: u64 = 12;
const CHILD: u64 = 13;
const PARAMETER: u64 = 14;
/// Key classes beside a level and its dependence.
const STRUCTURAL: u8 = 0x20;
const RENAMED: u8 = 0x40;

#[derive(Default)]
struct Scope {
    clauses: Option<Arc<ClauseIndex>>,
    environment: Option<Arc<Environment>>,
    /// The premises as content, exactly and renamed; `None` keeps identity.
    structure: Option<[[u64; 2]; 2]>,
}
impl Scope {
    fn identities(&self) -> [u64; 2] {
        let address = |address: usize| u64::try_from(address).unwrap_or(u64::MAX);
        [
            self.clauses
                .as_ref()
                .map_or(0, |index| address(Arc::as_ptr(index).addr())),
            self.environment
                .as_ref()
                .map_or(0, |environment| address(Arc::as_ptr(environment).addr())),
        ]
    }
    /// Whether any layer of the caller's clauses names this trait head.
    fn mentions(&self, item: crate::types::ItemId) -> bool {
        self.clauses.as_deref().is_some_and(|index| {
            index.layers().any(|(layer, _)| {
                layer.candidates.keys().any(|(head, _)| *head == item)
                    || layer.quantified.as_deref().is_some_and(|quantified| {
                        quantified.candidates.keys().any(|(head, _)| *head == item)
                    })
            })
        })
    }
}
#[derive(Default)]
struct State {
    phase: usize,
    rows: [[Row; 3]; 3],
    answers: [usize; Answer::ALL.len()],
    phase_keys: HashSet<(u8, u64, u64)>,
    keys: HashSet<(u8, u64, u64)>,
    /// The innermost root's caller and mode.
    scope: Scope,
    mode: u64,
    /// Name parameters by their argument slot rather than their identity.
    rename: bool,
    /// Caller changes whose premises were read as content, and those that
    /// kept their identity.
    scopes: [usize; 2],
    /// Every earlier caller stays alive, so an address names one caller.
    pinned: Vec<Scope>,
    key: Vec<u64>,
    regions: Vec<RegionId>,
    stack: Vec<TypeId>,
}

/// Count this thread's trait queries from now on.
pub fn start() {
    STATE.with(|state| {
        if let Ok(mut state) = state.try_borrow_mut() {
            *state = Some(Box::default());
            ACTIVE.store(true, Ordering::Relaxed);
        }
    });
}
/// The pipeline names the phase its next queries belong to.
pub fn enter(phase: Phase) {
    if active() {
        with(|state| state.phase = phase as usize);
    }
}
/// Every counted row, in phase then level order; empty unless started.
#[must_use]
pub fn rows() -> Vec<(Phase, Level, Row)> {
    let mut rows = Vec::new();
    with(|state| {
        for phase in Phase::ALL {
            for level in Level::ALL {
                rows.push((phase, level, state.rows[phase as usize][level as usize]));
            }
        }
    });
    rows
}
/// Every answer-table count, in [`Answer::ALL`] order; empty unless started.
#[must_use]
pub fn answer_rows() -> Vec<(Answer, usize)> {
    let mut rows = Vec::new();
    with(|state| rows.extend(Answer::ALL.into_iter().zip(state.answers)));
    rows
}
/// Caller changes whose premises were read as content, then those whose
/// free regions, openings or feature grants kept the caller's identity.
#[must_use]
pub fn scope_rows() -> Vec<(&'static str, usize)> {
    let mut rows = Vec::new();
    with(|state| rows.extend(["content", "identity"].into_iter().zip(state.scopes)));
    rows
}
/// Count one answer-table event.
pub(in crate::sema) fn answer(event: Answer) {
    if active() {
        with(|state| state.answers[event as usize] += 1);
    }
}
#[inline]
pub(in crate::sema) fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}
fn with(action: impl FnOnce(&mut State)) {
    STATE.with(|state| {
        if let Ok(mut state) = state.try_borrow_mut()
            && let Some(state) = state.as_deref_mut()
        {
            action(state);
        }
    });
}
/// Two independent hashes of a key.
fn pair(key: &[u64]) -> [u64; 2] {
    let mut fast = FxHasher::default();
    key.hash(&mut fast);
    let mut slow = std::hash::DefaultHasher::new();
    key.hash(&mut slow);
    [fast.finish(), slow.finish()]
}
fn same<T>(left: Option<&Arc<T>>, right: Option<&Arc<T>>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

/// A consumer's query under its own caller, before any assumption or search.
pub(in crate::sema) fn root(
    types: &Store,
    context: &mut Context,
    goal: &Goal,
    caller: &Caller<'_>,
    mode: u32,
) {
    with(|state| {
        let clauses = caller.assumptions.clause_index();
        if !same(state.scope.clauses.as_ref(), clauses)
            || !same(state.scope.environment.as_ref(), caller.environment)
        {
            let structure = state.premises(
                types,
                clauses.map(Arc::as_ref),
                caller.environment.map(Arc::as_ref),
            );
            state.scopes[usize::from(structure.is_none())] += 1;
            let previous = std::mem::replace(
                &mut state.scope,
                Scope {
                    clauses: clauses.cloned(),
                    environment: caller.environment.cloned(),
                    structure,
                },
            );
            state.pinned.push(previous);
        }
        state.mode = u64::from(mode)
            | (u64::from(caller.retain_sized) << 8)
            | (u64::from(caller.placeholders.is_some()) << 9);
        state.observe(types, context, goal, Level::Root);
    });
}
/// One goal a search evaluates under the innermost root's caller.
pub(in crate::sema) fn frame(types: &Store, context: &mut Context, goal: &Goal) {
    with(|state| state.observe(types, context, goal, Level::Frame));
}
/// One goal whose implementation candidates the search is about to assemble.
pub(in crate::sema) fn lookup(types: &Store, context: &mut Context, goal: &Goal) {
    with(|state| state.observe(types, context, goal, Level::Lookup));
}

impl State {
    fn observe(&mut self, types: &Store, context: &mut Context, goal: &Goal, level: Level) {
        self.key.clear();
        self.regions.clear();
        let parameters = if self.requirement(goal) {
            self.inputs(types, context, goal)
        } else {
            None
        };
        let row = &mut self.rows[self.phase][level as usize];
        row.queries += 1;
        let Some(parameters) = parameters else {
            row.open += 1;
            return;
        };
        row.regional += usize::from(!self.regions.is_empty());
        let dependent = parameters
            || goal
                .trait_arguments()
                .is_some_and(|(item, _)| self.scope.mentions(item));
        self.key.push(self.mode);
        let goal_end = self.key.len();
        if dependent {
            self.key.extend(self.scope.identities());
        }
        let class = ((level as u8) << 1) | u8::from(dependent);
        let (distinct, first) = self.insert(class);
        if dependent {
            self.structural(types, context, goal, level, goal_end);
        }
        let row = &mut self.rows[self.phase][level as usize];
        let (count, keys, firsts) = if dependent {
            (
                &mut row.dependent,
                &mut row.dependent_keys,
                &mut row.dependent_first,
            )
        } else {
            (
                &mut row.independent,
                &mut row.independent_keys,
                &mut row.independent_first,
            )
        };
        *count += 1;
        *keys += usize::from(distinct);
        *firsts += usize::from(first);
    }

    /// Whether the key is new in this phase and new in the census.
    fn insert(&mut self, class: u8) -> (bool, bool) {
        let [fast, slow] = pair(&self.key);
        let phase = u8::try_from(self.phase).unwrap_or(u8::MAX);
        let distinct = self.phase_keys.insert(((phase << 3) | class, fast, slow));
        let first = self.keys.insert((class, fast, slow));
        (distinct, first)
    }

    /// Count a dependent goal's key once more under the caller's premises as
    /// content, exactly and then with its parameters renamed.
    fn structural(
        &mut self,
        types: &Store,
        context: &mut Context,
        goal: &Goal,
        level: Level,
        goal_end: usize,
    ) {
        let [exact, renamed] = self.scope.structure.unwrap_or_else(|| {
            let identity = self.scope.identities();
            [identity, identity]
        });
        let class = ((level as u8) << 1) | 1;
        self.key.truncate(goal_end);
        self.key.push(u64::from(self.scope.structure.is_some()));
        self.key.extend(exact);
        let (distinct, first) = self.insert(class | STRUCTURAL);
        let row = &mut self.rows[self.phase][level as usize];
        row.structural_keys += usize::from(distinct);
        row.structural_first += usize::from(first);
        self.key.clear();
        self.regions.clear();
        self.rename = true;
        let complete = self.requirement(goal) && self.inputs(types, context, goal).is_some();
        self.rename = false;
        if !complete {
            return;
        }
        self.key
            .extend([self.mode, u64::from(self.scope.structure.is_some())]);
        self.key.extend(renamed);
        let (distinct, first) = self.insert(class | RENAMED);
        let row = &mut self.rows[self.phase][level as usize];
        row.renamed_keys += usize::from(distinct);
        row.renamed_first += usize::from(first);
    }

    /// The requirement's own identity; false keeps the query open.
    fn requirement(&mut self, goal: &Goal) -> bool {
        if let (Some((item, _)), Some(effect)) = (goal.trait_arguments(), goal.constness()) {
            self.key.extend([0, u64::from(item.0), effect as u64]);
            return true;
        }
        if goal.projection_result().is_some() {
            self.key.push(2);
            return true;
        }
        match goal.builtin_bound() {
            // The lifetime follows as an argument with a canonical index.
            Some(Bound::Outlives(_)) => self.key.extend([1, 0]),
            Some(bound @ (Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_))) => {
                let mut hasher = FxHasher::default();
                bound.hash(&mut hasher);
                self.key.extend([1, 1, hasher.finish()]);
            }
            _ => return false,
        }
        true
    }

    /// Append every input; `None` when one is open, else whether any input
    /// mentions a parameter.
    fn inputs(&mut self, types: &Store, context: &mut Context, goal: &Goal) -> Option<bool> {
        let mut parameters = false;
        for argument in goal.arguments() {
            match argument {
                Argument::Lifetime(region) => self.region(region),
                Argument::Const(_) => return None,
                Argument::Type(term) => {
                    let ty = context.find_identity(types, term).ok().flatten()?;
                    parameters |= types.contains_parameters(ty) != Some(false);
                    self.ty(types, ty);
                }
            }
        }
        Some(parameters)
    }

    fn region(&mut self, region: RegionId) {
        if region == RegionId::STATIC
            || region == RegionId::ERASED
            || region.bound_parts().is_some()
        {
            self.key.extend([FIXED_REGION, u64::from(region.0)]);
            return;
        }
        let index = self
            .regions
            .iter()
            .position(|seen| *seen == region)
            .unwrap_or_else(|| {
                self.regions.push(region);
                self.regions.len() - 1
            });
        self.key
            .extend([FREE_REGION, u64::try_from(index).unwrap_or(u64::MAX)]);
    }

    /// A region-free type is its interned identity. Structure is opened only
    /// to number free regions; kinds not listed keep their exact identity.
    fn ty(&mut self, types: &Store, root: TypeId) {
        self.stack.clear();
        self.stack.push(root);
        while let Some(ty) = self.stack.pop() {
            if types.contains_regions(ty) != Some(true)
                && !(self.rename && types.contains_parameters(ty) == Some(true))
            {
                self.key.extend([EXACT, u64::from(ty.0)]);
                continue;
            }
            match types.kind(ty) {
                Some(Kind::Parameter(id)) if self.rename => self.parameter(types, id),
                Some(Kind::Reference {
                    mutable,
                    region,
                    referent,
                }) => {
                    self.key.push(REFERENCE + u64::from(mutable));
                    self.region(region);
                    self.stack.push(referent);
                }
                Some(Kind::Pointer { mutable, pointee }) => {
                    self.key.extend([POINTER, u64::from(mutable)]);
                    self.stack.push(pointee);
                }
                Some(Kind::Slice(element)) => {
                    self.key.push(SLICE);
                    self.stack.push(element);
                }
                Some(Kind::Array { element, length }) => {
                    self.key.extend([ARRAY, length]);
                    self.stack.push(element);
                }
                Some(Kind::Tuple(list)) => {
                    let members = types.list(list).unwrap_or_default();
                    self.key
                        .extend([TUPLE, u64::try_from(members.len()).unwrap_or(u64::MAX)]);
                    self.stack.extend(members.iter().rev());
                }
                Some(Kind::Adt(item, arguments)) => self.nominal(types, item, arguments),
                _ => self.key.extend([EXACT, u64::from(ty.0)]),
            }
        }
    }

    /// A parameter by its identity, or renamed to its argument slot.
    fn parameter(&mut self, types: &Store, id: ParameterId) {
        let Some(parameter) = types.parameter(id).filter(|_| self.rename) else {
            self.key.extend([PARAMETER, u64::from(id.0)]);
            return;
        };
        let kind = match parameter.kind {
            ParameterKind::Type { sized } => u64::from(sized),
            ParameterKind::Const { ty } => 2 + u64::from(ty.0),
        };
        self.key
            .extend([PARAMETER + 1, u64::from(parameter.index), kind]);
    }

    fn nominal(
        &mut self,
        types: &Store,
        item: crate::types::ItemId,
        arguments: crate::types::GenericArgs,
    ) {
        self.key.extend([ADT, u64::from(item.0)]);
        let before = self.stack.len();
        for argument in types.arguments(arguments).into_iter().flatten() {
            match argument {
                Canonical::Lifetime(region) => self.region(region),
                Canonical::Const(constant) => {
                    self.key.extend([CONSTANT, u64::from(constant.0)]);
                }
                Canonical::Type(member) => {
                    self.key.push(CHILD);
                    self.stack.push(member);
                }
            }
        }
        self.stack[before..].reverse();
    }
}
