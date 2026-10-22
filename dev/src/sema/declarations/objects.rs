//! Object syntax schedules ordinary trait inputs before publishing canonical bounds.
mod binding_times;
mod bounds;
mod containers;
use crate::sema::traits::object_bounds::graph;
mod lifetimes;
mod selections;
use super::{
    Checker, DefId, Error, Kind, List, NodeKey, RegionId, Type, TypeId, Work,
    generics::bounds::{Written, lexical::Scope},
    regions,
};
use crate::support::hash::HashMap;
use crate::types::parameters::predicates::{Bound, Predicate};

#[derive(Default)]
pub(super) struct Syntax {
    pending: HashMap<NodeKey, Pending>,
    defaults: HashMap<NodeKey, RegionId>,
    containers: containers::Defaults,
    uses: Vec<(NodeKey, TypeId, DefId)>,
    times: binding_times::Times,
    members:
        HashMap<crate::types::parameters::clauses::Clause, std::sync::Arc<selections::Members>>,
    member_steps: usize,
}
struct Pending {
    principal: Option<Principal>,
    region: Option<RegionId>,
    autos: Vec<crate::types::auto_traits::AutoTrait>,
}
struct Principal {
    path: NodeKey,
    definition: DefId,
    scope: Option<Scope>,
}

impl Checker<'_> {
    pub(super) fn object_reference_default(
        &mut self,
        mut key: NodeKey,
        region: RegionId,
        context: regions::Context,
    ) {
        if matches!(
            context.position,
            regions::Position::Local(_) | regions::Position::Constant(_)
        ) && region != RegionId::STATIC
            && region.bound_parts().is_none()
        {
            return;
        }
        loop {
            match self.kind(key) {
                Kind::Group(node) | Kind::Attributed { value: node, .. } => key.node = node,
                Kind::TraitType { opaque: false, .. } => {
                    self.object_syntax.defaults.insert(key, region);
                    return;
                }
                _ => return,
            }
        }
    }

    pub(super) fn object_type(
        &mut self,
        key: NodeKey,
        bounds: List,
        context: regions::Context,
        finish: bool,
    ) -> Result<Option<TypeId>, Error> {
        if !finish {
            self.work.push(Work::Node(key, true, context));
            let pending = self.prepare_object(key, bounds, context)?;
            self.prepare_object_binding_times(context)?;
            self.object_syntax.pending.insert(key, pending);
            return Ok(None);
        }
        let pending = self
            .object_syntax
            .pending
            .remove(&key)
            .ok_or_else(|| self.error(key, "missing object annotation continuation"))?;
        let Some(principal) = &pending.principal else {
            return self.auto_object_type(key, &pending, context).map(Some);
        };
        let inner = principal
            .scope
            .map_or(context, |scope| scope.context(context));
        self.prepare_object_guarantees(key, principal.definition)?;
        let self_type = self
            .facts
            .associated
            .schema(principal.definition)
            .ok_or_else(|| self.error(key, "missing principal trait declaration"))?
            .self_type;
        let subject = self.bound_subject(key, principal.scope, self_type)?;
        let (bound, original) = self.bound_values(
            Written::User {
                path: principal.path,
                definition: principal.definition,
            },
            subject,
            subject,
            inner,
        )?;
        let predicate = Predicate { ty: subject, bound };
        let equalities = self.object_bindings(
            key,
            &pending,
            predicate,
            Predicate {
                ty: subject,
                bound: original,
            },
        )?;
        let bound_regions = self.object_bound_regions(key, principal.scope, predicate)?;
        let region = self.object_region(key, &pending, Some(predicate), context)?;
        let ty = self.complete_object_bounds(
            key,
            &pending,
            predicate,
            equalities,
            bound_regions,
            region,
        )?;
        self.object_syntax
            .uses
            .push((key, ty, principal.definition));
        Ok(Some(ty))
    }

    fn auto_object_type(
        &mut self,
        key: NodeKey,
        pending: &Pending,
        context: regions::Context,
    ) -> Result<TypeId, Error> {
        let region = self.object_region(key, pending, None, context)?;
        let shape = self
            .facts
            .types
            .intern_dynamic_shape(crate::types::dynamic::Bounds {
                principal: None,
                principal_arguments: 0,
                bindings: &[],
                auto_traits: &pending.autos,
            })
            .map_err(|e| self.error(key, e))?;
        self.intern(
            key,
            Type::Dynamic {
                shape,
                arguments: crate::types::GenericArgs::EMPTY,
                bound_regions: 0,
                region,
            },
        )
    }

    fn prepare_object(
        &mut self,
        key: NodeKey,
        bounds: List,
        context: regions::Context,
    ) -> Result<Pending, Error> {
        let mut principal = None;
        let mut region = None;
        let mut autos = Vec::new();
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
            let Kind::Bound {
                optional,
                quantified,
                binders,
                value,
            } = self.kind(source)
            else {
                return Err(self.error(source, "invalid object bound"));
            };
            if optional {
                return Err(self.error(source, "object bounds cannot be relaxed"));
            }
            let value = NodeKey { node: value, ..key };
            if self.lifetime_argument(value) {
                if quantified || region.is_some() {
                    return Err(self.error(source, "object requires at most one lifetime bound"));
                }
                region = Some(self.region(value, Some(value.node), context)?);
                continue;
            }
            let written = self.trait_bound(value)?;
            if let Written::Trait(Bound::Auto(marker)) = written {
                if quantified {
                    self.bound_scope(source, Some(binders), context, false)?;
                }
                autos.push(marker);
                continue;
            }
            let Written::User { path, definition } = written else {
                return Err(self.error(source, "object principal must be dyn compatible"));
            };
            if principal.is_some() {
                return Err(self.error(source, "object requires exactly one principal trait"));
            }
            let scope = self.bound_scope(source, quantified.then_some(binders), context, false)?;
            let scope = self.call_bound_scope(source, written, scope, context)?;
            let inner = scope.map_or(context, |scope| scope.context(context));
            self.schedule_associated_bindings(path, inner)?;
            self.work.push(Work::PathArguments(path, definition, inner));
            principal = Some(Principal {
                path,
                definition,
                scope,
            });
        }
        if principal.is_none() && autos.is_empty() {
            return Err(self.error(key, "object requires at least one trait"));
        }
        Ok(Pending {
            principal,
            region,
            autos,
        })
    }

    fn object_bound_regions(
        &mut self,
        key: NodeKey,
        scope: Option<Scope>,
        predicate: Predicate,
    ) -> Result<u32, Error> {
        match self.bound_clause(key, scope, predicate)? {
            crate::types::parameters::clauses::Clause::Predicate(_) => Ok(0),
            crate::types::parameters::clauses::Clause::ForAll(id) => self
                .facts
                .types
                .quantified(id)
                .map(|q| q.bound_regions)
                .ok_or_else(|| self.error(key, "invalid object trait binder")),
        }
    }

    pub(super) fn validate_objects(&mut self) -> Result<(), Error> {
        let mut queries = crate::sema::dyn_compat::Queries::default();
        let mut checked = crate::support::hash::HashSet::default();
        let uses = std::mem::take(&mut self.object_syntax.uses);
        for (source, ty, principal) in uses {
            let Some(Type::Dynamic {
                shape,
                arguments,
                bound_regions,
                ..
            }) = self.facts.types.kind(ty)
            else {
                return Err(self.error(source, "missing object formation type"));
            };
            if !checked.insert((shape, arguments, bound_regions)) {
                continue;
            }
            queries
                .demand(self.report, &self.facts, principal)
                .map_err(|e| self.error(source, e.to_string()))?;
            self.check_object_bindings(source, ty, principal, &mut queries)?;
        }
        Ok(())
    }
}
