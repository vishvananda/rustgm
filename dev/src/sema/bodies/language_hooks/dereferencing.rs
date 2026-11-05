//! Dereference syntax selects the actual source method and borrows its receiver once.
use super::super::{Checker, Error, NodeId, TermId, Type};
use crate::types::{ItemId, infer::predicates::Predicate};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn source_dereference(
        &mut self,
        node: NodeId,
        operand: NodeId,
        mutable: bool,
    ) -> Result<TermId, Error> {
        let selected = crate::sema::compiler_items::dereferencing::query(
            self.report,
            self.facts,
            self.key(node),
            mutable,
        )?;
        let subject = self.term(operand)?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application(subject, ItemId(selected.trait_definition.0), Vec::new()),
        ));
        self.complete_method_arguments(node, selected.method)?;
        let item = self.facts.definitions[selected.method.0 as usize]
            .ok_or_else(|| self.error(node, "source dereference lost its actual method item"))?;
        if !matches!(self.facts.types.kind(item),Some(Type::FunctionItem{item,signature,..}) if item.0==selected.method.0 && signature==selected.signature)
        {
            return Err(self.error(
                node,
                "source dereference method differs from its actual header",
            ));
        }
        let callable = self.generic_function(node, selected.method, item)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        let receiver = self.child(node, signature, 0)?;
        let receiver_subject = self.child(node, receiver, 0)?;
        self.shape_equation(node, subject, receiver_subject)?;
        let result = self.child(node, signature, 1)?;
        let target = self.child(node, result, 0)?;
        let pending = self.retain_language_borrow(
            node,
            operand,
            selected.method,
            callable,
            subject,
            receiver,
        );
        self.retain_dereference_call(node, mutable, pending, signature);
        Ok(target)
    }
}
