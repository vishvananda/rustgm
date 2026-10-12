//! Partition only selected array boundaries, then number their shared move bits.
use super::{Component, PathId, Paths};
use std::collections::HashMap;

struct Boundary {
    array: PathId,
    offset: u64,
    element: Option<PathId>,
    bit: u32,
}
struct Subarray {
    array: PathId,
    path: PathId,
    start: u64,
    end: u64,
}
#[derive(Default)]
pub(super) struct Arrays {
    boundaries: Vec<Boundary>,
    groups: HashMap<PathId, (usize, usize)>,
    subarrays: Vec<Subarray>,
}
enum Work {
    Enter(usize),
    Finish(usize),
    Boundary(usize),
    Gap,
}
impl Arrays {
    pub(super) fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
    fn boundary(&mut self, array: PathId, offset: u64, element: Option<PathId>) {
        self.boundaries.push(Boundary {
            array,
            offset,
            element,
            bit: 0,
        });
    }
    pub(super) fn add(
        &mut self,
        array: PathId,
        path: PathId,
        component: Component,
        length: u64,
    ) -> Result<(), &'static str> {
        if let std::collections::hash_map::Entry::Vacant(entry) = self.groups.entry(array) {
            entry.insert((0, 0));
            self.boundary(array, 0, None);
            self.boundary(array, length, None);
        }
        match component {
            Component::Index(index) if index < length => {
                self.boundary(array, index, Some(path));
                self.boundary(array, index + 1, None);
            }
            Component::Subarray { start, end } if start <= end && end <= length => {
                self.boundary(array, start, None);
                self.boundary(array, end, None);
                self.subarrays.push(Subarray {
                    array,
                    path,
                    start,
                    end,
                });
            }
            _ => return Err("pattern move range exceeds its array"),
        }
        if self.boundaries.len() > 1_000_000 {
            return Err("array move partition budget exceeded");
        }
        Ok(())
    }
    fn prepare(&mut self) {
        self.boundaries
            .sort_unstable_by_key(|b| (b.array.0, b.offset));
        self.boundaries.dedup_by(|later, earlier| {
            if later.array == earlier.array && later.offset == earlier.offset {
                earlier.element = earlier.element.or(later.element);
                true
            } else {
                false
            }
        });
        let mut start = 0;
        while start < self.boundaries.len() {
            let array = self.boundaries[start].array;
            let mut end = start + 1;
            while end < self.boundaries.len() && self.boundaries[end].array == array {
                end += 1;
            }
            self.groups.insert(array, (start, end));
            start = end;
        }
    }
    fn schedule_children(
        &self,
        paths: &Paths,
        index: usize,
        work: &mut Vec<Work>,
    ) -> Result<(), &'static str> {
        let id = PathId(u32::try_from(index).map_err(|_| "too many move paths")?);
        if let Some(&(start, end)) = self.groups.get(&id) {
            for index in (start..end).rev() {
                if index + 1 < end {
                    work.push(
                        self.boundaries[index]
                            .element
                            .map_or(Work::Gap, |id| Work::Enter(id.0 as usize)),
                    );
                }
                work.push(Work::Boundary(index));
            }
        } else {
            let mut child = paths.entries[index].child;
            while let Some(id) = child {
                work.push(Work::Enter(id.0 as usize));
                child = paths.entries[id.0 as usize].sibling;
            }
        }
        Ok(())
    }
    pub(super) fn number(mut self, paths: &mut Paths) -> Result<(), &'static str> {
        self.prepare();
        let mut work = Vec::new();
        let mut next = 0_u32;
        for root in 0..paths.entries.len() {
            if paths.entries[root].parent.is_some() {
                continue;
            }
            work.push(Work::Enter(root));
            while let Some(step) = work.pop() {
                match step {
                    Work::Enter(index) => {
                        let path = &mut paths.entries[index];
                        path.start = next;
                        next = next.checked_add(1).ok_or("too many move state bits")?;
                        work.push(Work::Finish(index));
                        self.schedule_children(paths, index, &mut work)?;
                    }
                    Work::Finish(index) => paths.entries[index].end = next,
                    Work::Boundary(index) => self.boundaries[index].bit = next,
                    Work::Gap => next = next.checked_add(1).ok_or("too many move state bits")?,
                }
            }
        }
        for range in &self.subarrays {
            let (start, end) = self.groups[&range.array];
            let boundaries = &self.boundaries[start..end];
            let first = boundaries
                .binary_search_by_key(&range.start, |b| b.offset)
                .map_err(|_| "missing subarray start boundary")?;
            let last = boundaries
                .binary_search_by_key(&range.end, |b| b.offset)
                .map_err(|_| "missing subarray end boundary")?;
            let path = &mut paths.entries[range.path.0 as usize];
            path.start = boundaries[first].bit;
            path.end = boundaries[last].bit;
        }
        paths.bits = next;
        Ok(())
    }
}
