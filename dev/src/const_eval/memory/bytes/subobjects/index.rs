//! Completed product fields are indexed once; an access visits intersecting storage.
use super::{Failure, Machine, Reader, TypeId};
#[derive(Clone, Copy)]
struct Span {
    index: u64,
    start: u64,
    end: u64,
}
#[derive(Clone, Copy)]
struct Bounds {
    first: u64,
    last: u64,
}
pub(super) struct Index {
    fields: Vec<Span>,
    bounds: Vec<Bounds>,
    leaves: usize,
}
impl Index {
    pub(super) fn retained_slots(&self) -> usize {
        self.fields
            .capacity()
            .saturating_mul(size_of::<Span>())
            .saturating_add(self.bounds.capacity().saturating_mul(size_of::<Bounds>()))
            .saturating_add(4 * size_of::<Self>())
            .div_ceil(size_of::<super::Constant>())
    }
    pub(super) fn visit(
        &self,
        offset: u64,
        inclusive: bool,
        limit: usize,
        mut found: impl FnMut(u64),
    ) -> Result<usize, Failure> {
        let mut stack = [0usize; usize::BITS as usize + 1];
        stack[0] = 1;
        let mut length = 1;
        let mut work = 0usize;
        while length != 0 {
            length -= 1;
            let next = stack[length];
            work = work
                .checked_add(1)
                .filter(|&work| work <= limit)
                .ok_or("symbolic subobject lookup work budget exceeded")?;
            let bounds = self
                .bounds
                .get(next)
                .ok_or("missing symbolic subobject index node")?;
            if bounds.first > offset
                || bounds.last < offset
                || (!inclusive && bounds.last == offset)
            {
                continue;
            }
            if next >= self.leaves {
                if let Some(field) = self.fields.get(next - self.leaves)
                    && field.start <= offset
                    && (offset < field.end || (inclusive && offset == field.end))
                {
                    found(field.index);
                }
            } else {
                if length.checked_add(2).is_none_or(|end| end > stack.len()) {
                    return Err("symbolic subobject index depth exceeds host bound".into());
                }
                stack[length] = next * 2 + 1;
                stack[length + 1] = next * 2;
                length += 2;
            }
        }
        Ok(work)
    }
}
impl Machine<'_, '_> {
    pub(super) fn index_subobjects(&mut self, ty: TypeId, count: usize) -> Result<Index, Failure> {
        let leaves = count
            .max(1)
            .checked_next_power_of_two()
            .ok_or("symbolic subobject index extent overflow")?;
        let nodes = leaves
            .checked_mul(2)
            .ok_or("symbolic subobject index size overflow")?;
        self.reserve_cells(
            count
                .saturating_mul(2 * size_of::<Span>())
                .saturating_add(nodes.saturating_mul(size_of::<Bounds>()))
                .saturating_add(4 * size_of::<Index>())
                .div_ceil(size_of::<super::Constant>()),
        )?;
        let mut fields = Vec::new();
        fields
            .try_reserve_exact(count)
            .map_err(|_| "symbolic subobject index allocation failed")?;
        for index in 0..count {
            self.charge(1)?;
            let field = Reader::new(self.evaluator.facts()).projection_layout(ty, index as u64)?;
            let extent = self.footprint(field.ty)?.size;
            fields.push(Span {
                index: index as u64,
                start: field.offset,
                end: field
                    .offset
                    .checked_add(extent)
                    .ok_or("symbolic subobject extent overflow")?,
            });
        }
        self.charge(
            count.saturating_mul((usize::BITS - count.max(1).leading_zeros()) as usize + 1),
        )?;
        fields.sort_unstable_by_key(|field| (field.start, field.index));
        let mut bounds = Vec::new();
        bounds
            .try_reserve_exact(nodes)
            .map_err(|_| "symbolic subobject tree allocation failed")?;
        bounds.resize(
            nodes,
            Bounds {
                first: u64::MAX,
                last: 0,
            },
        );
        self.charge(nodes)?;
        for (index, field) in fields.iter().enumerate() {
            bounds[leaves + index] = Bounds {
                first: field.start,
                last: field.end,
            };
        }
        for index in (1..leaves).rev() {
            bounds[index] = Bounds {
                first: bounds[index * 2].first.min(bounds[index * 2 + 1].first),
                last: bounds[index * 2].last.max(bounds[index * 2 + 1].last),
            };
        }
        Ok(Index {
            fields,
            bounds,
            leaves,
        })
    }
}
