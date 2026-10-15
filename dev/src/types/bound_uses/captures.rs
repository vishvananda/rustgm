//! Memoized occurrence queries for a fixed range of surrounding callable binders.
use crate::types::{
    GenericArgs, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    parameters::predicates::{Bound, Predicate},
    traits::ApplicationId,
};
use std::collections::HashMap;

type Key = (TypeId, u16);

pub(crate) struct Captures {
    count: u16,
    completed: HashMap<Key, bool>,
    applications: HashMap<ApplicationId, bool>,
    work: Vec<(Key, bool)>,
    pub(crate) steps: usize,
}
impl Captures {
    pub(crate) fn new(count: u16) -> Self {
        Self {
            count,
            completed: HashMap::new(),
            applications: HashMap::new(),
            work: Vec::new(),
            steps: 0,
        }
    }
    pub(crate) fn predicate(
        &mut self,
        store: &Store,
        value: Predicate,
    ) -> Result<bool, &'static str> {
        if self.collect(store, value.ty)? {
            return Ok(true);
        }
        match value.bound {
            Bound::Outlives(region) => Ok(self.region(region, 0)),
            Bound::Trait(id) => {
                if let Some(&found) = self.applications.get(&id) {
                    return Ok(found);
                }
                let application = store
                    .trait_application(id)
                    .ok_or("invalid capture trait application")?;
                let mut found = false;
                let arguments = store
                    .arguments(application.arguments)
                    .ok_or("invalid capture trait arguments")?;
                self.account(arguments.len())?;
                for argument in arguments {
                    found |= match argument {
                        Argument::Lifetime(region) => self.region(region, 0),
                        Argument::Type(ty) => self.collect(store, ty)?,
                        Argument::Const(_) => false,
                    };
                }
                self.applications.insert(id, found);
                Ok(found)
            }
            Bound::Copy | Bound::Clone | Bound::Sized => Ok(false),
        }
    }
    pub(crate) fn collect(&mut self, store: &Store, ty: TypeId) -> Result<bool, &'static str> {
        let root = (ty, 0);
        if let Some(found) = self.result(store, root)? {
            return Ok(found);
        }
        self.work.push((root, false));
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
        self.result(store, root)?.ok_or("missing capture result")
    }
    fn result(&self, store: &Store, key: Key) -> Result<Option<bool>, &'static str> {
        if store
            .escaping_binders(key.0)
            .ok_or("invalid capture type")?
            <= key.1
        {
            return Ok(Some(false));
        }
        Ok(self.completed.get(&key).copied())
    }
    fn child(&mut self, store: &Store, key: Key, finish: bool) -> Result<bool, &'static str> {
        if let Some(found) = self.result(store, key)? {
            return Ok(found);
        }
        if finish {
            return Err("missing capture dependency");
        }
        self.account(1)?;
        self.work.push((key, false));
        Ok(false)
    }
    fn inspect(
        &mut self,
        store: &Store,
        (ty, depth): Key,
        finish: bool,
    ) -> Result<bool, &'static str> {
        let mut found = false;
        let list = match store.kind(ty).ok_or("invalid capture type")? {
            Kind::Reference {
                region, referent, ..
            } => {
                found =
                    self.region(region, depth) | self.child(store, (referent, depth), finish)?;
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
            Kind::Adt(_, arguments) => {
                found = self.arguments(store, arguments, depth, finish)?;
                None
            }
            Kind::FunctionItem {
                signature,
                arguments,
                ..
            } => {
                found = self.arguments(store, arguments, depth, finish)?
                    | self.child(store, (signature, depth), finish)?;
                None
            }
            Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => None,
        };
        if let Some((list, depth)) = list {
            let children = store.list(list).ok_or("invalid capture type list")?;
            self.account(children.len())?;
            for &child in children {
                found |= self.child(store, (child, depth), finish)?;
            }
        }
        Ok(found)
    }
    fn arguments(
        &mut self,
        store: &Store,
        args: GenericArgs,
        depth: u16,
        finish: bool,
    ) -> Result<bool, &'static str> {
        let arguments = store.arguments(args).ok_or("invalid capture arguments")?;
        self.account(arguments.len())?;
        let mut found = false;
        for argument in arguments {
            found |= match argument {
                Argument::Lifetime(region) => self.region(region, depth),
                Argument::Type(ty) => self.child(store, (ty, depth), finish)?,
                Argument::Const(_) => false,
            };
        }
        Ok(found)
    }
    fn region(&self, region: RegionId, depth: u16) -> bool {
        region
            .bound_parts()
            .is_some_and(|(binder, _)| binder >= depth && binder - depth < self.count)
    }
    fn account(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err("bound capture work budget exceeded");
        }
        Ok(())
    }
}
