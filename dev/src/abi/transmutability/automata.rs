//! Hash-consed layout DAGs preserve repeats and correlated alternatives.
use super::Budget;
use crate::{
    support::{hash::HashMap, retained::Growth},
    types::{RegionId, TypeId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayoutId(pub u32);
/// Inclusive/exclusive byte interval; 256 is the uninitialized symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ByteRange {
    pub(super) start: u16,
    pub(super) end: u16,
}
impl ByteRange {
    pub const INITIALIZED: Self = Self { start: 0, end: 256 };
    pub const UNINITIALIZED: Self = Self { start: 0, end: 257 };
    pub const BOOL: Self = Self { start: 0, end: 2 };
    /// # Errors
    /// Rejects empty ranges and symbols outside the byte/uninitialized alphabet.
    pub const fn new(start: u16, end: u16) -> Result<Self, &'static str> {
        if start >= end || end > 257 {
            return Err("invalid layout byte interval");
        }
        Ok(Self { start, end })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Reference {
    pub referent: TypeId,
    pub region: RegionId,
    pub mutable: bool,
    pub size: u64,
    pub align: u32,
}
impl Ord for Reference {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            self.referent.0,
            self.region.0,
            self.mutable,
            self.size,
            self.align,
        )
            .cmp(&(
                other.referent.0,
                other.region.0,
                other.mutable,
                other.size,
                other.align,
            ))
    }
}
impl PartialOrd for Reference {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Node {
    Empty(u64),
    Epsilon,
    Byte(ByteRange),
    Reference(Reference),
    Sequence(LayoutId, LayoutId),
    Choice(LayoutId, LayoutId),
    Repeat(LayoutId, u64),
    Guard(LayoutId),
}
#[derive(Clone, Copy)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Cached independent language properties compose differently under products and alternatives"
)]
pub(super) struct Info {
    pub size: u64,
    pub inhabited: bool,
    pub bytes_only: bool,
    pub guarded: bool,
    pub initialized: bool,
    pub can_initialize: bool,
    pub uninit: bool,
}
#[derive(Clone, Copy)]
struct Record {
    node: Node,
    info: Info,
}
#[derive(Default)]
pub struct Language {
    records: Vec<Record>,
    index: HashMap<Node, LayoutId>,
}
impl Language {
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    pub(super) fn node(&self, id: LayoutId) -> Result<Node, &'static str> {
        self.records
            .get(id.0 as usize)
            .map(|r| r.node)
            .ok_or("invalid layout language identity")
    }
    pub(super) fn info(&self, id: LayoutId) -> Result<Info, &'static str> {
        self.records
            .get(id.0 as usize)
            .map(|r| r.info)
            .ok_or("invalid layout language identity")
    }
    fn intern(
        &mut self,
        node: Node,
        info: Info,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        budget.charge(1, 0)?;
        if let Some(&id) = self.index.get(&node) {
            return Ok(id);
        }
        let mut growth = Growth::vector::<Record>(self.records.len(), self.records.capacity(), 1);
        growth.include(Growth::map::<Node, LayoutId>(
            self.index.len(),
            self.index.capacity(),
            1,
        ));
        budget.charge(growth.work, growth.bytes)?;
        self.records
            .try_reserve(1)
            .map_err(|_| "layout language allocation failed")?;
        self.index
            .try_reserve(1)
            .map_err(|_| "layout language index allocation failed")?;
        let id = LayoutId(
            u32::try_from(self.records.len()).map_err(|_| "layout language identity overflow")?,
        );
        self.records.push(Record { node, info });
        self.index.insert(node, id);
        Ok(id)
    }
    /// # Errors
    /// Preserves the query's work/storage and allocation failures.
    pub fn epsilon(&mut self, budget: &mut Budget<'_>) -> Result<LayoutId, &'static str> {
        self.intern(
            Node::Epsilon,
            Info {
                size: 0,
                inhabited: true,
                bytes_only: true,
                guarded: false,
                initialized: true,
                can_initialize: true,
                uninit: true,
            },
            budget,
        )
    }
    /// # Errors
    /// Preserves the query's work/storage and allocation failures.
    pub fn empty(&mut self, size: u64, budget: &mut Budget<'_>) -> Result<LayoutId, &'static str> {
        self.intern(
            Node::Empty(size),
            Info {
                size,
                inhabited: false,
                bytes_only: true,
                guarded: false,
                initialized: true,
                can_initialize: false,
                uninit: false,
            },
            budget,
        )
    }
    /// # Errors
    /// Preserves the query's work/storage and allocation failures.
    pub fn byte(
        &mut self,
        range: ByteRange,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        self.intern(
            Node::Byte(range),
            Info {
                size: 1,
                inhabited: true,
                bytes_only: true,
                guarded: false,
                initialized: range.end <= 256,
                can_initialize: range.start < 256,
                uninit: range.end == 257,
            },
            budget,
        )
    }
    /// # Errors
    /// Rejects invalid alignment and preserves payment/allocation failures.
    pub fn reference(
        &mut self,
        value: Reference,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        if !value.align.is_power_of_two() {
            return Err("invalid layout reference alignment");
        }
        self.intern(
            Node::Reference(value),
            Info {
                size: 8,
                inhabited: true,
                bytes_only: false,
                guarded: false,
                initialized: false,
                can_initialize: false,
                uninit: false,
            },
            budget,
        )
    }
    /// A nominal safety guard is retained in structure, then interpreted for each query side.
    /// # Errors
    /// Rejects invalid child identities and preserves payment/allocation failures.
    pub fn nominal(
        &mut self,
        child: LayoutId,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        let mut info = self.info(child)?;
        info.guarded = true;
        self.intern(Node::Guard(child), info, budget)
    }
    /// # Errors
    /// Rejects size overflow/invalid children and preserves payment/allocation failures.
    pub fn repeat(
        &mut self,
        mut child: LayoutId,
        mut count: u64,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        let mut info = self.info(child)?;
        if count == 0 {
            return self.epsilon(budget);
        }
        if count == 1 || info.size == 0 {
            return Ok(child);
        }
        if let Node::Repeat(element, length) = self.node(child)? {
            count = count
                .checked_mul(length)
                .ok_or("layout repeat length overflow")?;
            child = element;
            info = self.info(child)?;
        }
        info.size = info
            .size
            .checked_mul(count)
            .ok_or("layout repeat size overflow")?;
        if !info.inhabited {
            return self.empty(info.size, budget);
        }
        self.intern(Node::Repeat(child, count), info, budget)
    }
    /// # Errors
    /// Rejects size overflow/invalid children and preserves payment/allocation failures.
    pub fn sequence(
        &mut self,
        left: LayoutId,
        right: LayoutId,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        let left_info = self.info(left)?;
        let right_info = self.info(right)?;
        let size = left_info
            .size
            .checked_add(right_info.size)
            .ok_or("layout sequence size overflow")?;
        if !left_info.inhabited || !right_info.inhabited {
            return self.empty(size, budget);
        }
        if matches!(self.node(left)?, Node::Epsilon) {
            return Ok(right);
        }
        if matches!(self.node(right)?, Node::Epsilon) {
            return Ok(left);
        }
        let (left_unit, left_count) =
            if let Node::Repeat(left_unit, left_count) = self.node(left)? {
                (left_unit, left_count)
            } else {
                (left, 1)
            };
        let (right_unit, right_count) =
            if let Node::Repeat(right_unit, right_count) = self.node(right)? {
                (right_unit, right_count)
            } else {
                (right, 1)
            };
        if left_unit == right_unit {
            return self.repeat(
                left_unit,
                left_count
                    .checked_add(right_count)
                    .ok_or("layout repeat length overflow")?,
                budget,
            );
        }
        self.intern(
            Node::Sequence(left, right),
            Info {
                size,
                inhabited: true,
                bytes_only: left_info.bytes_only && right_info.bytes_only,
                guarded: left_info.guarded || right_info.guarded,
                initialized: left_info.initialized && right_info.initialized,
                can_initialize: left_info.can_initialize && right_info.can_initialize,
                uninit: left_info.uninit && right_info.uninit,
            },
            budget,
        )
    }
    pub(super) fn either(
        &mut self,
        left: LayoutId,
        right: LayoutId,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        let a = self.info(left)?;
        let b = self.info(right)?;
        if a.size != b.size {
            return Err("layout alternatives have different widths");
        }
        if !a.inhabited {
            return Ok(right);
        }
        if !b.inhabited || left == right {
            return Ok(left);
        }
        let (left, right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        self.intern(
            Node::Choice(left, right),
            Info {
                size: a.size,
                inhabited: true,
                bytes_only: a.bytes_only && b.bytes_only,
                guarded: a.guarded || b.guarded,
                initialized: a.initialized && b.initialized,
                can_initialize: a.can_initialize || b.can_initialize,
                uninit: a.uninit || b.uninit,
            },
            budget,
        )
    }
    /// Balanced alternatives avoid copying an ever-growing prefix transition set.
    /// # Errors
    /// Rejects empty selections, width mismatches and payment/allocation failures.
    pub fn alternatives(
        &mut self,
        mut values: Vec<LayoutId>,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        if values.is_empty() {
            return Err("layout alternatives require an explicit empty language");
        }
        budget.charge(values.len().saturating_mul(usize::BITS as usize + 2), 0)?;
        values.sort_unstable();
        values.dedup();
        while values.len() > 1 {
            let mut end = 0;
            for i in (0..values.len()).step_by(2) {
                values[end] = if let Some(&right) = values.get(i + 1) {
                    self.either(values[i], right, budget)?
                } else {
                    values[i]
                };
                end += 1;
            }
            values.truncate(end);
        }
        Ok(values[0])
    }
}
