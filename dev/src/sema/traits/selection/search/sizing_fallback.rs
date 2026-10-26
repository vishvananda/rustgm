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
        if let Some(kind) = kind {
            let declaration = catalog
                .validated_role(kind)?
                .ok_or("source sizing goal lost its actual validated declaration")?;
            if matches!(
                declaration.origin,
                crate::resolve::body_origin::DeclarationOrigin::Imported(_)
            ) {
                facts.associated.require_imported_role(declaration)?;
            }
        }
        if kind.is_some()
            && goal.constness() != Some(crate::support::constness::Constness::Ordinary)
        {
            return Err(
                "source sizing applications require their actual ordinary trait effect".into(),
            );
        }
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
        frame.require_nominal_formed(facts, goal.ty)?;
        let shape = self.query.builtins.physical_shape(facts, goal.ty)?;
        if !shape.well_formed {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        let valid = if kind == LangItem::Sized || shape.sized {
            shape.sized
        } else {
            match facts
                .types
                .kind(goal.ty)
                .ok_or("missing source sizing subject")?
            {
                Kind::Never
                | Kind::Primitive(_)
                | Kind::Reference { .. }
                | Kind::Pattern { .. }
                | Kind::Pointer { .. }
                | Kind::Function { .. }
                | Kind::FunctionItem { .. }
                | Kind::Closure { .. }
                | Kind::Slice(_)
                | Kind::Dynamic { .. } => true,
                Kind::Foreign(_) => kind == LangItem::PointeeSized,
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
                    self.source_tail_sizing(facts, frame, goal.ty, kind)?
                }
            }
        };
        if !valid {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        match kind {
            LangItem::Sized | LangItem::MetaSized | LangItem::PointeeSized => {
                let node = self.source_builtin_node(frame, goal, shape.sized);
                let selected = self.query.push_source_sizing(
                    facts,
                    node,
                    frame.scope,
                    frame.first..frame.first,
                )?;
                Ok(Some(Action::Complete(Decision::Proven(selected))))
            }
            _ => Err("invalid source sizing fallback kind".into()),
        }
    }
    /// Reuse the same actual terminal as metadata normalization. In particular,
    /// an opaque external terminal has unit metadata without becoming `MetaSized`.
    fn source_tail_sizing(
        &mut self,
        facts: &Declarations,
        frame: &mut Frame,
        mut tail: crate::types::TypeId,
        requested: LangItem,
    ) -> Result<bool, String> {
        loop {
            frame.inputs.context.account_work(1).map_err(inference)?;
            self.charge(&mut frame.inputs.context)?;
            match crate::abi::metadata::declared_step(&facts.types, &facts.aggregates, tail)? {
                crate::abi::metadata::Step::Tail(child) => tail = child,
                crate::abi::metadata::Step::Complete(Metadata::Thin) => {
                    return Ok(!matches!(facts.types.kind(tail), Some(Kind::Foreign(_)))
                        || requested == LangItem::PointeeSized);
                }
                crate::abi::metadata::Step::Complete(Metadata::Length | Metadata::VTable(_)) => {
                    return Ok(true);
                }
                crate::abi::metadata::Step::Complete(Metadata::Unknown(_)) => {
                    return Err("source sizing requires a proved metadata tail".into());
                }
            }
        }
    }
    pub(super) fn complete_source_builtin(
        &mut self,
        frame: &mut Frame,
        goal: Predicate,
        is_sized: bool,
    ) -> Action {
        let node = self.source_builtin_node(frame, goal, is_sized);
        let selected = self.query.push(node, frame.scope, frame.first..frame.first);
        Action::Complete(Decision::Proven(selected))
    }
    fn source_builtin_node(&self, frame: &mut Frame, goal: Predicate, is_sized: bool) -> Node {
        let mut sized = Vec::new();
        if is_sized && self.caller.retain_sized {
            sized.push(goal.ty);
        }
        let end = frame.available_region();
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
            sized,
        }
    }
}
