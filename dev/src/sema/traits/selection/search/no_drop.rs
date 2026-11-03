//! The actual `NoDrop` marker proves components or ordinary source Copy, never a destructor guess.
use super::component_markers::Components;
use super::{Action, Decision, Declarations, Frame, Goal, Search, inference};
use crate::{
    support::{bindings::LangItem, constness::Constness},
    types::{GenericArgs, ItemId, Kind, infer, parameters::predicates::Bound, traits::Application},
};
/// Confinement of already selected builtin evidence, not a replacement for its proof.
pub(in crate::sema) fn no_drop_predicate(
    facts: &Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
) -> Result<bool, String> {
    let Some((id, constness)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("invalid source NoDrop application")?;
    let empty = facts
        .types
        .arguments(application.arguments)
        .ok_or("invalid source NoDrop arguments")?
        .next()
        .is_none();
    role(facts, application.item, empty, Some(constness))
}
fn role(
    facts: &Declarations,
    item: ItemId,
    empty: bool,
    constness: Option<Constness>,
) -> Result<bool, String> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some(declaration) = catalog.role(LangItem::BikeshedGuaranteedNoDrop)? else {
        return Ok(false);
    };
    if item.0 != declaration.definition.0 {
        return Ok(false);
    }
    catalog.validated_role(LangItem::BikeshedGuaranteedNoDrop)?;
    if matches!(
        declaration.origin,
        crate::resolve::body_origin::DeclarationOrigin::Imported(_)
    ) {
        facts.associated.require_imported_role(declaration)?;
    }
    if !empty {
        return Err("source NoDrop requires zero actual trait arguments".into());
    }
    Ok(constness == Some(Constness::Ordinary))
}
impl Search<'_> {
    pub(super) fn source_no_drop_goal(facts: &Declarations, goal: &Goal) -> Result<bool, String> {
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(false);
        };
        role(facts, item, arguments.is_empty(), goal.constness())
    }
    pub(super) fn source_no_drop_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if frame.selected.is_some()
            || frame.deferred
            || frame.partial.is_some()
            || !Self::source_no_drop_goal(facts, &frame.goal)?
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
        let components = match facts.types.kind(goal.ty) {
            Some(Kind::Reference { .. }) => {
                return Ok(Some(self.complete_source_builtin(frame, goal, false)));
            }
            Some(Kind::Adt(item, _))
                if facts
                    .source_bindings
                    .as_deref()
                    .ok_or("source NoDrop catalog disappeared")?
                    .validated_language(LangItem::ManuallyDrop)?
                    .is_some_and(|declaration| declaration.definition.0 == item.0) =>
            {
                return Ok(Some(self.complete_source_builtin(frame, goal, false)));
            }
            Some(Kind::Tuple(fields)) => Some(Components::List(fields)),
            Some(Kind::Array { element, .. } | Kind::ArrayTerm { element, .. }) => {
                Some(Components::Element(element))
            }
            _ => None,
        };
        let (components, bound) = if let Some(components) = components {
            (components, goal.bound)
        } else {
            let copy = crate::sema::compiler_items::copy_declaration(facts)?
                .ok_or("source NoDrop fallback requires actual source Copy")?;
            let application = facts.types.intern_trait_application(Application {
                item: ItemId(copy.0),
                arguments: GenericArgs::EMPTY,
            })?;
            (Components::Element(goal.ty), Bound::Trait(application))
        };
        self.start_component_markers(facts, frame, goal, components, bound)
            .map(Some)
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/no_drop_sources.rs"]
mod tests;
