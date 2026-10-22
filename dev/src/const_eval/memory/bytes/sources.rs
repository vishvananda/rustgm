//! Typed reads borrow either the exclusive local backing or an immutable union image.
use super::{Failure, Machine, Reader};
use super::{continuations::Source, relocations::Fragment};
pub(super) struct Data<'a> {
    pub bytes: &'a [u8],
    initialized: &'a [u64],
}
impl Data<'_> {
    pub(super) fn initialized(&self, index: usize) -> bool {
        index < self.bytes.len()
            && self
                .initialized
                .get(index / 64)
                .is_some_and(|word| word & (1u64 << (index % 64)) != 0)
    }
}
impl Machine<'_, '_> {
    pub(super) fn byte_data(&self, source: Source) -> Result<Data<'_>, Failure> {
        match source {
            Source::Backing(id) => {
                let backing = self.byte_backing(id)?;
                Ok(Data {
                    bytes: &backing.bytes,
                    initialized: &backing.initialized,
                })
            }
            Source::Image(value) => {
                let image = Reader::new(self.evaluator.facts()).image(value)?;
                Ok(Data {
                    bytes: image.bytes,
                    initialized: image.initialized,
                })
            }
        }
    }
    pub(super) fn source_fragments(
        &mut self,
        source: Source,
        offset: u64,
        length: u64,
    ) -> Result<Vec<Fragment>, Failure> {
        match source {
            Source::Backing(id) => self.root_fragments(id, offset, length),
            Source::Image(value) => self.image_fragments(value, offset, length),
        }
    }
    pub(super) fn source_has_fragments(
        &mut self,
        source: Source,
        offset: u64,
        length: u64,
    ) -> Result<bool, Failure> {
        let end = offset
            .checked_add(length)
            .ok_or("symbolic numeric span extent overflow")?;
        if length == 0 {
            return Ok(false);
        }
        let (count, found) = match source {
            Source::Backing(id) => {
                let Some(tree) = self.byte_backing(id)?.fragments.as_ref() else {
                    return Ok(false);
                };
                let previous = tree
                    .range(..offset)
                    .next_back()
                    .is_some_and(|(_, fragment)| {
                        fragment.offset + u64::from(fragment.length) > offset
                    });
                (
                    tree.len(),
                    previous || tree.range(offset..end).next().is_some(),
                )
            }
            Source::Image(value) => {
                let image = Reader::new(self.evaluator.facts()).image(value)?;
                let start = image.fragments.partition_point(|fragment| {
                    fragment.offset + u64::from(fragment.length) <= offset
                });
                (
                    image.fragments.len(),
                    image
                        .fragments
                        .get(start)
                        .is_some_and(|fragment| fragment.offset < end),
                )
            }
        };
        if count != 0 {
            self.fragment_query_work(count, usize::from(found))?;
        }
        Ok(found)
    }
}
