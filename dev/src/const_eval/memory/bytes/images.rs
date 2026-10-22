//! Immutable images and mutable backing exchange only selected symbolic intervals.
use super::relocations::Fragment;
use super::{Backing, Constant, Failure, Machine, Reader};
use crate::sema::values::ImageFragment;
impl Machine<'_, '_> {
    pub(super) fn image_fragments(
        &mut self,
        value: Constant,
        start: u64,
        length: u64,
    ) -> Result<Vec<Fragment>, Failure> {
        let end = start
            .checked_add(length)
            .ok_or("symbolic image span extent overflow")?;
        let image = Reader::new(self.evaluator.facts()).image(value)?;
        if end > image.bytes.len() as u64 {
            return Err("symbolic image span exceeds storage".into());
        }
        let first = image
            .fragments
            .partition_point(|fragment| fragment.offset + u64::from(fragment.length) <= start);
        let stop = image
            .fragments
            .partition_point(|fragment| fragment.offset < end);
        let count = stop.saturating_sub(first);
        self.fragment_query_work(image.fragments.len(), count)?;
        let retained = count
            .saturating_mul(2 * size_of::<Fragment>())
            .saturating_add(128)
            .div_ceil(size_of::<Constant>());
        self.reserve_cells(retained)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(retained)
            .ok_or("symbolic image snapshot retention overflow")?;
        let result = (|| {
            let mut fragments = Vec::new();
            fragments
                .try_reserve_exact(count)
                .map_err(|_| "symbolic image snapshot allocation failed")?;
            for index in first..stop {
                let fragment = Reader::new(self.evaluator.facts()).image(value)?.fragments[index];
                let from = start.max(fragment.offset);
                let to = end.min(fragment.offset + u64::from(fragment.length));
                let skipped = u8::try_from(from - fragment.offset)
                    .map_err(|_| "symbolic image source offset overflow")?;
                fragments.push(Fragment {
                    offset: from - start,
                    source_offset: fragment
                        .source_offset
                        .checked_add(skipped)
                        .ok_or("symbolic image source offset overflow")?,
                    length: u8::try_from(to - from)
                        .map_err(|_| "symbolic image fragment length overflow")?,
                    word: self.intern_word(fragment.target)?,
                });
            }
            Ok(fragments)
        })();
        self.slots = previous;
        result
    }
    pub(super) fn encode_image_fragments(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        value: Constant,
        length: u64,
    ) -> Result<(), Failure> {
        let fragments = self.image_fragments(value, 0, length)?;
        self.buffer_fragments(backing, offset, length, &fragments)
    }
    pub(super) fn source_word_targets(
        &mut self,
        source: super::continuations::Source,
        offset: u64,
        length: u64,
    ) -> Result<Vec<ImageFragment>, Failure> {
        let fragments = self.source_fragments(source, offset, length)?;
        self.reserve_cells(
            fragments
                .len()
                .saturating_mul(2 * size_of::<ImageFragment>() + 2 * size_of::<Fragment>())
                .saturating_add(128)
                .div_ceil(size_of::<Constant>()),
        )?;
        self.charge(fragments.len())?;
        let mut targets = Vec::new();
        targets
            .try_reserve_exact(fragments.len())
            .map_err(|_| "immutable symbolic image allocation failed")?;
        for fragment in fragments {
            targets.push(ImageFragment {
                offset: fragment.offset,
                source_offset: fragment.source_offset,
                length: fragment.length,
                target: self.word_target(fragment.word)?,
            });
        }
        Ok(targets)
    }
}
