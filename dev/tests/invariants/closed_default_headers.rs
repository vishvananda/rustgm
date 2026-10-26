//! A local body-bearing default modifier never escapes either old packet profile.
use super::*;
pub(super) fn reject_untransported_defaultness() -> Result {
    let text = "#![feature(min_specialization)]struct S;impl S{default fn f()->u8{1}}fn main(){}";
    let report = crate::test_source::resolve(text, true)?;
    crate::pipeline::analyze_source(&report)?;
    let (index, actual) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, definition)| {
            definition.kind == DefKind::Function && report.package.names.get(definition.name) == "f"
        })
        .ok_or("actual method header")?;
    let key = actual.node.ok_or("actual local function source")?;
    let modifier = report
        .active_defaultness(key)?
        .ok_or("positive defaultness fact")?;
    assert_eq!(modifier.node, key.node);
    assert_eq!(report.resolution.definitions[index].node, Some(key));
    let error = super::super::publication::require_defaultness_transport(&report, key)
        .err()
        .ok_or("untransported modifier accepted")?;
    assert_eq!(
        error.message,
        "source defaultness contract transport is not implemented"
    );
    assert_eq!(error.span.start, modifier.span.start);
    let mut forged = report;
    forged.package.units[key.file.0 as usize].source =
        forged.package.unit(key.file).source.physical_copy();
    let error = super::super::publication::require_defaultness_transport(&forged, key)
        .err()
        .ok_or("physical-copy forgery accepted")?;
    assert!(error.message.contains("incarnation"));
    Ok(())
}
