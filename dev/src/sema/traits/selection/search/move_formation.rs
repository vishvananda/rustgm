//! Optional body Copy keeps authenticated formation for a definitive move.
use super::{
    Caller, Context, Declarations, Frame, Goal, GraphTraversal, Search, Selection, allocations,
    continuation, inference,
};
use crate::sema::traits::selection::{Availability, Failure};

pub(in crate::sema) enum BodyCopy {
    Selection(Selection),
    MoveWithFormation(crate::sema::traits::obligations::Obligations),
}

pub(in crate::sema::traits::selection) fn classify_body_copy(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
) -> Result<BodyCopy, Failure> {
    if crate::sema::query_census::active() {
        let mode = Availability::Executable as u32 | (7 << 4);
        crate::sema::query_census::root(&facts.types, context, goal, &caller, mode);
    }
    let first = Frame::new(facts, context, goal, caller.first_region)?;
    let mut search = Search::assuming(&facts.types, caller, Availability::Executable)
        .map_err(|error| Failure::Invalid(error.into()))?;
    match search.begin_with_mode(facts, first, true)? {
        continuation::Progress::Ready(decision) => search
            .result_observed(facts, context, goal, decision, None)
            .map(BodyCopy::Selection),
        continuation::Progress::MoveWithFormation(root) => search
            .body_copy_formation(facts, context, root)
            .map(BodyCopy::MoveWithFormation)
            .map_err(Failure::Invalid),
        continuation::Progress::Pending(dependency) => Err(Failure::Pending(dependency)),
        continuation::Progress::Inferred => Err(Failure::Invalid(
            "body Copy classification returned partial inputs".into(),
        )),
    }
}

impl Search<'_> {
    fn body_copy_formation(
        mut self,
        facts: &mut Declarations,
        context: &mut Context,
        root: usize,
    ) -> Result<crate::sema::traits::obligations::Obligations, String> {
        context.account_work(self.query.steps).map_err(inference)?;
        context
            .account_nested(self.query.nested as usize)
            .map_err(inference)?;
        self.pay_extraction(context, |search| {
            let node = &search.query.nodes[root];
            if node.source.is_some() || node.provisional_size || node.cycle_dependent {
                return Err("invalid formation-only move evidence".into());
            }
            let mut proof = std::mem::replace(
                &mut search.query.allocations,
                allocations::Allocations::new(1),
            )
            .finish(node.end);
            let mut sized = Vec::new();
            search.collect_graph(
                facts,
                ([root], GraphTraversal::Unique),
                None,
                &mut proof,
                &mut sized,
                None,
            )?;
            if !sized.is_empty() {
                return Err("formation-only move retained unpaid sizing requirements".into());
            }
            Ok(proof)
        })
    }
}
