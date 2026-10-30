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

pub(super) fn bound(facts: &mut Declarations, marker: AutoTrait) -> Result<Bound, String> {
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("source Freeze requires actual source bindings")?
        .validated_language(role(marker).ok_or("source auto trait marker is not represented")?)?
        .ok_or("source Freeze declaration is absent")?;
    let payments =
        crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
            .map_err(|e| e.to_string())?;
    Ok(Bound::Trait(
        facts.types.intern_trait_application_observed::<true>(
            Application {
                item: ItemId(declaration.definition.0),
                arguments: GenericArgs::EMPTY,
            },
            &|steps, bytes| payments.charge(steps, bytes),
        )?,
    ))
}
impl Search<'_> {
    pub(super) fn source_auto_kind(
        facts: &Declarations,
        goal: &Goal,
    ) -> Result<Option<AutoTrait>, String> {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(None);
        };
        for marker in [
            AutoTrait::Freeze,
            AutoTrait::Send,
            AutoTrait::Sync,
            AutoTrait::Unpin,
            AutoTrait::UnsafeUnpin,
        ] {
            let role = role(marker).ok_or("source auto trait marker is not represented")?;
            if catalog
                .language(role)
                .is_none_or(|d| d.definition != DefId(item.0))
            {
                continue;
            }
            catalog.validated_language(role)?;
            if !arguments.is_empty() || goal.constness() != Some(Constness::Ordinary) {
                return Err("source auto trait requires an ordinary complete application".into());
            }
            return Ok(Some(marker));
        }
        Ok(None)
    }
    pub(super) fn source_auto_goal(facts: &Declarations, goal: &Goal) -> Result<bool, String> {
        Ok(Self::source_auto_kind(facts, goal)?.is_some())
    }
    pub(super) fn source_auto_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let Some(marker) = Self::source_auto_kind(facts, &frame.goal)? else {
            return Ok(None);
        };
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
        let (overridden, work) = facts
            .generics
            .implementations
            .as_ref()
            .map_or((false, 0), |registry| {
                registry.source_auto_override(item, kind)
            });
        frame.inputs.context.account_work(work).map_err(inference)?;
        if overridden {
            if matches!(
                self.query.availability,
                crate::sema::traits::implementations::Availability::Validated
                    | crate::sema::traits::implementations::Availability::Executable
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
                marker,
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

pub(super) const fn role(marker: AutoTrait) -> Option<LangItem> {
    match marker {
        AutoTrait::Freeze => Some(LangItem::Freeze),
        AutoTrait::Send => Some(LangItem::Send),
        AutoTrait::Sync => Some(LangItem::Sync),
        AutoTrait::Unpin => Some(LangItem::Unpin),
        AutoTrait::UnsafeUnpin => Some(LangItem::UnsafeUnpin),
        _ => None,
    }
}
pub(super) fn has_source(facts: &Declarations, marker: AutoTrait) -> bool {
    role(marker).is_some_and(|role| {
        facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.language(role))
            .is_some()
    })
}
