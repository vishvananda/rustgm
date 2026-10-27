//! The actual source Pointee trait is universal; metadata equality is separate.
use super::{Action, Decision, Declarations, Frame, Node, Search, Source, inference};
use crate::{
    sema::compiler_items::{discriminant, pointee},
    support::bindings::LangItem,
    types::infer,
};

impl Search<'_> {
    pub(super) fn source_pointee_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some((item, _)) = frame.goal.trait_arguments() else {
            return Ok(None);
        };
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let discriminant = catalog
            .role(LangItem::DiscriminantKind)?
            .is_some_and(|role| role.definition.0 == item.0);
        if !discriminant
            && catalog
                .role(LangItem::PointeeTrait)?
                .is_none_or(|role| role.definition.0 != item.0)
        {
            return Ok(None);
        }
        // A genuine caller premise retains its abstract associated result.
        // Universal source fallback must not replace it with an automatic equality.
        if frame.selected.is_some() {
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
        let source = if discriminant {
            match discriminant::select(facts, goal).map_err(|error| error.to_string())? {
                discriminant::Readiness::Unrelated => return Ok(None),
                discriminant::Readiness::Pending(member) => {
                    return Ok(Some(Action::Wait(
                        super::Dependency::SourceResultContracts(member),
                    )));
                }
                discriminant::Readiness::Ready(selected) => Source::Discriminant(selected),
            }
        } else {
            match pointee::select(facts, goal).map_err(|error| error.to_string())? {
                pointee::Readiness::Unrelated => return Ok(None),
                pointee::Readiness::Pending(member) => {
                    return Ok(Some(Action::Wait(
                        super::Dependency::SourceResultContracts(member),
                    )));
                }
                pointee::Readiness::Ready(selected) => Source::Pointee(selected),
            }
        };
        if frame.deferred || frame.partial.is_some() {
            return Ok(Some(Action::Complete(Decision::Deferred)));
        }
        let end = frame.available_region();
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(source),
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
