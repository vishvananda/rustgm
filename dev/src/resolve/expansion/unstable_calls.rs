//! An imported definition row's `#[unstable]` feature gates every call of it,
//! through any path or re-export. The crate must request the feature, or the
//! call must sit in an expansion whose definition may use it, as rustc checks
//! macro stability at the call's own span.
use super::{Error, Location, Report};
use crate::resolve::bindings::FeatureName;

/// Check each call of an imported row once the library requests are complete.
/// # Errors
/// Rejects an unrequested, unallowed feature by name, and a changed row.
pub(in crate::resolve) fn validate(report: &Report) -> Result<(), Error> {
    let Some(crates) = report.resolution.crates.as_deref() else {
        return Ok(());
    };
    let artifacts = crates.dependencies().artifacts();
    let mut session = None;
    let mut steps = 0;
    for invocation in &report.invocations {
        let site = Location {
            file: invocation.node.file,
            span: invocation.original,
        };
        let error = |message| Error::at(site.file, site.span.start, message);
        let imported = report
            .definitions
            .get(invocation.definition)
            .ok_or_else(|| error("macro call names an unknown definition"))?
            .imported;
        let Some(imported) = imported else {
            continue;
        };
        let feature = usize::try_from(imported.crate_id)
            .ok()
            .and_then(|id| artifacts.get(id.checked_sub(1)?))
            .and_then(crate::support::crates::Interface::macros)
            .and_then(|macros| macros.unstable(imported.row))
            .ok_or_else(|| error("imported macro stability row disappeared"))?;
        let Some(feature) = feature else {
            continue;
        };
        let name = FeatureName::new(feature).map_err(error)?;
        let requested = report
            .package
            .library_policy()
            .map(|library| library.requested_feature(&name, &mut steps))
            .transpose()
            .map_err(error)?
            .unwrap_or(false);
        if requested
            || session
                .get_or_insert_with(|| report.macro_allowances())
                .allows(site, &name)?
        {
            continue;
        }
        return Err(Error::at(
            site.file,
            site.span.start,
            format!("use of unstable library macro requires feature `{feature}`"),
        ));
    }
    Ok(())
}
