//! Actual source Freeze keeps canonical identity through every structural child.
use super::{Action, Decision, Declarations, Frame, Goal, Node, Search, Source, inference};
use crate::{
    resolve::model::DefId,
    support::{bindings::LangItem, constness::Constness},
    types::{
        GenericArgs, ItemId,
        auto_traits::{AutoTrait, Obligation},
        parameters::predicates::Bound,
        traits::Application,
    },
};

pub(super) fn bound(facts: &mut Declarations) -> Result<Bound, String> {
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("source Freeze requires actual source bindings")?
        .validated_language(LangItem::Freeze)?
        .ok_or("source Freeze declaration is absent")?;
    Ok(Bound::Trait(facts.types.intern_trait_application(
        Application {
            item: ItemId(declaration.definition.0),
            arguments: GenericArgs::EMPTY,
        },
    )?))
}
impl Search<'_> {
    pub(super) fn source_freeze_goal(facts: &Declarations, goal: &Goal) -> Result<bool, String> {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(false);
        };
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(false);
        };
        let Some(declaration) = catalog.language(LangItem::Freeze) else {
            return Ok(false);
        };
        if DefId(item.0) != declaration.definition {
            return Ok(false);
        }
        catalog.validated_language(LangItem::Freeze)?;
        if !arguments.is_empty() || goal.constness() != Some(Constness::Ordinary) {
            return Err("source Freeze requires an ordinary empty-argument application".into());
        }
        Ok(true)
    }
    pub(super) fn source_freeze_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if !Self::source_freeze_goal(facts, &frame.goal)? {
            return Ok(None);
        }
        if frame.deferred || frame.partial.is_some() || frame.selected.is_some() {
            return Ok(None);
        }
        // An applicable explicit header family replaces the structural default,
        // including when its declared prerequisites cannot be discharged.
        if frame.explicit_auto {
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
        let (item, _) = frame
            .goal
            .trait_arguments()
            .ok_or("source Freeze lost its application")?;
        let kind = facts
            .types
            .kind(goal.ty)
            .ok_or("source Freeze lost its subject")?;
        if facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|registry| registry.source_auto_override(item, kind))
        {
            if matches!(
                self.query.availability,
                crate::sema::traits::implementations::Availability::Validated
            ) && facts
                .generics
                .implementations
                .as_ref()
                .is_some_and(|registry| !registry.headers_validated())
            {
                return Err(
                    "source Freeze override requires validated implementation headers".into(),
                );
            }
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        frame.require_nominal_formed(facts, goal.ty)?;
        let shape = self.query.builtins.physical_shape(facts, goal.ty)?;
        if !shape.well_formed {
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        match self.auto_condition(
            facts,
            frame,
            Obligation {
                ty: goal.ty,
                marker: AutoTrait::Freeze,
            },
        )? {
            std::ops::ControlFlow::Break(action) => return Ok(Some(action)),
            std::ops::ControlFlow::Continue(false) => {
                return Ok(Some(Action::Complete(Decision::Absent)));
            }
            std::ops::ControlFlow::Continue(true) => {}
        }
        let children = frame
            .autos
            .as_mut()
            .map_or_else(Vec::new, |auto| std::mem::take(&mut auto.children));
        let end = frame.available_region();
        let node = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(Source::Builtin),
                children,
                first: frame.first,
                end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            frame.first..frame.first,
        );
        Ok(Some(Action::Complete(Decision::Proven(node))))
    }
}
