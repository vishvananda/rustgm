//! One authenticated selector kernel serves equality and projected-bound syntax.
use super::{Binding, Checker, DefId, Error, NodeKey};
use crate::types::{
    TypeId,
    parameters::predicates::{Bound, Predicate},
};
pub(super) enum Outcome {
    Pending,
    Equality(crate::sema::generics::Predicate),
    Bounds((TypeId, TypeId)),
}
impl Checker<'_> {
    pub(super) fn complete_binding_constraint(
        &mut self,
        source: NodeKey,
        binding: Binding,
    ) -> Result<Outcome, Error> {
        let Binding {
            path,
            predicate,
            original,
            scope,
            context,
            ..
        } = binding;
        let selector = self
            .report
            .resolution
            .associated_bindings
            .get(&source)
            .ok_or_else(|| self.error(source, "associated binding requires one member name"))?;
        let name = selector.name;
        let application = self.binding_projection(source, predicate, name)?;
        if self.prepare_binding_arguments(source, application, context)? {
            return Ok(Outcome::Pending);
        }
        let ty = self.applied_binding(source, application, context, false)?;
        let original_ty = if original == predicate {
            self.applied_binding(source, application, context, true)?
        } else {
            let original = self.binding_projection(source, original, name)?;
            self.applied_binding(source, original, context, true)?
        };
        if self
            .facts
            .associated
            .families
            .contains_key(&DefId(application.item.0))
        {
            // A constraint's result cannot erase the family selector's
            // input requirements. Retain that typed root at its own source.
            self.store(source, original_ty)?;
        }
        let Some(value) = self.associated_binding_rhs(source)? else {
            return Ok(Outcome::Bounds((ty, original_ty)));
        };
        let value = NodeKey {
            node: value,
            ..path
        };
        let result = self.ty(value)?;
        let original_result = self.facts.unreduced.node(value).unwrap_or(result);
        let value = Predicate {
            ty,
            bound: Bound::ProjectionEqual(result),
        };
        let original = self.bound_clause(
            source,
            scope,
            Predicate {
                ty: original_ty,
                bound: Bound::ProjectionEqual(original_result),
            },
        )?;
        self.facts.unreduced.retain_predicate(source, original);
        Ok(Outcome::Equality(crate::sema::generics::Predicate {
            source,
            value: self.bound_clause(source, scope, value)?,
        }))
    }
}
