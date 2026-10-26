//! Written quantifiers retain their lexical subject scope until clause publication.
use super::{Checker, Error, NodeKey, Predicate, regions};
use crate::{
    syntax::ast::List,
    types::{
        TypeId,
        parameters::clauses::{Clause, Quantified},
        substitute::Bound,
    },
};

#[derive(Clone, Copy)]
pub(in crate::sema::declarations) struct Scope {
    binder: regions::BinderId,
    subject_inside: bool,
}

impl Scope {
    pub(in crate::sema::declarations) fn count(self, regions: &regions::Regions) -> u32 {
        regions.count(self.binder)
    }
    pub fn context(self, parent: regions::Context) -> regions::Context {
        regions::Context {
            binder: Some(self.binder),
            ..parent
        }
    }

    pub fn subject_context(self, parent: regions::Context) -> regions::Context {
        if self.subject_inside {
            self.context(parent)
        } else {
            parent
        }
    }
}

impl Checker<'_> {
    pub(in crate::sema::declarations) fn call_bound_scope(
        &mut self,
        source: NodeKey,
        written: super::Written,
        scope: Option<Scope>,
        parent: regions::Context,
    ) -> Result<Option<Scope>, Error> {
        if scope.is_some() {
            return Ok(scope);
        }
        if let super::Written::User {
            definition, path, ..
        } = written
            && self.declared_callable(definition)
            && crate::sema::lifetimes::call_arguments(self.report, path)?.is_some()
        {
            return self.bound_scope(source, Some(List { start: 0, end: 0 }), parent, false);
        }
        Ok(None)
    }
    pub(in crate::sema::declarations) fn bound_scope(
        &mut self,
        source: NodeKey,
        binders: Option<List>,
        parent: regions::Context,
        subject_inside: bool,
    ) -> Result<Option<Scope>, Error> {
        let Some(binders) = binders else {
            return Ok(None);
        };
        let declared = self.declared_regions(source, binders)?;
        self.validate_bound_names(source, &declared)?;
        let binder = self.regions.binder(parent.binder, declared.clone());
        // An unused declared slot still quantifies the clause. Reserve slots in
        // declaration order before subject/argument scheduling can observe them.
        for definition in declared {
            self.regions
                .named(Some(binder), definition)
                .map_err(|e| self.error(source, e))?;
        }
        Ok(Some(Scope {
            binder,
            subject_inside,
        }))
    }

    fn validate_bound_names(
        &self,
        source: NodeKey,
        declared: &[crate::resolve::model::DefId],
    ) -> Result<(), Error> {
        use crate::resolve::model::Namespace;
        let mut work = 0_usize;
        for &id in declared {
            let definition = &self.report.resolution.definitions[id.0 as usize];
            if matches!(
                self.report.package.names.get(definition.name),
                "static" | "_"
            ) {
                return Err(self.error(source, "invalid quantified lifetime parameter name"));
            }
            let mut parent = self.report.resolution.scopes[definition.scope.0 as usize].parent;
            while let Some(id) = parent {
                work += 1;
                if work > 1_000_000 {
                    return Err(
                        self.error(source, "quantified lifetime naming work budget exceeded")
                    );
                }
                let scope = &self.report.resolution.scopes[id.0 as usize];
                if self.report.resolution.bindings.contains(
                    id,
                    definition.name,
                    Namespace::Lifetime,
                ) {
                    return Err(
                        self.error(source, "quantified lifetime shadows an enclosing lifetime")
                    );
                }
                if scope.barrier {
                    break;
                }
                parent = scope.parent;
            }
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn bound_subject(
        &mut self,
        source: NodeKey,
        scope: Option<Scope>,
        ty: TypeId,
    ) -> Result<TypeId, Error> {
        if scope.is_some_and(|scope| !scope.subject_inside) {
            self.capture_bound_type(source, ty)
        } else {
            Ok(ty)
        }
    }

    /// The uniform one-binder shift shares completed structural work across
    /// different clauses with the same subject. It carries no proof authority.
    pub(in crate::sema::declarations) fn capture_bound_type(
        &mut self,
        source: NodeKey,
        ty: TypeId,
    ) -> Result<TypeId, Error> {
        self.bound_captures
            .get_or_insert_with(|| Box::new(Bound::shift(1)))
            .apply(&mut self.facts.types, ty)
            .map_err(|e| self.error(source, e))
    }

    pub(in crate::sema::declarations) fn bound_clause(
        &mut self,
        source: NodeKey,
        scope: Option<Scope>,
        predicate: Predicate,
    ) -> Result<Clause, Error> {
        scope.map_or(Ok(Clause::Predicate(predicate)), |scope| {
            self.facts
                .types
                .intern_quantified(Quantified {
                    bound_regions: self.regions.count(scope.binder),
                    predicate,
                })
                .map(Clause::ForAll)
                .map_err(|e| self.error(source, e))
        })
    }
}
