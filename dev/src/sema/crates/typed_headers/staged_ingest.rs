//! A real `SourceClient` inventory installs neutral headers, never checked execution.
use super::{Declarations, Error, Headers, error, mapped_source, staged_headers::Stage};
use crate::resolve::{crates::Imported, expansion::Report};

pub(super) fn import(
    report: &Report,
    imported: &Imported,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let client = report
        .package
        .source_client()
        .ok_or_else(|| error("source headers require their authenticated source client report"))?;
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
    let index = client
        .require_inventory(imported.dependencies())
        .map_err(error)?;
    facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("source headers lack their authenticated role catalog"))?
        .require_provider_report(report)
        .map_err(error)?;
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
    if descriptor.identity() != client.provider() {
        return Err(error(
            "source headers changed their actual selected provider",
        ));
    }
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
