//! Source alias transport is lazy and belongs to the exact prepared caller frame.
use super::{Queries, pending_patterns::FrameId};
use crate::types::substitute::Bound;
impl Queries<'_> {
    pub(super) fn begin_source_regions(&mut self, id: FrameId) -> Result<(usize, usize), String> {
        let frame = self
            .pattern_frames
            .as_mut()
            .ok_or("missing source caller frames")?
            .frame(id)?;
        let prepared = frame
            .prepared
            .as_ref()
            .ok_or("missing source caller proof owner")?;
        let upper = prepared.caller_region_preflight()?;
        let prior = frame.source_regions.as_ref().map(|fold| {
            let (bytes, visits) = fold.retained_allocation();
            (size_of::<Bound>() + bytes, fold.steps, visits)
        });
        let old = prior.map_or(0, |value| value.0);
        self.steps = self.steps.saturating_add(prior.map_or(0, |value| value.2));
        self.check_pattern_storage(upper.saturating_sub(old))
            .map_err(|e| e.to_string())?;
        let frame = self
            .pattern_frames
            .as_mut()
            .ok_or("missing source caller frames")?
            .frame(id)?;
        if let Some((bytes, steps, _)) = prior {
            return Ok((bytes, steps));
        }
        let (fold, work) = frame
            .prepared
            .as_ref()
            .ok_or("missing source caller proof owner")?
            .caller_regions(self.typing.facts)?;
        frame.source_regions = Some(Box::new(fold));
        self.steps = self.steps.saturating_add(work);
        self.finish_source_regions(id, (0, 0))
    }
    pub(super) fn finish_source_regions(
        &mut self,
        id: FrameId,
        previous: (usize, usize),
    ) -> Result<(usize, usize), String> {
        let frames = self
            .pattern_frames
            .as_mut()
            .ok_or("missing source caller frames")?;
        let frame = frames.frame(id)?;
        let fold = frame
            .source_regions
            .as_ref()
            .ok_or("missing source alias folder")?;
        let (bytes, visits) = fold.retained_allocation();
        let bytes = size_of::<Bound>().saturating_add(bytes);
        let growth = bytes
            .checked_sub(previous.0)
            .ok_or("source alias storage moved backwards")?;
        self.steps = self
            .steps
            .saturating_add(visits)
            .saturating_add(fold.steps.saturating_sub(previous.1));
        let current = (bytes, fold.steps);
        frame.bytes = frame.bytes.saturating_add(growth);
        frames.children = frames.children.saturating_add(growth);
        self.check_pattern_storage(0).map_err(|e| e.to_string())?;
        Ok(current)
    }
}
