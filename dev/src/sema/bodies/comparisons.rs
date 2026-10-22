//! Equality syntax retains canonical member selection and two independent shared borrows.
use super::{Checker, Constructor, Error, NodeId, TermId, Type};
use crate::{
    support::comparisons::Method,
    types::{
        ItemId,
        infer::{arguments::Argument, predicates::Predicate},
    },
};
impl Checker<'_> {
    pub(super) fn comparison_readiness(
        &mut self,
        node: NodeId,
    ) -> Result<Option<super::Requirement>, Error> {
        let method = match self.kind(node) {
            crate::syntax::ast::Kind::Binary { op, .. } => match op {
                crate::lex::token::Punctuation::EqualEqual => Method::Eq,
                crate::lex::token::Punctuation::BangEqual => Method::Ne,
                crate::lex::token::Punctuation::Less => Method::Lt,
                crate::lex::token::Punctuation::LessEqual => Method::Le,
                crate::lex::token::Punctuation::Greater => Method::Gt,
                crate::lex::token::Punctuation::GreaterEqual => Method::Ge,
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        let declaration = self
            .report
            .resolution
            .builtin_traits
            .comparison(method.owner());
        let Some(declaration) = declaration else {
            return Ok(None);
        };
        if self
            .facts
            .associated
            .comparison(declaration.definition)
            .is_some()
        {
            return Ok(None);
        }
        let crate::syntax::ast::Kind::Binary { left, right, .. } = self.kind(node) else {
            return Err(self.error(node, "invalid comparison readiness site"));
        };
        if !self.uses_comparison(node, method, left, right)? {
            return Ok(None);
        }
        if matches!(
            self.availability,
            crate::sema::traits::implementations::Availability::Declarations
        ) {
            return Ok(Some(super::Requirement::Definition(declaration.definition)));
        }
        // Only this source-free latent family can be completed synchronously;
        // every ordinary Validated dependency retains its existing invariant.
        crate::sema::declarations::prepare_comparison(
            self.report,
            self.facts,
            declaration.definition,
        )?;
        Ok(None)
    }
    pub(super) fn uses_comparison(
        &mut self,
        node: NodeId,
        method: Method,
        left: NodeId,
        right: NodeId,
    ) -> Result<bool, Error> {
        if self
            .report
            .resolution
            .builtin_traits
            .comparison(method.owner())
            .is_none()
        {
            return Ok(false);
        }
        for operand in [left, right] {
            let term = self.term(operand)?;
            match self.head(node, term)? {
                Some(
                    Constructor::Primitive(_) | Constructor::Never | Constructor::Pointer { .. },
                ) => {}
                None => {
                    if !matches!(
                        self.state
                            .infer
                            .unresolved_domain(term)
                            .map_err(|e| self.error(node, e))?,
                        Some(super::Domain::Integer | super::Domain::Float)
                    ) {
                        return Ok(true);
                    }
                }
                _ => return Ok(true),
            }
        }
        Ok(false)
    }
    pub(super) fn comparison(
        &mut self,
        node: NodeId,
        method: Method,
        left: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        let declaration = self
            .report
            .resolution
            .builtin_traits
            .comparison(method.owner())
            .ok_or_else(|| self.error(node, "comparison requires PartialEq implementation"))?;
        let methods = declaration
            .methods
            .ok_or_else(|| self.error(node, "missing comparison methods"))?;
        let method = methods
            .get(method)
            .ok_or_else(|| self.error(node, "comparison method is not in canonical owner"))?;
        let actual_subject = self.term(left)?;
        let actual_right = self.term(right)?;
        let (subject, right_type) = if declaration.kind
            == crate::support::comparisons::Comparison::PartialOrd
            && self.sequence_operand(node, actual_subject)?
        {
            (
                self.inferred_regions(node, actual_subject)?,
                self.inferred_regions(node, actual_right)?,
            )
        } else {
            (actual_subject, actual_right)
        };
        let rhs = declaration
            .argument
            .ok_or_else(|| self.error(node, "missing comparison argument"))?;
        let bindings = [
            (
                self.operator_parameter(node, declaration.self_parameter)?,
                Argument::Type(subject),
            ),
            (
                self.operator_parameter(node, rhs)?,
                Argument::Type(right_type),
            ),
        ];
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application(
                subject,
                ItemId(declaration.definition.0),
                vec![Argument::Type(right_type)],
            ),
        ));
        let Some(Type::FunctionItem { signature, .. }) =
            self.facts.definitions[method.0 as usize].and_then(|ty| self.facts.types.kind(ty))
        else {
            return Err(self.error(node, "missing comparison method signature"));
        };
        let signature = self
            .state
            .infer
            .instantiate(&mut self.facts.types, signature, &bindings)
            .map_err(|e| self.error(node, e))?;
        let callable = self
            .state
            .infer
            .function_item(
                ItemId(method.0),
                signature,
                &[Argument::Type(subject), Argument::Type(right_type)],
            )
            .map_err(|e| self.error(node, e))?;
        let signature = self.call_signature(node, signature)?;
        let receiver = self.child(node, signature, 0)?;
        let actual = self.comparison_borrow(node, actual_subject, receiver)?;
        let pending =
            self.retain_language_borrow(node, left, method, callable, actual_subject, actual);
        self.state.call_types.methods[pending]
            .1
            .expect_receiver(receiver);
        let expected = self.child(node, signature, 1)?;
        let actual = self.comparison_borrow(node, actual_right, expected)?;
        self.state.call_types.field_receivers.push(actual_right);
        self.state
            .call_types
            .comparison_borrows
            .push((node, actual, expected));
        self.state.alignment.borrows.push((node, right));
        self.state.calls.push((node, signature));
        self.child(node, signature, 2)
    }
    fn sequence_operand(&mut self, node: NodeId, mut term: TermId) -> Result<bool, Error> {
        loop {
            match self.state.infer.head(&self.facts.types, term) {
                Ok(Some(Constructor::Reference { .. })) => {
                    term = self
                        .state
                        .infer
                        .child(&self.facts.types, term, 0)
                        .map_err(|error| self.error(node, error))?;
                }
                Ok(Some(
                    Constructor::Array(_)
                    | Constructor::ArrayTerm(_)
                    | Constructor::ArrayInfer(_)
                    | Constructor::Slice,
                )) => return Ok(true),
                Ok(_)
                | Err(
                    crate::types::infer::Error::Normalization
                    | crate::types::infer::Error::Unresolved,
                ) => return Ok(false),
                Err(error) => return Err(self.error(node, error)),
            }
        }
    }
    fn comparison_borrow(
        &mut self,
        node: NodeId,
        subject: TermId,
        expected: TermId,
    ) -> Result<TermId, Error> {
        let region =
            self.body_region(node, crate::sema::model::BodyRegionOrigin::ExpressionBorrow)?;
        let actual = self
            .state
            .infer
            .construct(
                Constructor::Reference {
                    mutable: false,
                    region,
                },
                &[subject],
            )
            .map_err(|e| self.error(node, e))?;
        self.shape_equation(node, actual, expected)?;
        Ok(actual)
    }
}
