//! Shared indices for ordered cleanup, statement and edge points.
use super::{Effect, Event};
use crate::support::hash::HashMap;
use crate::{
    body::{
        ir::{BlockId, Body},
        storage::Location,
    },
    borrow::regions::points::Point,
};

pub(crate) struct Index {
    pub(crate) blocks: Vec<std::ops::Range<usize>>,
    pub(crate) edges: HashMap<(BlockId, BlockId), std::ops::Range<usize>>,
    /// Point of each in-block location, dense by block-relative position.
    positions: Vec<u32>,
    /// Each block's first statement and the offset of its positions.
    extents: Vec<Extent>,
    /// Points recorded at any other location.
    others: HashMap<Location, Point>,
    pub(crate) count: u32,
}
#[derive(Clone, Copy)]
struct Extent {
    start: u32,
    end: u32,
    base: u32,
}
const NO_POINT: u32 = u32::MAX;
impl Index {
    /// The inspection or statement point recorded at this location.
    pub(crate) fn point(&self, location: Location) -> Option<Point> {
        if let Some(slot) = self.slot(location) {
            return self
                .positions
                .get(slot)
                .copied()
                .filter(|&point| point != NO_POINT)
                .map(Point);
        }
        self.others.get(&location).copied()
    }
    /// Dense slot of a location inside its block's statement extent.
    fn slot(&self, location: Location) -> Option<usize> {
        let Location::At { block, before } = location else {
            return None;
        };
        let extent = self.extents.get(block.0 as usize)?;
        (extent.start..=extent.end)
            .contains(&before)
            .then(|| extent.base as usize + (before - extent.start) as usize)
    }
    pub(crate) fn new(body: &Body, events: &[Event]) -> Result<Self, &'static str> {
        let count = u32::try_from(events.len())
            .ok()
            .filter(|n| *n <= 4_000_000)
            .ok_or("too many destruction liveness points")?;
        let mut extents = Vec::with_capacity(body.blocks.len());
        let mut positions = 0_u32;
        for block in &body.blocks {
            let width = block
                .end
                .checked_sub(block.start)
                .and_then(|width| width.checked_add(1))
                .ok_or("invalid destruction block extent")?;
            extents.push(Extent {
                start: block.start,
                end: block.end,
                base: positions,
            });
            positions = positions
                .checked_add(width)
                .filter(|&n| n != NO_POINT)
                .ok_or("too many destruction liveness points")?;
        }
        let mut result = Self {
            blocks: vec![0..0; body.blocks.len()],
            edges: HashMap::default(),
            positions: vec![NO_POINT; positions as usize],
            extents,
            others: HashMap::default(),
            count,
        };
        for (index, event) in events.iter().enumerate() {
            let range = match event.location {
                Location::At { block, .. } => result
                    .blocks
                    .get_mut(block.0 as usize)
                    .ok_or("invalid cleanup liveness block")?,
                Location::Edge { block, target } => {
                    result.edges.entry((block, target)).or_insert(0..0)
                }
            };
            if range.start == range.end {
                *range = index..index;
            }
            if range.end != index {
                return Err("interleaved destruction event lanes");
            }
            range.end = index + 1;
            if matches!(event.effect, Effect::Point | Effect::Statement) {
                let point = u32::try_from(index).map_err(|_| "too many cleanup points")?;
                if let Some(slot) = result.slot(event.location) {
                    result.positions[slot] = point;
                } else {
                    result.others.insert(event.location, Point(point));
                }
            }
        }
        Ok(result)
    }
}
