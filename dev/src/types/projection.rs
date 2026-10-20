//! Associated type identity and complete inputs, separate from normalization evidence.
mod outlives;
use super::{GenericArgs, ItemId, Properties, Property, Store, TypeId, traits::ApplicationId};
use std::collections::{HashMap, hash_map::Entry};

/// Uniform declaration promises, conditional on a proved complete application.
/// Kept separate from canonical properties when headers are typed after interning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Builtins {
    pub sized: bool,
    pub copies: super::copy::Properties,
    pub autos: u8,
}
impl Builtins {
    pub(crate) fn bound(bound: super::parameters::predicates::Bound) -> Self {
        use super::parameters::predicates::Bound;
        Self {
            autos: if let Bound::Auto(marker) = bound {
                1 << marker as u8
            } else {
                0
            },
            sized: matches!(bound, Bound::Sized | Bound::Copy | Bound::Clone),
            copies: super::copy::Properties {
                copy: bound == Bound::Copy,
                clone_: matches!(bound, Bound::Copy | Bound::Clone),
            },
        }
    }
    pub(crate) fn include(&mut self, other: Self) -> bool {
        let previous = *self;
        self.sized |= other.sized;
        self.copies.copy |= other.copies.copy;
        self.copies.clone_ |= other.copies.clone_;
        self.autos |= other.autos;
        *self != previous
    }
}

/// Immutable declaration promises, conditional on a well-formed application.
/// These facts do not prove parent formation or authorize normalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Schema {
    pub trait_item: ItemId,
    pub sized: bool,
    pub copies: super::copy::Properties,
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
    outlives: Option<HashMap<ItemId, Box<[outlives::Promise]>>>,
}

impl Store {
    /// Complete type and lifetime inputs without decomposing the projected result.
    pub(crate) fn projection_inputs(
        &self,
        application: Application,
    ) -> Result<impl Iterator<Item = super::arguments::Argument> + '_, &'static str> {
        let trait_ = self
            .trait_application(application.trait_application)
            .ok_or("invalid projection trait inputs")?;
        let trait_args = self
            .arguments(trait_.arguments)
            .ok_or("invalid projection trait arguments")?;
        let own_args = self
            .arguments(application.arguments)
            .ok_or("invalid projection family arguments")?;
        Ok(
            std::iter::once(super::arguments::Argument::Type(application.subject))
                .chain(trait_args)
                .chain(own_args),
        )
    }
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
    /// Rejects conflicting declaration facts or inconsistent builtin promises.
    pub fn declare_projection(&mut self, item: ItemId, schema: Schema) -> Result<(), &'static str> {
        if (schema.copies.copy && !schema.copies.clone_) || (schema.copies.clone_ && !schema.sized)
        {
            return Err("inconsistent associated type builtin promises");
        }
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
