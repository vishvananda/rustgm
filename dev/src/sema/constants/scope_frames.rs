//! Nested value owners keep their dispatcher proof and restoration state together.
use super::{
    Error, PatternScope, Queries, Work,
    instances::{Context, ValueScope},
    pending_patterns::{Frame, FrameId, Saved},
};
impl Queries<'_> {
    pub(super) fn begin_variant_value(
        &mut self,
        body: crate::resolve::unit::NodeKey,
        ty: crate::types::TypeId,
    ) -> Result<(), Error> {
        if matches!(self.value_scope, Some(ValueScope::Anonymous { body: old, ty: actual }) if old == body && actual == ty)
        {
            return Ok(());
        }
        let id = self.append_pattern_frame(Frame {
            source_regions: None,
            inputs: None,
            prepared: None,
            cursor: 0,
            saved: None,
            reservation: None,
            bytes: 0,
        })?;
        self.install_pattern_frame(
            id,
            None,
            Some(ValueScope::Anonymous { body, ty }),
            self.work.context,
        )?;
        self.work.push(Work::PatternFinish(id));
        Ok(())
    }
    pub(super) fn start_pattern_frame(&mut self, id: FrameId) -> Result<(), Error> {
        let scope = self
            .pattern_frames
            .as_mut()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pattern frames"))?
            .frame(id)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?
            .prepared
            .as_ref()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing prepared pattern scope"))?
            .values();
        let value = scope
            .owner
            .and_then(|owner| self.typing.facts.initializer_index(owner))
            .filter(|&index| self.typing.facts.initializers[index].semantic_source() == scope.body)
            .map(|index| ValueScope::Initializer {
                body: scope.body,
                index,
            })
            .or_else(|| self.value_scope.filter(|value| value.body() == scope.body));
        self.install_pattern_frame(id, Some(scope), value, self.work.context)?;
        self.work.push(Work::PatternNext(id));
        Ok(())
    }
    pub(super) fn begin_pattern_value(
        &mut self,
        index: usize,
        context: Context,
    ) -> Result<(), Error> {
        self.capture_pending_patterns()?;
        let initializer = self.typing.facts.initializers[index];
        let body = initializer.semantic_source();
        let needed = self.pattern_frames.as_ref().is_some_and(|frames| {
            !frames.active.is_empty()
                || frames.batches.get(&body).is_some_and(|scope| {
                    matches!(scope, super::pending_patterns::ScopeState::Queued(_))
                })
        });
        if !needed {
            return Ok(());
        }
        // Value substitution uses the caller's rigid parameters and proof scope.
        // Declaration-pattern eligibility enters its own queued PreparedScope.
        let scope = self.pattern_scope;
        let id = self.append_pattern_frame(Frame {
            source_regions: None,
            inputs: None,
            prepared: None,
            cursor: 0,
            saved: None,
            reservation: None,
            bytes: 0,
        })?;
        self.install_pattern_frame(
            id,
            scope,
            Some(ValueScope::Initializer { body, index }),
            context,
        )?;
        self.work.push(Work::PatternFinish(id));
        Ok(())
    }
    fn install_pattern_frame(
        &mut self,
        id: FrameId,
        scope: Option<PatternScope>,
        value: Option<ValueScope>,
        context: Context,
    ) -> Result<(), Error> {
        if self
            .pattern_frames
            .as_mut()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pattern frames"))?
            .frame(id)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?
            .saved
            .is_some()
        {
            return Err(Error::at(
                super::FileId(0),
                0,
                "pattern frame entered twice",
            ));
        }
        self.resume_pattern_dispatch()?;
        self.reserve_pattern_dispatch()?;
        let growth = self
            .pattern_frames
            .as_ref()
            .map_or(4 * size_of::<FrameId>(), |frames| {
                super::pending_patterns::vector_growth(&frames.active, 1)
            });
        self.check_pattern_storage(growth)?;
        let (bytes, visits) = self.pattern_dispatch.as_mut().map_or(
            (0, 0),
            crate::sema::dispatch::Dispatcher::retained_allocation,
        );
        self.steps = self.steps.saturating_add(visits);
        self.budget()?;
        let saved = Saved {
            scope: self.pattern_scope,
            dispatcher: self.pattern_dispatch.take(),
            value: self.value_scope,
            context,
            bytes: bytes.saturating_sub(size_of::<crate::sema::dispatch::Dispatcher>()),
        };
        let frames = self
            .pattern_frames
            .as_mut()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pattern frames"))?;
        frames.children = frames.children.saturating_add(saved.bytes);
        let frame = frames
            .frame(id)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?;
        frame.saved = Some(saved);
        frames.active.push(id);
        self.pattern_scope = scope;
        self.value_scope = value;
        self.check_pattern_storage(0)
    }
    pub(super) fn finish_pattern_frame(&mut self, id: FrameId) -> Result<(), Error> {
        self.resume_pattern_dispatch()?;
        self.reserve_pattern_dispatch()?;
        let frames = self
            .pattern_frames
            .as_mut()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pattern frames"))?;
        if frames.active.last() != Some(&id) {
            return Err(Error::at(
                super::FileId(0),
                0,
                "constant pattern frame ownership mismatch",
            ));
        }
        frames.active.pop();
        let mut frame = frames
            .slots
            .get_mut(id.0 as usize)
            .and_then(Option::take)
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing completed pattern frame"))?;
        let saved = frame
            .saved
            .take()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pattern scope restoration"))?;
        frames.children = frames
            .children
            .saturating_sub(frame.bytes)
            .saturating_sub(saved.bytes);
        let result = (|| {
            if let Some(dispatcher) = self.pattern_dispatch.take() {
                let scope = self
                    .pattern_scope
                    .ok_or_else(|| Error::at(super::FileId(0), 0, "missing pattern proof owner"))?;
                let proof = dispatcher.finish();
                if !proof.has_quantified() {
                    self.steps = self.steps.saturating_add(proof.regions.len());
                    self.budget()?;
                }
                frame
                    .reservation
                    .take()
                    .ok_or_else(|| {
                        Error::at(super::FileId(0), 0, "missing pattern proof reservation")
                    })?
                    .publish_pattern(proof, self.typing.facts, scope.body, scope.body)
                    .map_err(|e| Error::at(super::FileId(0), 0, e))?;
            }
            if let Some(prepared) = frame.prepared {
                let body = prepared.values().body;
                let state = self
                    .pattern_frames
                    .as_mut()
                    .and_then(|frames| frames.batches.get_mut(&body))
                    .ok_or_else(|| {
                        Error::at(super::FileId(0), 0, "missing active pattern owner")
                    })?;
                if !matches!(state, super::pending_patterns::ScopeState::Active) {
                    return Err(Error::at(
                        super::FileId(0),
                        0,
                        "pattern owner completion state changed",
                    ));
                }
                self.typing.publish_prepared_patterns(prepared)?;
                *self
                    .pattern_frames
                    .as_mut()
                    .and_then(|frames| frames.batches.get_mut(&body))
                    .ok_or_else(|| {
                        Error::at(super::FileId(0), 0, "missing completed pattern owner")
                    })? = super::pending_patterns::ScopeState::Complete;
            }
            Ok(())
        })();
        self.pattern_scope = saved.scope;
        self.pattern_dispatch = saved.dispatcher;
        self.value_scope = saved.value;
        self.work.context = saved.context;
        result
    }
    pub(super) fn abort_pattern_frames(&mut self) {
        while let Some(frames) = &mut self.pattern_frames {
            let Some(id) = frames.active.pop() else {
                break;
            };
            let Some(mut frame) = frames.slots.get_mut(id.0 as usize).and_then(Option::take) else {
                break;
            };
            let Some(saved) = frame.saved.take() else {
                break;
            };
            frames.children = frames
                .children
                .saturating_sub(frame.bytes)
                .saturating_sub(saved.bytes);
            self.pattern_scope = saved.scope;
            self.pattern_dispatch = saved.dispatcher;
            self.value_scope = saved.value;
            self.work.context = saved.context;
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/parked_patterns.rs"]
mod tests;
