//! One typed written-bound completion kernel for direct and projected subjects.
use super::{Checker, DefId, Error, Predicate, Subject, Syntax, Type, Written, regions};
use crate::types::TypeId;
impl Checker<'_> {
    pub(in crate::sema::declarations) fn complete_written_bound(
        &mut self,
        owner: (DefId, regions::Context),
        bound: Syntax,
        subject: (TypeId, TypeId),
        completed: &mut Vec<crate::sema::generics::Predicate>,
        alias_edges: &mut Vec<super::super::super::aliases::Edge>,
    ) -> Result<Option<super::super::bindings::Binding>, Error> {
        let (owner, context) = owner;
        let (ty, original_ty) = subject;
        let ty = self.bound_subject(bound.source, bound.scope, ty)?;
        let original_ty = self.bound_subject(bound.source, bound.scope, original_ty)?;
        if let Written::Alias { path, definition } = bound.bound {
            let edge = self.canonical_alias_edge(
                path,
                definition,
                ty,
                original_ty,
                bound.context(context),
                self.alias_scope_count(bound.scope),
            )?;
            self.complete_alias_bound(owner, edge, completed, alias_edges)?;
            return Ok(None);
        }
        let (value, original_bound) =
            self.bound_values(bound.bound, ty, original_ty, bound.context(context))?;
        if matches!(bound.subject, Subject::Parameter(_))
            && !matches!(self.facts.types.kind(ty), Some(Type::Parameter(_)))
        {
            return Err(self.error(bound.source, "trait bounds require a type parameter"));
        }
        let value = Predicate { ty, bound: value };
        let original = Predicate {
            ty: original_ty,
            bound: original_bound,
        };
        if original
            .contains_projections(&self.facts.types)
            .map_err(|e| self.error(bound.source, e))?
        {
            let original = self.bound_clause(bound.source, bound.scope, original)?;
            self.facts
                .unreduced
                .retain_predicate(bound.source, original);
        }
        completed.push(crate::sema::generics::Predicate {
            source: bound.source,
            value: self.bound_clause(bound.source, bound.scope, value)?,
        });
        if let Written::User { path, .. } = bound.bound
            && (self.associated_bindings.bindings(path).is_some()
                || crate::sema::lifetimes::call_arguments(self.report, path)?.is_some())
        {
            return Ok(Some(super::super::bindings::Binding {
                owner,
                path,
                predicate: value,
                original,
                scope: bound.scope,
                context: bound.context(context),
            }));
        }
        Ok(None)
    }
}
