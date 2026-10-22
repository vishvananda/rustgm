//! Lifetime-erased fingerprints only select candidates; complete input matching proves identity.
use crate::support::hash::FxHasher;
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, Kind, ListId, Store, TypeId, arguments::Argument, region_graph::Error,
    traits::ApplicationId,
};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Type(TypeId),
    List(ListId),
    Arguments(GenericArgs),
    Trait(ApplicationId),
}
#[derive(Clone, Default)]
pub(crate) struct Shapes {
    completed: HashMap<Key, u64>,
    work: Vec<(Key, bool)>,
    pub steps: usize,
}
impl Shapes {
    pub(crate) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.completed);
        total.vector(&self.work);
        total
    }
}
impl Shapes {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(count);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err(Error::Budget);
        }
        Ok(())
    }
    pub fn get(&mut self, types: &Store, ty: TypeId) -> Result<u64, Error> {
        let root = Key::Type(ty);
        if let Some(&found) = self.completed.get(&root) {
            return Ok(found);
        }
        self.work.push((root, false));
        while let Some((key, finish)) = self.work.pop() {
            self.charge(1)?;
            if self.completed.contains_key(&key) {
                continue;
            }
            if !finish {
                self.work.push((key, true));
            }
            let hash = self.inspect(types, key, finish)?;
            if finish {
                self.completed.insert(key, hash);
            }
        }
        self.completed.get(&root).copied().ok_or(Error::Identity)
    }
    fn child(&mut self, key: Key, finish: bool, hash: &mut FxHasher) -> Result<(), Error> {
        self.charge(1)?;
        if let Some(value) = self.completed.get(&key) {
            value.hash(hash);
        } else if finish {
            return Err(Error::Identity);
        } else {
            self.work.push((key, false));
        }
        Ok(())
    }
    fn inspect(&mut self, types: &Store, key: Key, finish: bool) -> Result<u64, Error> {
        let mut hash = FxHasher::default();
        std::mem::discriminant(&key).hash(&mut hash);
        match key {
            Key::Type(ty) => self.ty(types.kind(ty).ok_or(Error::Identity)?, finish, &mut hash)?,
            Key::List(list) => {
                let values = types.list(list).ok_or(Error::Identity)?;
                self.charge(values.len())?;
                values.len().hash(&mut hash);
                for &ty in values {
                    self.child(Key::Type(ty), finish, &mut hash)?;
                }
            }
            Key::Arguments(arguments) => {
                let values = types.arguments(arguments).ok_or(Error::Identity)?;
                self.charge(values.len())?;
                values.len().hash(&mut hash);
                for value in values {
                    std::mem::discriminant(&value).hash(&mut hash);
                    match value {
                        Argument::Lifetime(_) => {}
                        Argument::Const(value) => value.hash(&mut hash),
                        Argument::Type(ty) => self.child(Key::Type(ty), finish, &mut hash)?,
                    }
                }
            }
            Key::Trait(id) => {
                let application = types.trait_application(id).ok_or(Error::Identity)?;
                application.item.hash(&mut hash);
                self.child(Key::Arguments(application.arguments), finish, &mut hash)?;
            }
        }
        Ok(hash.finish())
    }
    fn ty(&mut self, kind: Kind, finish: bool, hash: &mut FxHasher) -> Result<(), Error> {
        std::mem::discriminant(&kind).hash(hash);
        match kind {
            Kind::Closure {
                identity,
                arguments,
                signature,
                environment,
            } => {
                identity.hash(hash);
                self.child(Key::Arguments(arguments), finish, hash)?;
                self.child(Key::Type(signature), finish, hash)?;
                self.child(Key::Type(environment), finish, hash)?;
            }
            Kind::Never => {}
            Kind::Primitive(value) => value.hash(hash),
            Kind::Parameter(value) => value.hash(hash),
            Kind::Reference {
                mutable, referent, ..
            } => {
                mutable.hash(hash);
                self.child(Key::Type(referent), finish, hash)?;
            }
            Kind::Pointer { mutable, pointee } => {
                mutable.hash(hash);
                self.child(Key::Type(pointee), finish, hash)?;
            }
            Kind::Slice(element) => self.child(Key::Type(element), finish, hash)?,
            Kind::Array { element, length } => {
                length.hash(hash);
                self.child(Key::Type(element), finish, hash)?;
            }
            Kind::ArrayTerm { element, length } => {
                length.hash(hash);
                self.child(Key::Type(element), finish, hash)?;
            }
            Kind::Tuple(list) => self.child(Key::List(list), finish, hash)?,
            Kind::Opaque(id, arguments) => {
                id.hash(hash);
                self.child(Key::Arguments(arguments), finish, hash)?;
            }
            Kind::Dynamic {
                shape,
                arguments,
                bound_regions,
                ..
            } => {
                shape.hash(hash);
                bound_regions.hash(hash);
                self.child(Key::Arguments(arguments), finish, hash)?;
            }
            Kind::Adt(item, arguments) => {
                item.hash(hash);
                self.child(Key::Arguments(arguments), finish, hash)?;
            }
            Kind::Function {
                bound_regions,
                parameters,
                result,
                unsafe_,
                convention,
            } => {
                bound_regions.hash(hash);
                unsafe_.hash(hash);
                convention.hash(hash);
                self.child(Key::List(parameters), finish, hash)?;
                self.child(Key::Type(result), finish, hash)?;
            }
            Kind::FunctionItem {
                item,
                arguments,
                signature,
            } => {
                item.hash(hash);
                self.child(Key::Arguments(arguments), finish, hash)?;
                self.child(Key::Type(signature), finish, hash)?;
            }
            Kind::Projection(application) => {
                application.item.hash(hash);
                self.child(Key::Type(application.subject), finish, hash)?;
                self.child(Key::Trait(application.trait_application), finish, hash)?;
                self.child(Key::Arguments(application.arguments), finish, hash)?;
            }
        }
        Ok(())
    }
}
