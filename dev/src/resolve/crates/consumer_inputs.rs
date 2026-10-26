//! A fresh consumer retains its own real reservation and one immutable catalog.
use super::{Imported, ReservationWork, error, unpaid_provided::Inputs};
use crate::{
    expand::config::Config,
    resolve::{
        bindings::{
            Context,
            consumer_policy::{ProviderAdmission, Request},
        },
        body_origin::{Catalog, ImportedDeclarationId},
        expansion::{self, Report},
        load::{InputFile, Sources},
        model::DefKind,
        package::Error,
    },
    support::crates::Dependencies,
};
use std::{ops::Range, rc::Rc, sync::Arc};

/// The only source-to-reservation handoff; neither report nor facts are public.
pub(crate) struct ConsumerInput {
    report: Report,
    reservation: ConsumerReservation,
}
/// Parsing stays at its actual resolver boundary; semantic callers hold no Config.
pub(crate) struct ConsumerRequest<'a, S> {
    root: InputFile,
    sources: &'a mut S,
    config: &'a Config,
    context: &'a Context,
}
impl<'a, S: Sources> ConsumerRequest<'a, S> {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the actual private consumer request awaits selected role/header and original-body admission"
        )
    )]
    pub(crate) fn new(
        root: InputFile,
        sources: &'a mut S,
        config: &'a Config,
        context: &'a Context,
    ) -> Self {
        Self {
            root,
            sources,
            config,
            context,
        }
    }
    pub(crate) fn require_inventory(
        &self,
        inventory: &Arc<Dependencies>,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(1)?;
        if self
            .context
            .dependencies
            .as_ref()
            .is_none_or(|actual| !Arc::ptr_eq(actual, inventory))
        {
            return Err(error(
                "collected provider request changed its actual inventory",
            ));
        }
        Ok(())
    }
    pub(crate) fn admit(
        self,
        admission: ProviderAdmission,
        work: &mut ReservationWork,
    ) -> Result<ConsumerInput, Error> {
        ConsumerInput::new(
            self.root,
            self.sources,
            self.config,
            self.context,
            admission,
            work,
        )
    }
}
impl ConsumerInput {
    pub(crate) fn new(
        root: InputFile,
        sources: &mut impl Sources,
        config: &Config,
        context: &Context,
        admission: ProviderAdmission,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        work.charge(root.source.text().len().saturating_add(1))?;
        work.reserve(size_of::<Self>())?;
        let request = Request::new(admission, config, work);
        let (report, seed) = expansion::analyze_staged(root, sources, config, context, request)?;
        let report = crate::resolve::analyze_expanded(report)?;
        let (report, reservation) = if let Some(seed) = seed {
            seed.finish(report, work)?
        } else {
            Inputs::reserve_consumer(report, work)?
        };
        Ok(Self {
            report,
            reservation,
        })
    }
    pub(crate) fn into_parts(self) -> (Report, ConsumerReservation) {
        (self.report, self.reservation)
    }
}

pub(crate) struct ConsumerReservation {
    pub(super) inventory: Arc<Dependencies>,
    pub(super) index: u32,
    catalog: Rc<Catalog>,
    pub(super) root: ImportedDeclarationId,
    definitions: Range<usize>,
    scopes: Range<usize>,
    definition_extent: usize,
    scope_extent: usize,
}
/// Kept on the staged loader's stack while genuine source declarations append.
/// These are the actual imported ranges and SAME catalog, never semantic readiness.
pub(in crate::resolve) struct ReservationSeed {
    inventory: Arc<Dependencies>,
    index: u32,
    catalog: Rc<Catalog>,
    root: ImportedDeclarationId,
    definitions: Range<usize>,
    scopes: Range<usize>,
}
impl ReservationSeed {
    pub(in crate::resolve) fn finish(
        self,
        report: Report,
        work: &mut ReservationWork,
    ) -> Result<(Report, ConsumerReservation), Error> {
        work.charge(1)?;
        work.reserve(size_of::<ConsumerReservation>())?;
        let reservation = ConsumerReservation {
            inventory: self.inventory,
            index: self.index,
            catalog: self.catalog,
            root: self.root,
            definitions: self.definitions,
            scopes: self.scopes,
            definition_extent: report.resolution.definitions.len(),
            scope_extent: report.resolution.scopes.len(),
        };
        reservation.require(&report, work)?;
        Ok((report, reservation))
    }
}
/// Called only from the genuine staged expansion engine, before its first scan.
pub(in crate::resolve) fn reserve_names(
    resolver: &mut super::Resolver<'_>,
    work: &mut ReservationWork,
) -> Result<Option<ReservationSeed>, Error> {
    let policy = resolver
        .package
        .staged_consumer()
        .cloned()
        .ok_or_else(|| error("named reservation requires its actual staged consumer"))?;
    policy.require_package(resolver.package).map_err(error)?;
    let Some(names) = policy.unpaid_namespace() else {
        return Ok(None);
    };
    if resolver.facts.crates.is_some() {
        return Err(error("named reservation cannot replace an imported owner"));
    }
    let inventory = Arc::clone(policy.inventory());
    let index = policy.index();
    let artifact = inventory
        .artifacts()
        .get(index as usize)
        .ok_or_else(|| error("named reservation lost its actual provider"))?;
    let rows = artifact
        .source_descriptor()
        .ok_or_else(|| error("named reservation lost its source descriptor"))?
        .rows();
    super::unpaid_provided::reserve_resolution(
        resolver.package,
        resolver.facts,
        &inventory,
        artifact,
        rows,
        work,
    )?;
    work.charge(5)?;
    work.reserve(size_of::<ReservationSeed>().saturating_add(size_of::<Imported>()))?;
    let first_definition = resolver.facts.definitions.len();
    let first_scope = resolver.facts.scopes.len();
    let external = resolver.new_scope(None, crate::resolve::package::ModuleId(0), true)?;
    let mut imported = Imported::new(Arc::clone(&inventory));
    let scope = resolver.reserve_artifact_root(artifact, &mut imported)?;
    super::original_declarations::named_unpaid(
        resolver,
        scope,
        artifact,
        &mut imported,
        names,
        work,
    )?;
    work.charge(inventory.aliases().iter().fold(1_usize, |count, alias| {
        count.saturating_add(alias.alias().len().saturating_add(1))
    }))?;
    resolver.bind_crate_aliases(external, &imported)?;
    resolver.facts.extern_scope = Some(external);
    let catalog = imported
        .origin_catalog()
        .map_err(error)?
        .ok_or_else(|| error("named reservation lost its actual completed catalog"))?;
    work.charge(4)?;
    let definition = catalog
        .definition_at(artifact.identity(), names.root)
        .map_err(error)?;
    let root = catalog.declaration(definition).map_err(error)?;
    catalog
        .validate_declaration(root, definition, DefKind::Module)
        .map_err(error)?;
    let seed = ReservationSeed {
        inventory,
        index,
        catalog,
        root,
        definitions: first_definition..resolver.facts.definitions.len(),
        scopes: first_scope..resolver.facts.scopes.len(),
    };
    resolver.facts.crates = Some(Box::new(imported));
    Ok(Some(seed))
}
impl ConsumerReservation {
    pub(crate) fn inventory(&self) -> &Arc<Dependencies> {
        &self.inventory
    }
    pub(crate) fn index(&self) -> u32 {
        self.index
    }
    pub(crate) fn retained_recipe_view(
        &self,
        work: &mut ReservationWork,
    ) -> Result<super::unpaid_provided::RecipeView<'_>, Error> {
        work.charge(5)?;
        let definition = self
            .catalog
            .declaration_identity(self.root, DefKind::Module)
            .map_err(error)?;
        let (provider, row, edition) = self
            .catalog
            .declaration_coordinate(self.root)
            .map_err(error)?;
        let descriptor = self
            .inventory
            .artifacts()
            .get(self.index as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or_else(|| error("retained header owner lost its actual descriptor"))?;
        if provider != descriptor.identity()
            || edition != descriptor.edition()
            || descriptor.profile()
                != crate::support::crates::generics::source::Profile::Source2024Provided
            || self.catalog.definition_at(provider, row).map_err(error)? != definition
            || descriptor
                .rows()
                .declarations
                .get(row.0 as usize)
                .is_none_or(|row| {
                    row.definition.kind
                        != crate::support::crates::generics::source::DefinitionKind::Module
                        || row.definition.parent.is_some()
                })
        {
            return Err(error(
                "retained header owner changed its actual catalog/root/inventory",
            ));
        }
        Ok(super::unpaid_provided::RecipeView::retained(self))
    }
    pub(crate) fn recipe_view<'a>(
        &'a self,
        report: &'a Report,
        work: &mut ReservationWork,
    ) -> Result<super::unpaid_provided::RecipeView<'a>, Error> {
        super::unpaid_provided::RecipeView::consumer(self, report, work)
    }
    pub(crate) fn require(&self, report: &Report, work: &mut ReservationWork) -> Result<(), Error> {
        work.charge(8)?;
        let policy = report
            .package
            .staged_consumer()
            .ok_or_else(|| error("private consumer reservation lost its genuine policy"))?;
        policy.require_package(&report.package).map_err(error)?;
        let imported = report
            .resolution
            .crates
            .as_deref()
            .ok_or_else(|| error("private consumer reservation lost its actual inventory"))?;
        let catalog = imported
            .origin_catalog()
            .map_err(error)?
            .ok_or_else(|| error("private consumer reservation lost its actual catalog"))?;
        if !Arc::ptr_eq(policy.inventory(), &self.inventory)
            || self.index != policy.index()
            || !std::ptr::eq(imported.dependencies(), self.inventory.as_ref())
            || !Rc::ptr_eq(&catalog, &self.catalog)
            || !self.definitions.contains(
                &(self
                    .catalog
                    .declaration_identity(self.root, DefKind::Module)
                    .map_err(error)?
                    .0 as usize),
            )
            || self.definitions.end > self.definition_extent
            || self.scopes.end > self.scope_extent
            || report.resolution.definitions.len() != self.definition_extent
            || report.resolution.scopes.len() != self.scope_extent
            || !report.names_resolved
        {
            return Err(error(
                "private consumer changed its actual report/catalog/reservation",
            ));
        }
        Ok(())
    }
    pub(crate) fn catalog(&self) -> &Rc<Catalog> {
        &self.catalog
    }
}

pub(super) fn bind(
    inputs: Inputs,
    mut imported: Imported,
    work: &mut ReservationWork,
) -> Result<(Report, ConsumerReservation), Error> {
    inputs.require(&inputs.inventory, inputs.provider_index, work)?;
    if inputs.report.resolution.crates.is_some()
        || imported.origin_catalog().map_err(error)?.is_some()
    {
        return Err(error(
            "private consumer cannot replace an existing imported owner",
        ));
    }
    work.charge(3)?;
    work.reserve(
        size_of::<Imported>() + size_of::<ConsumerReservation>() + 2 * size_of::<usize>(),
    )?;
    let Inputs {
        mut report,
        inventory,
        provider_index,
        catalog,
        root,
        definitions,
        scopes,
        ..
    } = inputs;
    imported.origins = catalog.into_slot().map_err(error)?;
    let catalog = imported
        .origin_catalog()
        .map_err(error)?
        .ok_or_else(|| error("private consumer catalog adoption disappeared"))?;
    report.resolution.crates = Some(Box::new(imported));
    let reservation = ConsumerReservation {
        inventory,
        index: provider_index,
        catalog,
        root,
        definitions,
        scopes,
        definition_extent: report.resolution.definitions.len(),
        scope_extent: report.resolution.scopes.len(),
    };
    reservation.require(&report, work)?;
    Ok((report, reservation))
}
