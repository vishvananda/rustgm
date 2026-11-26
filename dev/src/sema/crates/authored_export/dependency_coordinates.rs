//! Authored rows refer to an imported declaration in the publisher's dependency inventory.
use super::{Declarations, DefId, Report, Stage, g, s};
use crate::sema::associated::Owner;
use crate::{resolve::body_origin::Catalog, support::crates::Dependencies as Inventory};
use std::{rc::Rc, sync::Arc};

pub(super) struct Dependencies {
    catalog: Rc<Catalog>,
    inventory: Arc<Inventory>,
}

impl Stage {
    pub(super) fn is_dependency_owner(
        &self,
        facts: &Declarations,
        owner: &Owner,
    ) -> Result<bool, &'static str> {
        if self.bootstrap_bindings() && owner.source.is_none() {
            facts.associated.require_imported(*owner)?;
            return Ok(true);
        }
        Ok(false)
    }

    pub(super) fn prepare_dependency_coordinates(
        &mut self,
        report: &Report,
    ) -> Result<(), &'static str> {
        if !self.bootstrap_bindings() {
            return Ok(());
        }
        let Some(imported) = report.resolution.crates.as_deref() else {
            return Ok(());
        };
        let Some(catalog) = imported.origin_catalog()? else {
            return Ok(());
        };
        self.reserve(size_of::<Dependencies>())?;
        self.dependencies = Some(Box::new(Dependencies {
            catalog,
            inventory: imported.shared_dependencies(),
        }));
        Ok(())
    }

    pub(super) fn dependency_address(
        &mut self,
        actual: DefId,
    ) -> Result<s::DeclarationAddress, &'static str> {
        let count = self.dependencies.as_ref().map_or(0, |dependencies| {
            dependencies.inventory.publication_dependencies().len()
        });
        self.charge(4 + count.checked_ilog2().unwrap_or(0) as usize)?;
        let dependencies = self
            .dependencies
            .as_deref()
            .ok_or("source header names an unreserved actual declaration")?;
        let id = dependencies.catalog.declaration(actual)?;
        let (owner, declaration, _) = dependencies.catalog.declaration_coordinate(id)?;
        let index = dependencies
            .inventory
            .publication_dependencies()
            .binary_search(owner)
            .map_err(|_| "imported declaration is outside the publisher's dependency inventory")?;
        let ordinal =
            u32::try_from(index + 1).map_err(|_| "publication dependency ordinal overflow")?;
        Ok(s::DeclarationAddress {
            artifact: s::ArtifactRef(ordinal),
            declaration,
        })
    }
    pub(super) fn dependency_callable_binder(
        &mut self,
        actual: DefId,
    ) -> Result<(g::BinderRef, u32), &'static str> {
        let address = self.dependency_address(actual)?;
        let Some(dependencies) = self.dependencies.as_deref() else {
            return Err("source call target lost its actual dependency inventory");
        };
        self.charge(
            4 + dependencies
                .inventory
                .artifacts()
                .len()
                .checked_ilog2()
                .unwrap_or(0) as usize,
        )?;
        let dependencies = self
            .dependencies
            .as_deref()
            .ok_or("source call target lost its actual dependency inventory")?;
        let identity = dependencies
            .inventory
            .publication_dependencies()
            .get((address.artifact.0 - 1) as usize)
            .ok_or("source call target lost its actual dependency identity")?;
        let artifacts = dependencies.inventory.artifacts();
        let index = artifacts
            .binary_search_by(|artifact| artifact.identity().cmp(identity))
            .map_err(|_| "source call target lost its actual dependency artifact")?;
        let descriptor = artifacts[index]
            .source_descriptor()
            .filter(|descriptor| descriptor.profile().bootstrap_bindings())
            .ok_or("source call target requires its ordinary typed dependency")?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(address.declaration)? else {
            return Err("source call target has no actual dependency callable header");
        };
        super::authored_functions::opening_binder(&rows.functions[index as usize], &rows.binders)
    }
}
