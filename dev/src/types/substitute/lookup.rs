//! Lower captured coordinates into existing canonical identities without interning.
use super::{Bound, Key, WORK_LIMIT, rewrite};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    GenericArgs, ListId, RegionId, Store, TypeId,
    arguments::{Argument, ConstId},
    traits::ApplicationId,
};

enum Error {
    Absent,
    Invalid(&'static str),
}
impl From<&'static str> for Error {
    fn from(message: &'static str) -> Self {
        Self::Invalid(message)
    }
}
enum ResultKind {
    Present(TypeId),
    Absent,
    Pending,
}
/// A lookup belongs to one fixed surrounding prefix and immutable assumption
/// environment. A missing canonical identity cannot match that environment;
/// later unrelated type interning must not be used to add new assumptions.
pub(crate) struct Lookup {
    amount: u16,
    bound: Bound,
    absent: HashSet<Key>,
    applications: HashMap<ApplicationId, Option<ApplicationId>>,
}
impl Lookup {
    pub(crate) fn new(amount: u16) -> Self {
        Self {
            amount,
            bound: Bound::lower(amount),
            absent: HashSet::default(),
            applications: HashMap::default(),
        }
    }
    pub(crate) const fn steps(&self) -> usize {
        self.bound.steps
    }
    fn result(&self, store: &Store, key: Key) -> Result<ResultKind, &'static str> {
        if let Some(ty) = self.bound.result(store, key)? {
            Ok(ResultKind::Present(ty))
        } else if self.absent.contains(&key) {
            Ok(ResultKind::Absent)
        } else {
            Ok(ResultKind::Pending)
        }
    }
    pub(crate) fn ty(&mut self, store: &Store, ty: TypeId) -> Result<Option<TypeId>, &'static str> {
        let result = self.apply(store, ty);
        self.bound.work.clear();
        result
    }
    fn apply(&mut self, store: &Store, ty: TypeId) -> Result<Option<TypeId>, &'static str> {
        let key = (ty, 0);
        if matches!(self.result(store, key)?, ResultKind::Pending) {
            self.bound.queue(ty, 0)?;
        }
        while let Some((key, finish)) = self.bound.work.pop() {
            self.bound.steps += 1;
            if self.bound.steps > WORK_LIMIT {
                return Err("canonical lifetime lookup work budget exceeded");
            }
            if !matches!(self.result(store, key)?, ResultKind::Pending) {
                continue;
            }
            let kind = store.kind(key.0).ok_or("invalid lifetime lookup type")?;
            if !finish {
                self.bound.work.push((key, true));
                self.bound.dependencies(store, kind, key.1)?;
                continue;
            }
            let rewritten = rewrite::rebuild(
                &mut Read {
                    lookup: self,
                    store,
                },
                kind,
                key.1,
            );
            let found = match rewritten {
                Ok(kind) => store.find_kind(kind),
                Err(Error::Absent) => None,
                Err(Error::Invalid(message)) => return Err(message),
            };
            if let Some(id) = found {
                self.bound.completed.insert(key, id);
            } else {
                self.absent.insert(key);
            }
        }
        match self.result(store, key)? {
            ResultKind::Present(id) => Ok(Some(id)),
            ResultKind::Absent => Ok(None),
            ResultKind::Pending => Err("missing canonical lifetime lookup result"),
        }
    }
    pub(crate) fn application(
        &mut self,
        store: &Store,
        id: ApplicationId,
    ) -> Result<Option<ApplicationId>, &'static str> {
        if store
            .trait_properties(id)
            .ok_or("invalid lookup trait application")?
            .escaping
            == 0
        {
            return Ok(Some(id));
        }
        if let Some(result) = self.applications.get(&id) {
            return Ok(*result);
        }
        let original = store
            .trait_application(id)
            .ok_or("invalid lookup trait application")?;
        let arguments = store
            .arguments(original.arguments)
            .ok_or("invalid lookup arguments")?;
        self.bound.account_arguments(arguments.len())?;
        for argument in arguments {
            if let Argument::Type(ty) = argument {
                self.ty(store, ty)?;
            }
        }
        let result = rewrite::Folder::application(
            &mut Read {
                lookup: self,
                store,
            },
            id,
            0,
        );
        let found = match result {
            Ok(id) => Some(id),
            Err(Error::Absent) => None,
            Err(Error::Invalid(message)) => return Err(message),
        };
        self.applications.insert(id, found);
        Ok(found)
    }
}
struct Read<'a> {
    lookup: &'a mut Lookup,
    store: &'a Store,
}
impl rewrite::Folder for Read<'_> {
    type Error = Error;
    fn child(&mut self, ty: TypeId, depth: u16) -> Result<TypeId, Error> {
        match self.lookup.result(self.store, (ty, depth))? {
            ResultKind::Present(id) => Ok(id),
            ResultKind::Absent => Err(Error::Absent),
            ResultKind::Pending => Err(Error::Invalid("missing canonical lookup dependency")),
        }
    }
    fn region(&mut self, region: RegionId, depth: u16) -> Result<RegionId, Error> {
        if region
            .bound_parts()
            .is_some_and(|(binder, _)| binder >= depth && binder - depth < self.lookup.amount)
        {
            return Err(Error::Absent);
        }
        self.lookup.bound.region(region, depth).map_err(Into::into)
    }
    fn list(&mut self, list: ListId, depth: u16) -> Result<ListId, Error> {
        self.lookup.bound.children.clear();
        for &ty in self.store.list(list).ok_or("invalid lookup type list")? {
            let child = self.child(ty, depth)?;
            self.lookup.bound.children.push(child);
        }
        self.store
            .find_list(&self.lookup.bound.children)
            .ok_or(Error::Absent)
    }
    fn arguments(&mut self, args: GenericArgs, depth: u16) -> Result<GenericArgs, Error> {
        if let Some(arguments) = self.store.region_args(args) {
            self.lookup.bound.account_arguments(arguments.len())?;
            self.lookup.bound.regions.clear();
            for &region in arguments {
                let region = self.region(region, depth)?;
                self.lookup.bound.regions.push(region);
            }
            return self
                .store
                .find_regions(&self.lookup.bound.regions)
                .ok_or(Error::Absent);
        }
        let arguments = self
            .store
            .arguments(args)
            .ok_or("invalid lookup generic arguments")?;
        self.lookup.bound.account_arguments(arguments.len())?;
        self.lookup.bound.generic.clear();
        for argument in arguments {
            let argument = match argument {
                Argument::Type(ty) => Argument::Type(self.child(ty, depth)?),
                Argument::Lifetime(region) => Argument::Lifetime(self.region(region, depth)?),
                Argument::Const(id) => Argument::Const(id),
            };
            self.lookup.bound.generic.push(argument);
        }
        self.store
            .find_mixed_arguments(&self.lookup.bound.generic)
            .ok_or(Error::Absent)
    }
    fn application(&mut self, id: ApplicationId, depth: u16) -> Result<ApplicationId, Error> {
        let original = self
            .store
            .trait_application(id)
            .ok_or("invalid lookup trait")?;
        let arguments = self.arguments(original.arguments, depth)?;
        self.store
            .find_trait_application(crate::types::traits::Application {
                arguments,
                ..original
            })
            .ok_or(Error::Absent)
    }
    fn constant(&mut self, id: ConstId) -> Result<ConstId, Error> {
        Ok(id)
    }
    fn constraint(
        &mut self,
        constraint: crate::types::Constraint,
        base: TypeId,
    ) -> Result<crate::types::Constraint, Error> {
        let crate::types::Constraint::ScalarRange(id) = constraint else {
            return Ok(constraint);
        };
        let range = self
            .store
            .scalar_range(id)
            .ok_or("unknown scalar range lookup")?;
        self.lookup.bound.account_arguments(1)?;
        let attempted = std::cell::Cell::new(0_usize);
        let result = self.store.find_scalar_range(
            crate::types::ScalarRange { base, ..range },
            &|work, _| {
                attempted.set(attempted.get().saturating_add(work));
                Ok(())
            },
        );
        self.lookup.bound.account_arguments(attempted.get())?;
        let id = result?.ok_or(Error::Absent)?;
        Ok(crate::types::Constraint::ScalarRange(id))
    }
    fn bound_regions(&self, count: u32) -> u32 {
        count
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/captured_lookup.rs"]
mod tests;
