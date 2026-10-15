//! Associated type identity and complete inputs, separate from normalization evidence.
use super::{GenericArgs, ItemId, Properties, Property, Store, TypeId, traits::ApplicationId};
use std::collections::{HashMap, hash_map::Entry};

/// The declaring trait and result sizedness cannot change across applications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Schema {
    pub trait_item: ItemId,
    pub sized: bool,
}

/// Self, the trait's explicit arguments and the family's arguments remain separate.
/// Different applications may normalize to the same type; no reverse inference is valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Application {
    pub item: ItemId,
    pub subject: TypeId,
    pub trait_application: ApplicationId,
    pub arguments: GenericArgs,
}

#[derive(Default)]
pub(super) struct Storage {
    schemas: HashMap<ItemId, Schema>,
}

impl Store {
    pub(super) fn projection_properties(
        &self,
        application: Application,
    ) -> Result<Properties, &'static str> {
        let mut result = self
            .properties
            .get(application.subject.0 as usize)
            .copied()
            .ok_or("invalid projection subject")?;
        result.include(
            self.trait_properties(application.trait_application)
                .ok_or("invalid projection trait application")?,
        );
        self.argument_properties(application.arguments, &mut result)?;
        let schema = self
            .projection_schema(application.item)
            .ok_or("missing associated type declaration")?;
        result.set(Property::Sized, schema.sized);
        result.set(Property::Projections, true);
        Ok(result)
    }
    #[must_use]
    pub fn projection_schema(&self, item: ItemId) -> Option<Schema> {
        self.arguments
            .as_ref()?
            .projections
            .as_ref()?
            .schemas
            .get(&item)
            .copied()
    }

    /// Register one immutable associated declaration before interning its applications.
    /// # Errors
    /// Rejects a conflicting trait owner or result sizedness.
    pub fn declare_projection(&mut self, item: ItemId, schema: Schema) -> Result<(), &'static str> {
        let storage = self
            .arguments
            .get_or_insert_with(Default::default)
            .projections
            .get_or_insert_with(Default::default);
        match storage.schemas.entry(item) {
            Entry::Vacant(entry) => {
                entry.insert(schema);
                Ok(())
            }
            Entry::Occupied(entry) if *entry.get() == schema => Ok(()),
            Entry::Occupied(_) => Err("conflicting associated type declaration"),
        }
    }

    pub(super) fn valid_projection(&self, application: Application) -> bool {
        self.projection_schema(application.item)
            .is_some_and(|schema| {
                self.trait_application(application.trait_application)
                    .is_some_and(|trait_| trait_.item == schema.trait_item)
            })
            && self.kind(application.subject).is_some()
            && self.arguments(application.arguments).is_some()
    }
}
