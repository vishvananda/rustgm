//! Lifetime-erased fingerprints only select candidates; complete input matching proves identity.
use crate::support::hash::FxHasher;
use crate::support::hash::HashMap;
use crate::support::retained::Growth;
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
    /// An acyclic scalar/parameter/function graph has one type key per node
    /// and at most one list key per function. Pending edges include finish
    /// markers and each explicit list/result edge.
    pub(crate) fn callable_allocation_bound(nodes: usize, edges: usize) -> usize {
        let keys = nodes.saturating_mul(2);
        crate::support::retained::new_map::<Key, u64>(keys).saturating_add(
            keys.saturating_mul(2)
                .saturating_add(edges)
                .saturating_mul(4 * size_of::<(Key, bool)>()),
        )
    }
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
        self.get_observed::<false, Error>(types, ty, &|_, _| Ok(()))
    }
    pub(crate) fn get_charged(
        &mut self,
        types: &Store,
        ty: TypeId,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<u64, &'static str> {
        self.get_observed::<true, ChargedError>(types, ty, &|work, bytes| {
            charge(work, bytes).map_err(ChargedError::Retention)
        })
        .map_err(|error| match error {
            ChargedError::Semantic(Error::Budget) => "structural fingerprint work budget exceeded",
            ChargedError::Semantic(_) => "invalid structural fingerprint identity",
            ChargedError::Retention(message) => message,
        })
    }
    fn get_observed<const CHARGED: bool, E: From<Error>>(
        &mut self,
        types: &Store,
        ty: TypeId,
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<u64, E> {
        if CHARGED {
            charge(1, 0)?;
        }
        let root = Key::Type(ty);
        if let Some(&found) = self.completed.get(&root) {
            return Ok(found);
        }
        self.schedule::<CHARGED, E>(root, false, charge)?;
        while let Some((key, finish)) = self.work.pop() {
            self.account::<CHARGED, E>(1, charge)?;
            if self.completed.contains_key(&key) {
                continue;
            }
            if !finish {
                self.schedule::<CHARGED, E>(key, true, charge)?;
            }
            let hash = self.inspect::<CHARGED, E>(types, key, finish, charge)?;
            if finish {
                if CHARGED {
                    let growth =
                        Growth::map::<Key, u64>(self.completed.len(), self.completed.capacity(), 1);
                    charge(growth.work, growth.bytes)?;
                }
                self.completed.insert(key, hash);
            }
        }
        self.completed
            .get(&root)
            .copied()
            .ok_or_else(|| Error::Identity.into())
    }
    fn child<const CHARGED: bool, E: From<Error>>(
        &mut self,
        key: Key,
        finish: bool,
        hash: &mut FxHasher,
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<(), E> {
        self.account::<CHARGED, E>(1, charge)?;
        if let Some(value) = self.completed.get(&key) {
            value.hash(hash);
        } else if finish {
            return Err(Error::Identity.into());
        } else {
            self.schedule::<CHARGED, E>(key, false, charge)?;
        }
        Ok(())
    }
    fn inspect<const CHARGED: bool, E: From<Error>>(
        &mut self,
        types: &Store,
        key: Key,
        finish: bool,
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<u64, E> {
        let mut hash = FxHasher::default();
        std::mem::discriminant(&key).hash(&mut hash);
        match key {
            Key::Type(ty) => self.ty::<CHARGED, E>(
                types.kind(ty).ok_or(Error::Identity)?,
                finish,
                &mut hash,
                charge,
            )?,
            Key::List(list) => {
                let values = types.list(list).ok_or(Error::Identity)?;
                self.account::<CHARGED, E>(values.len(), charge)?;
                values.len().hash(&mut hash);
                for &ty in values {
                    self.child::<CHARGED, E>(Key::Type(ty), finish, &mut hash, charge)?;
                }
            }
            Key::Arguments(arguments) => {
                let values = types.arguments(arguments).ok_or(Error::Identity)?;
                self.account::<CHARGED, E>(values.len(), charge)?;
                values.len().hash(&mut hash);
                for value in values {
                    std::mem::discriminant(&value).hash(&mut hash);
                    match value {
                        Argument::Lifetime(_) => {}
                        Argument::Const(value) => value.hash(&mut hash),
                        Argument::Type(ty) => {
                            self.child::<CHARGED, E>(Key::Type(ty), finish, &mut hash, charge)?;
                        }
                    }
                }
            }
            Key::Trait(id) => {
                let application = types.trait_application(id).ok_or(Error::Identity)?;
                application.item.hash(&mut hash);
                self.child::<CHARGED, E>(
                    Key::Arguments(application.arguments),
                    finish,
                    &mut hash,
                    charge,
                )?;
            }
        }
        Ok(hash.finish())
    }
    fn ty<const CHARGED: bool, E: From<Error>>(
        &mut self,
        kind: Kind,
        finish: bool,
        hash: &mut FxHasher,
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<(), E> {
        std::mem::discriminant(&kind).hash(hash);
        match kind {
            kind @ (Kind::Closure { .. } | Kind::Function { .. } | Kind::FunctionItem { .. }) => {
                self.callable::<CHARGED, E>(kind, finish, hash, charge)?;
            }

            Kind::Never => {}
            Kind::Primitive(value) => value.hash(hash),
            Kind::Parameter(value) => value.hash(hash),
            Kind::Reference {
                mutable, referent, ..
            } => {
                mutable.hash(hash);
                self.child::<CHARGED, E>(Key::Type(referent), finish, hash, charge)?;
            }
            Kind::Pointer { mutable, pointee } => {
                mutable.hash(hash);
                self.child::<CHARGED, E>(Key::Type(pointee), finish, hash, charge)?;
            }
            Kind::Pattern { base, constraint } => {
                constraint.hash(hash);
                self.child::<CHARGED, E>(Key::Type(base), finish, hash, charge)?;
            }
            Kind::Slice(element) => {
                self.child::<CHARGED, E>(Key::Type(element), finish, hash, charge)?;
            }
            Kind::Array { element, length } => {
                length.hash(hash);
                self.child::<CHARGED, E>(Key::Type(element), finish, hash, charge)?;
            }
            Kind::ArrayTerm { element, length } => {
                length.hash(hash);
                self.child::<CHARGED, E>(Key::Type(element), finish, hash, charge)?;
            }
            Kind::Tuple(list) => self.child::<CHARGED, E>(Key::List(list), finish, hash, charge)?,
            Kind::Opaque(id, arguments) => {
                id.hash(hash);
                self.child::<CHARGED, E>(Key::Arguments(arguments), finish, hash, charge)?;
            }
            Kind::Dynamic {
                shape,
                arguments,
                bound_regions,
                ..
            } => {
                shape.hash(hash);
                bound_regions.hash(hash);
                self.child::<CHARGED, E>(Key::Arguments(arguments), finish, hash, charge)?;
            }
            Kind::Adt(item, arguments) => {
                item.hash(hash);
                self.child::<CHARGED, E>(Key::Arguments(arguments), finish, hash, charge)?;
            }

            Kind::Projection(application) => {
                application.item.hash(hash);
                self.child::<CHARGED, E>(Key::Type(application.subject), finish, hash, charge)?;
                self.child::<CHARGED, E>(
                    Key::Trait(application.trait_application),
                    finish,
                    hash,
                    charge,
                )?;
                self.child::<CHARGED, E>(
                    Key::Arguments(application.arguments),
                    finish,
                    hash,
                    charge,
                )?;
            }
        }
        Ok(())
    }
}

enum ChargedError {
    Semantic(Error),
    Retention(&'static str),
}
impl From<Error> for ChargedError {
    fn from(error: Error) -> Self {
        Self::Semantic(error)
    }
}
impl Shapes {
    fn schedule<const CHARGED: bool, E>(
        &mut self,
        key: Key,
        finish: bool,
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<(), E> {
        if CHARGED {
            let growth = Growth::vector::<(Key, bool)>(self.work.len(), self.work.capacity(), 1);
            charge(growth.work, growth.bytes)?;
        }
        self.work.push((key, finish));
        Ok(())
    }
    fn account<const CHARGED: bool, E: From<Error>>(
        &mut self,
        work: usize,
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<(), E> {
        if CHARGED {
            charge(work, 0)?;
        }
        self.charge(work).map_err(Into::into)
    }
}

impl Shapes {
    fn callable<const CHARGED: bool, E: From<Error>>(
        &mut self,
        kind: Kind,
        finish: bool,
        hash: &mut FxHasher,
        charge: &impl Fn(usize, usize) -> Result<(), E>,
    ) -> Result<(), E> {
        match kind {
            Kind::Closure {
                identity,
                arguments,
                signature,
                environment,
            } => {
                identity.hash(hash);
                self.child::<CHARGED, E>(Key::Arguments(arguments), finish, hash, charge)?;
                self.child::<CHARGED, E>(Key::Type(signature), finish, hash, charge)?;
                self.child::<CHARGED, E>(Key::Type(environment), finish, hash, charge)?;
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
                self.child::<CHARGED, E>(Key::List(parameters), finish, hash, charge)?;
                self.child::<CHARGED, E>(Key::Type(result), finish, hash, charge)?;
            }
            Kind::FunctionItem {
                item,
                arguments,
                signature,
            } => {
                item.hash(hash);
                self.child::<CHARGED, E>(Key::Arguments(arguments), finish, hash, charge)?;
                self.child::<CHARGED, E>(Key::Type(signature), finish, hash, charge)?;
            }
            _ => return Err(Error::Identity.into()),
        }
        Ok(())
    }
}
