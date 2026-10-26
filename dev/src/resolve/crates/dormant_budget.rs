//! The private moved-report reservation charges actual existing growth dimensions.
use super::{Dependencies, Error, Report, ReservationWork};
use crate::support::crates::{Interface, generics::source::ValidatedView};

pub(super) fn reserve(
    report: &Report,
    inventory: &Dependencies,
    artifact: &Interface,
    rows: ValidatedView<'_>,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    reserve_resolution(
        &report.package,
        &report.resolution,
        inventory,
        artifact,
        rows,
        work,
    )
}
pub(in crate::resolve::crates) fn reserve_resolution(
    package: &crate::resolve::package::Package,
    resolution: &crate::resolve::model::Resolution,
    inventory: &Dependencies,
    artifact: &Interface,
    rows: ValidatedView<'_>,
    work: &mut ReservationWork,
) -> Result<(), Error> {
    let metadata = inventory
        .artifacts()
        .iter()
        .try_fold(0_usize, |bytes, artifact| {
            work.charge(1)?;
            Ok::<_, Error>(
                bytes.saturating_add(
                    artifact
                        .generic_payload()
                        .map_or(0, crate::support::crates::generics::Payload::retained_bytes),
                ),
            )
        })?;
    work.reserve(
        metadata
            .saturating_add(size_of::<super::Inputs>())
            .saturating_add(
                4 * (size_of::<crate::resolve::model::Definition>()
                    + size_of::<crate::resolve::model::Scope>()
                    + size_of::<(
                        crate::support::crates::ExternalDef,
                        crate::resolve::model::DefId,
                    )>()),
            )
            .saturating_add(4 * artifact.identity().name().len()),
    )?;
    let lexical = rows
        .originals
        .iter()
        .try_fold(rows.declarations.len(), |count, original| {
            work.charge(1)?;
            Ok::<_, Error>(count.saturating_add(original.scopes.len()))
        })?;
    // Existing vector/interner payload can coexist with replacement buffers
    // while genuine cells append. Charge the moved report's actual size
    // dimensions too, not just the small incoming packet's row counts.
    work.reserve(
        resolution
            .definitions
            .capacity()
            .saturating_add(rows.declarations.len())
            .saturating_mul(4 * size_of::<crate::resolve::model::Definition>())
            .saturating_add(
                resolution
                    .scopes
                    .capacity()
                    .saturating_add(lexical)
                    .saturating_mul(4 * size_of::<crate::resolve::model::Scope>()),
            )
            .saturating_add(
                package
                    .names
                    .len()
                    .saturating_add(rows.declarations.len())
                    .saturating_mul(256),
            )
            .saturating_add(package.names.byte_len().saturating_mul(8)),
    )?;
    Ok(())
}
