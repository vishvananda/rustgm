//! Declared callable output requirements supply context, never selection permission.
use super::{Checker, Constructor, Error, NodeId, TermId};
use crate::support::hash::HashMap;
use crate::types::{
    closure::CallKind,
    infer::{arguments::Argument, predicates::Predicate},
};

#[derive(Default)]
pub(in crate::sema::bodies) struct Hints {
    data: Option<Box<Data>>,
}
#[derive(Default)]
struct Data {
    values: HashMap<TermId, CallableHint>,
    changes: Vec<(TermId, TermId)>,
    next: usize,
}
#[derive(Clone, Copy, Default)]
struct CallableHint {
    signature: Option<Hint>,
    kind: Option<CallKind>,
}
#[derive(Clone, Copy)]
struct Hint {
    inputs: TermId,
    result: TermId,
    regions: Option<u32>,
    order: usize,
}
impl CallableHint {
    fn merge(&mut self, other: Self) {
        self.kind = self.kind.into_iter().chain(other.kind).min();
        if let Some(signature) = other.signature
            && self
                .signature
                .is_none_or(|current| signature.order < current.order)
        {
            self.signature = Some(signature);
        }
    }
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
                    data.values.entry(target).or_default().merge(hint);
                }
            }
        }
        Ok(())
    }
}

impl Checker<'_> {
    pub(super) fn callable_closure_expectation(
        &mut self,
        node: NodeId,
        expected: TermId,
    ) -> Result<(Option<TermId>, Option<CallKind>), Error> {
        let object = self.object_closure_hint(node, expected)?;
        if let Some(signature) = object.signature
            && let Some(term) = self.contextual_signature(node, signature)?
        {
            return Ok((Some(term), object.kind));
        }
        let hint = self.generic_closure_hint(node, expected)?;
        let signature = hint
            .signature
            .map(|signature| self.contextual_signature(node, signature))
            .transpose()?
            .flatten();
        Ok((signature, object.kind.into_iter().chain(hint.kind).min()))
    }

    pub(super) fn opaque_closure_kind(
        &mut self,
        node: NodeId,
        expected: TermId,
    ) -> Result<Option<CallKind>, Error> {
        if self.report.package.uses_source_bindings() {
            return Ok(self.generic_closure_hint(node, expected)?.kind);
        }
        Ok(None)
    }

    pub(in crate::sema::bodies) fn retain_object_closure_expectation(
        &mut self,
        node: NodeId,
        subject: TermId,
        target: TermId,
    ) -> Result<(), Error> {
        let hint = self.object_closure_hint(node, target)?;
        if hint.signature.is_some() || hint.kind.is_some() {
            self.retain_closure_hint(node, subject, hint)?;
        }
        Ok(())
    }

    fn object_closure_hint(
        &mut self,
        node: NodeId,
        expected: TermId,
    ) -> Result<CallableHint, Error> {
        let Some(Constructor::Dynamic { shape, .. }) = self.contextual_head(node, expected)? else {
            return Ok(CallableHint::default());
        };
        let kind = if self.report.package.uses_source_bindings() {
            self.facts
                .types
                .dynamic_shape(shape)
                .ok_or_else(|| self.error(node, "missing contextual callable object"))?
                .principal
                .and_then(|item| self.contextual_call_kind(item))
        } else {
            None
        };
        let mut hint = CallableHint {
            signature: None,
            kind,
        };
        let Some(output) = self.facts.associated.callable_output() else {
            return Ok(hint);
        };
        let Some(binding) = self
            .state
            .infer
            .object_binding(&self.facts.types, expected, crate::types::ItemId(output.0))
            .map_err(|e| self.error(node, e))?
        else {
            return Ok(hint);
        };
        let [Argument::Type(inputs)] = binding.arguments.as_slice() else {
            return Err(self.error(node, "object callable context requires one input tuple"));
        };
        hint.signature = Some(Hint {
            inputs: *inputs,
            result: binding.result,
            regions: Some(binding.bound_regions),
            order: 0,
        });
        Ok(hint)
    }

    pub(in crate::sema::bodies) fn record_closure_expectation(
        &mut self,
        node: NodeId,
        predicate: &Predicate,
        regions: Option<u32>,
    ) -> Result<(), Error> {
        if let Some((item, _)) = predicate.trait_arguments() {
            return self.retain_closure_trait(node, predicate.subject(), item);
        }
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
        self.retain_closure_hint(
            node,
            subject,
            CallableHint {
                signature: Some(Hint {
                    inputs: *inputs,
                    result,
                    regions,
                    order: 0,
                }),
                kind: None,
            },
        )
    }

    pub(super) fn retain_closure_trait(
        &mut self,
        node: NodeId,
        subject: TermId,
        item: crate::types::ItemId,
    ) -> Result<(), Error> {
        // The bootstrap language specifies the receiver from capture use.
        if self.report.package.uses_source_bindings()
            && let Some(kind) = self.contextual_call_kind(item)
        {
            self.retain_closure_hint(
                node,
                subject,
                CallableHint {
                    signature: None,
                    kind: Some(kind),
                },
            )?;
        }
        Ok(())
    }

    fn contextual_call_kind(&self, item: crate::types::ItemId) -> Option<CallKind> {
        self.facts
            .associated
            .callable(crate::resolve::model::DefId(item.0))
            .map(CallKind::from_trait)
    }

    fn retain_closure_hint(
        &mut self,
        node: NodeId,
        subject: TermId,
        mut hint: CallableHint,
    ) -> Result<(), Error> {
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
        if let Some(signature) = &mut hint.signature {
            signature.order = data.next;
            data.next += 1;
        }
        data.values.entry(subject).or_default().merge(hint);
        Ok(())
    }

    /// The result may remain open while the closure body establishes its type.
    /// Only its input tuple needs the enclosing body's pending inference.
    pub(in crate::sema::bodies) fn contextual_closure_inputs_pending(
        &mut self,
        node: NodeId,
        expected: TermId,
    ) -> Result<bool, Error> {
        let Some(hint) = self.generic_closure_hint(node, expected)?.signature else {
            return Ok(false);
        };
        self.state
            .infer
            .inputs_fixed(&self.facts.types, hint.inputs)
            .map(|fixed| !fixed)
            .map_err(|e| self.error(node, e))
    }

    fn generic_closure_hint(
        &mut self,
        node: NodeId,
        expected: TermId,
    ) -> Result<CallableHint, Error> {
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
        Ok(self
            .state
            .call_types
            .closure_expectations
            .data
            .as_ref()
            .and_then(|data| data.values.get(&expected))
            .copied()
            .unwrap_or_default())
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
