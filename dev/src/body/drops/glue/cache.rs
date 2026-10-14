//! Recipe state survives append-only type and instance discovery without retaining borrows.
use super::{Catalog, Recipes};
use crate::{
    body::drops::Method,
    resolve::model::DefId,
    sema::members::Members,
    types::{ItemId, Store, TypeId, aggregate::Definitions, destruction::NeedsDrop},
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Nominal(ItemId),
    Structural(TypeId),
}
#[derive(Clone, Copy)]
pub(super) enum State {
    Active,
    Ready(Option<super::Id>),
    Failed(&'static str),
}
/// One canonical store and append-only completed definitions, with a fixed set
/// of destructor declarations. Reattaching does not redo completed queries.
pub struct Cache {
    pub(super) methods: HashMap<ItemId, DefId>,
    pub(super) needs: NeedsDrop,
    pub(super) states: HashMap<Key, State>,
    pub(super) storage: Recipes,
    pub(super) work: Vec<(TypeId, bool)>,
    retained: usize,
    pub(super) steps: usize,
}
impl Cache {
    /// # Errors
    /// Rejects duplicate declarations and oversized method tables.
    pub fn new(methods: &[Method]) -> Result<Self, &'static str> {
        let retained = methods
            .len()
            .checked_mul(2 * (size_of::<(ItemId, DefId)>() + size_of::<ItemId>()))
            .filter(|n| *n <= 64_000_000)
            .ok_or("destruction methods exceed 64 MB budget")?;
        let mut selected = HashMap::new();
        for method in methods {
            if selected.insert(method.item, method.function).is_some() {
                return Err("duplicate destructor recipe method");
            }
        }
        let needs = NeedsDrop::new(selected.keys().copied());
        Ok(Self {
            methods: selected,
            needs,
            states: HashMap::new(),
            storage: Recipes::default(),
            work: Vec::new(),
            retained,
            steps: 0,
        })
    }
    /// Reborrow the same semantic environment after new types/fields are published.
    #[must_use]
    pub fn attach<'a>(
        self,
        types: &'a Store,
        definitions: &'a Definitions,
        members: &'a Members,
    ) -> Catalog<'a> {
        Catalog {
            types,
            definitions,
            members,
            cache: self,
        }
    }
    #[must_use]
    pub fn into_recipes(self) -> Recipes {
        self.storage
    }
    pub(crate) const fn recipes(&self) -> &Recipes {
        &self.storage
    }
    pub(crate) fn resolve(
        &mut self,
        types: &mut Store,
        id: super::Id,
        instance: crate::types::instance::Instance,
    ) -> Result<(), &'static str> {
        let recipe = self
            .storage
            .entries
            .get_mut(id.0 as usize)
            .ok_or("invalid destructor selection recipe")?;
        let Some(super::Method::Unresolved(method)) = recipe.method else {
            return if recipe.method == Some(super::Method::Instance(instance)) {
                Ok(())
            } else {
                Err("destructor recipe is not awaiting this instance")
            };
        };
        crate::types::destruction::validate_method(types, instance, ItemId(method.0), recipe.ty)?;
        recipe.method = Some(super::Method::Instance(instance));
        Ok(())
    }
    pub(super) fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|n| *n <= 1_000_000)
            .ok_or("destruction recipe work budget exceeded")?;
        Ok(())
    }
    pub(super) fn retain<T>(&mut self, count: usize) -> Result<(), &'static str> {
        self.retained = count
            .checked_mul(size_of::<T>())
            .and_then(|n| self.retained.checked_add(n))
            .filter(|n| *n <= 64_000_000)
            .ok_or("destruction recipes exceed 64 MB budget")?;
        Ok(())
    }
}
