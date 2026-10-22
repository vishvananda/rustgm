//! Source alias transport is lazy and belongs to the exact prepared caller frame.
use super::{Queries, pending_patterns::FrameId};
use crate::types::substitute::Bound;
impl Queries<'_> {
    pub(super) fn begin_source_regions(&mut self, id: FrameId) -> Result<(usize, usize), String> {
        self.prepare_source_inputs(id)?;
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
        let mut work = 0;
        let fold = frame
            .prepared
            .as_ref()
            .ok_or("missing source caller proof owner")?
            .caller_regions_counted(
                self.typing.facts,
                frame.inputs.as_deref().map(|inputs| inputs.view()),
                &mut work,
            );
        self.steps = self.steps.saturating_add(work);
        frame.source_regions = Some(Box::new(fold?));
        self.finish_source_regions(id, (0, 0))
    }
    fn prepare_source_inputs(&mut self, id: FrameId) -> Result<(), String> {
        let frame = self
            .pattern_frames
            .as_mut()
            .ok_or("missing input caller frames")?
            .frame(id)?;
        if frame.inputs.is_some() {
            return Ok(());
        }
        let scope = frame
            .prepared
            .as_ref()
            .ok_or("missing prepared input caller")?
            .values();
        let (upper, work) =
            crate::sema::callee_premises::OriginalInputs::preflight(self.typing.facts, scope.owner);
        self.steps = self.steps.saturating_add(work);
        self.check_pattern_storage(upper)
            .map_err(|e| e.to_string())?;
        let inputs = crate::sema::callee_premises::OriginalInputs::new(
            self.typing.facts,
            scope.body,
            scope.owner,
            scope.binders,
        )?;
        let allocation = inputs.allocation();
        self.steps = self.steps.saturating_add(allocation.visits);
        let frames = self
            .pattern_frames
            .as_mut()
            .ok_or("missing prepared input frames")?;
        let frame = frames.frame(id)?;
        frame.inputs = Some(Box::new(inputs));
        frame.bytes = frame.bytes.saturating_add(allocation.bytes);
        frames.children = frames.children.saturating_add(allocation.bytes);
        self.check_pattern_storage(0).map_err(|e| e.to_string())
    }
    pub(super) fn canonical_pattern_input(
        &mut self,
        input: crate::sema::values::Instance,
    ) -> Result<crate::sema::values::Instance, String> {
        let Some(scope) = self.pattern_scope else {
            return Ok(input);
        };
        let open = self
            .typing
            .facts
            .types
            .arguments(input.arguments)
            .ok_or("missing pattern input arguments")?
            .any(|argument| {
                matches!(argument, crate::types::arguments::Argument::Type(ty)
                if self.typing.facts.types.needs_region_context(ty) == Some(true))
            });
        if !open {
            return Ok(input);
        }
        let frames = self
            .pattern_frames
            .as_mut()
            .ok_or("missing pattern input owner")?;
        let &current = frames
            .active
            .last()
            .ok_or("missing active pattern input owner")?;
        let direct = frames
            .frame(current)?
            .prepared
            .as_ref()
            .is_some_and(|p| super::parked_values::same_scope(p.values(), scope));
        let owner = if direct {
            current
        } else {
            let current_scope = frames
                .frame(current)?
                .saved
                .as_ref()
                .and_then(|saved| saved.scope)
                .ok_or("pattern input lacks its saved caller")?;
            if !super::parked_values::same_scope(current_scope, scope) {
                return Err("pattern input saved caller changed".into());
            }
            let parent = *frames
                .active
                .get(
                    frames
                        .active
                        .len()
                        .checked_sub(2)
                        .ok_or("missing pattern input parent")?,
                )
                .ok_or("missing pattern input parent")?;
            if !frames
                .frame(parent)?
                .prepared
                .as_ref()
                .is_some_and(|p| super::parked_values::same_scope(p.values(), scope))
            {
                return Err("pattern input belongs to another prepared parent".into());
            }
            parent
        };
        let previous = self.begin_source_regions(owner)?;
        let result = (|| {
            let frame = self
                .pattern_frames
                .as_mut()
                .ok_or("missing source input frame")?
                .frame(owner)?;
            let regions = frame
                .source_regions
                .as_deref_mut()
                .ok_or("missing source input aliases")?;
            let available = super::computation::SourceComputation::with_argument_regions(
                self.typing.facts,
                input,
                |_, observed| {
                    for &region in observed {
                        if !regions.projection_input_available(region)? {
                            return Ok::<_, &'static str>(false);
                        }
                    }
                    Ok(true)
                },
            )?;
            if available {
                input
                    .substitute(&mut self.typing.facts.types, regions)
                    .map_err(String::from)
            } else {
                Ok(input)
            }
        })();
        self.finish_source_regions(owner, previous)?;
        result
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
