//! Dereference syntax selects the actual source method and borrows its receiver once.
use super::super::{Checker, DefId, Error, NodeId, TermId, Type};
use crate::types::{ItemId, infer::predicates::Predicate};

/// Transient actual call inputs; final body/region proof supplies permission.
pub(in crate::sema::bodies) struct DereferenceCall {
    pub(in crate::sema::bodies) method: DefId,
    pub(in crate::sema::bodies) callable: TermId,
    pub(in crate::sema::bodies) signature: TermId,
    pub(in crate::sema::bodies) receiver: TermId,
    pub(in crate::sema::bodies) borrow: TermId,
    pub(in crate::sema::bodies) target: TermId,
}
const _: [(); 24] = [(); size_of::<DereferenceCall>()];

impl Checker<'_> {
    pub(in crate::sema::bodies) fn source_dereference(
        &mut self,
        node: NodeId,
        operand: NodeId,
        mutable: bool,
    ) -> Result<TermId, Error> {
        let subject = self.term(operand)?;
        let call = self.dereference_call(node, subject, mutable)?;
        let pending = self.retain_language_borrow(
            node,
            operand,
            call.method,
            call.callable,
            subject,
            call.borrow,
        );
        self.state.call_types.methods[pending]
            .1
            .expect_receiver(call.receiver);
        self.retain_dereference_call(node, mutable, pending, call.signature)?;
        Ok(call.target)
    }

    pub(in crate::sema::bodies) fn dereference_call(
        &mut self,
        node: NodeId,
        subject: TermId,
        mutable: bool,
    ) -> Result<DereferenceCall, Error> {
        let selected = crate::sema::compiler_items::dereferencing::query(
            self.report,
            self.facts,
            self.key(node),
            mutable,
        )?;
        let effect = self.callee_use_effect()?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application_with_effect(
                subject,
                ItemId(selected.trait_definition.0),
                Vec::new(),
                effect,
            ),
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
        let actual = self.operand_borrow(node, subject, receiver, mutable)?;
        let result = self.child(node, signature, 1)?;
        let target = self.child(node, result, 0)?;
        Ok(DereferenceCall {
            method: selected.method,
            callable,
            signature,
            receiver,
            borrow: actual,
            target,
        })
    }
}
