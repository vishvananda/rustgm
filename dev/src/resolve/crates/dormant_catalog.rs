//! A moved report owns genuine reserved cells; no lexical names or imports escape.
use super::{Imported, ReservationWork, error, original_declarations};
use crate::{
    resolve::{
        body_origin::{Catalog, Limits},
        expansion::Report,
        index::State,
        package::Error,
    },
    support::crates::{Dependencies, MAX_BYTES, MAX_COMPONENTS, generics::source::Profile},
};
use std::{ops::Range, sync::Arc};
#[path = "dormant_budget.rs"]
mod budget;
#[cfg(test)]
#[path = "../../../tests/invariants/dormant_domains.rs"]
mod tests;

pub(crate) struct Inputs {
    pub(super) report: Report,
    pub(super) inventory: Arc<Dependencies>,
    pub(super) provider_index: u32,
    pub(super) catalog: Catalog,
    pub(super) root: crate::resolve::body_origin::ImportedDeclarationId,
    root_definition: crate::resolve::model::DefId,
    pub(super) definitions: Range<usize>,
    pub(super) scopes: Range<usize>,
}
/// Borrowed reserved schema only. It exposes no semantic type, Graph or proof.
pub(crate) struct RecipeView<'a> {
    owner: &'a Inputs,
}
impl Inputs {
    pub(crate) fn into_provider(
        self,
        work: &mut ReservationWork,
    ) -> Result<(Arc<Dependencies>, u32), Error> {
        self.require(&self.inventory, self.provider_index, work)?;
        Ok((self.inventory, self.provider_index))
    }
    pub(crate) fn recipe_view<'a>(
        &'a self,
        inventory: &Dependencies,
        provider_index: u32,
        work: &mut ReservationWork,
    ) -> Result<RecipeView<'a>, Error> {
        self.require(inventory, provider_index, work)?;
        Ok(RecipeView { owner: self })
    }
    pub(crate) fn reserve(
        report: Report,
        inventory: Arc<Dependencies>,
        provider_index: u32,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        Self::reserve_owned(report, inventory, provider_index, work).map(|(inputs, _)| inputs)
    }
    pub(crate) fn reserve_consumer(
        report: Report,
        work: &mut ReservationWork,
    ) -> Result<(Report, super::ConsumerReservation), Error> {
        let source = report
            .package
            .staged_consumer()
            .ok_or_else(|| error("private reservation requires its actual staged consumer"))?;
        source.require_package(&report.package).map_err(error)?;
        work.charge(1)?;
        let inventory = Arc::clone(source.inventory());
        let index = source.index();
        let (inputs, imported) = Self::reserve_owned(report, inventory, index, work)?;
        super::consumer_inputs::bind(inputs, imported, work)
    }
    fn reserve_owned(
        mut report: Report,
        inventory: Arc<Dependencies>,
        provider_index: u32,
        work: &mut ReservationWork,
    ) -> Result<(Self, Imported), Error> {
        work.charge(1)?;
        let artifact = inventory
            .artifacts()
            .get(provider_index as usize)
            .ok_or_else(|| error("unpaid provider index is outside its actual inventory"))?;
        let descriptor = artifact
            .source_descriptor()
            .ok_or_else(|| error("unpaid provider lacks its actual source descriptor"))?;
        if !report.names_resolved || descriptor.profile() != Profile::Source2024Provided {
            return Err(error(
                "unpaid reservation requires resolved input and exact Provided data",
            ));
        }
        let rows = descriptor.rows();
        budget::reserve(&report, &inventory, artifact, rows, work)?;
        let definitions_before = report.resolution.definitions.len();
        let scopes_before = report.resolution.scopes.len();
        let mut state = State::default();
        state.facts = std::mem::take(&mut report.resolution);
        let mut imported = Imported::new(Arc::clone(&inventory));
        let input = {
            let mut resolver = state.bind(&mut report.package);
            work.charge(artifact.identity().name().len().saturating_add(1))?;
            let scope = resolver.reserve_artifact_root(artifact, &mut imported)?;
            original_declarations::unpaid_input(&mut resolver, scope, artifact, &imported, work)?
        };
        report.resolution = state.facts;
        let definitions = definitions_before..report.resolution.definitions.len();
        let scopes = scopes_before..report.resolution.scopes.len();
        let mut catalog = Catalog::new(Limits {
            bytes: MAX_BYTES.saturating_sub(work.retained),
            steps: MAX_COMPONENTS.saturating_sub(work.steps),
            ..Limits::default()
        })
        .map_err(error)?;
        // Reuse the actual catalog estimator. The caller-owned budget is charged
        // before its row/hash allocations; failed attempts retain their charge.
        let result = catalog
            .install_charged(&report.resolution.definitions, input, |peak| {
                work.reserve_raw(peak)
            })
            .and_then(|_| catalog.freeze_owned());
        work.charge(catalog.work())?;
        result.map_err(error)?;
        work.charge(rows.declarations.len().saturating_add(5))?;
        let ordinal = rows
            .declarations
            .iter()
            .position(|row| row.definition.parent.is_none())
            .ok_or_else(|| error("unpaid descriptor lost its actual module root"))?;
        let reference = crate::support::crates::owner_catalog::DeclarationRef(
            u32::try_from(ordinal).map_err(|_| error("unpaid module ordinal overflow"))?,
        );
        let root_definition = catalog
            .definition_at(artifact.identity(), reference)
            .map_err(error)?;
        let root = catalog.declaration(root_definition).map_err(error)?;
        let result = Self {
            report,
            inventory,
            provider_index,
            catalog,
            root,
            root_definition,
            definitions,
            scopes,
        };
        result.require(&result.inventory, provider_index, work)?;
        result.check_cells(work)?;
        Ok((result, imported))
    }
    pub(crate) fn definitions_len(&self) -> usize {
        self.report.resolution.definitions.len()
    }
    pub(crate) fn require(
        &self,
        inventory: &Dependencies,
        provider_index: u32,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(1)?;
        if !std::ptr::eq(self.inventory.as_ref(), inventory)
            || self.provider_index != provider_index
            || self.definitions.end != self.report.resolution.definitions.len()
            || self.scopes.end != self.report.resolution.scopes.len()
        {
            return Err(error(
                "unpaid inputs changed their actual inventory/report allocation",
            ));
        }
        let artifact = inventory
            .artifacts()
            .get(provider_index as usize)
            .ok_or_else(|| error("unpaid inputs lost their selected actual artifact"))?;
        let descriptor = artifact
            .source_descriptor()
            .filter(|descriptor| descriptor.profile() == Profile::Source2024Provided)
            .ok_or_else(|| error("unpaid inputs changed their explicit data profile"))?;
        let rows = descriptor.rows();
        work.charge(1)?;
        if self
            .catalog
            .declaration_identity(self.root, crate::resolve::model::DefKind::Module)
            .map_err(error)?
            != self.root_definition
        {
            return Err(error(
                "unpaid inputs belong to another actual catalog allocation",
            ));
        }
        if self.definitions.len() != rows.declarations.len()
            || self.catalog.body_count() != rows.originals.len()
        {
            return Err(error(
                "unpaid inputs omit part of their actual declaration/body family",
            ));
        }
        Ok(())
    }
    fn check_cells(&self, work: &mut ReservationWork) -> Result<(), Error> {
        let artifact = self
            .inventory
            .artifacts()
            .get(self.provider_index as usize)
            .ok_or_else(|| error("unpaid input cell inspection lost its selected artifact"))?;
        let rows = artifact
            .source_descriptor()
            .ok_or_else(|| error("unpaid input cell inspection lost its actual descriptor"))?
            .rows();
        for (ordinal, row) in rows.declarations.iter().enumerate() {
            // definition_at performs three catalog owner checks in addition
            // to this row visit. Charge all attempted work before that query.
            work.charge(4)?;
            let reference = crate::support::crates::owner_catalog::DeclarationRef(
                u32::try_from(ordinal).map_err(|_| error("unpaid declaration ordinal overflow"))?,
            );
            let definition = self
                .catalog
                .definition_at(artifact.identity(), reference)
                .map_err(error)?;
            let actual = self
                .report
                .resolution
                .definitions
                .get(definition.0 as usize)
                .ok_or_else(|| error("unpaid declaration cell is outside its actual report"))?;
            if !self.definitions.contains(&(definition.0 as usize))
                || actual.node.is_some()
                || row.name.as_deref() != Some(self.report.package.names.get(actual.name))
                || (actual.kind, actual.namespace)
                    != original_declarations::source_kind(row.definition.kind)
                || !self.scopes.contains(&(actual.scope.0 as usize))
                || actual
                    .members
                    .is_some_and(|scope| !self.scopes.contains(&(scope.0 as usize)))
                || actual.access
                    != if row.public {
                        crate::resolve::model::Access::Public
                    } else {
                        crate::resolve::model::Access::ExternalPrivate
                    }
            {
                return Err(error(
                    "unpaid declaration lost its actual monotonic resolver cell",
                ));
            }
        }
        Ok(())
    }
}
impl<'a> RecipeView<'a> {
    pub(crate) fn input_site(
        &self,
        owner: crate::support::crates::owner_catalog::DeclarationRef,
        site: u32,
        work: &mut ReservationWork,
    ) -> Result<crate::resolve::body_origin::ProofSite, Error> {
        let declaration = self.declaration(owner, work)?;
        work.charge(2)?;
        let site = self
            .owner
            .catalog
            .declaration_site(declaration, site)
            .map_err(error)?;
        self.owner
            .catalog
            .declaration_proof_site(declaration, site)
            .map_err(error)
    }
    pub(crate) fn body_site(
        &self,
        body: crate::support::crates::owner_catalog::BodyRef,
        site: u32,
        work: &mut ReservationWork,
    ) -> Result<crate::resolve::body_origin::ProofSite, Error> {
        let body = self.body(body, work)?;
        work.charge(2)?;
        let site = self.owner.catalog.site(body, site).map_err(error)?;
        self.owner.catalog.proof_site(body, site).map_err(error)
    }
    pub(crate) fn root(&self) -> crate::resolve::body_origin::ImportedDeclarationId {
        self.owner.root
    }
    pub(crate) fn rows(
        &self,
    ) -> Result<crate::support::crates::generics::source::ValidatedView<'a>, Error> {
        self.owner
            .inventory
            .artifacts()
            .get(self.owner.provider_index as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or_else(|| error("reserved recipe view lost its immutable descriptor"))
            .map(crate::support::crates::SourceDescriptor::rows)
    }
    pub(crate) fn definition(
        &self,
        row: crate::support::crates::owner_catalog::DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<crate::resolve::model::DefId, Error> {
        work.charge(4)?;
        let artifact = &self.owner.inventory.artifacts()[self.owner.provider_index as usize];
        self.owner
            .catalog
            .definition_at(artifact.identity(), row)
            .map_err(error)
    }
    pub(crate) fn declaration(
        &self,
        row: crate::support::crates::owner_catalog::DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<crate::resolve::body_origin::ImportedDeclarationId, Error> {
        let definition = self.definition(row, work)?;
        work.charge(1)?;
        self.owner.catalog.declaration(definition).map_err(error)
    }
    pub(crate) fn body(
        &self,
        row: crate::support::crates::owner_catalog::BodyRef,
        work: &mut ReservationWork,
    ) -> Result<crate::resolve::body_origin::ImportedBodyId, Error> {
        work.charge(2)?;
        let artifact = &self.owner.inventory.artifacts()[self.owner.provider_index as usize];
        self.owner
            .catalog
            .body_at(artifact.identity(), row)
            .map_err(error)
    }
}
