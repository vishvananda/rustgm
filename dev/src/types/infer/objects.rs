//! Object constraints preserve invariant trait inputs and a separate outer lifetime.
mod requirements;
use super::{Constructor, Context, Error, Kind, RegionId, Store, TermId};
use std::collections::HashMap;

/// Context-owned metadata keeps the common inference term compact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifetimesId(u32);

/// One associated equality retains the object's complete quantified inputs.
pub struct Binding {
    pub bound_regions: u32,
    pub arguments: Vec<super::arguments::Argument>,
    pub result: TermId,
}

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
    /// Inspect a written object equality without proving its concrete implementation.
    /// # Errors
    /// Rejects malformed object terms, bindings and exhausted inference work.
    pub fn object_binding(
        &mut self,
        store: &Store,
        target: TermId,
        member: super::ItemId,
    ) -> Result<Option<Binding>, Error> {
        let target = self.root(target)?;
        let Constructor::Dynamic {
            shape,
            arguments,
            lifetimes,
        } = self.materialize(store, target)?
        else {
            return Err(Error::InvalidIdentity);
        };
        let bound_regions = self.object_lifetimes(lifetimes)?.bound_regions;
        let value = store.dynamic_shape(shape).ok_or(Error::InvalidIdentity)?;
        let bindings = store
            .dynamic_bindings(shape)
            .ok_or(Error::InvalidIdentity)?;
        self.account_work(bindings.len().saturating_add(1))?;
        let arguments = self.application_arguments(target, arguments, 0)?;
        let mut start = value.principal_arguments as usize;
        for binding in bindings {
            let end = start
                .checked_add(binding.arguments as usize)
                .ok_or(Error::Overflow)?;
            if binding.item == member {
                let Some(&super::arguments::Argument::Type(result)) = arguments.get(end) else {
                    return Err(Error::InvalidIdentity);
                };
                return Ok(Some(Binding {
                    bound_regions,
                    arguments: arguments
                        .get(start..end)
                        .ok_or(Error::InvalidIdentity)?
                        .to_vec(),
                    result,
                }));
            }
            start = end.checked_add(1).ok_or(Error::Overflow)?;
        }
        Ok(None)
    }

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

    pub(crate) fn binder_regions(&self, kind: Constructor) -> Result<Option<u32>, Error> {
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
