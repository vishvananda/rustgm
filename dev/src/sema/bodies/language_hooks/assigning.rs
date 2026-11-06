//! Source compound operators borrow their actual mutable receiver exactly once.
use super::super::{Checker, Error, NodeId, TermId, Type};
use crate::{
    sema::compiler_items,
    support::language_hooks::Hook,
    syntax::ast::Kind,
    types::{
        ItemId, TypeId,
        infer::{Solved, arguments::Argument, predicates::Predicate},
    },
};
impl Checker<'_> {
    pub(in crate::sema::bodies) fn source_assignment_default_readiness(
        &self,
        node: NodeId,
    ) -> Result<Option<super::super::Requirement>, Error> {
        if self.facts.source_bindings.is_none()
            || !matches!(
                self.availability,
                crate::sema::traits::implementations::Availability::Declarations
            )
        {
            return Ok(None);
        }
        let Kind::Binary { op, .. } = self.kind(node) else {
            return Ok(None);
        };
        let Some(hook) = super::assignment_hook(op) else {
            return Ok(None);
        };
        compiler_items::binary_pending_default(self.report, self.facts, self.key(node), hook)
            .map(|binding| binding.map(super::super::Requirement::GenericDefault))
    }
    pub(in crate::sema::bodies) fn source_assignment_operator(
        &mut self,
        node: NodeId,
        hook: Hook,
        left: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        let selected =
            compiler_items::source_assignment(self.report, self.facts, self.key(node), hook)?;
        let subject = self.term(left)?;
        let rhs = self.term(right)?;
        let effect = self.callee_use_effect()?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application_with_effect(
                subject,
                ItemId(selected.definition.0),
                vec![Argument::Type(rhs)],
                effect,
            ),
        ));
        self.complete_method_arguments(node, selected.method)?;
        let item = self.facts.definitions[selected.method.0 as usize]
            .ok_or_else(|| self.error(node, "source assignment lost its actual callable"))?;
        if !matches!(self.facts.types.kind(item),Some(Type::FunctionItem{item,signature,..}) if item.0==selected.method.0 && signature==selected.signature)
        {
            return Err(self.error(
                node,
                "source assignment callable differs from its checked header",
            ));
        }
        let callable = self.generic_function(node, selected.method, item)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        let receiver = self.child(node, signature, 0)?;
        let referent = self.child(node, receiver, 0)?;
        self.shape_equation(node, subject, referent)?;
        let expected_rhs = self.child(node, signature, 1)?;
        self.shape_equation(node, rhs, expected_rhs)?;
        let result = self.child(node, signature, 2)?;
        let actual = self.operand_borrow(node, subject, receiver, true)?;
        let pending =
            self.retain_language_borrow(node, left, selected.method, callable, subject, actual);
        self.state.call_types.methods[pending]
            .1
            .expect_receiver(receiver);
        self.state.calls.push((node, signature));
        Ok(result)
    }
    pub(in crate::sema::bodies) fn source_scalar_assignment_solved(
        &self,
        node: NodeId,
        solved: &Solved,
    ) -> Result<bool, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        let Kind::Binary { op, left, right } = self.kind(node) else {
            return Ok(false);
        };
        if super::assignment_hook(op).is_none() {
            return Ok(false);
        }
        self.source_scalar_assignment_types(
            node,
            solved.types[self.term(left)?.0 as usize],
            solved.types[self.term(right)?.0 as usize],
            solved.types[self.term(node)?.0 as usize],
        )
    }
    pub(in crate::sema::bodies) fn erased_source_assignment(
        &self,
        node: NodeId,
    ) -> Result<bool, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        let Kind::Binary { op, left, right } = self.kind(node) else {
            return Ok(false);
        };
        if super::assignment_hook(op).is_none() {
            return Ok(false);
        }
        self.source_scalar_assignment_types(node, self.ty(left)?, self.ty(right)?, self.ty(node)?)
    }
    fn source_scalar_assignment_types(
        &self,
        node: NodeId,
        left: TypeId,
        right: TypeId,
        result: TypeId,
    ) -> Result<bool, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        let Kind::Binary { op, .. } = self.kind(node) else {
            return Ok(false);
        };
        let Some(operator) = super::super::scalar::compound_operator(op) else {
            return Ok(false);
        };
        let (Some(Type::Primitive(a)), Some(Type::Primitive(b))) =
            (self.facts.types.kind(left), self.facts.types.kind(right))
        else {
            return Ok(false);
        };
        if result != TypeId::UNIT || !super::super::scalar::binary(operator, a, b) {
            return Err(self.error(
                node,
                "source primitive assignment requires its exact scalar operands and unit result",
            ));
        }
        Ok(true)
    }
}
