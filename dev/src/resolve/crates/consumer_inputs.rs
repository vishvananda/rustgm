//! A fresh consumer retains its own real reservation and one immutable catalog.
use super::{Imported, ReservationWork, error, unpaid_provided::Inputs};
use crate::{
    resolve::{
        body_origin::{Catalog, ImportedDeclarationId},
        expansion::Report,
        model::DefKind,
        package::Error,
    },
    support::crates::Dependencies,
};
use std::{ops::Range, rc::Rc, sync::Arc};

pub(crate) struct ConsumerReservation {
    inventory: Arc<Dependencies>,
    index: u32,
    catalog: Rc<Catalog>,
    root: ImportedDeclarationId,
    definitions: Range<usize>,
    scopes: Range<usize>,
}
impl ConsumerReservation {
    pub(crate) fn require(&self, report: &Report, work: &mut ReservationWork) -> Result<(), Error> {
        work.charge(6)?;
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
            || report.resolution.definitions.len() != self.definitions.end
            || report.resolution.scopes.len() != self.scopes.end
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
    };
    reservation.require(&report, work)?;
    Ok((report, reservation))
}
