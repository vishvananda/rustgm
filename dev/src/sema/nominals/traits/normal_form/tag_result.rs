//! Discriminant data retains the already proved parent and requesting scope.
use super::{Declarations, Progress, Query, origin_results::Continuation, selection};
use crate::{
    sema::{
        compiler_items::discriminant::{self, Output, Readiness},
        traits::equalities,
    },
    types::{projection::Application, substitute::Resolution},
};

pub(super) struct Pending {
    selected: Option<discriminant::Selection>,
    application: Application,
}

impl Query {
    pub(super) fn begin_discriminant(
        mut self,
        facts: &mut Declarations,
    ) -> Result<Progress, &'static str> {
        match self.equality(facts)? {
            equalities::Selection::Proven(evidence) => {
                let ty = evidence.append_to(&mut self.state.obligations)?;
                return self.finish(facts, Some(Resolution::Type(ty)));
            }
            equalities::Selection::Ambiguous => {
                return Err("ambiguous associated type equality assumptions");
            }
            equalities::Selection::Absent => {}
        }
        let mut application = self
            .alias
            .associated()
            .ok_or("missing actual discriminant application")?;
        let goal = self
            .state
            .context
            .resolved_predicate(&mut facts.types, &self.goal)
            .map_err(|_| "source discriminant input did not complete")?;
        application.subject = goal.ty;
        application.trait_application = goal
            .bound
            .trait_parts()
            .ok_or("source discriminant lost its proved parent")?
            .0;
        crate::sema::header_attributes::charge_source_retention(
            facts,
            1,
            size_of::<Continuation>(),
        )
        .map_err(|_| "source discriminant continuation exceeds its budget")?;
        self.inherited = Some(Box::new(Continuation::Discriminant(Pending {
            selected: None,
            application,
        })));
        self.advance_discriminant(facts)
    }

    pub(super) fn advance_discriminant(
        mut self,
        facts: &mut Declarations,
    ) -> Result<Progress, &'static str> {
        let Some(Continuation::Discriminant(pending)) = self.inherited.as_deref_mut() else {
            return Err("missing source discriminant continuation");
        };
        if pending.selected.is_none() {
            match discriminant::for_result(facts, pending.application)
                .map_err(|_| "invalid source discriminant declaration use")?
            {
                Readiness::Ready(selected) => pending.selected = Some(selected),
                Readiness::Pending(id) => {
                    return self.wait_discriminant(
                        facts,
                        selection::Dependency::SourceResultContracts(id),
                    );
                }
                Readiness::Unrelated => {
                    return Err("source discriminant request lost its actual member");
                }
            }
        }
        match pending
            .selected
            .as_ref()
            .ok_or("missing source discriminant seal")?
            .output(facts, pending.application)
            .map_err(|_| "invalid current source discriminant result")?
        {
            Output::Ready(ty) => self.finish(facts, Some(Resolution::Type(ty))),
            Output::Fields(item) => {
                self.wait_discriminant(facts, selection::Dependency::Fields(item))
            }
            Output::Unresolved => self.finish(facts, Some(Resolution::Opaque)),
        }
    }

    fn wait_discriminant(
        self,
        facts: &Declarations,
        dependency: selection::Dependency,
    ) -> Result<Progress, &'static str> {
        crate::sema::header_attributes::charge_source_retention(facts, 1, size_of::<Self>())
            .map_err(|_| "source discriminant suspension exceeds its budget")?;
        Ok(Progress::Pending(dependency, Box::new(self)))
    }
}
