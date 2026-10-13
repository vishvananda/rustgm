//! Elaborate destructor targets once and retain their lifetime environments.
mod headers;
mod receivers;
use super::{BinderId, Checker, DefId, Error, NodeKey, Type, TypeId, regions};
use crate::{
    sema::destructors::Implementation,
    types::outlives::{Bounds, Queries},
};
use std::collections::HashSet;

impl Checker<'_> {
    pub(super) fn prepare_destructors(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        for header in headers::collect(self.report, &self.facts)? {
            let key = header.source;
            let method = header
                .method
                .ok_or_else(|| self.error(key, "Drop impl is missing its drop method"))?;
            let declared = self.declared_regions(key, self.item(key)?.generics)?;
            let binder = self.regions.binder(None, declared.clone());
            for parameter in declared {
                self.regions
                    .named(Some(binder), parameter)
                    .map_err(|e| self.error(key, e))?;
            }
            let bounds = self.function_bounds(key, binder)?;
            self.node_work(
                header.target.file,
                header.target.node,
                regions::Context {
                    binder: Some(binder),
                    position: regions::Position::Impl,
                },
            );
            self.drain(steps, limit)?;
            let target = self.ty(header.target)?;
            let Some(Type::Adt(item, arguments)) = self.facts.types.kind(target) else {
                return Err(self.error(key, "Drop requires a local struct or enum target"));
            };
            let arguments = self
                .facts
                .types
                .region_args(arguments)
                .ok_or_else(|| self.error(key, "missing Drop target lifetimes"))?;
            let mut seen = HashSet::new();
            for &argument in arguments {
                if !matches!(argument.bound_parts(), Some((0, index)) if u32::from(index) < self.regions.count(binder))
                    || !seen.insert(argument)
                {
                    return Err(self.error(
                        key,
                        "Drop target requires distinct lifetime parameters without specialization",
                    ));
                }
            }
            self.facts
                .destructors
                .insert(Implementation {
                    source: key,
                    self_definition: header.self_definition,
                    method,
                    target,
                    item,
                    parameters: self.regions.count(binder),
                    bounds,
                })
                .map_err(|e| self.error(key, e))?;
            self.drop_binders.insert(header.self_definition, binder);
            self.complete(header.self_definition, key, target)?;
        }
        Ok(())
    }
    pub(super) fn validate_destructors(&self) -> Result<(), Error> {
        if self.facts.destructors.implementations().is_empty() {
            return Ok(());
        }
        let bounds = Bounds::infer(
            &self.facts.types,
            &self.facts.aggregates,
            &self.facts.lifetimes.nominal_bounds,
        )
        .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        let mut queries = Queries::new(&self.facts.types, &bounds, &self.facts.lifetimes.callables);
        for implementation in self.facts.destructors.implementations() {
            let key = implementation.source;
            if self
                .facts
                .derives
                .get(&implementation.item)
                .is_some_and(|p| p.copy)
            {
                return Err(self.error(key, "a type cannot implement both Copy and Drop"));
            }
            let Some(Type::Adt(_, arguments)) = self.facts.types.kind(implementation.target) else {
                return Err(self.error(key, "missing nominal Drop target"));
            };
            let arguments = self
                .facts
                .types
                .region_args(arguments)
                .ok_or_else(|| self.error(key, "missing Drop target arguments"))?;
            let assumptions = queries
                .requirements(implementation.target)
                .map_err(|e| self.error(key, e))?;
            crate::sema::destructors::check_bounds(implementation, arguments, assumptions)
                .map_err(|e| self.error(key, e))?;
        }
        Ok(())
    }
    pub(super) fn signature_binder(
        &mut self,
        owner: Option<DefId>,
        parent: Option<BinderId>,
        declared: Vec<DefId>,
    ) -> BinderId {
        if let Some(implementation) = owner.and_then(|id| self.facts.destructors.method(id)) {
            self.regions
                .capture(self.drop_binders[&implementation.self_definition], declared)
        } else {
            self.regions.binder(parent, declared)
        }
    }
    pub(super) fn drop_self_type(
        &mut self,
        key: NodeKey,
        id: DefId,
        context: regions::Context,
    ) -> Result<TypeId, Error> {
        if crate::sema::lifetimes::arguments(self.report, key, false)?.is_some() {
            return Err(self.error(key, "Self cannot take lifetime arguments"));
        }
        let implementation = self
            .facts
            .destructors
            .self_type(id)
            .ok_or_else(|| self.error(key, "missing Drop Self type"))?;
        let target = implementation.target;
        let binder = self.drop_binders[&id];
        let mut mapping = Vec::with_capacity(implementation.parameters as usize);
        for index in 0..implementation.parameters {
            let slot = u16::try_from(index)
                .map_err(|_| self.error(key, "too many impl lifetime slots"))?;
            mapping.push(
                self.regions
                    .captured_region(context.binder, binder, slot)
                    .map_err(|e| self.error(key, e))?,
            );
        }
        let args = self
            .facts
            .types
            .intern_regions(&mapping)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .lifetimes
            .substitute(&mut self.facts.types, args, target)
            .map_err(|e| self.error(key, e))
    }
}
