//! Defining signatures retain bound scopes through argument and equality scheduling.
use super::generics::{
    bindings::Binding,
    bounds::{Written, lexical::Scope},
};
use super::{Checker, DefId, Error, Kind, NodeKey, Position, Type, TypeId, Work, regions};
use crate::{
    sema::opaque::Declaration,
    types::{
        copy, opaque,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

pub(super) struct Pending {
    owner: DefId,
    declared: Vec<Declared>,
    precise: Option<NodeKey>,
    promises: copy::Properties,
    application: Option<TypeId>,
    explicit: Option<Box<super::capture_bounds::ExplicitLifetimes>>,
}
#[derive(Clone, Copy)]
struct Declared {
    source: NodeKey,
    written: Written,
    scope: Option<Scope>,
}

impl Checker<'_> {
    pub(super) fn opaque_result(
        &mut self,
        key: NodeKey,
        bounds: crate::syntax::ast::List,
        context: regions::Context,
        finish: bool,
    ) -> Result<Option<TypeId>, Error> {
        if !finish {
            self.work.push(Work::Node(key, true, context));
            let pending = self.prepare_opaque(key, bounds, context)?;
            self.opaque_syntax.insert(key, pending);
            return Ok(None);
        }
        let mut pending = self
            .opaque_syntax
            .remove(&key)
            .ok_or_else(|| self.error(key, "missing opaque signature continuation"))?;
        let application = if let Some(application) = pending.application {
            application
        } else {
            let id = self
                .facts
                .types
                .declare_opaque(opaque::Declaration {
                    trait_result: self.facts.associated.trait_owner(pending.owner).is_some(),
                    sized: true,
                    copies: pending.promises,
                })
                .map_err(|e| self.error(key, e))?;
            let captures = self.opaque_captures(
                key,
                pending.owner,
                context,
                pending.precise,
                pending.explicit.as_deref(),
            )?;
            let captures = self
                .facts
                .types
                .intern_arguments(&captures)
                .map_err(|e| self.error(key, e))?;
            let application = self.intern(key, Type::Opaque(id, captures))?;
            pending.application = Some(application);
            application
        };
        let work_start = self.work.len();
        let (bounds, waiting) = self.opaque_clauses(&pending, application, context)?;
        if waiting {
            let arguments = self.work.split_off(work_start);
            self.work.push(Work::Node(key, true, context));
            self.work.extend(arguments);
            self.opaque_syntax.insert(key, pending);
            return Ok(None);
        }
        super::capture_bounds::check_promises(&self.facts.types, application, &bounds)
            .map_err(|e| self.error(key, e))?;
        let Some(Type::Opaque(id, _)) = self.facts.types.kind(application) else {
            return Err(self.error(key, "invalid completed opaque signature"));
        };
        self.facts
            .opaques
            .owners
            .entry(pending.owner)
            .or_default()
            .push(id);
        self.facts.opaques.declarations.insert(
            id,
            Declaration {
                owner: pending.owner,
                source: key,
                application,
                bounds,
                hidden: None,
                inferred: None,
                free_regions: Vec::new(),
            },
        );
        Ok(Some(application))
    }

    fn prepare_opaque(
        &mut self,
        key: NodeKey,
        bounds: crate::syntax::ast::List,
        context: regions::Context,
    ) -> Result<Pending, Error> {
        let owner = context.binder.and_then(|id| self.regions.owner(id));
        let Some(owner) = owner.filter(|_| matches!(context.position, Position::Output)) else {
            return Err(self.error(key, "impl Trait requires a function parameter or result"));
        };
        let mut pending = Pending {
            owner,
            declared: Vec::new(),
            precise: None,
            promises: copy::Properties::default(),
            application: None,
            explicit: self.legacy_opaque_captures(key, owner)?,
        };
        let nodes = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(bounds)
            .to_vec();
        for node in nodes {
            let source = NodeKey { node, ..key };
            if matches!(self.kind(source), Kind::PreciseCapture(_)) {
                if pending.precise.replace(source).is_some() {
                    return Err(self.error(source, "opaque result has multiple capture bounds"));
                }
                continue;
            }
            let Kind::Bound {
                optional,
                constness,
                quantified,
                binders,
                value,
            } = self.kind(source)
            else {
                return Err(self.error(source, "invalid opaque result bound"));
            };
            if optional {
                return Err(self.error(source, "opaque result bounds cannot be relaxed"));
            }
            let scope = self.bound_scope(source, quantified.then_some(binders), context, false)?;
            let value = NodeKey { node: value, ..key };
            let written = if self.lifetime_argument(value) {
                Written::Outlives(value)
            } else {
                self.qualified_trait_bound(value, constness)?
            };
            let scope = self.call_bound_scope(source, written, scope, context)?;
            let context = scope.map_or(context, |scope| scope.context(context));
            if let Written::Outlives(lifetime) = written
                && let Some(explicit) = &mut pending.explicit
            {
                let region = self.region(lifetime, Some(lifetime.node), context)?;
                explicit.regions.insert(region);
            }
            if let Written::Trait(bound) = written {
                pending.promises.copy |= bound == Bound::Copy;
                pending.promises.clone_ |= matches!(bound, Bound::Copy | Bound::Clone);
            }
            if let Written::User {
                path, definition, ..
            } = written
            {
                self.bound_subject_inputs(
                    path,
                    owner,
                    pending
                        .explicit
                        .as_deref_mut()
                        .map(|captures| &mut captures.names),
                )?;
                self.schedule_associated_bindings(path, context)?;
                self.work
                    .push(Work::PathArguments(path, definition, context));
            }
            pending.declared.push(Declared {
                source,
                written,
                scope,
            });
        }
        if !pending
            .declared
            .iter()
            .any(|b| !matches!(b.written, Written::Outlives(_)))
        {
            return Err(self.error(key, "impl Trait requires at least one trait bound"));
        }
        Ok(pending)
    }

    fn opaque_clauses(
        &mut self,
        pending: &Pending,
        application: TypeId,
        context: regions::Context,
    ) -> Result<(Vec<Clause>, bool), Error> {
        let mut bounds = Vec::new();
        let mut waiting = false;
        for &Declared {
            source,
            written,
            scope,
        } in &pending.declared
        {
            let context = scope.map_or(context, |scope| scope.context(context));
            let ty = self.bound_subject(source, scope, application)?;
            let (bound, original) = self.bound_values(written, ty, ty, context)?;
            let predicate = Predicate { ty, bound };
            bounds.push(self.bound_clause(source, scope, predicate)?);
            if let Written::User { path, .. } = written {
                let mut equalities = Vec::new();
                waiting |= self.complete_associated_bindings(
                    Binding {
                        owner: pending.owner,
                        path,
                        predicate,
                        original: Predicate {
                            ty,
                            bound: original,
                        },
                        scope,
                        context,
                    },
                    &mut equalities,
                )?;
                bounds.extend(equalities.into_iter().map(|p| p.value));
            }
        }
        Ok((bounds, waiting))
    }
}
