//! Requested inherited results retain defining conditions on the existing search stack.
use super::{Completion, Normalization, ResultContinuation};
use crate::{
    sema::{
        projections::Definition,
        specialization::{DefiningConditions, SelectedMethod},
        traits::selection::search::{
            Action, Decision, Declarations, Frame, Node, Search, bound_frames::ClauseRequest,
        },
    },
    support::{hash::HashMap, retained::Growth},
    types::lifetime_proof::ScopeId,
};

pub(super) struct Inherited {
    completed: Completion,
    method: SelectedMethod,
    conditions: DefiningConditions,
    cursor: usize,
    children: Vec<usize>,
    universes: HashMap<u32, ScopeId>,
}
impl Search<'_> {
    pub(super) fn stage_inherited_result(
        &mut self,
        facts: &mut Declarations,
        normalization: &mut Normalization,
        completed: Completion,
    ) -> Result<Option<Completion>, String> {
        let source = self.query.nodes[completed.node]
            .source
            .clone()
            .ok_or("missing selected result proof")?;
        let Some(method) = Definition::method(facts, completed.opened.application, source)? else {
            return Ok(Some(completed));
        };
        let Some(conditions) = method.projection_conditions(facts)? else {
            return Ok(Some(completed));
        };
        method.charge(1, size_of::<ResultContinuation>() + size_of::<usize>())?;
        let children = vec![completed.node];
        normalization.inherited = Some(Box::new(ResultContinuation::Inherited(Inherited {
            completed,
            method,
            conditions,
            children,
            cursor: 0,
            universes: HashMap::default(),
        })));
        #[cfg(test)]
        crate::sema::traits::selection::copy_trace::record(|| {
            let pending = normalization
                .inherited
                .as_deref()
                .and_then(|value| match value {
                    ResultContinuation::Inherited(value) => Some(value),
                    ResultContinuation::Metadata(_) => None,
                });
            format!(
                "specialization-rpit-search capacities={:?}",
                pending.map(|value| (
                    value.conditions.clauses.capacity(),
                    value.conditions.outlives.capacity(),
                    value.children.capacity()
                ))
            )
        });
        Ok(None)
    }
    pub(super) fn advance_inherited_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let normalization = frame
            .normalization
            .as_mut()
            .ok_or("missing inherited normalization")?;
        let Some(ResultContinuation::Inherited(pending)) = normalization.inherited.as_deref()
        else {
            return Err("missing defining result conditions".into());
        };
        pending.method.current(facts)?;
        pending.method.charge(1, 0)?;
        if pending.cursor < pending.conditions.clauses.len() {
            return Ok(Action::Descend);
        }
        let pending = normalization
            .inherited
            .take()
            .ok_or("missing completed inherited result")?;
        let ResultContinuation::Inherited(pending) = *pending else {
            return Err("metadata continuation used inherited result".into());
        };
        let completed = pending.finish(&mut self.query, normalization.scope)?;
        self.finish_normalization(facts, frame, completed)?;
        Ok(Action::Continue)
    }
    pub(in crate::sema::traits::selection::search) fn inherited_result_child(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Frame>, String> {
        let Some(normalization) = frame.normalization.as_mut() else {
            return Ok(None);
        };
        let Some(ResultContinuation::Inherited(pending)) = normalization.inherited.as_deref_mut()
        else {
            return Ok(None);
        };
        let clause = *pending
            .conditions
            .clauses
            .get(pending.cursor)
            .ok_or("missing inherited result condition")?;
        self.clause_frame::<true>(
            facts,
            &mut frame.inputs.context,
            ClauseRequest {
                clause,
                scope: normalization.scope,
            },
            &mut pending.universes,
            &|steps, bytes| pending.method.charge(steps, bytes),
        )
        .map(Some)
    }
    pub(super) fn accept_inherited_result(
        normalization: &mut Normalization,
        decision: Decision,
    ) -> Result<(), String> {
        let Some(ResultContinuation::Inherited(pending)) = normalization.inherited.as_deref_mut()
        else {
            return Err("missing inherited result condition owner".into());
        };
        match decision {
            Decision::Proven(node) => {
                let growth =
                    Growth::vector::<usize>(pending.children.len(), pending.children.capacity(), 1);
                pending.method.charge(1 + growth.work, growth.bytes)?;
                pending.children.push(node);
                pending.cursor += 1;
            }
            Decision::Absent | Decision::Deferred => normalization.stopped = Some(decision),
            Decision::Refuted(_) => {
                return Err("negative inherited condition was not classified".into());
            }
        }
        Ok(())
    }
}
impl Inherited {
    fn finish(
        self,
        query: &mut crate::sema::traits::selection::search::Query,
        scope: ScopeId,
    ) -> Result<Completion, String> {
        let growth = query.node_growth();
        self.method
            .charge(1 + self.children.len() + growth.work, growth.bytes)?;
        let parent = &query.nodes[self.completed.node];
        let node = Node {
            debt_free: false,
            cycle_dependent: false,
            provisional_size: false,
            goal: parent.goal,
            source: parent.source.clone(),
            first: parent.first,
            end: parent.end.max(query.allocations.end()),
            children: self.children,
            equalities: Vec::new(),
            outlives: self.conditions.outlives,
            type_outlives: Vec::new(),
            sized: Vec::new(),
        };
        let first = node.first;
        let node = query.push(node, scope, first..first);
        Ok(Completion {
            node,
            ..self.completed
        })
    }
}
