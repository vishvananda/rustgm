//! Source sizing fallbacks retain the actual trait goal and ordinary formation duties.
use super::{
    Action, Decision, Declarations, Frame, Goal, Node, Predicate, Search, Source, inference,
};
use crate::{
    abi::metadata::Metadata,
    resolve::model::DefId,
    support::bindings::LangItem,
    types::{Kind, aggregate::MetadataPreparation},
};
impl Search<'_> {
    pub(super) fn source_sizing_kind(
        facts: &Declarations,
        goal: &Goal,
    ) -> Result<Option<LangItem>, String> {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(None);
        };
        let kind = catalog.sizing_kind(DefId(item.0))?;
        if kind.is_some() && !arguments.is_empty() {
            return Err("source sizing application has unexpected arguments".into());
        }
        Ok(kind)
    }
    pub(super) fn source_sizing_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some(kind) = Self::source_sizing_kind(facts, &frame.goal)? else {
            return Ok(None);
        };
        // A pending or selected source candidate remains authoritative here.
        if frame.deferred || frame.partial.is_some() || frame.selected.is_some() {
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
        let shape = self.query.builtins.check(facts, goal.ty)?;
        if self
            .query
            .builtins
            .take_trait_proof()
            .is_some_and(|proof| !proof.is_empty())
        {
            return Err(
                "source sizing formation requires retained trait prerequisite obligations".into(),
            );
        }
        if !shape.well_formed {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        let valid = if shape.sized {
            true
        } else {
            match facts
                .types
                .kind(goal.ty)
                .ok_or("missing source sizing subject")?
            {
                Kind::Never
                | Kind::Primitive(_)
                | Kind::Reference { .. }
                | Kind::Pointer { .. }
                | Kind::Function { .. }
                | Kind::FunctionItem { .. }
                | Kind::Closure { .. }
                | Kind::Slice(_)
                | Kind::Dynamic { .. } => true,
                Kind::Array { .. } | Kind::ArrayTerm { .. } | Kind::Parameter(_) => false,
                Kind::Projection(_) | Kind::Opaque(..) => {
                    return Err(
                        "source sizing requires a proved normalized or declared subject".into(),
                    );
                }
                Kind::Adt(..) | Kind::Tuple(_) => {
                    match facts
                        .aggregates
                        .prepare_physical_metadata(&mut facts.types, goal.ty)?
                    {
                        MetadataPreparation::Fields(item) => {
                            return Ok(Some(Action::Wait(super::Dependency::Fields(item))));
                        }
                        MetadataPreparation::Unresolved(_) => {
                            return Err("source sizing requires a proved metadata tail".into());
                        }
                        MetadataPreparation::Ready => {}
                    }
                    match facts.layouts.prepared_pointer_metadata(
                        &facts.types,
                        &facts.aggregates,
                        goal.ty,
                    )? {
                        Metadata::Thin | Metadata::Length | Metadata::VTable(_) => true,
                        Metadata::Unknown(_) => {
                            return Err("source sizing requires a proved metadata tail".into());
                        }
                    }
                }
            }
        };
        if !valid {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        match kind {
            LangItem::MetaSized | LangItem::PointeeSized => {
                Ok(Some(self.complete_source_builtin(frame, goal, shape.sized)))
            }
            _ => Err("invalid source sizing fallback kind".into()),
        }
    }
    pub(super) fn complete_source_builtin(
        &mut self,
        frame: &mut Frame,
        goal: Predicate,
        is_sized: bool,
    ) -> Action {
        let mut sized = Vec::new();
        if is_sized && self.caller.retain_sized {
            sized.push(goal.ty);
        }
        let end = frame.available_region();
        let node = self.query.push(
            Node {
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
                sized,
            },
            frame.scope,
            frame.first..frame.first,
        );
        Action::Complete(Decision::Proven(node))
    }
}
