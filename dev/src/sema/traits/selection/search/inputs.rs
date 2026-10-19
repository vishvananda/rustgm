//! Trait input normal forms retain original graphs, shared holes and normalization proofs.
use super::normalization::{Normalization, Purpose};
use super::{Action, Decision, Declarations, Frame, Goal, Node, Search, inference};
use crate::types::{
    TypeId,
    infer::{self, TermId},
    lifetime_proof::{Obligations, ScopeId},
};
use std::collections::HashMap;

pub(super) struct Inputs {
    original: Goal,
    roots: Vec<(TermId, TypeId)>,
    replacements: HashMap<TermId, TypeId>,
    cursor: usize,
    pub ready: bool,
}
impl Search<'_> {
    pub(super) fn start_inputs(
        facts: &mut Declarations,
        frame: &mut Frame,
    ) -> Result<Option<Action>, String> {
        if let Some(original) = frame.canonical
            && !original.contains_projections(&facts.types)?
        {
            return Ok(None);
        }
        let mut roots = Vec::new();
        let mut scopes = Obligations::new(frame.first);
        for (scope, term) in frame
            .inputs
            .context
            .predicate_normalization_roots(
                &facts.types,
                &frame.goal,
                ScopeId::ROOT,
                |parent, count| {
                    scopes
                        .scope(parent, count)
                        .map_err(|_| infer::Error::Budget)
                },
            )
            .map_err(inference)?
        {
            if scope != ScopeId::ROOT {
                return Err("quantified trait premises require scoped selection evidence".into());
            }
            let ty = match frame.inputs.context.resolved(&mut facts.types, term) {
                Ok(ty) => ty,
                Err(infer::Error::Unresolved) => {
                    return Ok(Some(Action::Complete(Decision::Deferred)));
                }
                Err(error) => return Err(inference(error)),
            };
            roots.push((term, ty));
        }
        let Some(&(_, first)) = roots.first() else {
            return Ok(None);
        };
        let original = frame.goal.map_inputs(Ok).map_err(inference)?;
        let purpose = Purpose::Inputs(Inputs {
            original,
            roots,
            replacements: HashMap::new(),
            cursor: 0,
            ready: false,
        });
        frame.normalization = Some(Box::new(Normalization::new(facts, frame, first, purpose)?));
        Ok(Some(Action::Continue))
    }

    pub(super) fn advance_inputs(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        value: TypeId,
    ) -> Result<Action, String> {
        let normalization = frame
            .normalization
            .as_mut()
            .ok_or("missing trait input query")?;
        let Purpose::Inputs(inputs) = &mut normalization.purpose else {
            return Err("invalid trait input normalization purpose".into());
        };
        inputs
            .replacements
            .insert(inputs.roots[inputs.cursor].0, value);
        inputs.cursor += 1;
        if let Some(&(_, ty)) = inputs.roots.get(inputs.cursor) {
            let before = normalization.fold.steps();
            normalization.step = normalization.fold.start(&mut facts.types, ty)?;
            frame
                .inputs
                .context
                .account_work(normalization.fold.steps().saturating_sub(before))
                .map_err(inference)?;
            return Ok(Action::Continue);
        }
        frame.goal = frame
            .inputs
            .context
            .rewritten_projection_inputs(&facts.types, &inputs.original, &inputs.replacements)
            .map_err(inference)?;
        inputs.ready = true;
        if frame.goal.builtin_bound().is_some() {
            return self.complete_builtin(facts, frame).map(Action::Complete);
        }
        match self.assume(facts, frame) {
            Ok(Some(action)) => return Ok(action),
            Ok(None) => {}
            Err(infer::Error::Normalization) => return Ok(Action::Complete(Decision::Deferred)),
            Err(error) => return Err(inference(error)),
        }
        if let Some(action) = self.derive_result(facts, frame)? {
            return Ok(action);
        }
        self.candidates(facts, frame)
    }

    pub(super) fn input_evidence(
        &mut self,
        facts: &mut Declarations,
        frame: &mut Frame,
        decision: Decision,
    ) -> Result<Decision, String> {
        let Decision::Proven(selected) = decision else {
            return Ok(decision);
        };
        if let Some(inputs) = frame.normalized_inputs.take() {
            frame.inputs.context.account_work(2).map_err(inference)?;
            self.charge(&mut frame.inputs.context)?;
            return Ok(Decision::Proven(
                self.inherit_inputs(selected, inputs.proof),
            ));
        }
        let Some(normalization) = &mut frame.normalization else {
            return Ok(decision);
        };
        let Purpose::Inputs(inputs) = &normalization.purpose else {
            return Ok(decision);
        };
        if !inputs.ready {
            return Err("trait input proof completed before normalization".into());
        }
        let goal = if let Some(original) = frame.canonical {
            original
        } else {
            // Only ordinary shared holes can be refined by normal-form selection.
            // Projection inputs were already resolved before normalization began.
            frame
                .inputs
                .context
                .equate_proved_predicate(
                    &mut facts.types,
                    &frame.goal,
                    self.query.nodes[selected].goal,
                )
                .map_err(inference)?;
            frame
                .inputs
                .context
                .resolved_predicate(&mut facts.types, &inputs.original)
                .map_err(inference)?
        };
        let source = self.query.nodes[selected].source;
        let end = normalization.end.max(self.query.nodes[selected].end);
        let children = normalization.proof.into_iter().chain([selected]).collect();
        let index = self.query.push(
            Node {
                goal,
                source,
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
        self.charge(&mut frame.inputs.context)?;
        Ok(Decision::Proven(index))
    }

    pub(super) fn inherit_inputs(&mut self, selected: usize, prefix: Option<usize>) -> usize {
        let Some(prefix) = prefix else {
            return selected;
        };
        // A completed result may outlive the candidate which normalized its
        // inputs. Keep the complete proof graph, including earlier existential
        // intervals: such nodes cannot enter the region-free completed cache.
        let node = &self.query.nodes[selected];
        let first = node.first.min(self.query.nodes[prefix].first);
        let end = node.end.max(self.query.nodes[prefix].end);
        let (goal, source) = (node.goal, node.source);
        let scope = self.query.node_scope(selected);
        let index = self.query.push(
            Node {
                goal,
                source,
                children: vec![prefix, selected],
                first,
                end,
                equalities: Vec::new(),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
                sized: Vec::new(),
            },
            scope,
            first..first,
        );
        self.query.reused = true;
        index
    }
}
