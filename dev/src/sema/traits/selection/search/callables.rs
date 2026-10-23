//! Language callable evidence owns its equations and witnesses, without a source impl.
use super::{Action, Decision, Declarations, Frame, Node, Search, Source, inference};
use crate::{
    resolve::model::DefId,
    sema::traits::callables::{self, Matched},
    types::infer,
};

use super::mechanical::Pending;

impl Search<'_> {
    /// Input inference comes from the selected language signature, never from
    /// treating an arbitrary associated projection as an injective constructor.
    pub(super) fn infer_callable_equality(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        if frame.equality_hint.is_some() {
            return Ok(
                if self.query.refining && frame.scope == self.query.input_scope() {
                    Action::Yield
                } else {
                    Action::Complete(Decision::Deferred)
                },
            );
        }
        let root = frame.goal.subject();
        if frame
            .inputs
            .context
            .projection_item(&facts.types, root)
            .map_err(inference)?
            .is_none()
        {
            return Ok(Action::Complete(Decision::Deferred));
        }
        let application = frame
            .inputs
            .context
            .projection_application(&facts.types, root)
            .map_err(inference)?;
        if facts.associated.callable_output() != Some(DefId(application.item.0))
            || !application.arguments.is_empty()
        {
            return Ok(Action::Complete(Decision::Deferred));
        }
        let first = frame.available_region().max(self.query.allocations.floor());
        let matched = match callables::match_signature(
            facts,
            &mut frame.inputs.context,
            &application.predicate,
            first,
        ) {
            Ok(matched) => matched,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                return Ok(Action::Complete(Decision::Deferred));
            }
            Err(infer::Error::Mismatch | infer::Error::Occurs) => {
                return Ok(Action::Complete(Decision::Absent));
            }
            Err(error) => return Err(inference(error)),
        };
        let output = frame
            .goal
            .projection_result()
            .ok_or("missing callable equality result")?;
        match frame
            .inputs
            .context
            .equate_scoped_terms(&facts.types, matched.output, output)
        {
            Ok(()) => {}
            Err(infer::Error::Mismatch | infer::Error::Occurs) => {
                return Ok(Action::Complete(Decision::Absent));
            }
            Err(infer::Error::Normalization) => return Ok(Action::Complete(Decision::Deferred)),
            Err(error) => return Err(inference(error)),
        }
        self.query.allocations.record(matched.regions.end);
        self.query.reserve(frame.scope, matched.regions.clone())?;
        frame.equality_hint = Some(Box::new(matched));
        // Ordinary normalization still proves the complete original application.
        Ok(Action::Continue)
    }

    pub(super) fn start_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some((item, _)) = frame.goal.trait_arguments() else {
            return Ok(None);
        };
        let Some(kind) = facts.associated.callable(DefId(item.0)) else {
            return Ok(None);
        };
        // Completed normalization can retain an opaque associated result. Its
        // caller and declared result permissions have already been tried. Such
        // a subject cannot acquire a language implementation from an unknown
        // physical signature, nor keep a stronger call kind spuriously pending.
        let normalized = frame.normalized_inputs.is_some()
            || frame
                .normalization
                .as_deref()
                .and_then(super::normalization::Normalization::original_inputs)
                .is_some();
        if normalized
            && frame
                .inputs
                .context
                .projection_item(&facts.types, frame.goal.subject())
                .map_err(inference)?
                .is_some()
        {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        if let Some(action) = self.start_forwarding(facts, frame, kind)? {
            return Ok(Some(action));
        }
        let first = frame.available_region().max(self.query.allocations.floor());
        match callables::match_signature(facts, &mut frame.inputs.context, &frame.goal, first) {
            Ok(matched) => {
                self.query.allocations.record(matched.regions.end);
                self.query.reserve(frame.scope, matched.regions.clone())?;
                frame.language = Some(Box::new(Pending::Direct(matched)));
                self.complete_callable(facts, frame).map(Some)
            }
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                Ok(Some(Action::Complete(Decision::Deferred)))
            }
            Err(infer::Error::Mismatch | infer::Error::Occurs) => {
                Ok(Some(Action::Complete(Decision::Absent)))
            }
            Err(error) => Err(inference(error)),
        }
    }

    pub(super) fn complete_callable(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        if matches!(frame.language.as_deref(), Some(Pending::Forward(_))) {
            return self.complete_forwarding(facts, frame);
        }
        let Some(Pending::Direct(matched)) = frame.language.as_deref() else {
            return Err("missing callable equations".into());
        };
        match matched.permitted(&facts.types).map_err(inference)? {
            Some(true) => {}
            Some(false) => return Ok(Action::Complete(Decision::Absent)),
            None => {
                return Ok(
                    if self.query.refining && frame.scope == self.query.input_scope() {
                        Action::Yield
                    } else {
                        Action::Complete(Decision::Deferred)
                    },
                );
            }
        }
        let Matched {
            output, regions, ..
        } = matched;
        let resolved = frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
            .and_then(|goal| {
                frame
                    .inputs
                    .context
                    .resolved(&mut facts.types, *output)
                    .map(|output| (goal, output))
            });
        let (goal, output) = match resolved {
            Ok(values) => values,
            Err(infer::Error::Unresolved)
                if self.query.refining && frame.scope == self.query.input_scope() =>
            {
                return Ok(Action::Yield);
            }
            Err(infer::Error::Unresolved) => return Ok(Action::Complete(Decision::Deferred)),
            Err(error) => return Err(inference(error)),
        };
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::Callable { output }),
                children: Vec::new(),
                first: frame.first,
                end: regions.end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            regions.clone(),
        );
        Ok(Action::Complete(Decision::Proven(node)))
    }
}
