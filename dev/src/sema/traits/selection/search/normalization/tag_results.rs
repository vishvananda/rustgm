//! Logical discriminant results reuse the parent proof and existing continuation.
use super::{Completion, Normalization, ResultContinuation, written_results::ResultEquality};
use crate::{
    sema::{
        compiler_items::discriminant::{self, Output, Readiness},
        traits::selection::search::{Action, Declarations, Frame, Search, inference},
    },
    types::{projection::Application, substitute::Resolution},
};

pub(super) struct Pending {
    completed: Completion,
    written: ResultEquality,
    application: Application,
    selected: Option<discriminant::Selection>,
}

impl Search<'_> {
    pub(super) fn stage_discriminant_result(
        &mut self,
        facts: &mut Declarations,
        context: &mut super::super::Context,
        normalization: &mut Normalization,
        completed: Completion,
    ) -> Result<Option<Completion>, String> {
        let Some(mut application) = completed
            .opened
            .application
            .associated()
            .filter(|a| discriminant::is_result(facts, a.item))
        else {
            return Ok(Some(completed));
        };
        let goal = context
            .resolved_predicate(&mut facts.types, &completed.opened.goal)
            .map_err(inference)?;
        application.subject = goal.ty;
        application.trait_application = goal
            .bound
            .trait_parts()
            .ok_or("source discriminant lost its proved parent")?
            .0;
        let (written, completed) =
            self.resolve_result_equality(facts, context, normalization, completed)?;
        crate::sema::header_attributes::charge_source_retention(
            facts,
            1,
            size_of::<ResultContinuation>(),
        )
        .map_err(|e| e.to_string())?;
        normalization.inherited = Some(Box::new(ResultContinuation::Discriminant(Pending {
            completed,
            written,
            application,
            selected: None,
        })));
        Ok(None)
    }

    pub(super) fn advance_discriminant_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let normalization = frame
            .normalization
            .as_mut()
            .ok_or("missing discriminant normalization")?;
        let Some(ResultContinuation::Discriminant(pending)) =
            normalization.inherited.as_deref_mut()
        else {
            return Err("missing source discriminant result owner".into());
        };
        let resolution = match pending.written {
            ResultEquality::Type(ty) => Some(Resolution::Type(ty)),
            ResultEquality::Ambiguous => None,
            ResultEquality::Absent => {
                if pending.selected.is_none() {
                    match discriminant::for_result(facts, pending.application)
                        .map_err(|e| e.to_string())?
                    {
                        Readiness::Ready(selected) => pending.selected = Some(selected),
                        Readiness::Pending(id) => {
                            return Ok(Action::Wait(
                                super::super::Dependency::SourceResultContracts(id),
                            ));
                        }
                        Readiness::Unrelated => {
                            return Err("source discriminant lost its actual result".into());
                        }
                    }
                }
                match pending
                    .selected
                    .as_ref()
                    .ok_or("missing discriminant seal")?
                    .output(facts, pending.application)
                    .map_err(|e| e.to_string())?
                {
                    Output::Ready(ty) => Some(Resolution::Type(ty)),
                    Output::Fields(item) => {
                        return Ok(Action::Wait(super::super::Dependency::Fields(item)));
                    }
                    Output::Unresolved => Some(Resolution::Opaque),
                }
            }
        };
        let pending = normalization
            .inherited
            .take()
            .ok_or("missing completed discriminant result")?;
        let ResultContinuation::Discriminant(pending) = *pending else {
            return Err("discriminant publication used another result owner".into());
        };
        self.publish_normalization(
            facts,
            frame,
            pending.completed.equations_start,
            resolution,
            pending.completed.node,
        )?;
        Ok(Action::Continue)
    }
}
