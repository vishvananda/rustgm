//! Associated equality premises retain normalization proof and explicit callable equations.
use super::normalization::{Normalization, Purpose};
use super::{Action, Decision, Declarations, Frame, Node, Search, Source, inference};
use crate::types::{TypeId, infer};

pub(super) struct Equality {
    right: Option<TypeId>,
    left: Option<TypeId>,
}
impl Search<'_> {
    pub(super) fn start_equality(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        if frame.equality_hint.is_none()
            && let Some(decision) = self.cached_query(facts, frame)?
        {
            return Ok(Action::Complete(decision));
        }
        let left = match frame
            .inputs
            .context
            .resolved(&mut facts.types, frame.goal.subject())
        {
            Ok(ty) => ty,
            Err(infer::Error::Unresolved) => return self.infer_callable_equality(facts, frame),
            Err(error) => return Err(inference(error)),
        };
        let result = frame
            .goal
            .projection_result()
            .ok_or("missing equality result")?;
        let right = match frame.inputs.context.resolved(&mut facts.types, result) {
            Ok(ty) => Some(ty),
            Err(infer::Error::Unresolved) => None,
            Err(error) => return Err(inference(error)),
        };
        frame.normalization = Some(Box::new(Normalization::new(
            facts,
            frame,
            left,
            Purpose::Equality(Equality { right, left: None }),
        )?));
        Ok(Action::Continue)
    }

    pub(super) fn advance_equality(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        value: TypeId,
    ) -> Result<Action, String> {
        let normalization = frame
            .normalization
            .as_mut()
            .ok_or("missing equality query")?;
        let Purpose::Equality(equality) = &mut normalization.purpose else {
            return Err("invalid equality normalization purpose".into());
        };
        if equality.left.is_none() {
            equality.left = Some(value);
            if let Some(right) = equality.right {
                let before = normalization.fold.steps();
                normalization.step = normalization.fold.start(&mut facts.types, right)?;
                frame
                    .inputs
                    .context
                    .account_work(normalization.fold.steps().saturating_sub(before))
                    .map_err(inference)?;
                return Ok(Action::Continue);
            }
        }
        self.complete_equality(facts, frame, value)
            .map(Action::Complete)
    }

    fn complete_equality(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        value: TypeId,
    ) -> Result<Decision, String> {
        let normalization = frame
            .normalization
            .as_mut()
            .ok_or("missing equality completion")?;
        let Purpose::Equality(equality) = &mut normalization.purpose else {
            return Err("invalid equality completion purpose".into());
        };
        let left = frame
            .inputs
            .context
            .known(
                &facts.types,
                equality.left.ok_or("missing normalized equality subject")?,
            )
            .map_err(inference)?;
        let right = if equality.right.is_some() {
            frame
                .inputs
                .context
                .known(&facts.types, value)
                .map_err(inference)?
        } else {
            frame
                .goal
                .projection_result()
                .ok_or("missing equality result")?
        };
        match frame
            .inputs
            .context
            .equate_scoped_terms(&facts.types, left, right)
        {
            Ok(()) => {}
            Err(infer::Error::Mismatch | infer::Error::Occurs) => return Ok(Decision::Absent),
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                return Ok(Decision::Deferred);
            }
            Err(error) => return Err(inference(error)),
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved) => return Ok(Decision::Deferred),
            Err(error) => return Err(inference(error)),
        };
        let node = self.query.push(
            Node {
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::Equality),
                children: normalization.proof.into_iter().collect(),
                first: frame.first,
                end: normalization.end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            frame
                .equality_hint
                .as_ref()
                .map_or(frame.first..frame.first, |hint| hint.regions.clone()),
        );
        Ok(Decision::Proven(node))
    }
}
