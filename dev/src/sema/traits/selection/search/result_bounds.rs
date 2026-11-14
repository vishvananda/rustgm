//! Declared object promises and proved projection results share scoped selection.
use super::ambient::Cursor;
use super::{Action, Decision, Declarations, Frame, Node, Search, Source, inference};
use crate::{
    sema::traits::{assumptions::Selection, results},
    support::constness::Constness,
    types::{
        Kind, infer,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

/// A conditional associated promise also owes its const-qualified parent.
/// This lazy continuation reuses the scoped clause stack and owns no caller grant.
pub(super) struct Pending {
    goal: Predicate,
    source: Source,
    parent: Predicate,
    regions: std::ops::Range<u32>,
    cursor: Cursor,
}
impl Pending {
    pub(super) fn regions(&self) -> std::ops::Range<u32> {
        self.regions.clone()
    }
    pub(super) fn retained_child(&self) -> Option<usize> {
        self.cursor.suspended
    }
    pub(super) fn condition(&mut self) -> (Option<Clause>, &mut Cursor) {
        (
            (self.cursor.index == 0).then_some(self.parent.into()),
            &mut self.cursor,
        )
    }
}

impl Search<'_> {
    pub(super) fn derive_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        let ty = match frame
            .inputs
            .context
            .resolved(&mut facts.types, frame.goal.subject())
        {
            Ok(ty) => ty,
            Err(infer::Error::Unresolved) => return Ok(None),
            Err(error) => return Err(inference(error)),
        };
        let (item, _) = frame
            .goal
            .trait_arguments()
            .ok_or("missing result query trait")?;
        let Some((rules, steps)) = results::traits(facts, ty, item)? else {
            return Ok(None);
        };
        frame
            .inputs
            .context
            .account_work(steps)
            .map_err(inference)?;
        let first = frame.available_region().max(self.query.allocations.floor());
        // A conditional declaration can prove an always-const result only by
        // proving its parent always-const below. Ordinary result/object rules
        // retain the ordinary qualifier matching policy.
        let query = if matches!(facts.types.kind(ty), Some(Kind::Projection(_)))
            && frame.goal.constness() == Some(Constness::Const)
        {
            let (item, arguments) = frame
                .goal
                .trait_arguments()
                .ok_or("missing const result query trait")?;
            Some(
                crate::types::infer::predicates::Predicate::trait_application_with_effect(
                    frame.goal.subject(),
                    item,
                    arguments.to_vec(),
                    Constness::Conditional,
                ),
            )
        } else {
            None
        };
        let selected = match rules.select_scoped(
            &mut facts.types,
            &mut frame.inputs.context,
            query.as_ref().unwrap_or(&frame.goal),
            first,
        ) {
            Ok(Selection::Proven(proof)) => proof,
            Ok(Selection::Absent) => return Ok(None),
            Ok(Selection::Deferred) | Err(infer::Error::Normalization) => {
                return Ok(Some(Action::Complete(Decision::Deferred)));
            }
            Err(error) => return Err(inference(error)),
        };
        self.query.reserve(frame.scope, selected.regions.clone())?;
        self.query.allocations.record(selected.regions.end);
        let goal = frame
            .inputs
            .context
            .resolved_predicate(&mut facts.types, &frame.goal)
            .map_err(inference)?;
        self.complete_result(facts, frame, ty, goal, selected)
    }

    fn complete_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        ty: crate::types::TypeId,
        goal: Predicate,
        selected: crate::sema::traits::assumptions::ScopedEvidence,
    ) -> Result<Option<Action>, String> {
        let source = Source::result(&selected).map_err(inference)?;
        if let Some(Kind::Projection(application)) = facts.types.kind(ty)
            && selected
                .predicate
                .bound
                .trait_parts()
                .is_some_and(|(_, effect)| effect == Constness::Conditional)
            && let Some((_, effect)) = goal.bound.trait_parts()
            && effect != Constness::Ordinary
        {
            facts.charge_caller_inputs(
                1,
                size_of::<Pending>() + size_of::<super::mechanical::Pending>() + size_of::<usize>(),
            )?;
            frame.language = Some(Box::new(super::mechanical::Pending::Result(Box::new(
                Pending {
                    goal,
                    source,
                    parent: Predicate {
                        ty: application.subject,
                        bound: Bound::trait_application(application.trait_application, effect),
                    },
                    regions: selected.regions,
                    cursor: Cursor::with_capacity(1),
                },
            ))));
            return self.advance_callable_conditions(facts, frame).map(Some);
        }
        self.publish_result(facts, frame, goal, source, selected.regions, Vec::new())
            .map(Some)
    }

    pub(super) fn finish_result_bound(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Action, String> {
        let Some(super::mechanical::Pending::Result(pending)) = frame.language.as_deref_mut()
        else {
            return Err("missing conditional result proof".into());
        };
        if pending.cursor.index != 1
            || pending.cursor.stopped.is_some()
            || pending.cursor.suspended.is_some()
        {
            return Err("conditional result published before its parent was proved".into());
        }
        let Some(super::mechanical::Pending::Result(pending)) =
            frame.language.take().map(|pending| *pending)
        else {
            return Err("conditional result lost its completed parent".into());
        };
        self.publish_result(
            facts,
            frame,
            pending.goal,
            pending.source,
            pending.regions,
            pending.cursor.children,
        )
    }

    fn publish_result(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        goal: Predicate,
        source: Source,
        regions: std::ops::Range<u32>,
        children: Vec<usize>,
    ) -> Result<Action, String> {
        // Ordinary declared-result queries retain their existing selector
        // account and exact region interval. Only the new parent continuation
        // allocates caller-input storage and carries its completed child range.
        let end = if children.is_empty() {
            regions.end
        } else {
            let growth = self.query.node_growth();
            facts.charge_caller_inputs(growth.work, growth.bytes)?;
            regions.end.max(self.query.allocations.end())
        };
        let index = self.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal,
                source: Some(source),
                children,
                first: frame.first,
                end,
                equalities: std::mem::take(&mut frame.inputs.context.region_equalities),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            frame.scope,
            regions,
        );
        // input_evidence also attaches every ordinary parent normalization
        // child, including existential scopes; neither effect proof replaces it.
        Ok(Action::Complete(Decision::Proven(index)))
    }
}
