//! Metadata keeps its real parent and optional Sized child on this proof stack.
use super::{Completion, Normalization, ResultContinuation};
use crate::{
    sema::{
        compiler_items::pointee::{MetadataPlan, MetadataStep},
        traits::selection::search::{
            Action, Decision, Declarations, Frame, Node, Predicate, Search, Source, inference,
        },
    },
    types::{TypeId, infer::Instantiation, parameters::predicates::Bound, substitute::Resolution},
};
pub(super) struct Pending {
    completed: Completion,
    plan: MetadataPlan,
    needed: Option<TypeId>,
    sized: Sizing,
}
#[derive(Clone, Copy)]
enum Sizing {
    NotRequested,
    Proven(usize),
    Opaque,
}
impl Search<'_> {
    pub(super) fn stage_metadata_result(
        &mut self,
        facts: &mut Declarations,
        context: &mut super::super::Context,
        normalization: &mut Normalization,
        completed: Completion,
    ) -> Result<Option<Completion>, String> {
        let Some(Source::Pointee(selected)) = self.query.nodes[completed.node].source.as_ref()
        else {
            return Ok(Some(completed));
        };
        let mut application = completed
            .opened
            .application
            .associated()
            .ok_or("source Pointee proof is not a Metadata application")?;
        let goal = context
            .resolved_predicate(&mut facts.types, &completed.opened.goal)
            .map_err(inference)?;
        application.subject = goal.ty;
        application.trait_application = goal
            .bound
            .trait_parts()
            .ok_or("source Metadata lost its parent")?
            .0;
        crate::sema::header_attributes::charge_source_retention(
            facts,
            1,
            size_of::<ResultContinuation>(),
        )
        .map_err(|error| error.to_string())?;
        let plan = MetadataPlan::new(facts, selected, application)?;
        #[cfg(test)]
        crate::sema::copy_trace::record(|| format!("source-metadata-search {application:?}"));
        normalization.inherited = Some(Box::new(ResultContinuation::Metadata(Pending {
            completed,
            plan,
            needed: None,
            sized: Sizing::NotRequested,
        })));
        Ok(None)
    }
    pub(super) fn advance_metadata_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let normalization = frame
            .normalization
            .as_mut()
            .ok_or("missing metadata normalization")?;
        let Some(ResultContinuation::Metadata(pending)) = normalization.inherited.as_deref_mut()
        else {
            return Err("missing source Metadata plan".into());
        };
        let resolution = if !matches!(pending.sized, Sizing::NotRequested) {
            Some(if matches!(pending.sized, Sizing::Proven(_)) {
                crate::sema::header_attributes::charge_source_retention(facts, 1, 0)
                    .map_err(|error| error.to_string())?;
                Resolution::Type(TypeId::UNIT)
            } else {
                Resolution::Opaque
            })
        } else {
            match pending.plan.advance(facts)? {
                MetadataStep::Ready(ty) => Some(Resolution::Type(ty)),
                MetadataStep::Fields(item) => {
                    return Ok(Action::Wait(super::super::Dependency::Fields(item)));
                }
                MetadataStep::Sized(ty) => {
                    pending.needed = Some(ty);
                    return Ok(Action::Descend);
                }
            }
        };
        let pending = normalization
            .inherited
            .take()
            .ok_or("missing completed Metadata plan")?;
        let ResultContinuation::Metadata(pending) = *pending else {
            return Err("inherited result used source metadata publication".into());
        };
        let equations_start = pending.completed.equations_start;
        let node = self.complete_metadata_proof(facts, &pending, normalization.scope)?;
        self.publish_normalization(facts, frame, equations_start, resolution, node)?;
        Ok(Action::Continue)
    }
    pub(in crate::sema::traits::selection::search) fn metadata_result_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        let Some(normalization) = frame.normalization.as_mut() else {
            return Ok(None);
        };
        let Some(ResultContinuation::Metadata(pending)) = normalization.inherited.as_deref_mut()
        else {
            return Ok(None);
        };
        let ty = pending.needed.ok_or("missing Metadata Sized child input")?;
        let first = normalization.end.max(self.query.allocations.floor());
        let mut substitution =
            Instantiation::new(&mut frame.inputs.context, &facts.types, &[]).map_err(inference)?;
        let goal = frame
            .inputs
            .context
            .instantiate_predicate(
                &mut facts.types,
                Predicate {
                    ty,
                    bound: Bound::Sized,
                },
                &mut substitution,
            )
            .map_err(inference)?;
        let mut child = Frame::new(facts, &mut frame.inputs.context, &goal, first)?;
        child.scope = normalization.scope;
        Ok(Some(child))
    }
    pub(super) fn accept_metadata_result(
        normalization: &mut Normalization,
        decision: Decision,
    ) -> Result<(), String> {
        let Some(ResultContinuation::Metadata(pending)) = normalization.inherited.as_deref_mut()
        else {
            return Err("missing Metadata Sized child owner".into());
        };
        pending
            .needed
            .take()
            .ok_or("missing requested Metadata Sized input")?;
        pending.sized = match decision {
            Decision::Proven(node) => Sizing::Proven(node),
            Decision::Absent | Decision::Deferred => Sizing::Opaque,
            Decision::Refuted(_) => {
                return Err("negative Metadata Sized child was not classified".into());
            }
        };
        Ok(())
    }
    fn complete_metadata_proof(
        &mut self,
        facts: &Declarations,
        pending: &Pending,
        scope: crate::types::lifetime_proof::ScopeId,
    ) -> Result<usize, String> {
        let Sizing::Proven(sized) = pending.sized else {
            return Ok(pending.completed.node);
        };
        let growth = self.query.node_growth();
        crate::sema::header_attributes::charge_source_retention(
            facts,
            2 + growth.work,
            growth.bytes.saturating_add(2 * size_of::<usize>()),
        )
        .map_err(|error| error.to_string())?;
        let parent = &self.query.nodes[pending.completed.node];
        let node = Node {
            goal: parent.goal,
            source: parent.source.clone(),
            first: parent.first,
            end: parent.end.max(self.query.nodes[sized].end),
            debt_free: false,
            cycle_dependent: false,
            provisional_size: false,
            children: vec![pending.completed.node, sized],
            equalities: Vec::new(),
            outlives: Vec::new(),
            type_outlives: Vec::new(),
            sized: Vec::new(),
        };
        let first = node.first;
        Ok(self.query.push(node, scope, first..first))
    }
}
