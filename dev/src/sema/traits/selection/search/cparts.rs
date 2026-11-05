//! Const Destruct retains ordinary const Drop and fully substituted field proofs.
use super::component_markers::Components;
use super::{Action, Decision, Declarations, Frame, Predicate, Search, inference};
use crate::{
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, Kind,
        aggregate::{FieldDestruction, Kind as AggregateKind},
        parameters::predicates::Bound,
        traits::Application,
    },
};
impl Search<'_> {
    pub(super) fn const_destruct_components(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        goal: Predicate,
    ) -> Result<Action, String> {
        match facts.types.kind(goal.ty) {
            Some(Kind::Tuple(fields)) => self.start_component_markers(
                facts,
                frame,
                goal,
                Components::List(fields),
                goal.bound,
            ),
            Some(
                Kind::Array { element, .. }
                | Kind::ArrayTerm { element, .. }
                | Kind::Slice(element),
            ) => self.start_component_markers(
                facts,
                frame,
                goal,
                Components::Element(element),
                goal.bound,
            ),
            Some(Kind::Closure { .. }) => self.start_capture_markers(facts, frame, goal),
            Some(Kind::Adt(item, _)) => {
                let definition = *facts
                    .aggregates
                    .get(item)
                    .ok_or("const Destruct nominal fields are pending")?;
                let policy = facts
                    .aggregates
                    .destruction(item)
                    .ok_or("const Destruct nominal policy is pending")?;
                let mut predicates = Vec::new();
                if policy != FieldDestruction::Never {
                    if facts.destructors.has_declared(item) {
                        if facts.destructors.item(item).is_none() {
                            return Ok(Action::Complete(Decision::Deferred));
                        }
                        let drop = facts
                            .source_bindings
                            .as_deref()
                            .ok_or("const Destruct lost its source catalog")?
                            .validated_language(LangItem::Drop)?
                            .ok_or("const Destruct requires its actual Drop declaration")?;
                        let application = facts.types.intern_trait_application(Application {
                            item: ItemId(drop.definition.0),
                            arguments: GenericArgs::EMPTY,
                        })?;
                        let (_, effect) = goal
                            .bound
                            .trait_parts()
                            .ok_or("const Destruct lost its effect")?;
                        predicates.push(Predicate {
                            ty: goal.ty,
                            bound: Bound::trait_application(application, effect),
                        });
                    }
                    if policy == FieldDestruction::Fields && definition.kind != AggregateKind::Union
                    {
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
                    }
                }
                self.start_component_predicates(facts, frame, goal, &predicates)
            }
            Some(_) => Ok(self.complete_source_builtin(frame, goal, false)),
            None => Err("const Destruct subject is missing".into()),
        }
    }
}
