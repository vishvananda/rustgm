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
        let subject = self.term(left)?;
        let right_type = self.term(right)?;
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
        let actual = self.comparison_borrow(node, subject, receiver)?;
        let pending = self.retain_language_borrow(node, left, method, callable, subject, actual);
        self.state.call_types.methods[pending]
            .1
            .expect_receiver(receiver);
        let expected = self.child(node, signature, 1)?;
        let actual = self.comparison_borrow(node, right_type, expected)?;
        self.state.call_types.field_receivers.push(right_type);
        self.state
            .call_types
            .comparison_borrows
            .push((node, actual, expected));
        self.state.alignment.borrows.push((node, right));
        self.state.calls.push((node, signature));
        self.child(node, signature, 2)
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
