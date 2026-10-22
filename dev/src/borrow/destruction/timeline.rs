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
    pub(crate) points: HashMap<Location, Point>,
    pub(crate) count: u32,
}
impl Index {
    pub(crate) fn new(body: &Body, events: &[Event]) -> Result<Self, &'static str> {
        let count = u32::try_from(events.len())
            .ok()
            .filter(|n| *n <= 4_000_000)
            .ok_or("too many destruction liveness points")?;
        let mut result = Self {
            blocks: vec![0..0; body.blocks.len()],
            edges: HashMap::default(),
            points: HashMap::default(),
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
                result.points.insert(
                    event.location,
                    Point(u32::try_from(index).map_err(|_| "too many cleanup points")?),
                );
            }
        }
        Ok(result)
    }
}
