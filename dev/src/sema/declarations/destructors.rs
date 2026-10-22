//! Elaborate destructor targets once and retain their lifetime environments.
mod coverage;
mod headers;
mod receivers;
use super::{Checker, Error, Type, heads};
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
            let environment =
                self.impl_header(header.self_definition, key, header.target, steps, limit)?;
            let target = environment.schema.self_type;
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
                    environment.schema.captures,
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
                    parameters: environment.schema.captures,
                    bounds: environment.schema.bounds.clone(),
                    coverage,
                })
                .map_err(|e| self.error(key, e))?;
            self.complete_impl_owner(header.self_definition, key, environment, steps, limit)?;
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
        let mut trait_proofs = crate::support::hash::HashMap::default();
        for index in 0..self.facts.destructors.implementations().len() {
            if let Some(proof) = self.validate_drop_traits(index)? {
                trait_proofs.insert(index, proof);
            }
        }
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        for (index, implementation) in self.facts.destructors.implementations().iter().enumerate() {
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
                trait_proofs.get(&index),
            )
            .map_err(|e| self.error(key, e))?;
        }
        Ok(())
    }
}
