//! An authenticated crate inventory installs neutral headers, never checked execution.
use super::{Declarations, Error, Headers, error, mapped_source, staged_headers::Stage};
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
    let index = if let Some(client) = report.package.source_client() {
        client
            .require_inventory(imported.dependencies())
            .map_err(error)?
    } else {
        let index = imported
            .dependencies()
            .artifacts()
            .iter()
            .position(|artifact| {
                artifact
                    .source_descriptor()
                    .is_some_and(|descriptor| descriptor.profile().bootstrap_bindings())
            })
            .ok_or_else(|| {
                error("source headers require their authenticated source client report")
            })?;
        u32::try_from(index).map_err(|_| error("bootstrap crate artifact index overflow"))?
    };
    authenticate_bindings(report, imported, facts, index)?;
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
    let artifact = imported
        .dependencies()
        .artifacts()
        .get(index as usize)
        .ok_or_else(|| error("source provider index is outside its authenticated inventory"))?;
    let descriptor = artifact
        .source_descriptor()
        .ok_or_else(|| error("source headers require their complete actual descriptor"))?;
    for (position, artifact) in imported.dependencies().artifacts().iter().enumerate() {
        if position != index as usize && artifact.generic_payload().is_some() {
            return Err(error(
                "mixed generic source inventories require their wider complete import profile",
            ));
        }
    }
    let mut headers = Headers {
        base: imported.dependencies().artifacts().iter().fold(
            catalog.retained_bytes(),
            |bytes, artifact| {
                bytes.saturating_add(
                    artifact
                        .generic_payload()
                        .map_or(0, crate::support::crates::generics::Payload::retained_bytes),
                )
            },
        ),
        ..Headers::default()
    };
    headers.reserve(size_of::<Headers>())?;
    let context = mapped_source::Context {
        imported,
        catalog: &catalog,
        descriptor,
        provider_index: index,
    };
    Stage::prepare(&context, facts, &mut headers)?.install(&context, facts, &mut headers)?;
    headers.observed.add(size_of::<Headers>());
    facts.crates.get_or_insert_default().originals = Some(Box::new(headers));
    Ok(())
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
