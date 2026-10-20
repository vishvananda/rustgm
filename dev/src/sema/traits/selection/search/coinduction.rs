//! Coinduction requires the same full predicate, scope and uninterrupted auto path.
use super::{Decision, Declarations, Frame, Node, Predicate, Search, Source, inference};

impl Search<'_> {
    pub(super) fn auto_cycle(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        goal: Predicate,
    ) -> Result<Option<Decision>, String> {
        if let Some(&(scope, segment)) = self.query.auto_active.get(&goal) {
            self.query.cycles += 1;
            if scope != frame.scope || segment != frame.auto_segment {
                return Ok(Some(Decision::Absent));
            }
            // Every intervening implementation/field still has to finish. The
            // transitive dependency flag keeps this assumption and its ancestors
            // out of completed caches while any sibling obligation is open.
            let index = self.query.push(
                Node {
                    cycle_dependent: true,
                    provisional_size: false,
                    goal,
                    source: Some(Source::Builtin),
                    children: Vec::new(),
                    first: frame.first,
                    end: frame.available_region(),
                    equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                    outlives: Vec::new(),
                    type_outlives: Vec::new(),
                    sized: Vec::new(),
                },
                frame.scope,
                frame.first..frame.first,
            );
            return Ok(Some(Decision::Proven(index)));
        }
        if let Some(&decision) = self
            .query
            .auto_provisional
            .get(&frame.auto_segment)
            .and_then(|values| values.get(&(frame.scope, goal)))
        {
            self.query.reused = true;
            return Ok(Some(decision));
        }
        let (shape, work) = facts.types.auto_shape_key(goal)?;
        frame.inputs.context.account_work(work).map_err(inference)?;
        if !self.query.active.insert(shape) {
            self.query.cycles += 1;
            return Ok(Some(Decision::Absent));
        }
        frame.key = Some(shape);
        self.query
            .auto_active
            .insert(goal, (frame.scope, frame.auto_segment));
        Ok(None)
    }
}

impl super::Query {
    /// These answers may depend on an ancestor that has not checked every field
    /// yet. Only the same contiguous auto segment may use them. Any failed child
    /// invalidates the entire segment, across its scopes, before another query.
    /// Auto traits expose no associated outputs: repeated exact predicates can
    /// share the full proof's private witnesses in the same scope. A bare cycle
    /// assumption has no completed frame key and must never replace that proof.
    pub(super) fn remember_auto(&mut self, frame: &Frame, goal: Predicate, decision: Decision) {
        if frame.key.is_some()
            && matches!(
                goal.bound,
                crate::types::parameters::predicates::Bound::Auto(_)
            )
        {
            self.auto_provisional
                .entry(frame.auto_segment)
                .or_default()
                .insert((frame.scope, goal), decision);
        }
    }

    pub(super) fn invalidate_auto(&mut self, frame: &Frame) {
        self.auto_provisional.remove(&frame.auto_segment);
    }

    pub(super) fn prepare_auto_child(&mut self, frame: &Frame, child: &mut Frame) {
        if !child.started {
            if matches!(
                frame.goal.builtin_bound(),
                Some(crate::types::parameters::predicates::Bound::Auto(_))
            ) && !frame
                .autos
                .as_ref()
                .is_some_and(|pending| pending.revealing())
            {
                child.auto_segment = frame.auto_segment;
            } else {
                self.next_auto_segment += 1;
                child.auto_segment = self.next_auto_segment;
            }
        }
    }
}
