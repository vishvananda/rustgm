//! An authenticated crate inventory installs neutral headers, never checked execution.
use super::{Declarations, Error, Headers, error, mapped_source, staged_headers::Installation};
use crate::resolve::{crates::Imported, expansion::Report};

pub(super) fn import(
    report: &Report,
    imported: &Imported,
    facts: &mut Declarations,
) -> Result<(), Error> {
    if report
        .resolution
        .crates
        .as_deref()
        .is_none_or(|actual| !std::ptr::eq(actual, imported))
    {
        return Err(error(
            "source headers changed their actual resolver inventory",
        ));
    }
    let indices = active_indices(report, imported)?;
    let catalog = imported
        .origin_catalog()
        .map_err(error)?
        .ok_or_else(|| error("source headers lack their completed origin catalog"))?;
    if facts
        .crates
        .as_ref()
        .is_none_or(|linkage| linkage.originals.is_some())
    {
        return Err(error(
            "source headers cannot replace an existing or absent semantic linkage",
        ));
    }
    let dependencies = imported.dependencies();
    let mut headers = Headers {
        base: catalog
            .retained_bytes()
            .saturating_add(dependencies.generic_bytes()),
        ..Headers::default()
    };
    headers
        .reserve(size_of::<Headers>().saturating_add(crate::support::retained::vector(&indices)))?;
    let mut installation = Installation::new(facts, &mut headers)?;
    for index in indices {
        authenticate_bindings(report, imported, facts, index)?;
        let descriptor = dependencies.artifacts()[index as usize]
            .source_descriptor()
            .ok_or_else(|| error("source headers require their complete actual descriptor"))?;
        let context = mapped_source::Context {
            imported,
            catalog: &catalog,
            descriptor,
            provider_index: index,
        };
        installation.add(&context, facts, &mut headers)?;
    }
    installation.finish(facts, &mut headers)?;
    headers.observed.add(size_of::<Headers>());
    facts.crates.get_or_insert_default().originals = Some(Box::new(headers));
    Ok(())
}

fn active_indices(report: &Report, imported: &Imported) -> Result<Vec<u32>, Error> {
    let dependencies = imported.dependencies();
    let selected = report
        .package
        .source_client()
        .map(|client| client.require_inventory(dependencies))
        .transpose()
        .map_err(error)?;
    let mut indices = Vec::new();
    for position in dependencies.installation_order() {
        let artifact = &dependencies.artifacts()[position];
        if artifact.generic_payload().is_none() {
            continue;
        }
        let index =
            u32::try_from(position).map_err(|_| error("typed crate artifact index overflow"))?;
        let descriptor = artifact.source_descriptor();
        let bootstrap =
            descriptor.is_some_and(|descriptor| descriptor.profile().bootstrap_bindings());
        match selected {
            Some(actual) if actual == index && !bootstrap && descriptor.is_some() => {
                indices.push(index);
            }
            None if bootstrap => indices.push(index),
            None if indices.is_empty() && descriptor.is_some() => {
                return Err(error(
                    "source headers require their authenticated source client report",
                ));
            }
            _ => {
                return Err(error(
                    "mixed generic source inventories require their wider complete import profile",
                ));
            }
        }
    }
    if indices.is_empty() {
        return Err(error(
            "source headers require their complete actual descriptor",
        ));
    }
    Ok(indices)
}

/// Keep binding verification constant per loan of an imported original.
pub(super) fn authenticate_bindings(
    report: &Report,
    imported: &Imported,
    facts: &Declarations,
    index: u32,
) -> Result<(), Error> {
    let descriptor = imported
        .dependencies()
        .artifacts()
        .get(index as usize)
        .and_then(crate::support::crates::Interface::source_descriptor)
        .ok_or_else(|| error("imported crate lost its actual binding descriptor"))?;
    if descriptor.profile().bootstrap_bindings() {
        if report.package.source_core().is_some()
            || report.package.source_client().is_some()
            || report.package.staged_consumer().is_some()
            || facts.source_bindings.is_some()
        {
            return Err(error(
                "bootstrap crate metadata requires ordinary compiler bindings",
            ));
        }
        return Ok(());
    }
    let client = report
        .package
        .source_client()
        .ok_or_else(|| error("source headers require their authenticated source client report"))?;
    if client
        .require_inventory(imported.dependencies())
        .map_err(error)?
        != index
        || descriptor.identity() != client.provider()
    {
        return Err(error(
            "source headers changed their actual selected provider",
        ));
    }
    facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("source headers lack their authenticated role catalog"))?
        .require_provider_report(report)
        .map_err(error)
}
