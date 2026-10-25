//! Removed Test owners retain diagnostic obligations without demanding descendants.
use super::{Declarations, Error, NodeKey, Report, WarningCode};

pub(super) fn collect(
    report: &Report,
    facts: &mut Declarations,
    owner: NodeKey,
) -> Result<(), Error> {
    let Some(removal) = report
        .removed_functions()
        .and_then(|removals| removals.get(owner))
    else {
        return Ok(());
    };
    // The actual Test consumer diagnoses the first remaining literal test
    // attribute once, including a duplicate whose later input is never demanded.
    let Some(&duplicate) = removal.duplicates.first() else {
        return Ok(());
    };
    let unit = report.package.unit(owner.file);
    let offset = unit
        .configured
        .effects
        .get(duplicate)
        .ok_or_else(|| Error::at(owner.file, 0, "removed Test duplicate effect disappeared"))?
        .attribute
        .attribute
        .span
        .start;
    facts.must_use_diagnostic(
        report,
        owner,
        offset,
        0,
        WarningCode::DuplicateMacroAttributes,
        "duplicated test attribute".into(),
    )
}
