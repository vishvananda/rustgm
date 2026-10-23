//! Marker premises check abstract Clone applications; they never select a method body.
use super::{Action, Decision, Declarations, Frame, Goal, Node, Search, Source, inference};
use crate::{
    support::{bindings::LangItem, constness::Constness},
    types::{
        infer,
        parameters::predicates::{Bound, Predicate},
    },
};

fn source_callable_role(facts: &Declarations, goal: &Goal) -> Result<bool, String> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some((item, arguments)) = goal.trait_arguments() else {
        return Ok(false);
    };
    let Some(role) = [LangItem::Copy, LangItem::Clone].into_iter().find(|&role| {
        catalog
            .language(role)
            .is_some_and(|declaration| declaration.definition.0 == item.0)
    }) else {
        return Ok(false);
    };
    catalog.validated_language(role)?;
    if !arguments.is_empty() {
        return Err("source callable copying role has unexpected arguments".into());
    }
    // A nonordinary Clone application can be valid Rust. This ordinary marker
    // profile supplies neither its const permission nor an executable method.
    Ok(goal.constness() == Some(Constness::Ordinary))
}

impl Search<'_> {
    pub(super) fn source_callable_marker_goal(
        facts: &Declarations,
        frame: &mut Frame,
    ) -> Result<bool, String> {
        if !source_callable_role(facts, &frame.goal)? {
            return Ok(false);
        }
        Ok(matches!(
            frame
                .inputs
                .context
                .head(&facts.types, frame.goal.subject())
                .map_err(inference)?,
            Some(
                infer::Constructor::Function { .. }
                    | infer::Constructor::FunctionItem(..)
                    | infer::Constructor::FunctionItemArgs(..)
            )
        ))
    }

    /// Compiler-defined Copy/Clone markers for actual function items/pointers.
    /// Input formation remains independent; signature inputs/results need not Copy.
    pub(super) fn source_callable_marker_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if frame.selected.is_some()
            || frame.deferred
            || frame.partial.is_some()
            || !Self::source_callable_marker_goal(facts, frame)?
        {
            return Ok(None);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(infer::Error::Unresolved | infer::Error::Normalization) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        frame.require_nominal_formed(facts, goal.ty)?;
        Ok(Some(self.complete_source_builtin(frame, goal, false)))
    }

    pub(super) fn clone_marker_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some(canonical) = facts.cloning.as_deref().map(|r| r.canonical) else {
            return Ok(None);
        };
        let Some((item, arguments)) = frame.goal.trait_arguments() else {
            return Ok(None);
        };
        if item.0 != canonical.definition.0
            || !arguments.is_empty()
            || frame.selected.is_some()
            || frame.deferred
            || frame.partial.is_some()
        {
            return Ok(None);
        }
        let goal = match frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
        {
            Ok(goal) => goal,
            Err(crate::types::infer::Error::Unresolved) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        // All real candidates and their header waits were handled first. This
        // result carries no implementation identity; concrete dispatch rejects it.
        for bound in [Bound::Clone, Bound::Sized] {
            if !self
                .query
                .builtins
                .prove(facts, Predicate { ty: goal.ty, bound })?
            {
                return Ok(Some(Action::Complete(Decision::Absent)));
            }
        }
        let end = frame.available_region();
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::Builtin),
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
        Ok(Some(Action::Complete(Decision::Proven(node))))
    }
}
