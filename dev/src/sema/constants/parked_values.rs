//! An initializer temporarily borrows its exact parked caller's proof allocator.
use super::{PatternScope, Queries, instances::ValueScope, pending_patterns::FrameId};
use crate::sema::{dispatch::Dispatcher, model::Declarations, traits::obligations::Reservation};
impl Queries<'_> {
    pub(super) fn with_parked_caller<T>(
        &mut self,
        aliases: usize,
        use_caller: impl FnOnce(
            &mut Dispatcher,
            &mut Declarations,
            PatternScope,
            &mut Reservation,
            &mut crate::types::substitute::Bound,
            [std::ops::Range<usize>; 2],
        ) -> Result<T, String>,
    ) -> Result<T, String> {
        self.check_pattern_storage(0).map_err(|e| e.to_string())?;
        let (child, parent, scope) = self.parked_caller()?;
        let canonical = self.begin_source_regions(parent)?;
        let growth = self
            .pattern_frames
            .as_mut()
            .ok_or("missing source caller frames")?
            .frame(parent)?
            .source_regions
            .as_ref()
            .ok_or("missing source caller aliases")?
            .projection_anchor_growth(aliases)?;
        self.check_pattern_storage(growth)
            .map_err(|e| e.to_string())?;
        let frames = self
            .pattern_frames
            .as_mut()
            .ok_or("missing source caller frames")?;
        let previous = frames
            .frame(child)?
            .saved
            .as_ref()
            .ok_or("missing parked caller")?
            .bytes;
        let remaining = frames
            .children
            .checked_sub(previous)
            .ok_or("parked caller storage underflow")?;
        let saved = frames
            .frame(child)?
            .saved
            .as_mut()
            .ok_or("missing parked caller")?;
        let mut dispatcher = saved.dispatcher.take();
        let existing = dispatcher.is_some();
        saved.bytes = 0;
        frames.children = remaining;
        let result = (|| {
            if dispatcher.is_none() {
                let first = u32::try_from(self.typing.facts.body_regions.len() + 1)
                    .map_err(|_| "source caller region overflow")?;
                dispatcher = Some(Dispatcher::pattern(
                    self.typing.facts,
                    scope.binders,
                    first,
                    scope.owner,
                )?);
            }
            let dispatcher = dispatcher.as_mut().ok_or("missing source dispatcher")?;
            let parent = self
                .pattern_frames
                .as_mut()
                .ok_or("missing source caller frames")?
                .frame(parent)?;
            if !existing && parent.reservation.is_some() {
                return Err("source caller reservation has no original dispatcher".into());
            }
            let publication = parent
                .prepared
                .as_ref()
                .ok_or("missing prepared source caller")?
                .caller_publication()?;
            let reservation = parent.reservation.get_or_insert_with(Default::default);
            // A fresh dispatcher starts at this frontier. An existing one resumes
            // only through its original reservation, recording every foreign gap.
            if existing {
                dispatcher.resume_pattern(self.typing.facts, reservation)?;
            }
            let regions = parent
                .source_regions
                .as_deref_mut()
                .ok_or("missing source caller aliases")?;
            use_caller(
                dispatcher,
                self.typing.facts,
                scope,
                reservation,
                regions,
                publication,
            )
        })();
        self.restore_parked_caller(child, dispatcher)?;
        self.finish_source_regions(parent, canonical)?;
        self.check_pattern_storage(0).map_err(|e| e.to_string())?;
        result
    }
    fn parked_caller(&mut self) -> Result<(FrameId, FrameId, PatternScope), String> {
        let Some(ValueScope::Initializer { body, index }) = self.value_scope else {
            return Err("source computation requires its current initializer scope".into());
        };
        if !self
            .typing
            .facts
            .initializers
            .get(index)
            .is_some_and(|initializer| initializer.semantic_source() == body)
        {
            return Err("source computation initializer ownership changed".into());
        }
        let frames = self
            .pattern_frames
            .as_mut()
            .ok_or("missing source caller frames")?;
        let count = frames.active.len();
        let &child = frames.active.last().ok_or("missing source value frame")?;
        let &parent = frames
            .active
            .get(count.checked_sub(2).ok_or("missing source caller frame")?)
            .ok_or("missing source caller frame")?;
        let value = frames.frame(child)?;
        if value.prepared.is_some() {
            return Err("source value is not owned by its active initializer frame".into());
        }
        let scope = value
            .saved
            .as_ref()
            .and_then(|saved| saved.scope)
            .ok_or("source caller is not a pattern body")?;
        if self
            .pattern_scope
            .is_none_or(|current| !same_scope(current, scope))
        {
            return Err("source value caller scope changed".into());
        }
        let actual = frames
            .frame(parent)?
            .prepared
            .as_ref()
            .ok_or("source caller lacks its prepared proof owner")?
            .values();
        if !same_scope(actual, scope) {
            return Err("source caller prepared owner changed".into());
        }
        Ok((child, parent, scope))
    }
    fn restore_parked_caller(
        &mut self,
        id: FrameId,
        mut dispatcher: Option<Dispatcher>,
    ) -> Result<(), String> {
        let (bytes, visits) = dispatcher
            .as_mut()
            .map_or((0, 0), Dispatcher::retained_allocation);
        self.steps = self.steps.saturating_add(visits);
        let bytes = bytes.saturating_sub(size_of::<Dispatcher>());
        let frames = self
            .pattern_frames
            .as_mut()
            .ok_or("missing source caller frames")?;
        let saved = frames
            .frame(id)?
            .saved
            .as_mut()
            .ok_or("missing source caller restoration")?;
        if saved.dispatcher.is_some() || saved.bytes != 0 {
            return Err("source caller restored twice".into());
        }
        saved.dispatcher = dispatcher;
        saved.bytes = bytes;
        frames.children = frames.children.saturating_add(bytes);
        Ok(())
    }
}

fn same_scope(left: PatternScope, right: PatternScope) -> bool {
    left.body == right.body
        && left.owner == right.owner
        && left.binders == right.binders
        && left.end == right.end
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_node_does_not_share_another_prepared_owner_or_binder() {
        let original = PatternScope {
            body: crate::resolve::unit::NodeKey {
                file: crate::support::source::FileId(0),
                node: crate::syntax::ast::NodeId::new(1),
            },
            owner: Some(crate::resolve::model::DefId(2)),
            binders: 1,
            end: 3,
        };
        assert!(same_scope(original, original));
        assert!(!same_scope(
            original,
            PatternScope {
                owner: Some(crate::resolve::model::DefId(3)),
                ..original
            }
        ));
        assert!(!same_scope(
            original,
            PatternScope {
                binders: 2,
                ..original
            }
        ));
        assert!(!same_scope(original, PatternScope { end: 4, ..original }));
    }
}
