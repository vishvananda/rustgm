//! Conditional lifetime promises on opaque results, never on their inputs.
use super::{Application, Store};
use crate::types::{ItemId, RegionId, arguments::Argument};
use std::collections::hash_map::Entry;

impl Store {
    /// Publish the completed declaration contract once. Unlike canonical shape
    /// flags, this metadata is consumed only by scoped lifetime proof queries.
    pub(crate) fn declare_projection_outlives(
        &mut self,
        item: ItemId,
        regions: &[RegionId],
    ) -> Result<(), &'static str> {
        if regions.is_empty() {
            return Ok(());
        }
        let storage = self
            .arguments
            .as_mut()
            .and_then(|arguments| arguments.projections.as_mut())
            .ok_or("missing associated lifetime declarations")?;
        if !storage.schemas.contains_key(&item) {
            return Err("missing associated lifetime schema");
        }
        match storage
            .outlives
            .get_or_insert_with(Default::default)
            .entry(item)
        {
            Entry::Vacant(entry) => {
                entry.insert(regions.into());
                Ok(())
            }
            Entry::Occupied(entry) if entry.get().as_ref() == regions => Ok(()),
            Entry::Occupied(_) => Err("conflicting associated lifetime promises"),
        }
    }

    /// The frontend separately proves every parent application. This query only
    /// maps its result's declared region promises, without inferring input bounds.
    pub(crate) fn projection_outlives(
        &self,
        application: Application,
        depth: u16,
    ) -> Result<impl Iterator<Item = Result<Option<RegionId>, &'static str>> + '_, &'static str>
    {
        let arguments = self
            .trait_application(application.trait_application)
            .ok_or("missing associated lifetime application")?
            .arguments;
        let regions = self
            .arguments
            .as_ref()
            .and_then(|arguments| arguments.projections.as_ref())
            .and_then(|storage| storage.outlives.as_ref())
            .and_then(|outlives| outlives.get(&application.item))
            .map_or(&[][..], AsRef::as_ref);
        Ok(regions.iter().map(move |&region| {
            if region == RegionId::STATIC {
                return Ok(Some(region));
            }
            let Some((0, index)) = region.bound_parts() else {
                return Err("invalid associated lifetime template");
            };
            let argument = self
                .arguments(arguments)
                .ok_or("missing associated lifetime arguments")?
                .nth(usize::from(index));
            let Some(Argument::Lifetime(region)) = argument else {
                return Err("associated lifetime argument kind mismatch");
            };
            match region.bound_parts() {
                Some((binder, index)) if binder >= depth => {
                    RegionId::bound(binder - depth, index).map(Some)
                }
                Some(_) => Ok(None),
                None => Ok(Some(region)),
            }
        }))
    }
}
