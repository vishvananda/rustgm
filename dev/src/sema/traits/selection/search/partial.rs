//! A unique partial candidate retains its conditions; input progress is not proof.
use super::{Context, Declarations, Frame, Search, inference};
use crate::types::substitute::Bound;

impl Frame {
    pub(super) fn publish(
        &mut self,
        facts: &mut Declarations,
        source: &mut Context,
    ) -> Result<bool, String> {
        // Frames within the same proof scope share their lifetime identities.
        // The root's universal boundary uses its separately retained export fold.
        let mut identity = Bound::parameters(&facts.types, &[])?;
        self.inputs
            .publish(&mut facts.types, source, &mut identity)
            .map_err(inference)
    }

    pub(super) fn refresh(
        &mut self,
        facts: &mut Declarations,
        source: &mut Context,
    ) -> Result<(), String> {
        self.inputs
            .refresh(&facts.types, source)
            .map_err(inference)?;
        if let Some(trial) = &mut self.trial {
            trial
                .inputs
                .refresh(&facts.types, &mut self.inputs.context)
                .map_err(inference)?;
        }
        Ok(())
    }
}

impl Search<'_> {
    pub(super) fn publish_trial(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<(), String> {
        if frame.language.is_some() || frame.equality_hint.is_some() {
            // Callable equations already live in this frame's retained input graph.
            return Ok(());
        }
        let trial = frame
            .trial
            .as_mut()
            .ok_or("missing selected partial candidate")?;
        let mut identity = Bound::parameters(&facts.types, &[])?;
        trial
            .inputs
            .publish(&mut facts.types, &mut frame.inputs.context, &mut identity)
            .map_err(inference)?;
        self.charge(&mut frame.inputs.context)
    }

    pub(super) fn accept_partial(
        &mut self,
        facts: &mut Declarations,
        parent: &mut Frame,
        mut child: Frame,
    ) -> Result<(), String> {
        if parent.normalizing() {
            return Err("partial normalization child requires retained input evidence".into());
        }
        if matches!(
            parent.language.as_deref(),
            Some(super::mechanical::Pending::Unsize(_))
        ) {
            return self.accept_partial_unsizing(facts, parent, child);
        }
        if matches!(
            parent.language.as_deref(),
            Some(super::mechanical::Pending::Forward(_))
        ) {
            return self.accept_partial_forwarding(facts, parent, child);
        }
        let trial = parent
            .trial
            .as_mut()
            .ok_or("missing partial condition parent")?;
        let index = *trial
            .pending
            .get(trial.cursor)
            .ok_or("missing partial condition index")?;
        child.publish(facts, &mut trial.inputs.context)?;
        trial.end = trial.end.max(self.query.allocations.end());
        let slot = self.query.retain_partial(child)?;
        trial.wakeups.retain_child(index, slot);
        trial.defer_condition(facts, index)?;
        trial.cursor += 1;
        self.charge(&mut trial.inputs.context)
    }
}

impl super::Query {
    pub(super) fn retain_partial(&mut self, mut child: Frame) -> Result<usize, String> {
        let slot = if let Some(slot) = child.suspended_slot {
            slot
        } else {
            let slot = self.suspended.len();
            child.suspended_slot = Some(slot);
            self.suspended.push(None);
            slot
        };
        let stored = self
            .suspended
            .get_mut(slot)
            .ok_or("invalid partial child slot")?;
        if stored.replace(child).is_some() {
            return Err("partial child slot already occupied".into());
        }
        Ok(slot)
    }
}
