//! Primitive methods compete with ordinary impl candidates and retain normal input proof.
use super::{Action, Decision, Declarations, Frame, Node, Search, Source, inference};
use crate::sema::primitive_methods::{self, Candidate};

impl Search<'_> {
    pub(super) fn language_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        if let Some(action) = Self::source_unsizing_result(facts, frame)? {
            return Ok(action);
        }
        if let Some(action) = self.source_sizing_result(facts, frame)? {
            return Ok(action);
        }
        if let Some(action) = self.source_tuple_result(facts, frame)? {
            return Ok(action);
        }
        if let Some(action) = self.source_destruct_result(facts, frame)? {
            return Ok(action);
        }
        if let Some(action) = self.source_freeze_result(facts, frame)? {
            return Ok(action);
        }
        if let Some(action) = self.clone_marker_result(facts, frame)? {
            return Ok(action);
        }
        match primitive_methods::candidate(
            facts,
            &mut frame.inputs.context,
            &frame.goal,
            frame.inference != super::frames::Inference::Disabled,
        ) {
            Ok(Candidate::Present(descriptor)) => {
                // An automatic method never suppresses a second matching impl.
                if frame.selected.is_some() {
                    return Ok(Action::Complete(Decision::Deferred));
                }
                let goal = frame
                    .inputs
                    .context
                    .resolved_predicate(&mut facts.types, &frame.goal)
                    .map_err(inference)?;
                let end = frame.available_region();
                let node = self.query.push(
                    Node {
                        debt_free: false,
                        cycle_dependent: false,
                        provisional_size: false,
                        goal,
                        source: Some(Source::LanguageOperation { descriptor }),
                        children: Vec::new(),
                        first: frame.first,
                        end,
                        equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                        outlives: Vec::new(),
                        type_outlives: Vec::new(),
                        sized: Vec::new(),
                    },
                    frame.scope,
                    end..end,
                );
                frame.selected = Some(super::frames::Chosen::Positive(node));
            }
            Ok(Candidate::Conditional {
                descriptor,
                prerequisite,
            }) => {
                return Ok(Self::start_comparison(frame, descriptor, prerequisite));
            }
            Ok(Candidate::Unrelated) => {}
            Ok(Candidate::Deferred)
            | Err(
                crate::types::infer::Error::Normalization | crate::types::infer::Error::Unresolved,
            ) => frame.deferred = true,
            Err(error) => return Err(inference(error)),
        }
        Ok(Action::Complete(
            if frame.deferred || frame.partial.is_some() {
                Decision::Deferred
            } else {
                frame
                    .selected
                    .map_or(Decision::Absent, super::frames::Chosen::decision)
            },
        ))
    }
}
