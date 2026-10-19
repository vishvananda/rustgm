//! Conditional lifetime promises on opaque results, never on their inputs.
use super::{Application, Store};
use crate::types::{ItemId, RegionId, arguments::Argument};
use std::collections::hash_map::Entry;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Promise {
    Static,
    Parent(u16),
    Family(u16),
}

impl Promise {
    fn new(region: RegionId, captures: u32) -> Result<Self, &'static str> {
        if region == RegionId::STATIC {
            return Ok(Self::Static);
        }
        let Some((0, index)) = region.bound_parts() else {
            return Err("invalid associated lifetime template");
        };
        Ok(if u32::from(index) < captures {
            Self::Parent(index)
        } else {
            Self::Family(
                u16::try_from(u32::from(index) - captures)
                    .map_err(|_| "invalid family lifetime slot")?,
            )
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/family_outlives.rs"]
mod tests;

impl Store {
    /// Publish the completed declaration contract once. Unlike canonical shape
    /// flags, this metadata is consumed only by scoped lifetime proof queries.
    pub(crate) fn declare_projection_outlives(
        &mut self,
        item: ItemId,
        captures: u32,
        regions: &[RegionId],
    ) -> Result<(), &'static str> {
        if regions.is_empty() {
            return Ok(());
        }
        let promises: Box<[_]> = regions
            .iter()
            .map(|&region| Promise::new(region, captures))
            .collect::<Result<_, _>>()?;
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
                entry.insert(promises);
                Ok(())
            }
            Entry::Occupied(entry) if *entry.get() == promises => Ok(()),
            Entry::Occupied(_) => Err("conflicting associated lifetime promises"),
        }
    }

    /// The frontend separately proves every complete application. This query only
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
        Ok(regions.iter().map(move |&promise| {
            let (arguments, index) = match promise {
                Promise::Static => return Ok(Some(RegionId::STATIC)),
                Promise::Parent(index) => (arguments, index),
                Promise::Family(index) => (application.arguments, index),
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
