//! Source binary inference retains the actual trait call before primitive erasure.
use super::{Checker, Constructor, Error, NodeId, TermId, Type};
use crate::{
    lex::token::Punctuation,
    sema::compiler_items,
    support::language_hooks::Hook,
    syntax::ast::Kind,
    types::{
        ItemId, Primitive, TypeId,
        infer::{Domain, Solved, arguments::Argument, predicates::Predicate},
    },
};

#[cfg(test)]
#[path = "../../../../tests/invariants/binop_operand_types.rs"]
mod builtin_tests;

/// An operand type as rustc's builtin binary rule classifies it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Operand {
    Integral,
    Float,
    Bool,
    Char,
    Other,
}

impl Checker<'_> {
    /// rustc's builtin binary rule (`is_builtin_binop` and
    /// `enforce_builtin_binop_types`). When neither operand is an unknown type
    /// and, under one shared reference, both are integral or both floating
    /// (bitwise operators also accept two bools, comparisons any two primitive
    /// scalars), the operand types are unified before any implementation is
    /// selected. Shifts relate only their result to the left operand. Returns
    /// the builtin result type for arithmetic, bitwise and shift operators.
    /// Raw and function pointer comparisons keep ordinary trait selection.
    pub(in crate::sema::bodies) fn builtin_binary(
        &mut self,
        node: NodeId,
        op: Punctuation,
        left: TermId,
        right: TermId,
    ) -> Result<Option<TermId>, Error> {
        let op = super::super::scalar::compound_operator(op).unwrap_or(op);
        let (Some((left, a)), Some((right, b))) = (
            self.builtin_operand(node, left)?,
            self.builtin_operand(node, right)?,
        ) else {
            return Ok(None);
        };
        let numeric = a == b && matches!(a, Operand::Integral | Operand::Float);
        let (unify, result) = match op {
            Punctuation::ShiftLeft | Punctuation::ShiftRight => {
                (false, a == Operand::Integral && b == Operand::Integral)
            }
            Punctuation::Plus
            | Punctuation::Minus
            | Punctuation::Star
            | Punctuation::Slash
            | Punctuation::Percent => (numeric, numeric),
            Punctuation::And | Punctuation::Or | Punctuation::Caret => {
                let bitwise = numeric || (a == Operand::Bool && b == Operand::Bool);
                (bitwise, bitwise)
            }
            Punctuation::EqualEqual
            | Punctuation::BangEqual
            | Punctuation::Less
            | Punctuation::LessEqual
            | Punctuation::Greater
            | Punctuation::GreaterEqual => (a != Operand::Other && b != Operand::Other, false),
            _ => (false, false),
        };
        if unify {
            self.equate(node, left, right)?;
        }
        Ok(result.then_some(left))
    }
    /// One shared reference layer is peeled; general unknowns are not builtin.
    fn builtin_operand(
        &mut self,
        node: NodeId,
        term: TermId,
    ) -> Result<Option<(TermId, Operand)>, Error> {
        let (term, head) = match self.settled_head(node, term)? {
            Some(Constructor::Reference { mutable: false, .. }) => {
                let referent = self.child(node, term, 0)?;
                (referent, self.settled_head(node, referent)?)
            }
            head => (term, head),
        };
        let operand = match head {
            Some(Constructor::Primitive(primitive)) => match primitive {
                Primitive::Bool => Operand::Bool,
                Primitive::Char => Operand::Char,
                primitive if primitive.integer().is_some() => Operand::Integral,
                primitive if primitive.is_float() => Operand::Float,
                _ => Operand::Other,
            },
            Some(_) => Operand::Other,
            None => match self
                .state
                .infer
                .unresolved_domain(term)
                .map_err(|error| self.error(node, error))?
            {
                Some(Domain::Integer) => Operand::Integral,
                Some(Domain::Float) => Operand::Float,
                Some(Domain::Any) | None => return Ok(None),
            },
        };
        Ok(Some((term, operand)))
    }
    pub(in crate::sema::bodies) fn binary_readiness(
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
                | Punctuation::Slash
                | Punctuation::Star
                | Punctuation::Percent
                | Punctuation::Caret
                | Punctuation::And
                | Punctuation::ShiftRight),
            ..
        } = self.kind(node)
        else {
            return Ok(None);
        };
        let hook = super::binary_hook(op)
            .ok_or_else(|| self.error(node, "source binary default lost its actual operator"))?;
        if let Some(binding) =
            compiler_items::binary_pending_default(self.report, self.facts, self.key(node), hook)?
        {
            return Ok(Some(super::super::Requirement::GenericDefault(binding)));
        }
        // A declaration header, such as an impl self-type length, can need the
        // operation members before the fixed passes prepare them.
        compiler_items::binary_pending_member(self.report, self.facts, self.key(node), hook)
            .map(|member| member.map(super::super::Requirement::Definition))
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
        let Kind::Binary { op, .. } = self.kind(node) else {
            return Err(self.error(node, "source binary lost its actual operator"));
        };
        if let Some(builtin) = self.builtin_binary(node, op, subject, rhs)? {
            // The selected Output must still agree with the builtin result.
            self.equate(node, builtin, result)?;
            return Ok(builtin);
        }
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
                | Punctuation::Slash
                | Punctuation::Star
                | Punctuation::Percent
                | Punctuation::Caret
                | Punctuation::And
                | Punctuation::ShiftRight,
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
        if left == right
            && matches!(self.facts.types.kind(left), Some(Type::Pointer { .. }))
            && matches!(op, Punctuation::EqualEqual | Punctuation::BangEqual)
        {
            if self.facts.types.kind(result) != Some(Type::Primitive(crate::types::Primitive::Bool))
            {
                return Err(self.error(node, "source pointer comparison result must be bool"));
            }
            if self.callee_use_effect()? != crate::support::constness::Constness::Ordinary {
                return Err(self.error(
                    node,
                    "raw pointer comparisons are not allowed in const contexts",
                ));
            }
            // The actual source trait/where proof remains. Only operator syntax
            // is primitive; explicit method calls execute their real source body.
            return Ok(true);
        }
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
        if self.erased_source_assignment(node)? || self.erased_source_index(node) {
            return Ok(true);
        }
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
                | Punctuation::Slash
                | Punctuation::Star
                | Punctuation::Percent
                | Punctuation::Caret
                | Punctuation::And
                | Punctuation::ShiftRight,
            left,
            right,
        } = self.kind(node)
        else {
            return Ok(false);
        };
        self.source_scalar_binary(node, self.ty(left)?, self.ty(right)?, self.ty(node)?)
    }
}
