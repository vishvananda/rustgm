//! Shared DAG folds for captured binders and the required local lifetime slot count.
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, Kind, ListId, RegionId, Store, TypeId,
    arguments::Argument,
    parameters::predicates::{Bound, Predicate},
    traits::ApplicationId,
};
#[cfg(test)]
#[path = "../../../tests/invariants/slot_groups.rs"]
mod tests;

type Key = (TypeId, u16);
type ScopedApplications<V> = HashMap<(ApplicationId, u16), V>;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Group {
    Arguments(GenericArgs),
    Types(ListId),
}
type Groups<V> = HashMap<(Group, u16), V>;

pub(crate) struct Surrounding {
    count: u16,
}
pub(crate) struct LocalSlots(u16);
pub(crate) type Captures = Occurrences<Surrounding>;
pub(crate) type Slots = Occurrences<LocalSlots>;

pub(crate) trait Query {
    type Value: Copy + Default;
    fn merge(a: Self::Value, b: Self::Value) -> Self::Value;
    fn decisive(value: Self::Value) -> bool;
    fn region(&self, region: RegionId, depth: u16) -> Self::Value;
}
impl Query for Surrounding {
    type Value = bool;
    fn merge(a: bool, b: bool) -> bool {
        a | b
    }
    fn decisive(value: bool) -> bool {
        value
    }
    fn region(&self, region: RegionId, depth: u16) -> bool {
        region
            .bound_parts()
            .is_some_and(|(binder, _)| binder >= depth && binder - depth < self.count)
    }
}
impl Query for LocalSlots {
    type Value = u32;
    fn merge(a: u32, b: u32) -> u32 {
        a.max(b)
    }
    fn decisive(_: u32) -> bool {
        false
    }
    fn region(&self, region: RegionId, depth: u16) -> u32 {
        match region.bound_parts() {
            Some((binder, index)) if Some(binder) == depth.checked_add(self.0) => {
                u32::from(index) + 1
            }
            _ => 0,
        }
    }
}

pub(crate) struct Occurrences<Q: Query> {
    query: Q,
    completed: HashMap<Key, Q::Value>,
    applications: HashMap<ApplicationId, Q::Value>,
    // Ordinary capture queries never allocate the scoped application table.
    scoped_applications: Option<Box<ScopedApplications<Q::Value>>>,
    groups: Option<Box<Groups<Q::Value>>>,
    work: Vec<(Key, bool)>,
    pub(crate) steps: usize,
}
impl Captures {
    pub(crate) fn clause(
        &mut self,
        store: &Store,
        clause: crate::types::parameters::clauses::Clause,
    ) -> Result<bool, &'static str> {
        use crate::types::parameters::clauses::Clause;
        match clause {
            Clause::Predicate(predicate) => self.predicate_inside(store, predicate, 0),
            Clause::ForAll(id) => {
                let value = store.quantified(id).ok_or("invalid capture clause")?;
                self.predicate_inside(store, value.predicate, 1)
            }
        }
    }
    pub(crate) fn new(count: u16) -> Self {
        Self::query(Surrounding { count })
    }
}
impl Default for Slots {
    fn default() -> Self {
        Self::at(0)
    }
}
impl Slots {
    /// Summarize one fixed surrounding binder, keeping nested callable locals separate.
    pub(crate) fn at(depth: u16) -> Self {
        Self::query(LocalSlots(depth))
    }
}
impl<Q: Query> Occurrences<Q> {
    fn query(query: Q) -> Self {
        Self {
            query,
            completed: HashMap::default(),
            applications: HashMap::default(),
            scoped_applications: None,
            groups: None,
            work: Vec::new(),
            steps: 0,
        }
    }
    pub(crate) fn predicate(
        &mut self,
        store: &Store,
        value: Predicate,
    ) -> Result<Q::Value, &'static str> {
        self.predicate_inside(store, value, 0)
    }
    fn predicate_inside(
        &mut self,
        store: &Store,
        value: Predicate,
        depth: u16,
    ) -> Result<Q::Value, &'static str> {
        let subject = self.collect_inside(store, value.ty, depth)?;
        if Q::decisive(subject) {
            return Ok(subject);
        }
        let bound = match value.bound {
            Bound::ProjectionEqual(result) => self.collect_inside(store, result, depth),
            Bound::Outlives(region) => Ok(self.region(region, depth)),
            Bound::Trait(id) | Bound::ConstTrait(id) | Bound::MaybeConstTrait(id) => {
                let cached = if depth == 0 {
                    self.applications.get(&id)
                } else {
                    self.scoped_applications
                        .as_ref()
                        .and_then(|cache| cache.get(&(id, depth)))
                };
                if let Some(&found) = cached {
                    return Ok(Q::merge(subject, found));
                }
                let application = store
                    .trait_application(id)
                    .ok_or("invalid capture trait application")?;
                self.arguments(store, application.arguments, depth, false)?;
                self.drain(store)?;
                let found = self.arguments(store, application.arguments, depth, true)?;
                if depth == 0 {
                    self.applications.insert(id, found);
                } else {
                    self.scoped_applications
                        .get_or_insert_with(Default::default)
                        .insert((id, depth), found);
                }
                Ok(found)
            }
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_) => Ok(Q::Value::default()),
        }?;
        Ok(Q::merge(subject, bound))
    }
    pub(crate) fn collect(&mut self, store: &Store, ty: TypeId) -> Result<Q::Value, &'static str> {
        self.collect_inside(store, ty, 0)
    }
    fn collect_inside(
        &mut self,
        store: &Store,
        ty: TypeId,
        depth: u16,
    ) -> Result<Q::Value, &'static str> {
        let root = (ty, depth);
        if let Some(found) = self.result(store, root)? {
            return Ok(found);
        }
        self.work.push((root, false));
        self.drain(store)?;
        self.result(store, root)?.ok_or("missing capture result")
    }
    fn drain(&mut self, store: &Store) -> Result<(), &'static str> {
        while let Some((key, finish)) = self.work.pop() {
            self.account(1)?;
            if self.result(store, key)?.is_some() {
                continue;
            }
            if !finish {
                self.work.push((key, true));
            }
            let found = self.inspect(store, key, finish)?;
            if finish {
                self.completed.insert(key, found);
            }
        }
        Ok(())
    }
    fn result(&self, store: &Store, key: Key) -> Result<Option<Q::Value>, &'static str> {
        if store
            .escaping_binders(key.0)
            .ok_or("invalid capture type")?
            <= key.1
        {
            return Ok(Some(Q::Value::default()));
        }
        Ok(self.completed.get(&key).copied())
    }
    fn child(&mut self, store: &Store, key: Key, finish: bool) -> Result<Q::Value, &'static str> {
        if let Some(found) = self.result(store, key)? {
            return Ok(found);
        }
        if finish {
            return Err("missing capture dependency");
        }
        self.account(1)?;
        self.work.push((key, false));
        Ok(Q::Value::default())
    }
    fn inspect(
        &mut self,
        store: &Store,
        (ty, depth): Key,
        finish: bool,
    ) -> Result<Q::Value, &'static str> {
        let mut found = Q::Value::default();
        let list = match store.kind(ty).ok_or("invalid capture type")? {
            Kind::Reference {
                region, referent, ..
            } => {
                found = Q::merge(
                    self.region(region, depth),
                    self.child(store, (referent, depth), finish)?,
                );
                None
            }
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element)
            | Kind::Pointer {
                pointee: element, ..
            } => {
                found = self.child(store, (element, depth), finish)?;
                None
            }
            Kind::Tuple(list) => Some((list, depth)),
            Kind::Function {
                parameters, result, ..
            } => {
                let nested = depth
                    .checked_add(1)
                    .ok_or("capture binder depth overflow")?;
                found = self.child(store, (result, nested), finish)?;
                Some((parameters, nested))
            }
            Kind::Closure {
                arguments,
                signature,
                environment,
                ..
            } => {
                found = Q::merge(
                    self.arguments(store, arguments, depth, finish)?,
                    Q::merge(
                        self.child(store, (signature, depth), finish)?,
                        self.child(store, (environment, depth), finish)?,
                    ),
                );
                None
            }
            Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) => {
                found = self.arguments(store, arguments, depth, finish)?;
                None
            }
            Kind::Dynamic {
                arguments, region, ..
            } => {
                let nested = depth
                    .checked_add(1)
                    .ok_or("capture binder depth overflow")?;
                found = Q::merge(
                    self.region(region, depth),
                    self.arguments(store, arguments, nested, finish)?,
                );
                None
            }
            Kind::Projection(application) => {
                let trait_ = store
                    .trait_application(application.trait_application)
                    .ok_or("invalid capture projection trait")?;
                found = Q::merge(
                    self.child(store, (application.subject, depth), finish)?,
                    Q::merge(
                        self.arguments(store, trait_.arguments, depth, finish)?,
                        self.arguments(store, application.arguments, depth, finish)?,
                    ),
                );
                None
            }
            Kind::FunctionItem {
                signature,
                arguments,
                ..
            } => {
                found = Q::merge(
                    self.arguments(store, arguments, depth, finish)?,
                    self.child(store, (signature, depth), finish)?,
                );
                None
            }
            Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => None,
        };
        if let Some((list, depth)) = list {
            found = Q::merge(found, self.list(store, list, depth, finish)?);
        }
        Ok(found)
    }
    fn cached_group(&self, group: Group, depth: u16) -> Option<Q::Value> {
        self.groups.as_ref()?.get(&(group, depth)).copied()
    }
    fn remember_group(&mut self, group: Group, depth: u16, value: Q::Value, finish: bool) {
        if finish {
            self.groups
                .get_or_insert_with(Default::default)
                .insert((group, depth), value);
        }
    }
    fn list(
        &mut self,
        store: &Store,
        list: ListId,
        depth: u16,
        finish: bool,
    ) -> Result<Q::Value, &'static str> {
        let key = Group::Types(list);
        if let Some(found) = self.cached_group(key, depth) {
            return Ok(found);
        }
        let children = store.list(list).ok_or("invalid capture type list")?;
        if children.is_empty() {
            return Ok(Q::Value::default());
        }
        self.account(children.len())?;
        let mut found = Q::Value::default();
        for &child in children {
            found = Q::merge(found, self.child(store, (child, depth), finish)?);
        }
        self.remember_group(key, depth, found, finish);
        Ok(found)
    }
    fn arguments(
        &mut self,
        store: &Store,
        args: GenericArgs,
        depth: u16,
        finish: bool,
    ) -> Result<Q::Value, &'static str> {
        if args == GenericArgs::EMPTY {
            return Ok(Q::Value::default());
        }
        let key = Group::Arguments(args);
        if let Some(found) = self.cached_group(key, depth) {
            return Ok(found);
        }
        let arguments = store.arguments(args).ok_or("invalid capture arguments")?;
        self.account(arguments.len())?;
        let mut found = Q::Value::default();
        for argument in arguments {
            let value = match argument {
                Argument::Lifetime(region) => self.region(region, depth),
                Argument::Type(ty) => self.child(store, (ty, depth), finish)?,
                Argument::Const(_) => Q::Value::default(),
            };
            found = Q::merge(found, value);
        }
        self.remember_group(key, depth, found, finish);
        Ok(found)
    }
    fn region(&self, region: RegionId, depth: u16) -> Q::Value {
        self.query.region(region, depth)
    }
    fn account(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err("bound capture work budget exceeded");
        }
        Ok(())
    }
}

impl<Q: Query> Occurrences<Q> {
    pub(crate) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.completed);
        total.map(&self.applications);
        total.vector(&self.work);
        if let Some(value) = &self.scoped_applications {
            total.boxed(value.as_ref());
            total.map(value);
        }
        if let Some(value) = &self.groups {
            total.boxed(value.as_ref());
            total.map(value);
        }
        total
    }
}
