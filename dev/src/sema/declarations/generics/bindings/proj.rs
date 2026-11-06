//! Projected bounds reuse decoded written bounds and the normal owner continuation.
use super::{Binding, Checker, DefId, Error, Kind, NodeKey, Work, regions};
use crate::{
    sema::declarations::generics::bounds::{Subject, Syntax},
    support::{
        hash::{HashMap, HashSet},
        retained::Growth,
    },
};
type Key = (NodeKey, Binding);
type Root = (NodeKey, regions::Context);
#[derive(Default)]
pub(super) struct Pending {
    pub(super) interpretations: HashMap<Key, Prepared>,
    pub(super) runs: HashMap<Key, Box<super::nest::Run>>,
    owners: HashMap<DefId, Owner>,
}
#[derive(Default)]
struct Owner {
    roots: Vec<Root>,
    completed: bool,
}
pub(super) struct Prepared {
    pub(super) syntax: Vec<Syntax>,
    pub(super) retained: bool,
}
impl Checker<'_> {
    pub(in crate::sema::declarations) fn projected_bound_owner(&self, owner: DefId) -> bool {
        self.associated_bindings
            .projected
            .as_ref()
            .is_some_and(|p| p.owners.contains_key(&owner))
    }
    pub(in crate::sema::declarations) fn charge_bound_staging(
        &mut self,
        work: usize,
        bytes: usize,
    ) -> Result<(), Error> {
        if self.report.package.source_core().is_some()
            || self.report.package.source_client().is_some()
        {
            let catalog = self.facts.source_bindings.as_deref().ok_or_else(|| {
                Error::unlocated("projected bounds require their actual source catalog")
            })?;
            catalog
                .require_provider_report(self.report)
                .map_err(Error::unlocated)?;
            crate::sema::header_attributes::charge_source_retention(&self.facts, work, bytes)
        } else {
            crate::sema::header_attributes::reserve_source_contracts(&mut self.facts, work, bytes)
        }
    }
    pub(in crate::sema::declarations) fn prepare_projected_owner(
        &mut self,
        owner: DefId,
    ) -> Result<(), Error> {
        if self
            .associated_bindings
            .projected
            .as_ref()
            .is_some_and(|p| p.owners.contains_key(&owner))
        {
            return Ok(());
        }
        let (length, capacity) = self
            .associated_bindings
            .projected
            .as_ref()
            .map_or((0, 0), |p| (p.owners.len(), p.owners.capacity()));
        let growth = Growth::map::<DefId, Owner>(length, capacity, 1);
        let boxed = if self.associated_bindings.projected.is_none() {
            size_of::<Pending>()
        } else {
            0
        };
        self.charge_bound_staging(
            growth.work.saturating_add(1),
            growth.bytes.saturating_add(boxed),
        )?;
        self.associated_bindings
            .projected
            .get_or_insert_default()
            .owners
            .insert(owner, Owner::default());
        Ok(())
    }
    pub(in crate::sema::declarations) fn take_projected_roots(
        &mut self,
        owner: DefId,
        syntax_bytes: usize,
    ) -> Result<Option<Vec<Root>>, Error> {
        let Some(stage) = self
            .associated_bindings
            .projected
            .as_ref()
            .and_then(|p| p.owners.get(&owner))
        else {
            return Ok(None);
        };
        if !stage.completed {
            self.charge_bound_staging(1, syntax_bytes)?;
        }
        let stage = self
            .associated_bindings
            .projected
            .as_mut()
            .and_then(|p| p.owners.get_mut(&owner))
            .ok_or_else(|| Error::unlocated("projected roots lost their actual owner"))?;
        stage.completed = true;
        Ok(Some(std::mem::take(&mut stage.roots)))
    }
    pub(in crate::sema::declarations) fn stage_projected_roots(
        &mut self,
        owner: DefId,
        roots: &mut Vec<Root>,
    ) -> Result<(), Error> {
        let stage = self
            .associated_bindings
            .projected
            .as_ref()
            .and_then(|p| p.owners.get(&owner))
            .ok_or_else(|| Error::unlocated("projected roots lost their actual owner"))?;
        if stage.completed {
            return self.append_completed_bound_roots(owner, roots);
        }
        let growth = Growth::vector::<Root>(stage.roots.len(), stage.roots.capacity(), roots.len());
        self.charge_bound_staging(growth.work.saturating_add(roots.len()), growth.bytes)?;
        let stage = self
            .associated_bindings
            .projected
            .as_mut()
            .and_then(|p| p.owners.get_mut(&owner))
            .ok_or_else(|| Error::unlocated("projected roots lost their actual owner"))?;
        stage.roots.append(roots);
        Ok(())
    }
    pub(super) fn prepare_projected_bounds(
        &mut self,
        source: NodeKey,
        binding: Binding,
    ) -> Result<(), Error> {
        self.prepare_projected_owner(binding.owner)?;
        let Kind::Constraint {
            value: None,
            bounds,
            ..
        } = self.kind(source)
        else {
            return Err(self.error(source, "projected bounds require their actual constraint"));
        };
        let count = usize::try_from(bounds.end - bounds.start)
            .map_err(|_| self.error(source, "too many projected bounds"))?;
        let (length, capacity) = self
            .associated_bindings
            .projected
            .as_ref()
            .map_or((0, 0), |p| {
                (p.interpretations.len(), p.interpretations.capacity())
            });
        let growth = Growth::map::<Key, Prepared>(length, capacity, 1);
        let boxed = if self.associated_bindings.projected.is_none() {
            size_of::<Pending>()
        } else {
            0
        };
        let bytes =
            count.saturating_mul(size_of::<Syntax>() + size_of::<crate::syntax::ast::NodeId>());
        self.charge_bound_staging(
            count.saturating_add(growth.work).saturating_add(1),
            bytes.saturating_add(growth.bytes).saturating_add(boxed),
        )?;
        let mut syntax = Vec::with_capacity(count);
        self.bound_list(
            source,
            Subject::Type(source),
            bounds,
            &HashSet::default(),
            (binding.context, binding.scope.map(super::Scope::inside)),
            &mut syntax,
        )?;
        for bound in syntax.iter().rev() {
            if let Some((path, definition)) = bound.trait_path() {
                let context = bound.context(binding.context);
                self.schedule_associated_bindings(path, context)?;
                self.work
                    .push(Work::PathArguments(path, definition, context));
            }
        }
        self.associated_bindings
            .projected
            .get_or_insert_default()
            .interpretations
            .insert(
                (source, binding),
                Prepared {
                    syntax,
                    retained: false,
                },
            );
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/staging_continuations.rs"]
mod tests;
