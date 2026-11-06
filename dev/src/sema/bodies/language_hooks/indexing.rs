//! Source index calls return real borrowed places; exact builtins erase after proof.
use super::super::{Checker, Error, NodeId, TermId, Type};
use crate::{
    sema::{compiler_items, model::Operation},
    support::language_hooks::Hook,
    syntax::ast::Kind,
    types::{
        ItemId, Primitive, TypeId,
        infer::{Solved, arguments::Argument, predicates::Predicate},
    },
};
impl Checker<'_> {
    pub(in crate::sema::bodies) fn source_index_operator(
        &mut self,
        node: NodeId,
        value: NodeId,
        index: NodeId,
        subject: TermId,
        mutable: bool,
    ) -> Result<TermId, Error> {
        let selected =
            compiler_items::source_index(self.report, self.facts, self.key(node), mutable)?;
        if matches!(
            self.facts.types.kind(selected.signature),
            Some(Type::Function { unsafe_: true, .. })
        ) {
            self.retain_index_unsafe_context(node)?;
        }
        let idx = self.term(index)?;
        let effect = self.callee_use_effect()?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application_with_effect(
                subject,
                ItemId(selected.definition.0),
                vec![Argument::Type(idx)],
                effect,
            ),
        ));
        self.complete_method_arguments(node, selected.method)?;
        let item = self.facts.definitions[selected.method.0 as usize]
            .ok_or_else(|| self.error(node, "source indexing lost its actual callable"))?;
        if !matches!(self.facts.types.kind(item),Some(Type::FunctionItem{item,signature,..}) if item.0==selected.method.0 && signature==selected.signature)
        {
            return Err(self.error(
                node,
                "source indexing callable differs from its checked header",
            ));
        }
        let callable = self.generic_function(node, selected.method, item)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        let receiver = self.child(node, signature, 0)?;
        let referent = self.child(node, receiver, 0)?;
        self.shape_equation(node, subject, referent)?;
        let expected = self.child(node, signature, 1)?;
        self.shape_equation(node, idx, expected)?;
        let reference = self.child(node, signature, 2)?;
        let output = self.child(node, reference, 0)?;
        let actual = self.operand_borrow(node, subject, receiver, mutable)?;
        let pending =
            self.retain_language_borrow(node, value, selected.method, callable, subject, actual);
        self.state.call_types.methods[pending]
            .1
            .expect_receiver(receiver);
        self.retain_language_index_call(
            node,
            if mutable { Hook::IndexMut } else { Hook::Index },
            pending,
            signature,
        );
        Ok(output)
    }
    pub(in crate::sema::bodies) fn source_primitive_index_solved(
        &self,
        node: NodeId,
        solved: &Solved,
    ) -> Result<bool, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        let Kind::Index { index, .. } = self.kind(node) else {
            return Ok(false);
        };
        let Some(array) = self.state.pending_indexes.get(&node) else {
            return Ok(false);
        };
        let actual = solved.types[array.0 as usize];
        let idx = solved.types[self.term(index)?.0 as usize];
        let result = solved.types[self.term(node)?.0 as usize];
        let Some(Type::Array { element, .. } | Type::Slice(element)) =
            self.facts.types.kind(actual)
        else {
            return Ok(false);
        };
        Ok(idx == TypeId::primitive(Primitive::Usize) && result == element)
    }
    pub(in crate::sema::bodies) fn check_index_receiver_policy(
        &mut self,
        node: NodeId,
        value: NodeId,
        solved: &Solved,
    ) -> Result<(), Error> {
        if self.facts.source_bindings.is_none() || !matches!(self.kind(node), Kind::Index { .. }) {
            return Ok(());
        }
        let selected = self
            .facts
            .methods
            .calls
            .get(&self.key(node))
            .copied()
            .ok_or_else(|| {
                self.error(
                    node,
                    "source index receiver policy lost its actual selected method",
                )
            })?;
        let Some(Operation::Call { signature, .. }) = self
            .facts
            .node(self.key(node))
            .and_then(|fact| fact.operation)
        else {
            return Err(self.error(node, "source index receiver policy lost its actual Call"));
        };
        let Some(Type::Function { parameters, .. }) = self.facts.types.kind(signature) else {
            return Err(self.error(node, "source index receiver policy lost its signature"));
        };
        let Some([receiver, _]) = self.facts.types.list(parameters) else {
            return Err(self.error(
                node,
                "source index receiver policy changed its actual inputs",
            ));
        };
        let pending = self
            .language_index_method(node)
            .and_then(|index| self.state.call_types.methods.get(index))
            .filter(|(source, _)| *source == node)
            .map(|(_, pending)| pending)
            .ok_or_else(|| {
                self.error(node, "source index receiver policy lost its solved borrow")
            })?;
        let [callable, actual, expected] = pending.value_types(solved);
        let (dereferences, borrow) = pending.capture_receiver();
        // Publication retains the real operand borrow and separately records
        // its directed relation to the opened callee receiver. Region aliases
        // may differ; neither place identity nor that relation is interchangeable.
        if selected.method != pending.method()
            || selected.callable != callable
            || selected.borrow != Some(actual)
            || borrow.map(|term| solved.types[term.0 as usize]) != Some(actual)
            || selected.receiver != expected
            || expected != *receiver
            || selected.dereferences != dereferences
            || dereferences != 0
            || !matches!(self.facts.types.kind(selected.callable),Some(Type::FunctionItem{item,..}) if item.0==selected.method.0)
        {
            return Err(self.error(
                node,
                "source index receiver policy changed its checked borrow or callable",
            ));
        }
        self.check_published_raw_receiver(node, value, selected.method)
    }
    pub(in crate::sema::bodies) fn erased_source_index(&self, node: NodeId) -> bool {
        // The existing operation is published only after the solved source
        // classifier and ordinary role/implementation/effect proof succeeded.
        self.facts.source_bindings.is_some()
            && matches!(self.kind(node), Kind::Index { .. })
            && self.facts.node(self.key(node)).is_some_and(|fact| {
                matches!(
                    fact.operation,
                    Some(
                        Operation::Index { .. }
                            | Operation::IndexParameter { .. }
                            | Operation::IndexSlice
                    )
                )
            })
    }
}
