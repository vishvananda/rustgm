//! Pattern publication follows dependency completion, never provisional scalar ranges.
use super::{
    Error, Queries, Work,
    instances::ValueScope,
    pending_patterns::{self, Frame, FrameId, Frames, ScopeState},
};
use crate::{resolve::unit::NodeKey, sema::bodies::PreparedScope, support::retained::Allocation};
impl Queries<'_> {
    fn pattern_error(message: &'static str) -> Error {
        Error::at(super::FileId(0), 0, message)
    }
    pub(super) fn check_pattern_storage(&mut self, additional: usize) -> Result<(), Error> {
        let mut charge = self
            .pattern_frames
            .as_deref()
            .map_or_else(Allocation::default, Frames::allocation);
        if self.typing.pending_body() {
            charge.include(self.typing.pending_pattern_allocation());
        }
        if let Some(dispatcher) = &mut self.pattern_dispatch {
            let (bytes, visits) = dispatcher.retained_allocation();
            // The active Dispatcher header is held directly by Queries.
            charge.bytes = charge.bytes.saturating_add(bytes);
            charge.visits = charge.visits.saturating_add(visits);
        }
        if charge.bytes.saturating_add(additional) > pending_patterns::LIMIT {
            return Err(Self::pattern_error(
                "constant pattern continuation storage exceeds 64 MiB budget",
            ));
        }
        self.steps = self.steps.saturating_add(charge.visits);
        self.budget()
    }
    fn pattern_batch_growth(&self, count: usize) -> usize {
        self.pattern_frames.as_ref().map_or_else(
            || {
                size_of::<Frames>()
                    + crate::support::retained::new_map::<NodeKey, ScopeState>(count)
            },
            |frames| {
                crate::support::retained::new_map::<NodeKey, ScopeState>(
                    frames.batches.len().saturating_add(count),
                )
                .saturating_sub(crate::support::retained::map(&frames.batches))
            },
        )
    }
    pub(super) fn capture_pending_patterns(&mut self) -> Result<(), Error> {
        if self.typing.pending_body() || !self.typing.has_pending_patterns() {
            return Ok(());
        }
        let pending = self.typing.pending_pattern_allocation();
        let growth = self.pattern_batch_growth(self.typing.pending_pattern_scopes());
        self.check_pattern_storage(pending.bytes.saturating_mul(4).saturating_add(growth))?;
        if let Some((_, scopes)) = self.typing.take_prepared_patterns()? {
            self.retain_prepared_patterns(scopes)?;
        }
        Ok(())
    }
    pub(super) fn retain_prepared_patterns(
        &mut self,
        scopes: Vec<PreparedScope>,
    ) -> Result<(), Error> {
        let mut charge = Allocation::default();
        for scope in &scopes {
            charge.include(scope.allocation());
        }
        self.steps = self.steps.saturating_add(charge.visits);
        self.check_pattern_storage(
            charge
                .bytes
                .saturating_add(self.pattern_batch_growth(scopes.len())),
        )?;
        let frames = self.pattern_frames.get_or_insert_with(Default::default);
        for scope in scopes {
            let owner = scope.values().body;
            if frames.batches.contains_key(&owner) {
                return Err(Self::pattern_error("constant pattern owner prepared twice"));
            }
            frames.batches.insert(owner, ScopeState::Queued(scope));
        }
        frames.children = frames.children.saturating_add(charge.bytes);
        self.check_pattern_storage(0)
    }
    pub(super) fn begin_pending_patterns(&mut self, source: NodeKey) -> Result<bool, Error> {
        self.capture_pending_patterns()?;
        let owner = self.value_scope.map_or(source, ValueScope::body);
        self.schedule_pattern_owner(owner, Some(source))
    }
    pub(super) fn schedule_pattern_owner(
        &mut self,
        owner: NodeKey,
        retry: Option<NodeKey>,
    ) -> Result<bool, Error> {
        let Some(frames) = &mut self.pattern_frames else {
            return Ok(false);
        };
        let Some(state) = frames.batches.get_mut(&owner) else {
            return Ok(false);
        };
        if matches!(state, ScopeState::Complete) {
            return Ok(false);
        }
        if matches!(state, ScopeState::Active) {
            // Preparing a literal within this body's current pattern does not
            // demand the body's uncompleted CFG. Root demands remain cycles.
            let root=retry.is_none_or(|source|source==owner || self.value_scope.is_some_and(|value| {
                matches!(value,ValueScope::Initializer{index,..} if self.typing.facts.initializers[index].value==source)
            }));
            return if root {
                Err(crate::sema::model::Declarations::error(
                    self.report,
                    owner,
                    "cycle in constant pattern dependencies",
                ))
            } else {
                Ok(false)
            };
        }
        let ScopeState::Queued(scope) = std::mem::replace(state, ScopeState::Active) else {
            return Err(Self::pattern_error("missing queued pattern owner"));
        };
        if let Some(source) = retry {
            self.work.push(Work::PrepareNode(source));
        }
        let bytes = scope.allocation().bytes;
        let id = self.append_pattern_frame(Frame {
            prepared: Some(scope),
            cursor: 0,
            saved: None,
            reservation: None,
            bytes,
        })?;
        self.work.push(Work::PatternStart(id));
        Ok(true)
    }
    pub(super) fn append_pattern_frame(&mut self, frame: Frame) -> Result<FrameId, Error> {
        let growth = self
            .pattern_frames
            .as_ref()
            .map_or(4 * size_of::<Option<Frame>>(), |frames| {
                pending_patterns::vector_growth(&frames.slots, 1)
            });
        self.check_pattern_storage(growth)?;
        let frames = self.pattern_frames.get_or_insert_with(Default::default);
        let id = FrameId(
            u32::try_from(frames.slots.len())
                .map_err(|_| Error::at(super::FileId(0), 0, "too many constant pattern frames"))?,
        );
        frames.slots.push(Some(frame));
        Ok(id)
    }
    pub(super) fn reserve_pattern_dispatch(&mut self) -> Result<(), Error> {
        let Some(dispatcher) = &self.pattern_dispatch else {
            return Ok(());
        };
        let Some(scope) = self.pattern_scope else {
            return Ok(());
        };
        let Some(frames) = &mut self.pattern_frames else {
            return Ok(());
        };
        let Some(&id) = frames.active.last() else {
            return Ok(());
        };
        let frame = frames
            .frame(id)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?;
        let reservation = frame.reservation.get_or_insert_with(Default::default);
        dispatcher
            .reserve_pattern(self.typing.facts, scope.body, reservation)
            .map_err(|e| Error::at(super::FileId(0), 0, e))
    }
    pub(super) fn resume_pattern_dispatch(&mut self) -> Result<(), Error> {
        let Some(dispatcher) = &mut self.pattern_dispatch else {
            return Ok(());
        };
        let Some(frames) = &mut self.pattern_frames else {
            return Ok(());
        };
        let Some(&id) = frames.active.last() else {
            return Ok(());
        };
        let frame = frames
            .frame(id)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?;
        let Some(reservation) = frame.reservation.as_mut() else {
            return Ok(());
        };
        dispatcher
            .resume_pattern(self.typing.facts, reservation)
            .map_err(|e| Error::at(super::FileId(0), 0, e))
    }
}
