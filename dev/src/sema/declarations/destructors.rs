//! Elaborate destructor targets once and retain their lifetime environments.
mod coverage;
mod headers;
mod heads;
mod receivers;
use super::{BinderId, Checker, DefId, Error, NodeKey, Type, TypeId, regions};
use crate::{sema::destructors::Implementation, types::outlives::Queries};

pub(super) struct Pending {
    header: headers::Header,
    owner: Option<crate::types::ItemId>,
}
impl Checker<'_> {
    pub(super) fn reserve_destructors(&mut self) -> Result<Vec<Pending>, Error> {
        let headers = headers::collect(self.report, &self.facts)?;
        let owners = {
            let mut heads = heads::Heads::new(self.report, &self.facts);
            headers
                .iter()
                .map(|h| heads.target(h.target))
                .collect::<Result<Vec<_>, _>>()?
        };
        for &owner in owners.iter().flatten() {
            self.facts.destructors.declare(owner);
        }
        Ok(headers
            .into_iter()
            .zip(owners)
            .map(|(header, owner)| Pending { header, owner })
            .collect())
    }
    pub(super) fn prepare_destructors(
        &mut self,
        pending: Vec<Pending>,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        for Pending { header, owner } in pending {
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
            let context = regions::Context {
                binder: Some(binder),
                position: regions::Position::Impl,
            };
            self.declaration_parameters(header.self_definition, key, context)?;
            self.drain(steps, limit)?;
            self.node_work(header.target.file, header.target.node, context);
            self.drain(steps, limit)?;
            let target = self.ty(header.target)?;
            let Some(Type::Adt(item, arguments)) = self.facts.types.kind(target) else {
                return Err(self.error(key, "Drop requires a local struct or enum target"));
            };
            if owner != Some(item) {
                return Err(self.error(key, "Drop target disagrees with its declared nominal head"));
            }
            let coverage = self
                .drop_target_coverage(
                    header.self_definition,
                    item,
                    arguments,
                    self.regions.count(binder),
                )
                .map_err(|e| self.error(key, e))?;
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
                    coverage,
                })
                .map_err(|e| self.error(key, e))?;
            self.drop_binders.insert(header.self_definition, binder);
            self.complete(header.self_definition, key, target)?;
            self.schedule_bounds(header.self_definition, context)?;
            self.drain(steps, limit)?;
            self.facts
                .generics
                .complete_environment(&self.facts.types, header.self_definition)
                .map_err(|e| self.error(key, e))?;
            if self
                .facts
                .generics
                .environments
                .contains_key(&header.self_definition)
            {
                self.facts
                    .generics
                    .parents
                    .insert(method, header.self_definition);
            }
        }
        Ok(())
    }
    pub(super) fn validate_destructors(&mut self) -> Result<(), Error> {
        if self.facts.destructors.implementations().is_empty() {
            return Ok(());
        }
        for index in 0..self.facts.destructors.implementations().len() {
            self.validate_drop_traits(index)?;
        }
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        for implementation in self.facts.destructors.implementations() {
            let key = implementation.source;
            let Some(Type::Adt(_, arguments)) = self.facts.types.kind(implementation.target) else {
                return Err(self.error(key, "missing nominal Drop target"));
            };
            let arguments = self
                .facts
                .types
                .arguments(arguments)
                .ok_or_else(|| self.error(key, "missing Drop target arguments"))?
                .filter_map(|argument| {
                    if let crate::types::arguments::Argument::Lifetime(region) = argument {
                        Some(region)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let assumptions = queries
                .symbolic_requirements(implementation.target)
                .map_err(|e| self.error(key, e))?;
            crate::sema::destructors::check_bounds(
                implementation,
                &arguments,
                &assumptions,
                &self.facts.types,
                self.facts
                    .generics
                    .predicates_in_scope(implementation.method)
                    .map(|p| p.value),
            )
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
