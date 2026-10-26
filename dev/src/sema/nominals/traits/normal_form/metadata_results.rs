//! Source metadata uses this query's existing caller, region frontier and proof.
use super::{Declarations, Progress, Query, origin_results::Continuation, selection};
use crate::{
    sema::compiler_items::pointee::{self, MetadataPlan, MetadataStep},
    types::{
        TypeId,
        infer::predicates::Predicate as Goal,
        parameters::predicates::{Bound, Predicate},
        substitute::Resolution,
    },
};
pub(super) struct Pending {
    plan: MetadataPlan,
    goal: Option<Goal>,
    pending: Option<Box<selection::Pending>>,
}
impl Pending {
    pub(super) fn allocation_end(&self, first: u32) -> u32 {
        self.pending
            .as_ref()
            .map_or(first, |pending| first.max(pending.allocation_end()))
    }
}
impl Query {
    pub(super) fn prepare_metadata(
        &mut self,
        facts: &mut Declarations,
        selected: &pointee::Selection,
    ) -> Result<(), &'static str> {
        let mut application = self
            .alias
            .associated()
            .ok_or("source Pointee evidence is not a Metadata result")?;
        let goal = self
            .state
            .context
            .resolved_predicate(&mut facts.types, &self.goal)
            .map_err(|_| "source Metadata lacks complete selected input")?;
        let (trait_application, _) = goal
            .bound
            .trait_parts()
            .ok_or("source Metadata has no parent trait")?;
        application.subject = goal.ty;
        application.trait_application = trait_application;
        crate::sema::header_attributes::charge_source_retention(
            facts,
            1,
            size_of::<Continuation>(),
        )
        .map_err(|_| "source Metadata continuation exceeds its budget")?;
        let plan = MetadataPlan::new(facts, selected, application)
            .map_err(|_| "source Metadata has an invalid current declaration")?;
        #[cfg(test)]
        crate::sema::copy_trace::record(|| format!("source-metadata-outer {application:?}"));
        self.inherited = Some(Box::new(Continuation::Metadata(Pending {
            plan,
            goal: None,
            pending: None,
        })));
        Ok(())
    }
    pub(super) fn advance_metadata(
        mut self,
        facts: &mut Declarations,
    ) -> Result<Progress, &'static str> {
        let Some(Continuation::Metadata(pending)) = self.inherited.as_deref_mut() else {
            return Err("missing source Metadata continuation");
        };
        match pending
            .plan
            .advance(facts)
            .map_err(|_| "invalid or exhausted source Metadata plan")?
        {
            MetadataStep::Ready(ty) => self.finish(facts, Some(Resolution::Type(ty))),
            MetadataStep::Fields(item) => {
                self.wait_metadata(facts, selection::Dependency::Fields(item))
            }
            MetadataStep::Sized(ty) => self.metadata_sized(facts, ty),
        }
    }
    fn metadata_sized(
        mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<Progress, &'static str> {
        let Some(Continuation::Metadata(pending)) = self.inherited.as_deref_mut() else {
            return Err("missing source Metadata Sized obligation");
        };
        if pending.goal.is_none() {
            pending.goal = Some(self.state.projection_goal(
                facts,
                Predicate {
                    ty,
                    bound: Bound::Sized,
                },
            )?);
        }
        let goal = pending
            .goal
            .as_ref()
            .ok_or("missing source Metadata Sized goal")?;
        let progress = if let Some(query) = pending.pending.take() {
            query.resume(facts, &mut self.state.context, goal, self.frontier)
        } else {
            crate::sema::header_attributes::charge_source_retention(
                facts,
                1,
                size_of::<selection::Pending>()
                    + size_of::<crate::sema::traits::assumptions::Assumptions>()
                    + 2 * size_of::<usize>(),
            )
            .map_err(|_| "source Metadata proof exceeds retention budget")?;
            let caller = self.state.owned_caller(&mut self.assumptions, false);
            selection::begin_canonical(
                facts,
                &mut self.state.context,
                goal,
                caller,
                (self.state.availability, None),
                None,
            )
        }
        .map_err(|_| "invalid or exhausted source Metadata Sized proof")?;
        match progress {
            selection::Progress::Pending(dependency, query) => {
                pending.pending = Some(query);
                self.wait_metadata(facts, dependency)
            }
            selection::Progress::Ready(selected) => {
                if self.state.accept_selection(selected)?.is_none() {
                    return self.finish(facts, Some(Resolution::Opaque));
                }
                let unit = facts
                    .types
                    .intern(crate::types::Kind::Tuple(crate::types::ListId::EMPTY))?;
                self.finish(facts, Some(Resolution::Type(unit)))
            }
            selection::Progress::Inferred { .. } => {
                Err("source Metadata Sized proof cannot refine its input")
            }
        }
    }
    fn wait_metadata(
        self,
        facts: &Declarations,
        dependency: selection::Dependency,
    ) -> Result<Progress, &'static str> {
        crate::sema::header_attributes::charge_source_retention(facts, 1, size_of::<Self>())
            .map_err(|_| "source Metadata suspension exceeds its budget")?;
        Ok(Progress::Pending(dependency, Box::new(self)))
    }
}
