//! Compact transient child/query buckets avoid one allocation per binder.
use crate::support::crates::generics::Work;

pub(super) struct Buckets {
    offsets: Vec<u32>,
    items: Vec<u32>,
}
impl Buckets {
    pub(super) fn new<I>(
        groups: usize,
        items: usize,
        keys: I,
        work: &mut Work,
    ) -> Result<Self, &'static str>
    where
        I: Clone + Iterator<Item = Result<(u32, u32), &'static str>>,
    {
        work.charge(
            groups
                .saturating_add(1)
                .saturating_mul(2)
                .saturating_add(items),
        )?;
        let mut offsets = vec![0_u32; groups.saturating_add(1)];
        let mut count = 0_usize;
        for key in keys.clone() {
            work.charge(1)?;
            let (group, _) = key?;
            let offset = offsets
                .get_mut(group as usize + 1)
                .ok_or("source binder bucket has a foreign owner")?;
            *offset = offset
                .checked_add(1)
                .ok_or("source binder bucket count overflow")?;
            count = count.saturating_add(1);
        }
        if count != items {
            return Err("source binder bucket count changed");
        }
        for index in 1..offsets.len() {
            work.charge(1)?;
            offsets[index] = offsets[index]
                .checked_add(offsets[index - 1])
                .ok_or("source binder bucket prefix overflow")?;
        }
        let mut cursors = offsets.clone();
        let mut values = vec![0; items];
        for key in keys {
            work.charge(1)?;
            let (group, value) = key?;
            let cursor = cursors
                .get_mut(group as usize)
                .ok_or("source binder bucket owner disappeared")?;
            *values
                .get_mut(*cursor as usize)
                .ok_or("source binder bucket slot overflow")? = value;
            *cursor += 1;
        }
        Ok(Self {
            offsets,
            items: values,
        })
    }
    pub(super) fn get(&self, group: u32) -> Result<&[u32], &'static str> {
        let start = self
            .offsets
            .get(group as usize)
            .copied()
            .ok_or("source binder bucket is unknown")?;
        let end = self
            .offsets
            .get(group as usize + 1)
            .copied()
            .ok_or("source binder bucket endpoint is unknown")?;
        self.items
            .get(start as usize..end as usize)
            .ok_or("source binder bucket range is invalid")
    }
}
