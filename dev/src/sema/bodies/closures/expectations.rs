//! Declared callable output requirements supply context, never selection permission.
use super::{Checker, Constructor, Error, NodeId, TermId};
use crate::types::infer::{arguments::Argument, predicates::Predicate};
use std::collections::HashMap;

#[derive(Default)]
pub(in crate::sema::bodies) struct Hints {
    data: Option<Box<Data>>,
}
#[derive(Default)]
struct Data {
    values: HashMap<TermId, Hint>,
    changes: Vec<(TermId, TermId)>,
    next: usize,
}
#[derive(Clone, Copy)]
struct Hint {
    inputs: TermId,
    result: TermId,
    regions: Option<u32>,
    order: usize,
}
impl Hints {
    fn refresh(
        &mut self,
        context: &mut crate::types::infer::Context,
    ) -> Result<(), crate::types::infer::Error> {
        if let Some(data) = &mut self.data {
            context.exchange_root_changes(&mut data.changes)?;
            for (source, target) in data.changes.drain(..) {
                if let Some(hint) = data.values.remove(&source) {
                    let current = data.values.entry(target).or_insert(hint);
                    if hint.order < current.order {
                        *current = hint;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Checker<'_> {
    pub(super) fn object_closure_expectation(
        &mut self,
        node: NodeId,
        expected: TermId,
    ) -> Result<Option<TermId>, Error> {
        if !matches!(
            self.contextual_head(node, expected)?,
            Some(Constructor::Dynamic { .. })
        ) {
            return Ok(None);
        }
        let Some(output) = self.facts.associated.callable_output() else {
            return Ok(None);
        };
        let Some(binding) = self
            .state
            .infer
            .object_binding(&self.facts.types, expected, crate::types::ItemId(output.0))
            .map_err(|e| self.error(node, e))?
        else {
            return Ok(None);
        };
        let [Argument::Type(inputs)] = binding.arguments.as_slice() else {
            return Err(self.error(node, "object callable context requires one input tuple"));
        };
        self.contextual_signature(
            node,
            Hint {
                inputs: *inputs,
                result: binding.result,
                regions: Some(binding.bound_regions),
                order: 0,
            },
        )
    }

    pub(in crate::sema::bodies) fn record_closure_expectation(
        &mut self,
        node: NodeId,
        predicate: &Predicate,
        regions: Option<u32>,
    ) -> Result<(), Error> {
        let Some(result) = predicate.projection_result() else {
            return Ok(());
        };
        let item = self
            .state
            .infer
            .projection_item(&self.facts.types, predicate.subject())
            .map_err(|e| self.error(node, e))?;
        if item.is_none()
            || item.map(|item| crate::resolve::model::DefId(item.0))
                != self.facts.associated.callable_output()
        {
            return Ok(());
        }
        let application = self
            .state
            .infer
            .projection_application(&self.facts.types, predicate.subject())
            .map_err(|e| self.error(node, e))?;
        let subject = application.predicate.subject();
        let Some((_, [Argument::Type(inputs)])) = application.predicate.trait_arguments() else {
            return Err(self.error(node, "callable expectation requires one argument tuple"));
        };
        if !application.arguments.is_empty() {
            return Err(self.error(node, "callable output has unexpected family arguments"));
        }
        self.state
            .call_types
            .closure_expectations
            .refresh(&mut self.state.infer)
            .map_err(|e| self.error(node, e))?;
        let subject = self
            .state
            .infer
            .root(subject)
            .map_err(|e| self.error(node, e))?;
        self.state.infer.track_root_changes();
        let data = self
            .state
            .call_types
            .closure_expectations
            .data
            .get_or_insert_with(Default::default);
        let order = data.next;
        data.next += 1;
        data.values
            .entry(subject)
            // Keep declaration order. Every original requirement still needs
            // proof; this first contextual signature adds no trait authority.
            .or_insert(Hint {
                inputs: *inputs,
                result,
                regions,
                order,
            });
        Ok(())
    }

    pub(super) fn generic_closure_expectation(
        &mut self,
        node: NodeId,
        expected: TermId,
    ) -> Result<Option<TermId>, Error> {
        self.state
            .call_types
            .closure_expectations
            .refresh(&mut self.state.infer)
            .map_err(|e| self.error(node, e))?;
        let expected = self
            .state
            .infer
            .root(expected)
            .map_err(|e| self.error(node, e))?;
        let Some(hint) = self
            .state
            .call_types
            .closure_expectations
            .data
            .as_ref()
            .and_then(|data| data.values.get(&expected))
            .copied()
        else {
            return Ok(None);
        };
        self.contextual_signature(node, hint)
    }

    fn contextual_signature(&mut self, node: NodeId, hint: Hint) -> Result<Option<TermId>, Error> {
        if !matches!(self.head(node, hint.inputs)?, Some(Constructor::Tuple)) {
            return Ok(None);
        }
        let parameters = self
            .state
            .infer
            .arity(&self.facts.types, hint.inputs)
            .map_err(|e| self.error(node, e))?
            .ok_or_else(|| self.error(node, "missing contextual argument tuple"))?;
        let mut children = Vec::with_capacity(parameters + 1);
        for index in 0..parameters {
            children.push(self.child(node, hint.inputs, index)?);
        }
        children.push(hint.result);
        if hint.regions.is_none() {
            children = self
                .state
                .infer
                .map_region_roots(
                    &mut self.facts.types,
                    &children,
                    &mut crate::types::substitute::Bound::shift(1),
                )
                .map_err(|e| self.error(node, e))?;
        }
        self.state
            .infer
            .construct(
                Constructor::Function {
                    bound_regions: hint.regions.unwrap_or(0),
                    parameters: u32::try_from(parameters)
                        .map_err(|_| self.error(node, "too many contextual inputs"))?,
                    unsafe_: false,
                    convention: crate::types::callable::Convention::RUST,
                },
                &children,
            )
            .map(Some)
            .map_err(|e| self.error(node, e))
    }
}
