//! Canonical trait applications preserve declaration identity and all explicit arguments.
use super::{GenericArgs, ItemId, Properties, Store};
use crate::support::hash::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ApplicationId(pub u32);

/// Self is the predicate subject, separate from the trait's written parameters.
/// Declaration checking supplies the trait identity and validates parameter kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Application {
    pub item: ItemId,
    pub arguments: GenericArgs,
}

#[derive(Default)]
pub(super) struct Storage {
    applications: Vec<Application>,
    properties: Vec<Properties>,
    index: HashMap<Application, ApplicationId>,
}

impl Store {
    pub(super) fn trait_tables_bytes(&self) -> usize {
        self.arguments
            .as_ref()
            .and_then(|storage| storage.traits.as_deref())
            .map_or(0, |storage| {
                size_of::<Storage>()
                    .saturating_add(crate::support::retained::vector(&storage.applications))
                    .saturating_add(crate::support::retained::vector(&storage.properties))
                    .saturating_add(crate::support::retained::map(&storage.index))
            })
    }
    /// Bound a known miss's real append/copy/rehash costs without interning or proof.
    pub(crate) fn trait_application_growth(&self) -> crate::support::retained::Growth {
        use crate::support::retained::Growth;
        let storage = self
            .arguments
            .as_ref()
            .and_then(|storage| storage.traits.as_deref());
        let mut growth = Growth::default();
        if let Some(storage) = storage {
            growth.include(Growth::vector::<Application>(
                storage.applications.len(),
                storage.applications.capacity(),
                1,
            ));
            growth.include(Growth::vector::<Properties>(
                storage.properties.len(),
                storage.properties.capacity(),
                1,
            ));
            growth.include(Growth::map::<Application, ApplicationId>(
                storage.index.len(),
                storage.index.capacity(),
                1,
            ));
        } else {
            growth.bytes = size_of::<Storage>().saturating_add(if self.arguments.is_none() {
                size_of::<super::arguments::Storage>()
            } else {
                0
            });
            growth.include(Growth::vector::<Application>(0, 0, 1));
            growth.include(Growth::vector::<Properties>(0, 0, 1));
            growth.include(Growth::map::<Application, ApplicationId>(0, 0, 1));
        }
        growth
    }
    pub(super) fn trait_application_count(&self) -> usize {
        self.arguments
            .as_ref()
            .and_then(|storage| storage.traits.as_ref())
            .map_or(0, |storage| storage.applications.len())
    }
    pub(super) fn refresh_trait_properties(
        &mut self,
        id: ApplicationId,
    ) -> Result<(), &'static str> {
        let application = self
            .trait_application(id)
            .ok_or("invalid trait application")?;
        let mut properties = Properties::default();
        self.argument_properties(application.arguments, &mut properties)?;
        let storage = self
            .arguments
            .as_mut()
            .and_then(|storage| storage.traits.as_mut())
            .ok_or("missing trait application storage")?;
        storage.properties[id.0 as usize] = properties;
        Ok(())
    }
    pub(crate) fn find_trait_application(&self, application: Application) -> Option<ApplicationId> {
        self.arguments
            .as_ref()?
            .traits
            .as_ref()?
            .index
            .get(&application)
            .copied()
    }
    pub(super) fn trait_properties(&self, id: ApplicationId) -> Option<Properties> {
        self.arguments
            .as_ref()?
            .traits
            .as_ref()?
            .properties
            .get(id.0 as usize)
            .copied()
    }
    #[must_use]
    pub fn trait_application(&self, id: ApplicationId) -> Option<Application> {
        self.arguments
            .as_ref()?
            .traits
            .as_ref()?
            .applications
            .get(id.0 as usize)
            .copied()
    }

    /// Repeated complete applications share identity; different trait declarations
    /// never become equal merely because their parameter lists are equal.
    /// # Errors
    /// Rejects invalid argument identities and compact arena overflow.
    pub fn intern_trait_application(
        &mut self,
        application: Application,
    ) -> Result<ApplicationId, &'static str> {
        if self.arguments(application.arguments).is_none() {
            return Err("invalid trait application arguments");
        }
        if let Some(&id) = self
            .arguments
            .as_ref()
            .and_then(|s| s.traits.as_ref())
            .and_then(|s| s.index.get(&application))
        {
            return Ok(id);
        }
        let mut properties = Properties::default();
        self.argument_properties(application.arguments, &mut properties)?;
        let storage = self
            .arguments
            .get_or_insert_with(Default::default)
            .traits
            .get_or_insert_with(Default::default);
        let id = ApplicationId(
            u32::try_from(storage.applications.len())
                .map_err(|_| "too many canonical trait applications")?,
        );
        storage.applications.push(application);
        storage.properties.push(properties);
        storage.index.insert(application, id);
        Ok(id)
    }
}
