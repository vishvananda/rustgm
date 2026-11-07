//! Source unary typing keeps real trait/effect obligations before scalar erasure.
use super::{Checker, Constructor, Error, NodeId, TermId, Type};
use crate::{
    sema::compiler_items,
    syntax::ast::{Kind, Unary},
    types::{ItemId, TypeId, infer::predicates::Predicate},
};

impl Checker<'_> {
    /// A declaration header, such as an array length, can need the Neg or Not
    /// members before the fixed passes prepare them.
    pub(in crate::sema::bodies) fn unary_readiness(
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
        let Kind::Unary {
            op: op @ (Unary::Negate | Unary::Not),
            ..
        } = self.kind(node)
        else {
            return Ok(None);
        };
        compiler_items::unary_pending_member(
            self.report,
            self.facts,
            self.key(node),
            op == Unary::Negate,
        )
        .map(|member| member.map(super::super::Requirement::Definition))
    }
    pub(in crate::sema::bodies) fn source_unary_operator(
        &mut self,
        node: NodeId,
        operator: Unary,
        operand: NodeId,
    ) -> Result<TermId, Error> {
        let site = self.key(node);
        let selected = match operator {
            Unary::Not => compiler_items::source_not(self.report, self.facts, site)?,
            Unary::Negate => compiler_items::source_neg(self.report, self.facts, site)?,
            _ => return Err(self.error(node, "invalid source unary operation")),
        };
        let subject = self.term(operand)?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application(subject, ItemId(selected.definition.0), Vec::new()),
        ));
        self.complete_method_arguments(node, selected.method)?;
        let ty = self.facts.definitions[selected.method.0 as usize]
            .ok_or_else(|| self.error(node, "source unary lost its actual callable"))?;
        if !matches!(self.facts.types.kind(ty), Some(Type::FunctionItem {item,signature,..})
            if item.0 == selected.method.0 && signature == selected.signature)
        {
            return Err(self.error(
                node,
                "source unary callable differs from its checked header",
            ));
        }
        // This is the ordinary source-method instantiator, including inherited
        // clauses and the actual const use effect. No fabricated library header.
        let callable = self.generic_function(node, selected.method, ty)?;
        let signature = self.child(node, callable, 0)?;
        // The authenticated method has no late binders. Its complete signature
        // already has an ordinary pending normalization; do not demand Output
        // before connecting Self to the actual operand and contextual numbers.
        let receiver = self.child(node, signature, 0)?;
        self.shape_equation(node, subject, receiver)?;
        let result = self.child(node, signature, 1)?;
        self.retain_language_method(node, selected.method, callable, subject);
        self.state.calls.push((node, signature));
        // An Output that waits for its impl's definition leaves the body loop
        // to suspend on it; classifying the operand never demands it here.
        let scalar = match self.settled_head(node, subject)? {
            Some(Constructor::Primitive(primitive)) => {
                super::super::scalar::unary(operator, primitive)
            }
            None => matches!(
                (
                    operator,
                    self.state
                        .infer
                        .unresolved_domain(subject)
                        .map_err(|error| self.error(node, error))?
                ),
                (
                    Unary::Not | Unary::Negate,
                    Some(super::super::Domain::Integer)
                ) | (Unary::Negate, Some(super::super::Domain::Float))
            ),
            _ => false,
        };
        if scalar {
            // Match scalar type inference without skipping the method's clauses.
            Ok(subject)
        } else {
            Ok(result)
        }
    }

    /// A negative literal pattern calls no method, but like rustc it still
    /// requires the `neg` language item and proves `T: Neg` for its type.
    pub(in crate::sema::bodies) fn source_negative_pattern(
        &mut self,
        node: NodeId,
        term: TermId,
    ) -> Result<(), Error> {
        let site = self.key(node);
        let selected = compiler_items::source_neg(self.report, self.facts, site)?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application(term, ItemId(selected.definition.0), Vec::new()),
        ));
        Ok(())
    }

    pub(in crate::sema::bodies) fn source_scalar_unary_type(
        &self,
        node: NodeId,
        operand: TypeId,
    ) -> bool {
        let Kind::Unary {
            op: operator @ (Unary::Not | Unary::Negate),
            ..
        } = self.kind(node)
        else {
            return false;
        };
        self.facts.source_bindings.is_some()
            && matches!(self.facts.types.kind(operand), Some(Type::Primitive(p))
                if super::super::scalar::unary(operator, p))
    }

    pub(in crate::sema::bodies) fn erased_source_unary(&self, node: NodeId) -> Result<bool, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        let Kind::Unary {
            op: Unary::Not | Unary::Negate,
            operand,
        } = self.kind(node)
        else {
            return Ok(false);
        };
        Ok(self.source_scalar_unary_type(node, self.ty(operand)?))
    }
}
