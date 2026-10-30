//! Canonical source inputs are derived once, never supplied by site-only claims.
use super::{Error, Headers, error};
use crate::{
    resolve::{
        body_origin::{Catalog, ImportedDeclarationId},
        crates::Imported,
        model::{DefId, DefKind},
    },
    sema::declaration_inputs::Input,
    support::crates::{SourceDescriptor, generics::source as s},
    types::{RegionId, Store, TypeId, parameters::predicates::Predicate},
};
mod application_rows;
#[path = "predicate_remap/translated_inputs.rs"]
pub(super) mod translation;
use std::sync::Arc;

/// The graph is the canonical remap owned by the same staged header ingest.
/// It is not a new declaration or body index and never carries paid proof.
pub(super) struct Context<'a> {
    imported: &'a Imported,
    catalog: &'a Catalog,
    descriptor: SourceDescriptor<'a>,
    graph: &'a super::mapped_source::Graph,
    applications: Vec<Option<Predicate>>,
}

/// Fields stay private so arbitrary predicates cannot acquire canonical-prefix
/// status by retaining a valid source site. Only the typed remapper constructs it.
pub(in crate::sema) struct Prepared {
    declaration: ImportedDeclarationId,
    inputs: Arc<[Input]>,
    self_subjects: Box<[bool]>,
}
impl Prepared {
    pub(super) fn all(&self) -> &Arc<[Input]> {
        &self.inputs
    }
    pub(super) fn subjects(&self) -> &[bool] {
        &self.self_subjects
    }
    pub(super) fn require_declaration(
        &self,
        declaration: ImportedDeclarationId,
    ) -> Result<(), Error> {
        if self.declaration != declaration {
            return Err(error(
                "source trait inputs changed their actual imported declaration",
            ));
        }
        Ok(())
    }
    pub(in crate::sema) fn into_parts(self) -> (ImportedDeclarationId, Arc<[Input]>, Box<[bool]>) {
        (self.declaration, self.inputs, self.self_subjects)
    }
}
impl<'a> Context<'a> {
    pub(super) fn finish(
        mut self,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Vec<Option<Predicate>>, Error> {
        let count = self.applications.len();
        budget.reserve(count.saturating_mul(size_of::<Predicate>()))?;
        for index in 0..if self.descriptor.profile() == s::Profile::Source2024ModuleConversions {
            0
        } else {
            count
        } {
            let index = u32::try_from(index)
                .map_err(|_| error("source trait application ordinal overflow"))?;
            self.application(s::TraitApplicationRef(index), store, budget)?;
        }
        let applications = self.applications;
        budget.observed.vector(&applications);
        Ok(applications)
    }
    pub(super) fn new(
        imported: &'a Imported,
        catalog: &'a Catalog,
        descriptor: SourceDescriptor<'a>,
        graph: &'a super::mapped_source::Graph,
        budget: &mut Headers,
    ) -> Result<Self, Error> {
        budget.charge(1)?;
        let rows = descriptor.rows();
        graph.require(imported, catalog, descriptor, budget)?;
        let count = rows.trait_applications.len();
        budget.reserve(count.saturating_mul(size_of::<Option<Predicate>>()))?;
        Ok(Self {
            imported,
            catalog,
            descriptor,
            graph,
            applications: vec![None; count],
        })
    }
    pub(super) fn trait_inputs(
        &mut self,
        definition: DefId,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Prepared, Error> {
        budget.charge(1)?;
        let declaration = self.catalog.declaration(definition).map_err(error)?;
        self.catalog
            .validate_declaration(declaration, definition, DefKind::Trait)
            .map_err(error)?;
        let (owner, ordinal, _) = self
            .catalog
            .declaration_coordinate(declaration)
            .map_err(error)?;
        if owner != self.descriptor.identity()
            || self
                .imported
                .generic_definition(owner, ordinal)
                .map_err(error)?
                != definition
        {
            return Err(error(
                "source trait input remap has a foreign catalog/descriptor owner",
            ));
        }
        let mut port = self.port(Some((ordinal, declaration)));
        let prepared =
            translation::trait_inputs(&mut port, &mut self.applications, ordinal, store, budget)?;
        budget.observed.add(
            prepared
                .inputs
                .len()
                .saturating_mul(size_of::<Input>())
                .saturating_add(2 * size_of::<usize>())
                .saturating_add(
                    prepared
                        .self_subjects
                        .len()
                        .saturating_mul(size_of::<bool>()),
                ),
        );
        Ok(prepared)
    }
    pub(super) fn declaration_inputs(
        &mut self,
        definition: DefId,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Arc<[Input]>, Error> {
        budget.charge(1)?;
        let declaration = self.catalog.declaration(definition).map_err(error)?;
        let (owner, ordinal, _) = self
            .catalog
            .declaration_coordinate(declaration)
            .map_err(error)?;
        if owner != self.descriptor.identity()
            || self
                .imported
                .generic_definition(owner, ordinal)
                .map_err(error)?
                != definition
        {
            return Err(error(
                "source function inputs belong to another actual import owner",
            ));
        }
        let rows = self.descriptor.rows();
        let (kind, inputs) = match rows.header(ordinal).map_err(error)? {
            Some(s::HeaderRef::Function(index)) => {
                (DefKind::Function, rows.functions[index as usize].inputs)
            }
            Some(s::HeaderRef::Implementation(index)) => (
                DefKind::SelfType,
                rows.implementations[index as usize].inputs,
            ),
            _ => {
                return Err(error(
                    "source declaration inputs lack their exact completed header",
                ));
            }
        };
        self.catalog
            .validate_declaration(declaration, definition, kind)
            .map_err(error)?;
        let mut port = self.port(Some((ordinal, declaration)));
        let (inputs, _) = translation::inputs(
            &mut port,
            &mut self.applications,
            ordinal,
            inputs,
            None,
            store,
            budget,
        )?;
        budget.observed.add(
            inputs
                .len()
                .saturating_mul(size_of::<Input>())
                .saturating_add(2 * size_of::<usize>()),
        );
        Ok(inputs)
    }
    fn port(
        &self,
        site_owner: Option<(
            crate::support::crates::owner_catalog::DeclarationRef,
            ImportedDeclarationId,
        )>,
    ) -> translation::Ordinary<'a> {
        translation::Ordinary {
            imported: self.imported,
            catalog: self.catalog,
            descriptor: self.descriptor,
            graph: self.graph,
            site_owner,
        }
    }
}
