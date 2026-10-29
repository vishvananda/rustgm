//! Source binary inference retains the actual trait call before primitive erasure.
use super::{Checker, Error, NodeId, TermId, Type};
use crate::{
    lex::token::Punctuation,
    sema::compiler_items,
    support::language_hooks::Hook,
    syntax::ast::Kind,
    types::{
        ItemId, TypeId,
        infer::{Solved, arguments::Argument, predicates::Predicate},
    },
};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn binary_default_readiness(
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
        let Kind::Binary {
            op:
                op @ (Punctuation::ShiftLeft
                | Punctuation::Or
                | Punctuation::Plus
                | Punctuation::Minus
                | Punctuation::Slash),
            ..
        } = self.kind(node)
        else {
            return Ok(None);
        };
        let hook = super::binary_hook(op)
            .ok_or_else(|| self.error(node, "source binary default lost its actual operator"))?;
        compiler_items::binary_pending_default(self.report, self.facts, self.key(node), hook)
            .map(|binding| binding.map(super::super::Requirement::GenericDefault))
    }
    pub(in crate::sema::bodies) fn source_binary_operator(
        &mut self,
        node: NodeId,
        hook: Hook,
        left: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        let selected =
            compiler_items::source_binary(self.report, self.facts, self.key(node), hook)?;
        if !matches!(
            self.facts.types.kind(selected.rhs),
            Some(Type::Parameter(_))
        ) {
            return Err(self.error(node, "source binary descriptor lost its actual Rhs"));
        }
        let subject = self.term(left)?;
        let rhs = self.term(right)?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application(
                subject,
                ItemId(selected.definition.0),
                vec![Argument::Type(rhs)],
            ),
        ));
        self.complete_method_arguments(node, selected.method)?;
        let ty = self.facts.definitions[selected.method.0 as usize]
            .ok_or_else(|| self.error(node, "source binary lost its actual callable"))?;
        if !matches!(self.facts.types.kind(ty), Some(Type::FunctionItem { item, signature, .. })
            if item.0 == selected.method.0 && signature == selected.signature)
        {
            return Err(self.error(
                node,
                "source binary callable differs from its checked header",
            ));
        }
        // The existing function instantiator schedules Output normalization and
        // actual inherited/const clauses. Connect both operands before it resumes.
        let callable = self.generic_function(node, selected.method, ty)?;
        let signature = self.child(node, callable, 0)?;
        let receiver = self.child(node, signature, 0)?;
        self.shape_equation(node, subject, receiver)?;
        let expected_rhs = self.child(node, signature, 1)?;
        self.shape_equation(node, rhs, expected_rhs)?;
        let result = self.child(node, signature, 2)?;
        self.retain_language_method(node, selected.method, callable, subject);
        self.state.calls.push((node, signature));
        Ok(result)
    }

    pub(in crate::sema::bodies) fn source_scalar_binary_solved(
        &self,
        node: NodeId,
        left: TypeId,
        solved: &Solved,
    ) -> Result<bool, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        let Kind::Binary {
            op:
                Punctuation::ShiftLeft
                | Punctuation::Or
                | Punctuation::Plus
                | Punctuation::Minus
                | Punctuation::Slash,
            right,
            ..
        } = self.kind(node)
        else {
            return Ok(false);
        };
        let right = solved.types[self.term(right)?.0 as usize];
        let result = solved.types[self.term(node)?.0 as usize];
        self.source_scalar_binary(node, left, right, result)
    }

    fn source_scalar_binary(
        &self,
        node: NodeId,
        left: TypeId,
        right: TypeId,
        result: TypeId,
    ) -> Result<bool, Error> {
        let Kind::Binary { op, .. } = self.kind(node) else {
            return Ok(false);
        };
        let (Some(Type::Primitive(a)), Some(Type::Primitive(b))) =
            (self.facts.types.kind(left), self.facts.types.kind(right))
        else {
            return Ok(false);
        };
        if !super::super::scalar::binary(op, a, b) || result != left {
            return Err(self.error(node, "source primitive binary result or operands require unsupported scalar representation"));
        }
        Ok(true)
    }

    pub(in crate::sema::bodies) fn source_scalar_comparison_solved(
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
        if !matches!(
            op,
            Punctuation::EqualEqual
                | Punctuation::BangEqual
                | Punctuation::Less
                | Punctuation::LessEqual
                | Punctuation::Greater
                | Punctuation::GreaterEqual
        ) {
            return Ok(false);
        }
        self.source_scalar_comparison(
            node,
            op,
            solved.types[self.term(left)?.0 as usize],
            solved.types[self.term(right)?.0 as usize],
            solved.types[self.term(node)?.0 as usize],
        )
    }
    fn source_scalar_comparison(
        &self,
        node: NodeId,
        op: Punctuation,
        left: TypeId,
        right: TypeId,
        result: TypeId,
    ) -> Result<bool, Error> {
        let (Some(Type::Primitive(a)), Some(Type::Primitive(b))) =
            (self.facts.types.kind(left), self.facts.types.kind(right))
        else {
            return Ok(false);
        };
        if !super::super::scalar::binary(op, a, b)
            || self.facts.types.kind(result) != Some(Type::Primitive(crate::types::Primitive::Bool))
        {
            return Err(self.error(
                node,
                "source comparison operands/result require unsupported scalar representation",
            ));
        }
        Ok(true)
    }
    pub(in crate::sema::bodies) fn erased_source_operator(
        &self,
        node: NodeId,
    ) -> Result<bool, Error> {
        if self.erased_source_unary(node)? {
            return Ok(true);
        }
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        if let Kind::Binary { op, left, right } = self.kind(node)
            && matches!(
                op,
                Punctuation::EqualEqual
                    | Punctuation::BangEqual
                    | Punctuation::Less
                    | Punctuation::LessEqual
                    | Punctuation::Greater
                    | Punctuation::GreaterEqual
            )
        {
            return self.source_scalar_comparison(
                node,
                op,
                self.ty(left)?,
                self.ty(right)?,
                self.ty(node)?,
            );
        }
        let Kind::Binary {
            op:
                Punctuation::ShiftLeft
                | Punctuation::Or
                | Punctuation::Plus
                | Punctuation::Minus
                | Punctuation::Slash,
            left,
            right,
        } = self.kind(node)
        else {
            return Ok(false);
        };
        self.source_scalar_binary(node, self.ty(left)?, self.ty(right)?, self.ty(node)?)
    }
}
