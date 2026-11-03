//! A completed header and real Assume value supply no unproved layout relation.
use super::{Action, Decision, Declarations, Frame, Search, inference};
use crate::{
    support::bindings::LangItem,
    types::{arguments::Argument, infer},
};
impl Search<'_> {
    pub(super) fn source_transmutability_result(
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some((item, _)) = frame.goal.trait_arguments() else {
            return Ok(None);
        };
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(role) = catalog.language(LangItem::TransmuteTrait) else {
            return Ok(None);
        };
        if item.0 != role.definition.0 || frame.selected.is_some() {
            return Ok(None);
        }
        catalog.validated_language(LangItem::TransmuteTrait)?;
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
        let (application, _) = goal
            .bound
            .trait_parts()
            .ok_or("transmutability lost its actual trait application")?;
        let application = facts
            .types
            .trait_application(application)
            .ok_or("transmutability lost its canonical application")?;
        let mut arguments = facts
            .types
            .arguments(application.arguments)
            .ok_or("transmutability lost its canonical arguments")?;
        let (Some(Argument::Type(_source)), Some(Argument::Const(value)), None) =
            (arguments.next(), arguments.next(), arguments.next())
        else {
            return Err("transmutability requires its actual source type and Assume value".into());
        };
        let _assumptions = crate::sema::compiler_items::transmutability::decode(facts, value)?;
        Err(
            "source transmutability requires its all-values layout, validity and lifetime relation"
                .into(),
        )
    }
}
