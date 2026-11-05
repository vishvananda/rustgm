//! One paid sparse original index belongs to the complete source body family.
use super::SourceRows;
use crate::support::crates::generics::Work;
use crate::support::crates::{MAX_BYTES, owner_catalog::BodyRef};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Lookup {
    originals: Box<[u32]>,
}
impl Lookup {
    pub(super) fn build(
        rows: &SourceRows,
        work: &mut Work,
    ) -> Result<Option<std::sync::Arc<Self>>, &'static str> {
        if rows.compiler_operations.is_empty() {
            return Ok(None);
        }
        let count = rows.body_owners.len();
        work.charge(count.saturating_add(rows.originals.len()))?;
        if count.saturating_mul(size_of::<u32>()) > MAX_BYTES {
            return Err("source body index exceeds storage budget");
        }
        let mut originals = vec![u32::MAX; count];
        for (index, original) in rows.originals.iter().enumerate() {
            let slot = originals
                .get_mut(original.body.0 as usize)
                .ok_or("source original has an unknown body owner")?;
            if *slot != u32::MAX {
                return Err("source original repeats its body owner");
            }
            *slot = u32::try_from(index).map_err(|_| "source original index overflow")?;
        }
        Ok(Some(std::sync::Arc::new(Self {
            originals: originals.into_boxed_slice(),
        })))
    }
    pub(crate) fn original(&self, body: BodyRef) -> Option<usize> {
        let &index = self.originals.get(body.0 as usize)?;
        (index != u32::MAX).then_some(index as usize)
    }
    pub(crate) fn bytes(&self) -> usize {
        (size_of::<Self>() + 2 * size_of::<usize>())
            .saturating_add(self.originals.len().saturating_mul(size_of::<u32>()))
    }
}
pub(crate) fn original(index: Option<&Lookup>, body: BodyRef) -> Result<usize, &'static str> {
    index.map_or(Ok(body.0 as usize), |index| {
        index
            .original(body)
            .ok_or("compiler operation has no ordinary original code")
    })
}

pub(crate) type Shared = Option<std::sync::Arc<Lookup>>;
