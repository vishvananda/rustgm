//! Operators select explicit library identities through ordinary trait obligations.
mod mutation;
#[path = "language_hooks/binary_operator.rs"]
mod source_binary;
#[path = "language_hooks/unary_operator.rs"]
mod source_unary;
use super::{Checker, Constructor, Error, NodeId, TermId, Type};
use crate::{
    support::language_hooks::Hook,
    types::{
        ItemId,
        infer::{arguments::Argument, predicates::Predicate},
    },
};
pub(super) use mutation::Places;
impl Checker<'_> {
    pub(super) fn language_operator(
        &mut self,
        node: NodeId,
        kind: Hook,
        left: NodeId,
        right: Option<NodeId>,
    ) -> Result<TermId, Error> {
        let declaration = self
            .report
            .resolution
            .builtin_traits
            .language(kind)
            .ok_or_else(|| {
                self.error(node, "operator requires its library trait implementation")
            })?;
        let mut subject = self.term(left)?;
        if kind.indexing() {
            subject = self.autoderef(node, subject)?;
        }
        let mut arguments = vec![Argument::Type(subject)];
        let id = self.operator_parameter(node, declaration.self_parameter)?;
        let mut bindings = vec![(id, Argument::Type(subject))];
        let mut trait_arguments = Vec::new();
        if let Some(right) = right {
            let value = self.term(right)?;
            let formal = declaration
                .argument
                .ok_or_else(|| self.error(node, "missing operator argument parameter"))?;
            let id = self.operator_parameter(node, formal)?;
            bindings.push((id, Argument::Type(value)));
            arguments.push(Argument::Type(value));
            trait_arguments.push(Argument::Type(value));
        }
        let goal = Predicate::trait_application(
            subject,
            ItemId(declaration.definition.0),
            trait_arguments,
        );
        self.state.call_types.obligations.push((node, goal));
        let ty = self.facts.definitions[declaration.method.0 as usize]
            .ok_or_else(|| self.error(node, "missing operator method type"))?;
        let Some(Type::FunctionItem { signature, .. }) = self.facts.types.kind(ty) else {
            return Err(self.error(node, "operator method is not callable"));
        };
        let signature = self
            .state
            .infer
            .instantiate(&mut self.facts.types, signature, &bindings)
            .map_err(|e| self.error(node, e))?;
        self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, signature)?;
        let callable = self
            .state
            .infer
            .function_item(ItemId(declaration.method.0), signature, &arguments)
            .map_err(|e| self.error(node, e))?;
        let signature = self.call_signature(node, signature)?;
        let arity = usize::from(right.is_some()) + 1;
        let result = self.child(node, signature, arity)?;
        if kind.borrowed() {
            let expected = self.child(node, signature, 0)?;
            let pending = self.retain_language_borrow(
                node,
                left,
                declaration.method,
                callable,
                subject,
                expected,
            );
            if kind.indexing() {
                self.retain_language_index_call(node, kind, pending, signature);
            } else {
                self.state.calls.push((node, signature));
            }
        } else {
            self.retain_language_method(node, declaration.method, callable, subject);
            self.state.calls.push((node, signature));
        }
        if kind.indexing() {
            self.child(node, result, 0)
        } else {
            Ok(result)
        }
    }
    pub(super) fn operator_parameter(
        &self,
        node: NodeId,
        definition: crate::resolve::model::DefId,
    ) -> Result<crate::types::parameters::ParameterId, Error> {
        let formal = self.facts.definitions[definition.0 as usize]
            .ok_or_else(|| self.error(node, "missing operator parameter type"))?;
        crate::sema::generics::parameter_id(
            &self.facts.types,
            crate::types::arguments::Argument::Type(formal),
        )
        .map_err(|e| self.error(node, e))
    }
    pub(super) fn uses_language_operator(
        &mut self,
        node: NodeId,
        kind: Hook,
        operand: NodeId,
    ) -> Result<bool, Error> {
        if self
            .report
            .resolution
            .builtin_traits
            .language(kind)
            .is_none()
        {
            return Ok(false);
        }
        let term = self.term(operand)?;
        Ok(!matches!(
            self.head(node, term)?,
            None | Some(Constructor::Primitive(_) | Constructor::Never)
        ))
    }
    pub(super) fn uses_language_assignment(
        &mut self,
        node: NodeId,
        kind: Hook,
        left: NodeId,
        right: NodeId,
    ) -> Result<bool, Error> {
        if self
            .report
            .resolution
            .builtin_traits
            .language(kind)
            .is_none()
        {
            return Ok(false);
        }
        for operand in [left, right] {
            let term = self.term(operand)?;
            match self.head(node, term)? {
                Some(Constructor::Primitive(_) | Constructor::Never) => {}
                None => {
                    let domain = self
                        .state
                        .infer
                        .unresolved_domain(term)
                        .map_err(|e| self.error(node, e))?;
                    if !matches!(domain, Some(super::Domain::Integer | super::Domain::Float)) {
                        return Ok(true);
                    }
                }
                _ => return Ok(true),
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/operator_authority.rs"]
mod binary_guards;

pub(super) const fn unary_hook(operator: crate::syntax::ast::Unary) -> Option<Hook> {
    use crate::syntax::ast::Unary;
    match operator {
        Unary::Negate => Some(Hook::Neg),
        Unary::Not => Some(Hook::Not),
        _ => None,
    }
}
pub(super) const fn binary_hook(operator: crate::lex::token::Punctuation) -> Option<Hook> {
    use crate::lex::token::Punctuation;
    match operator {
        Punctuation::Plus => Some(Hook::Add),
        Punctuation::Minus => Some(Hook::Sub),
        Punctuation::Star => Some(Hook::Mul),
        Punctuation::Slash => Some(Hook::Div),
        Punctuation::Percent => Some(Hook::Rem),
        Punctuation::And => Some(Hook::BitAnd),
        Punctuation::Or => Some(Hook::BitOr),
        Punctuation::Caret => Some(Hook::BitXor),
        Punctuation::ShiftLeft => Some(Hook::Shl),
        Punctuation::ShiftRight => Some(Hook::Shr),
        _ => None,
    }
}
pub(super) const fn assignment_hook(operator: crate::lex::token::Punctuation) -> Option<Hook> {
    use crate::lex::token::Punctuation;
    match operator {
        Punctuation::PlusEqual => Some(Hook::AddAssign),
        Punctuation::MinusEqual => Some(Hook::SubAssign),
        Punctuation::StarEqual => Some(Hook::MulAssign),
        Punctuation::SlashEqual => Some(Hook::DivAssign),
        Punctuation::PercentEqual => Some(Hook::RemAssign),
        Punctuation::AndEqual => Some(Hook::BitAndAssign),
        Punctuation::OrEqual => Some(Hook::BitOrAssign),
        Punctuation::CaretEqual => Some(Hook::BitXorAssign),
        Punctuation::ShiftLeftEqual => Some(Hook::ShlAssign),
        Punctuation::ShiftRightEqual => Some(Hook::ShrAssign),
        _ => None,
    }
}
