//! Actual automatic trait identities reuse normal scoped children and coinduction.
use super::component_markers::Components;
use super::{Action, Decision, Declarations, Frame, Goal, Predicate, Search, inference};
use crate::{
    resolve::model::DefId,
    sema::associated::OwnerKind,
    support::{bindings::LangItem, constness::Constness},
    types::{ItemId, Kind},
};

/// Consume an already selected structural proof; header identity alone is not proof.
pub(in crate::sema) fn predicate(
    facts: &Declarations,
    predicate: Predicate,
) -> Result<bool, String> {
    let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or("automatic evidence application is absent")?;
    let Some(owner) = facts
        .associated
        .owner(DefId(application.item.0))
        .and_then(|id| facts.associated.get(id))
    else {
        return Ok(false);
    };
    let OwnerKind::Trait {
        definition,
        auto_: true,
        constant: false,
        ..
    } = owner.kind
    else {
        return Ok(false);
    };
    // Version thirteen: an imported auto trait's own packet admitted it.
    if owner.source.is_none() {
        facts.associated.imported_automatic_trait(definition)?;
    }
    if definition != DefId(application.item.0)
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|a| a.len() != 0)
    {
        return Err("automatic evidence differs from its actual complete application".into());
    }
    Search::require_automatic_header(facts, application.item)?;
    Ok(true)
}

impl Search<'_> {
    fn automatic_identity(
        facts: &Declarations,
        goal: &Goal,
    ) -> Result<Option<(ItemId, bool)>, String> {
        let Some((item, arguments)) = goal.trait_arguments() else {
            return Ok(None);
        };
        let Some(owner) = facts
            .associated
            .owner(DefId(item.0))
            .and_then(|id| facts.associated.get(id))
        else {
            return Ok(None);
        };
        let OwnerKind::Trait {
            definition,
            auto_: true,
            constant,
            ..
        } = owner.kind
        else {
            return Ok(None);
        };
        if definition != DefId(item.0) {
            return Err("automatic trait declaration identity changed".into());
        }
        // This compiler-owned rule retains its existing separate semantic producer.
        if facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.language(LangItem::BikeshedGuaranteedNoDrop))
            .is_some_and(|d| d.definition == definition)
        {
            return Ok(None);
        }
        if constant || !arguments.is_empty() || goal.constness() != Some(Constness::Ordinary) {
            return Err(
                "automatic trait requires its ordinary complete zero-input application".into(),
            );
        }
        Ok(Some((item, owner.source.is_some())))
    }

    pub(super) fn declared_automatic_goal(
        facts: &Declarations,
        goal: &Goal,
    ) -> Result<bool, String> {
        Ok(Self::automatic_identity(facts, goal)?.is_some())
    }

    pub(super) fn automatic_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if let Some(action) = self.source_auto_result(facts, frame)? {
            return Ok(Some(action));
        }
        let Some((item, _)) = Self::automatic_identity(facts, &frame.goal)? else {
            return Ok(None);
        };
        if Self::source_auto_kind(facts, &frame.goal)?.is_some()
            || frame.deferred
            || frame.partial.is_some()
            || frame.selected.is_some()
            || frame.explicit_auto
        {
            return Ok(None);
        }
        Self::require_automatic_header(facts, item)?;
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
        let kind = facts
            .types
            .kind(goal.ty)
            .ok_or("automatic trait subject is absent")?;
        if facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|r| r.source_auto_override(item, kind))
        {
            if matches!(
                self.query.availability,
                crate::sema::traits::implementations::Availability::Validated
                    | crate::sema::traits::implementations::Availability::Executable
            ) && facts
                .generics
                .implementations
                .as_ref()
                .is_some_and(|r| !r.headers_validated())
            {
                return Err(
                    "automatic trait override requires validated implementation headers".into(),
                );
            }
            return Ok(Some(Action::Complete(Decision::Absent)));
        }
        frame.require_nominal_formed(facts, goal.ty)?;
        self.automatic_components(facts, frame, goal, kind)
            .map(Some)
    }

    fn require_automatic_header(facts: &Declarations, item: ItemId) -> Result<(), String> {
        let id = DefId(item.0);
        let owner_id = facts
            .associated
            .owner(id)
            .ok_or("automatic trait owner is absent")?;
        let schema = facts
            .associated
            .schema(id)
            .ok_or("automatic trait schema is incomplete")?;
        let header = facts
            .generics
            .traits
            .as_deref()
            .and_then(|t| t.header(item))
            .ok_or("automatic trait canonical header is incomplete")?;
        // An imported auto trait keeps its packet's admitted inputs; it is
        // re-checked by name under its version-thirteen profile.
        if facts
            .associated
            .get(owner_id)
            .is_some_and(|o| o.source.is_none())
        {
            facts.associated.imported_automatic_trait(id)?;
        } else {
            header.source_inputs()?;
        }
        if schema.self_type != header.self_type
            || schema.captures != 0
            || !schema.bounds.is_empty()
            || !header.parameters.is_empty()
            || !header.regions.is_empty()
            || facts
                .associated
                .owner_members(owner_id)
                .is_none_or(|m| !m.is_empty())
        {
            return Err(
                "automatic trait differs from its actual complete declaration contract".into(),
            );
        }
        Ok(())
    }

    fn automatic_components(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        goal: Predicate,
        kind: Kind,
    ) -> Result<Action, String> {
        match kind {
            Kind::Primitive(_)
            | Kind::Never
            | Kind::Function { .. }
            | Kind::FunctionItem { .. } => self.start_component_predicates(facts, frame, goal, &[]),
            Kind::Tuple(fields) => self.start_component_markers(
                facts,
                frame,
                goal,
                Components::List(fields),
                goal.bound,
            ),
            Kind::Pattern { base, .. } => self.start_component_markers(
                facts,
                frame,
                goal,
                Components::Element(base),
                goal.bound,
            ),
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => self.start_component_markers(
                facts,
                frame,
                goal,
                Components::Element(element),
                goal.bound,
            ),
            Kind::Reference { referent, .. } => self.start_component_markers(
                facts,
                frame,
                goal,
                Components::Element(referent),
                goal.bound,
            ),
            Kind::Pointer { pointee, .. } => self.start_component_markers(
                facts,
                frame,
                goal,
                Components::Element(pointee),
                goal.bound,
            ),
            Kind::Closure { .. } => self.start_capture_markers(facts, frame, goal),
            Kind::Adt(item, _) => {
                if facts.aggregates.get(item).is_none() {
                    return Ok(Action::Wait(
                        crate::sema::traits::selection::Dependency::Fields(item),
                    ));
                }
                if let Some(ty) = facts.aggregates.phantom_argument(&facts.types, goal.ty)? {
                    return self.start_component_markers(
                        facts,
                        frame,
                        goal,
                        Components::Element(ty),
                        goal.bound,
                    );
                }
                let mut predicates = Vec::new();
                let mut fields = facts.aggregates.semantic_fields(&facts.types, goal.ty)?;
                while let Some(ty) = fields.next(&mut facts.types)? {
                    fields.reserve(&mut predicates, 1)?;
                    predicates.push(Predicate {
                        ty,
                        bound: goal.bound,
                    });
                }
                frame
                    .inputs
                    .context
                    .account_work(fields.work())
                    .map_err(inference)?;
                drop(fields);
                self.start_component_predicates(facts, frame, goal, &predicates)
            }
            Kind::Parameter(_) | Kind::Projection(_) => Ok(Action::Complete(Decision::Absent)),
            Kind::Opaque(..) | Kind::Dynamic { .. } | Kind::Future { .. } | Kind::Foreign(_) => {
                Err("custom automatic trait requires its actual abstract/storage profile".into())
            }
        }
    }
}
