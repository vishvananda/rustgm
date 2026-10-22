//! Lifetime guarantees depend on the principal declaration, not object spelling.
#[cfg(test)]
#[path = "../../../tests/invariants/erased_promises.rs"]
mod tests;
use super::{ItemId, Store};
use crate::support::hash::HashMap;
use crate::types::{GenericArgs, Kind, RegionId, TypeId, arguments::Argument};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Promise {
    Static,
    Argument(usize),
}
impl Store {
    pub(crate) fn dynamic_outlives_declared(&self, item: ItemId) -> bool {
        self.dynamics
            .as_ref()
            .is_some_and(|s| s.outlives.contains_key(&item))
    }
    pub(crate) fn declare_dynamic_outlives(
        &mut self,
        item: ItemId,
        arguments: GenericArgs,
        regions: &[RegionId],
    ) -> Result<(), &'static str> {
        let slots: HashMap<_, _> = self
            .arguments(arguments)
            .ok_or("invalid object lifetime template")?
            .enumerate()
            .filter_map(|(index, arg)| match arg {
                Argument::Lifetime(region) => Some((region, index)),
                _ => None,
            })
            .collect();
        let promises = regions
            .iter()
            .map(|&region| {
                if region == RegionId::STATIC {
                    Ok(Promise::Static)
                } else {
                    slots
                        .get(&region)
                        .copied()
                        .map(Promise::Argument)
                        .ok_or("object guarantee names a non-input lifetime")
                }
            })
            .collect::<Result<Box<[_]>, _>>()?;
        let storage = self.dynamics.get_or_insert_with(Default::default);
        match storage.outlives.entry(item) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(promises);
            }
            std::collections::hash_map::Entry::Occupied(entry) if *entry.get() == promises => {}
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err("conflicting object lifetime guarantees");
            }
        }
        Ok(())
    }
    pub(crate) fn dynamic_requirements(
        &self,
        ty: TypeId,
        depth: u16,
    ) -> Result<
        impl Iterator<Item = Result<Option<crate::types::outlives::Pair>, &'static str>> + '_,
        &'static str,
    > {
        let Some(Kind::Dynamic {
            shape,
            arguments,
            region,
            ..
        }) = self.kind(ty)
        else {
            return Err("object lifetime query requires a dynamic type");
        };
        let shape = self
            .dynamic_shape(shape)
            .ok_or("invalid object lifetime shape")?;
        let promises = self
            .dynamics
            .as_ref()
            .and_then(|s| shape.principal.and_then(|item| s.outlives.get(&item)))
            .map_or(&[][..], AsRef::as_ref);
        let nested = depth
            .checked_add(1)
            .ok_or("object lifetime depth overflow")?;
        let longer = crate::types::outlives::free(region, depth)?;
        Ok(promises.iter().map(move |&promise| {
            let shorter = match promise {
                Promise::Static => Some(RegionId::STATIC),
                Promise::Argument(index) => {
                    let Some(Argument::Lifetime(region)) =
                        self.arguments(arguments).and_then(|mut a| a.nth(index))
                    else {
                        return Err("object lifetime input kind mismatch");
                    };
                    crate::types::outlives::free(region, nested)?
                }
            };
            Ok(longer.zip(shorter))
        }))
    }
}
