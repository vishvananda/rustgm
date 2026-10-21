//! Declared callable output requirements supply context, never selection permission.
use super::{Checker, Constructor, Error, NodeId, TermId};
use crate::types::infer::{arguments::Argument, predicates::Predicate};
use std::collections::HashMap;

#[derive(Default)]
pub(in crate::sema::bodies) struct Hints {
    #[expect(
        clippy::box_collection,
        reason = "ordinary calls and parked bodies leave the callable context index unallocated"
    )]
    values: Option<Box<HashMap<TermId, Hint>>>,
}
#[derive(Clone, Copy)]
struct Hint {
    inputs: TermId,
    result: TermId,
    regions: Option<u32>,
}

impl Checker<'_> {
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
            .values
            .get_or_insert_with(Default::default)
            .entry(subject)
            // Keep declaration order. Every original requirement still needs
            // proof; this first contextual signature adds no trait authority.
            .or_insert(Hint {
                inputs: *inputs,
                result,
                regions,
            });
        Ok(())
    }

    pub(super) fn generic_closure_expectation(
        &mut self,
        node: NodeId,
        expected: TermId,
    ) -> Result<Option<TermId>, Error> {
        let Some(hint) = self
            .state
            .call_types
            .closure_expectations
            .values
            .as_ref()
            .and_then(|values| values.get(&expected))
            .copied()
        else {
            return Ok(None);
        };
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
