//! Operators select explicit library identities through ordinary trait obligations.
use super::{Checker, Constructor, Error, NodeId, TermId, Type};
use crate::{
    support::language_hooks::Hook,
    types::{
        ItemId,
        infer::{arguments::Argument, predicates::Predicate},
    },
};
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
        if kind == Hook::Index {
            subject = self.autoderef(node, subject)?;
        }
        let mut arguments = vec![Argument::Type(subject)];
        let formal = self.facts.definitions[declaration.self_parameter.0 as usize]
            .ok_or_else(|| self.error(node, "missing operator Self parameter"))?;
        let id = crate::sema::generics::parameter_id(
            &self.facts.types,
            crate::types::arguments::Argument::Type(formal),
        )
        .map_err(|e| self.error(node, e))?;
        let mut bindings = vec![(id, Argument::Type(subject))];
        let mut trait_arguments = Vec::new();
        if let Some(right) = right {
            let value = self.term(right)?;
            let formal = declaration
                .argument
                .and_then(|id| self.facts.definitions[id.0 as usize])
                .ok_or_else(|| self.error(node, "missing operator argument parameter"))?;
            let id = crate::sema::generics::parameter_id(
                &self.facts.types,
                crate::types::arguments::Argument::Type(formal),
            )
            .map_err(|e| self.error(node, e))?;
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
        if kind == Hook::Index {
            let expected = self.child(node, signature, 0)?;
            self.retain_language_index(node, left, declaration.method, callable, subject, expected);
        } else {
            self.retain_language_method(node, declaration.method, callable, subject);
        }
        self.state.calls.push((node, signature));
        if kind == Hook::Index {
            self.child(node, result, 0)
        } else {
            Ok(result)
        }
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
}

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
