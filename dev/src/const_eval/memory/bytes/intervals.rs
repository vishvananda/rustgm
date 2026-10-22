//! Interval edits touch intersecting provenance only; byte payloads stay separate.
use super::{
    Constant, Failure, Machine,
    relocations::{Fragment, Fragments, fragment_slots},
};
fn intersecting(tree: &Fragments, start: u64, end: u64) -> impl Iterator<Item = &Fragment> {
    tree.range(..start)
        .next_back()
        .into_iter()
        .filter(move |(_, fragment)| {
            fragment
                .offset
                .checked_add(u64::from(fragment.length))
                .is_none_or(|end| end > start)
        })
        .map(|(_, fragment)| fragment)
        .chain(tree.range(start..end).map(|(_, fragment)| fragment))
}
impl Machine<'_, '_> {
    pub(super) fn fragment_query_work(
        &mut self,
        length: usize,
        intersected: usize,
    ) -> Result<(), Failure> {
        let height = usize::BITS - length.saturating_add(1).leading_zeros();
        self.charge(
            (height as usize)
                .saturating_add(1)
                .saturating_mul(16)
                .saturating_add(intersected.saturating_mul(4)),
        )
    }
    pub(super) fn clip_fragments(
        &mut self,
        tree: &Fragments,
        start: u64,
        length: u64,
    ) -> Result<Vec<Fragment>, Failure> {
        let end = start
            .checked_add(length)
            .ok_or("symbolic span extent overflow")?;
        if length == 0 {
            return Ok(Vec::new());
        }
        let count = intersecting(tree, start, end).count();
        self.fragment_query_work(tree.len(), count)?;
        self.reserve_cells(
            count
                .saturating_mul(2 * size_of::<Fragment>())
                .saturating_add(128)
                .div_ceil(size_of::<Constant>()),
        )?;
        let mut clipped = Vec::new();
        clipped
            .try_reserve_exact(count)
            .map_err(|_| "symbolic span snapshot allocation failed")?;
        for fragment in intersecting(tree, start, end) {
            let fragment_end = fragment
                .offset
                .checked_add(u64::from(fragment.length))
                .ok_or("symbolic fragment extent overflow")?;
            let from = start.max(fragment.offset);
            let to = end.min(fragment_end);
            let skipped = u8::try_from(from - fragment.offset)
                .map_err(|_| "symbolic fragment source overflow")?;
            clipped.push(Fragment {
                offset: from - start,
                source_offset: fragment
                    .source_offset
                    .checked_add(skipped)
                    .ok_or("symbolic fragment source overflow")?,
                length: u8::try_from(to - from).map_err(|_| "symbolic fragment length overflow")?,
                word: fragment.word,
            });
        }
        Ok(clipped)
    }
    pub(super) fn edit_fragments(
        &mut self,
        tree: &mut Option<Box<Fragments>>,
        start: u64,
        length: u64,
        incoming: &[Fragment],
    ) -> Result<(), Failure> {
        let end = start
            .checked_add(length)
            .ok_or("symbolic destination extent overflow")?;
        if length == 0 {
            return if incoming.is_empty() {
                Ok(())
            } else {
                Err("nonempty symbolic fragments in an empty span".into())
            };
        }
        if tree.is_none() && incoming.is_empty() {
            return Ok(());
        }
        let count = tree
            .as_ref()
            .map_or(0, |tree| intersecting(tree, start, end).count());
        let old_len = tree.as_ref().map_or(0, |tree| tree.len());
        self.fragment_query_work(old_len, count.saturating_add(incoming.len()))?;
        let height = (usize::BITS - old_len.saturating_add(1).leading_zeros()) as usize;
        let nodes = incoming
            .len()
            .saturating_add(height)
            .saturating_add(3)
            .saturating_mul(512);
        let temporary = count
            .saturating_add(incoming.len())
            .saturating_add(2)
            .saturating_mul(2 * size_of::<Fragment>())
            .saturating_add(128);
        self.reserve_cells(
            nodes
                .saturating_add(temporary)
                .div_ceil(size_of::<Constant>()),
        )?;
        let mut removed = Vec::new();
        removed
            .try_reserve_exact(count)
            .map_err(|_| "symbolic destination edit allocation failed")?;
        if let Some(tree) = tree.as_ref() {
            removed.extend(intersecting(tree, start, end).copied());
        }
        let mut edits = Vec::new();
        edits
            .try_reserve_exact(incoming.len().saturating_add(2))
            .map_err(|_| "symbolic destination split allocation failed")?;
        split_edges(&removed, start, end, &mut edits)?;
        let mut previous = 0;
        for fragment in incoming {
            let stop = fragment
                .offset
                .checked_add(u64::from(fragment.length))
                .filter(|&stop| stop <= length)
                .ok_or("symbolic input exceeds its snapshot")?;
            if fragment.offset < previous
                || fragment.length == 0
                || u16::from(fragment.source_offset) + u16::from(fragment.length) > 8
            {
                return Err("symbolic input has invalid word coverage".into());
            }
            self.word_target(fragment.word)?;
            edits.push(Fragment {
                offset: start
                    .checked_add(fragment.offset)
                    .ok_or("symbolic input offset overflow")?,
                ..*fragment
            });
            previous = stop;
        }
        if tree.is_none() && edits.is_empty() {
            return Ok(());
        }
        let tree = tree.get_or_insert_with(|| Box::new(Fragments::new()));
        for fragment in removed {
            tree.remove(&fragment.offset);
        }
        for fragment in edits {
            insert_coalesced(tree, fragment);
        }
        Ok(())
    }
    pub(super) fn buffer_fragments(
        &mut self,
        backing: &mut super::Backing,
        start: u64,
        length: u64,
        incoming: &[Fragment],
    ) -> Result<(), Failure> {
        let before = backing
            .fragments
            .as_ref()
            .map_or(0, |tree| fragment_slots(tree));
        self.edit_fragments(&mut backing.fragments, start, length, incoming)?;
        let after = backing
            .fragments
            .as_ref()
            .map_or(0, |tree| fragment_slots(tree));
        self.slots = self
            .slots
            .checked_sub(before)
            .and_then(|slots| slots.checked_add(after))
            .ok_or("symbolic temporary storage accounting overflow")?;
        Ok(())
    }
}
fn insert_coalesced(tree: &mut Fragments, mut fragment: Fragment) {
    if let Some((&offset, &previous)) = tree.range(..fragment.offset).next_back()
        && previous.word == fragment.word
        && previous.offset + u64::from(previous.length) == fragment.offset
        && u16::from(previous.source_offset) + u16::from(previous.length)
            == u16::from(fragment.source_offset)
    {
        tree.remove(&offset);
        fragment.offset = previous.offset;
        fragment.source_offset = previous.source_offset;
        fragment.length += previous.length;
    }
    let end = fragment.offset + u64::from(fragment.length);
    if let Some(&next) = tree.get(&end)
        && next.word == fragment.word
        && u16::from(fragment.source_offset) + u16::from(fragment.length)
            == u16::from(next.source_offset)
    {
        tree.remove(&end);
        fragment.length += next.length;
    }
    tree.insert(fragment.offset, fragment);
}

/// Only the first and last intersected intervals can retain bytes outside the edit.
fn split_edges(
    removed: &[Fragment],
    start: u64,
    end: u64,
    edits: &mut Vec<Fragment>,
) -> Result<(), Failure> {
    for fragment in removed {
        let stop = fragment
            .offset
            .checked_add(u64::from(fragment.length))
            .ok_or("symbolic fragment extent overflow")?;
        if fragment.offset < start {
            edits.push(Fragment {
                length: u8::try_from(start - fragment.offset)
                    .map_err(|_| "symbolic prefix length overflow")?,
                ..*fragment
            });
        }
        if stop > end {
            let skipped = u8::try_from(end - fragment.offset)
                .map_err(|_| "symbolic suffix source overflow")?;
            edits.push(Fragment {
                offset: end,
                source_offset: fragment
                    .source_offset
                    .checked_add(skipped)
                    .ok_or("symbolic suffix source overflow")?,
                length: u8::try_from(stop - end).map_err(|_| "symbolic suffix length overflow")?,
                word: fragment.word,
            });
        }
    }
    Ok(())
}
impl Machine<'_, '_> {
    pub(super) fn root_fragments(
        &mut self,
        id: super::Id,
        start: u64,
        length: u64,
    ) -> Result<Vec<Fragment>, Failure> {
        let tree = self
            .memory
            .bytes
            .roots
            .get_mut(id.0 as usize)
            .ok_or("missing symbolic backing")?
            .fragments
            .take();
        // The storage-wide retained counter continues charging this detached tree.
        let result = tree.as_ref().map_or_else(
            || Ok(Vec::new()),
            |tree| self.clip_fragments(tree, start, length),
        );
        self.memory
            .bytes
            .roots
            .get_mut(id.0 as usize)
            .ok_or("symbolic backing disappeared")?
            .fragments = tree;
        result
    }
    pub(super) fn commit_root_fragments(
        &mut self,
        id: super::Id,
        start: u64,
        length: u64,
        incoming: &[Fragment],
    ) -> Result<(), Failure> {
        let mut tree = self
            .memory
            .bytes
            .roots
            .get_mut(id.0 as usize)
            .ok_or("missing symbolic destination backing")?
            .fragments
            .take();
        let before = tree.as_ref().map_or(0, |tree| fragment_slots(tree));
        let result = self.edit_fragments(&mut tree, start, length, incoming);
        let after = tree.as_ref().map_or(0, |tree| fragment_slots(tree));
        self.memory
            .bytes
            .roots
            .get_mut(id.0 as usize)
            .ok_or("symbolic destination backing disappeared")?
            .fragments = tree;
        result?;
        self.memory.bytes.fragment_slots = self
            .memory
            .bytes
            .fragment_slots
            .checked_sub(before)
            .and_then(|slots| slots.checked_add(after))
            .ok_or("symbolic backing retention overflow")?;
        Ok(())
    }
}
