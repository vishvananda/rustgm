//! Whole builtin assumptions have immutable scope and indexed complete-subject queries.
use super::{
    Abilities, Environment,
    instantiation::{Matches, Pattern},
};
use crate::support::hash::{HashMap, HashSet};
use crate::support::retained::Growth;
use crate::types::{
    Store, TypeId,
    parameters::{clauses::Clause, predicates::Bound},
    region_graph::Error,
    structural::Shapes,
    substitute,
};
use std::sync::Arc;

#[derive(Clone, Default)]
pub(super) struct Index {
    pub region_mapping: bool,
    entries: Vec<Clause>,
    unique: HashSet<Clause>,
    subjects: HashSet<Subject>,
    candidates: HashMap<u64, Vec<Subject>>,
    shapes: Shapes,
    autos: u8,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Subject {
    pattern: Pattern,
    bound: Bound,
}
impl Index {
    pub(super) fn callable_allocation_bound(clauses: usize, nodes: usize, edges: usize) -> usize {
        use crate::support::retained::new_map;
        size_of::<Self>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(
                clauses.saturating_mul(4 * (size_of::<Clause>() + size_of::<Subject>())),
            )
            .saturating_add(new_map::<Clause, ()>(clauses))
            .saturating_add(new_map::<Subject, ()>(clauses))
            .saturating_add(new_map::<u64, Vec<Subject>>(clauses))
            .saturating_add(Shapes::callable_allocation_bound(nodes, edges))
    }
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.vector(&self.entries);
        total.set(&self.unique);
        total.set(&self.subjects);
        total.map(&self.candidates);
        for values in self.candidates.values() {
            total.vector(values);
        }
        total.include(self.shapes.allocation());
        total
    }
}
impl Index {
    pub fn clauses(&self) -> impl Iterator<Item = Clause> + '_ {
        self.entries.iter().copied()
    }
    pub fn insert(&mut self, types: &Store, clause: Clause) -> Result<(), &'static str> {
        self.insert_observed::<false>(types, clause, &|_, _| Ok(()))
    }
    fn insert_observed<const CHARGED: bool>(
        &mut self,
        types: &Store,
        clause: Clause,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if CHARGED {
            charge(1, 0)?;
        }
        if self.unique.contains(&clause) {
            return Ok(());
        }
        self.region_mapping |= clause.has_free_regions(types)?;
        if self.entries.len() >= 1_000_000 {
            return Err("builtin assumption storage budget exceeded");
        }
        let (predicate, quantified) = match clause {
            Clause::UnstableFeature(_) => {
                return Err("non-type requirement cannot grant a type builtin");
            }
            Clause::Predicate(predicate) => (predicate, false),
            Clause::ForAll(id) => (
                types
                    .quantified(id)
                    .ok_or("invalid quantified builtin assumption")?
                    .predicate,
                true,
            ),
        };
        if !matches!(
            predicate.bound,
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_)
        ) {
            return Err("builtin assumption requires a builtin marker bound");
        }
        if let Bound::Auto(marker) = predicate.bound {
            self.autos |= 1 << marker as u8;
        }
        let shape = if CHARGED {
            self.shapes.get_charged(types, predicate.ty, charge)?
        } else {
            self.shapes.get(types, predicate.ty).map_err(error)?
        };
        let subject = Subject {
            pattern: Pattern {
                ty: predicate.ty,
                quantified,
            },
            bound: predicate.bound,
        };
        if CHARGED {
            self.prepay_entry(subject, shape, charge)?;
        }
        if self.subjects.insert(subject) {
            self.candidates.entry(shape).or_default().push(subject);
        }
        self.entries.push(clause);
        self.unique.insert(clause);
        Ok(())
    }
    pub fn substitute(
        &self,
        types: &mut Store,
        fold: &mut substitute::Bound,
    ) -> Result<Self, &'static str> {
        self.substitute_observed::<false>(types, fold, &|_, _| Ok(()))
    }
    pub(super) fn substitute_charged(
        &self,
        types: &mut Store,
        fold: &mut substitute::Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        self.substitute_observed::<true>(types, fold, charge)
    }
    fn substitute_observed<const CHARGED: bool>(
        &self,
        types: &mut Store,
        fold: &mut substitute::Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Self, &'static str> {
        let mut mapped = Self::default();
        for clause in self.clauses() {
            let clause = if CHARGED {
                charge(1, 0)?;
                fold.clause_charged(types, clause, charge)?
            } else {
                fold.clause(types, clause)?
            };
            mapped.insert_observed::<CHARGED>(types, clause, charge)?;
        }
        Ok(mapped)
    }
}
#[derive(Clone, Copy, Default)]
struct Permissions {
    abilities: Abilities,
    sized: bool,
    autos: u8,
}
pub(crate) struct Query {
    environment: Arc<Environment>,
    shapes: Shapes,
    matches: Matches,
    completed: HashMap<(TypeId, u16), Permissions>,
    requests: usize,
}
impl Query {
    fn new(environment: Arc<Environment>) -> Self {
        Self {
            environment,
            shapes: Shapes::default(),
            matches: Matches::default(),
            completed: HashMap::default(),
            requests: 0,
        }
    }
    pub(crate) fn steps(&self) -> usize {
        self.requests
            .saturating_add(self.shapes.steps)
            .saturating_add(self.matches.steps())
    }
    fn get(&mut self, types: &Store, ty: TypeId, depth: u16) -> Result<Permissions, &'static str> {
        self.requests = self.requests.saturating_add(1);
        if self.steps() > 1_000_000 {
            return Err("builtin assumption query work budget exceeded");
        }
        if let Some(&found) = self.completed.get(&(ty, depth)) {
            return Ok(found);
        }
        let key = self.shapes.get(types, ty).map_err(error)?;
        let index = self
            .environment
            .builtins
            .as_ref()
            .ok_or("missing builtin assumption index")?;
        let mut permissions = Permissions::default();
        for predicate in index.candidates.get(&key).into_iter().flatten() {
            self.requests = self.requests.saturating_add(1);
            if !self
                .matches
                .accepts(
                    types,
                    ty,
                    predicate.pattern,
                    depth,
                    self.environment.regions.as_deref(),
                )
                .map_err(error)?
            {
                continue;
            }
            if let Bound::Auto(marker) = predicate.bound {
                permissions.autos |= 1 << marker as u8;
            } else {
                permissions.sized = true;
                permissions.abilities.copy |= predicate.bound == Bound::Copy;
                permissions.abilities.clone_ |=
                    matches!(predicate.bound, Bound::Copy | Bound::Clone);
            }
            if permissions.abilities.copy && permissions.autos == index.autos {
                break;
            }
        }
        if self.steps() > 1_000_000 {
            return Err("builtin assumption query work budget exceeded");
        }
        self.completed.insert((ty, depth), permissions);
        Ok(permissions)
    }
}
impl Environment {
    pub(crate) fn matching_auto(
        self: &Arc<Self>,
        types: &Store,
        obligation: crate::types::auto_traits::Obligation,
        query: &mut Option<Box<Query>>,
    ) -> Result<bool, &'static str> {
        Ok(self
            .matching_permissions(types, obligation.ty, 0, query)?
            .autos
            & (1 << obligation.marker as u8)
            != 0)
    }
    pub(crate) fn has_region_builtins(&self) -> bool {
        self.builtins.is_some()
    }
    pub(crate) fn with_region_equalities(
        mut self,
        mut equalities: crate::types::relations::RegionEqualities,
    ) -> Self {
        if let Some(aliases) = self.body_aliases() {
            equalities = equalities.with_body_aliases(Arc::clone(aliases));
        }
        if !equalities.is_empty() || equalities.body_aliases().is_some() {
            self.regions = Some(Arc::new(equalities));
            self.region_mapping = true;
        }
        self
    }
    fn check_builtin_query(self: &Arc<Self>, query: Option<&Query>) -> Result<(), &'static str> {
        if query.is_some_and(|q| !Arc::ptr_eq(self, &q.environment)) {
            return Err("builtin query belongs to a different parameter environment");
        }
        Ok(())
    }
    fn matching_permissions(
        self: &Arc<Self>,
        types: &Store,
        ty: TypeId,
        depth: u16,
        query: &mut Option<Box<Query>>,
    ) -> Result<Permissions, &'static str> {
        self.check_builtin_query(query.as_deref())?;
        if self.builtins.is_none() {
            return Ok(Permissions::default());
        }
        query
            .get_or_insert_with(|| Box::new(Query::new(Arc::clone(self))))
            .get(types, ty, depth)
    }
    pub(crate) fn assumed_with(
        self: &Arc<Self>,
        types: &Store,
        ty: TypeId,
        query: &mut Option<Box<Query>>,
    ) -> Result<Abilities, &'static str> {
        let ordinary = self.assumed(ty);
        if ordinary.copy {
            self.check_builtin_query(query.as_deref())?;
            return Ok(ordinary);
        }
        let quantified = self.matching_permissions(types, ty, 0, query)?.abilities;
        Ok(Abilities {
            copy: ordinary.copy || quantified.copy,
            clone_: ordinary.clone_ || quantified.clone_,
        })
    }
    pub(crate) fn matching_sized(
        self: &Arc<Self>,
        types: &Store,
        ty: TypeId,
        depth: u16,
        query: &mut Option<Box<Query>>,
    ) -> Result<bool, &'static str> {
        if depth == 0 && self.assumes_sized(ty) {
            self.check_builtin_query(query.as_deref())?;
            return Ok(true);
        }
        Ok(self.matching_permissions(types, ty, depth, query)?.sized)
    }
}
fn error(error: Error) -> &'static str {
    match error {
        Error::Budget => "builtin assumption matching work budget exceeded",
        _ => "invalid builtin assumption matching input",
    }
}

impl Index {
    fn prepay_entry(
        &self,
        subject: Subject,
        shape: u64,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let mut growth = Growth::vector::<Clause>(self.entries.len(), self.entries.capacity(), 1);
        growth.include(Growth::map::<Clause, ()>(
            self.unique.len(),
            self.unique.capacity(),
            1,
        ));
        growth.include(Growth::map::<Subject, ()>(
            self.subjects.len(),
            self.subjects.capacity(),
            usize::from(!self.subjects.contains(&subject)),
        ));
        if !self.subjects.contains(&subject) {
            growth.include(Growth::map::<u64, Vec<Subject>>(
                self.candidates.len(),
                self.candidates.capacity(),
                usize::from(!self.candidates.contains_key(&shape)),
            ));
            let (length, capacity) = self
                .candidates
                .get(&shape)
                .map_or((0, 0), |values| (values.len(), values.capacity()));
            growth.include(Growth::vector::<Subject>(length, capacity, 1));
        }
        charge(1 + growth.work, growth.bytes)
    }
}
