//! Object constraints preserve invariant trait inputs and a separate outer lifetime.
mod requirements;
use super::{Constructor, Context, Error, Kind, RegionId, Store, TermId};
use std::collections::HashMap;

/// Context-owned metadata keeps the common inference term compact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimesId(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Lifetimes {
    pub bound_regions: u32,
    pub region: RegionId,
}

#[derive(Default)]
pub(super) struct Storage {
    values: Vec<Lifetimes>,
    index: HashMap<Lifetimes, LifetimesId>,
}

impl Context {
    pub(super) fn object_lifetimes(&self, id: LifetimesId) -> Result<Lifetimes, Error> {
        self.arguments
            .as_ref()
            .and_then(|arguments| arguments.objects.as_ref())
            .and_then(|objects| objects.values.get(id.0 as usize))
            .copied()
            .ok_or(Error::InvalidIdentity)
    }

    pub(super) fn intern_object_lifetimes(
        &mut self,
        bound_regions: u32,
        region: RegionId,
    ) -> Result<LifetimesId, Error> {
        let objects = self
            .arguments
            .get_or_insert_with(Default::default)
            .objects
            .get_or_insert_with(Default::default);
        let value = Lifetimes {
            bound_regions,
            region,
        };
        if let Some(&id) = objects.index.get(&value) {
            return Ok(id);
        }
        let id = LifetimesId(u32::try_from(objects.values.len()).map_err(|_| Error::Overflow)?);
        objects.values.push(value);
        objects.index.insert(value, id);
        Ok(id)
    }

    pub(super) fn binder_regions(&self, kind: Constructor) -> Result<Option<u32>, Error> {
        Ok(match kind {
            Constructor::Function { bound_regions, .. } => Some(bound_regions),
            Constructor::Dynamic { lifetimes, .. } => {
                Some(self.object_lifetimes(lifetimes)?.bound_regions)
            }
            _ => None,
        })
    }

    pub(super) fn object_equation(
        &mut self,
        left: Constructor,
        right: Constructor,
        record_regions: bool,
    ) -> Result<(), Error> {
        let (
            Constructor::Dynamic {
                shape: a,
                arguments: aa,
                lifetimes: al,
                ..
            },
            Constructor::Dynamic {
                shape: b,
                arguments: ba,
                lifetimes: bl,
                ..
            },
        ) = (left, right)
        else {
            return Err(Error::InvalidIdentity);
        };
        if a != b {
            return Err(Error::Mismatch);
        }
        self.equate_arguments(aa, ba, record_regions)?;
        let ar = self.object_lifetimes(al)?.region;
        let br = self.object_lifetimes(bl)?.region;
        if record_regions && ar != br {
            self.region_equalities.push((ar, br));
        }
        Ok(())
    }

    pub(super) fn object_constructor(
        &mut self,
        store: &Store,
        kind: Kind,
        children: &mut Vec<TermId>,
    ) -> Result<Constructor, Error> {
        let Kind::Dynamic {
            shape,
            arguments,
            bound_regions,
            region,
        } = kind
        else {
            return Err(Error::InvalidIdentity);
        };
        Ok(Constructor::Dynamic {
            shape,
            arguments: self.materialize_arguments(store, arguments, children)?,
            lifetimes: self.intern_object_lifetimes(bound_regions, region)?,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn object_metadata_does_not_grow_common_terms() {
        assert_eq!(size_of::<super::Constructor>(), 16);
        assert_eq!(size_of::<super::super::Term>(), 32);
    }
}
