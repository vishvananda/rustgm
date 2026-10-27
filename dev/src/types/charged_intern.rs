//! Observed producers prepay canonical growth through the same existing interners.
use super::{GenericArgs, Kind, ListId, RegionId, Store, TypeId, arguments::Argument, traits};
use crate::support::retained::Growth;
impl Store {
    pub(super) fn node_interning_growth(&self) -> Growth {
        let mut growth = Growth::vector::<Kind>(self.kinds.len(), self.kinds.capacity(), 1);
        growth.include(Growth::vector::<super::Properties>(
            self.properties.len(),
            self.properties.capacity(),
            1,
        ));
        growth.include(Growth::map::<Kind, TypeId>(
            self.index.len(),
            self.index.capacity(),
            1,
        ));
        growth
    }
    pub(crate) fn intern_list_observed<const CHARGED: bool>(
        &mut self,
        values: &[TypeId],
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<ListId, &'static str> {
        if CHARGED {
            if let Some(id) = self
                .children
                .find_charged(values, |work| charge(work.saturating_mul(2), 0))?
            {
                return Ok(ListId(id));
            }
            let growth = self.children.interning_growth(values.len());
            charge(values.len().saturating_add(growth.work), growth.bytes)?;
        }
        self.intern_list(values)
    }
    pub(crate) fn intern_regions_observed<const CHARGED: bool>(
        &mut self,
        values: &[RegionId],
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<GenericArgs, &'static str> {
        if CHARGED {
            if let Some(id) = self
                .regions
                .find_charged(values, |work| charge(work.saturating_mul(2), 0))?
            {
                return Ok(GenericArgs(id));
            }
            let growth = self.regions.interning_growth(values.len());
            charge(growth.work, growth.bytes)?;
        }
        self.intern_regions(values)
    }
    pub(crate) fn intern_arguments_observed<const CHARGED: bool>(
        &mut self,
        values: &[Argument],
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<GenericArgs, &'static str> {
        if CHARGED {
            charge(values.len().saturating_add(1), 0)?;
            if values
                .iter()
                .all(|arg| matches!(arg, Argument::Lifetime(_)))
            {
                let growth = Growth::vector::<RegionId>(0, 0, values.len());
                charge(values.len().saturating_add(growth.work), growth.bytes)?;
                let regions = values
                    .iter()
                    .filter_map(|arg| {
                        if let Argument::Lifetime(region) = arg {
                            Some(*region)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                return self.intern_regions_observed::<true>(&regions, charge);
            }
            if let Some(id) =
                self.find_mixed_arguments_charged(values, |work| charge(work.saturating_mul(2), 0))?
            {
                return Ok(id);
            }
            let growth = self.trait_arguments_growth(values.len());
            charge(values.len().saturating_add(growth.work), growth.bytes)?;
        }
        self.intern_arguments(values)
    }
    pub(crate) fn intern_trait_application_observed<const CHARGED: bool>(
        &mut self,
        application: traits::Application,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<traits::ApplicationId, &'static str> {
        if CHARGED {
            charge(1, 0)?;
            if let Some(id) = self.find_trait_application(application) {
                return Ok(id);
            }
            let count = self
                .arguments(application.arguments)
                .ok_or("invalid observed trait application arguments")?
                .len();
            let growth = self.trait_application_growth();
            charge(count.saturating_add(growth.work), growth.bytes)?;
        }
        self.intern_trait_application(application)
    }
}
