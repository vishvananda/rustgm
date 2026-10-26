//! Source unary typing keeps real trait/effect obligations before scalar erasure.
use super::{Checker, Constructor, Error, NodeId, TermId, Type};
use crate::{
    sema::compiler_items,
    syntax::ast::{Kind, Unary},
    types::{ItemId, TypeId, infer::predicates::Predicate},
};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn source_not_operator(
        &mut self,
        node: NodeId,
        operand: NodeId,
    ) -> Result<TermId, Error> {
        let site = self.key(node);
        let selected = compiler_items::source_not(self.report, self.facts, site)?;
        let subject = self.term(operand)?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application(subject, ItemId(selected.definition.0), Vec::new()),
        ));
        self.complete_method_arguments(node, selected.method)?;
        let ty = self.facts.definitions[selected.method.0 as usize]
            .ok_or_else(|| self.error(node, "source Not lost its actual callable"))?;
        if !matches!(self.facts.types.kind(ty), Some(Type::FunctionItem {item,signature,..})
            if item.0 == selected.method.0 && signature == selected.signature)
        {
            return Err(self.error(node, "source Not callable differs from its checked header"));
        }
        // This is the ordinary source-method instantiator, including inherited
        // clauses and the actual const use effect. No fabricated library header.
        let callable = self.generic_function(node, selected.method, ty)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        let receiver = self.child(node, signature, 0)?;
        self.shape_equation(node, subject, receiver)?;
        let result = self.child(node, signature, 1)?;
        self.retain_language_method(node, selected.method, callable, subject);
        self.state.calls.push((node, signature));
        if matches!(self.head(node, subject)?, Some(Constructor::Primitive(p))
            if super::super::scalar::unary(Unary::Not, p))
        {
            // Match scalar type inference without skipping the method's clauses.
            Ok(subject)
        } else {
            Ok(result)
        }
    }

    pub(in crate::sema::bodies) fn source_scalar_not_type(
        &self,
        node: NodeId,
        operand: TypeId,
    ) -> bool {
        self.facts.source_bindings.is_some()
            && matches!(self.kind(node), Kind::Unary { op: Unary::Not, .. })
            && matches!(self.facts.types.kind(operand), Some(Type::Primitive(p))
                if super::super::scalar::unary(Unary::Not, p))
    }

    pub(in crate::sema::bodies) fn erased_source_not(&self, node: NodeId) -> Result<bool, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        let Kind::Unary {
            op: Unary::Not,
            operand,
        } = self.kind(node)
        else {
            return Ok(false);
        };
        Ok(self.source_scalar_not_type(node, self.ty(operand)?))
    }
}
