//! Foreign numeric identities and warm memo substitutions are not header authority.
use super::{TestResult, owner_usage, provider, sites};
use crate::{
    resolve::{
        bindings::{FeatureName, KnownFeature},
        unit::NodeKey,
    },
    support::source::SourceFile,
};

fn allowed(feature: &str) -> TestResult<crate::resolve::expansion::Report> {
    provider(
        &format!("#[allow_internal_unstable({feature})]macro_rules!route{{()=>{{probe()}};}}"),
        "route!()",
    )
}

#[test]
fn published_item_reauthentication_uses_its_actual_attributed_owner() -> TestResult {
    use crate::resolve::bindings::panic_contracts;
    use std::rc::Rc;
    let report = allowed("panic_internals")?;
    let site = sites(&report, "probe")?[0];
    let invocation = report.invocations.first().ok_or("actual invocation")?;
    let definition = &report.definitions[invocation.definition];
    let item = definition.node.ok_or("actual macro item")?;
    let snapshot = definition.header.as_ref().ok_or("actual header")?;
    let published = panic_contracts::published(&report.package, item, Some(snapshot), 1)?;
    assert_ne!(published.source_owner(), item);
    // A bare item is not the configured wrapper: retain that strict opponent.
    assert!(panic_contracts::checked(&report.package, item).is_err());
    let header = panic_contracts::checked(&report.package, published.source_owner())?
        .ok_or("authenticated configured owner")?;
    assert_eq!(header.summary().node, item);
    assert!(Rc::ptr_eq(
        &header.into_data().ok_or("owned header")?,
        snapshot
    ));
    let mut session = report.macro_allowances();
    for _ in 0..2 {
        assert!(session.allows(site, &KnownFeature::PanicInternals.name())?);
    }
    Ok(())
}

#[test]
fn same_numbered_reports_and_foreign_memo_rows_keep_actual_owner_identity() -> TestResult {
    let left = allowed("panic_internals")?;
    let right = allowed("core_intrinsics")?;
    let a = sites(&left, "probe")?[0];
    let b = sites(&right, "probe")?[0];
    assert_eq!(a, b);
    assert_eq!(
        left.package.syntax_context(a.file, a.span.start)?,
        right.package.syntax_context(b.file, b.span.start)?
    );
    let mut first = left.macro_allowances();
    let mut second = right.macro_allowances();
    let panic = KnownFeature::PanicInternals.name();
    let intrinsic = KnownFeature::CoreIntrinsics.name();
    assert!(first.allows(a, &panic)?);
    assert!(!second.allows(b, &panic)?);
    assert!(second.allows(b, &intrinsic)?);
    let foreign = first
        .memo
        .as_ref()
        .ok_or("left memo")?
        .headers
        .values()
        .next()
        .ok_or("left header")?
        .clone();
    let owner = *second
        .memo
        .as_ref()
        .ok_or("right memo")?
        .headers
        .keys()
        .next()
        .ok_or("right owner")?;
    second
        .memo
        .as_mut()
        .ok_or("right memo")?
        .headers
        .insert(owner, foreign);
    assert!(second.allows(b, &panic).is_err());
    Ok(())
}

fn replace_source(
    report: &mut crate::resolve::expansion::Report,
    before: &str,
    after: &str,
) -> TestResult {
    assert_eq!(before.len(), after.len());
    let unit = &mut report.package.units[0];
    let physical = unit.source.physical_len();
    let old = unit.source.text();
    assert!(old.contains(before));
    let changed = old.replacen(before, after, 1);
    let mut replacement = SourceFile::new(changed.as_bytes()[..physical].to_vec())?;
    if changed.len() > physical {
        replacement.append(&changed[physical + 1..])?;
    }
    assert_eq!(replacement.text(), changed);
    assert_eq!(replacement.physical_len(), physical);
    unit.source = replacement;
    Ok(())
}

#[test]
fn report_entry_reauthenticates_equal_width_and_equivalent_unicode_source_lexemes() -> TestResult {
    for (before, after) in [
        ("panic_internals", "core_intrinsics"),
        ("fixture_a\u{301}\u{323}", "fixture_a\u{323}\u{301}"),
    ] {
        let mut report = allowed(before)?;
        let site = sites(&report, "probe")?[0];
        replace_source(&mut report, before, after)?;
        assert!(
            report
                .macro_allowances()
                .allows(site, &FeatureName::new(before)?)
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn changed_definition_frame_and_missing_warm_header_fail_before_membership() -> TestResult {
    let mut report = allowed("core_intrinsics")?;
    let site = sites(&report, "probe")?[0];
    let definition = report.invocations[0].definition;
    assert!(!report.definitions[definition].edition_panic);
    report.definitions[definition].edition_panic = true;
    assert!(
        report
            .macro_allowances()
            .allows(site, &KnownFeature::CoreIntrinsics.name())
            .is_err()
    );
    report.definitions[definition].edition_panic = false;
    report.invocations[0].definition = usize::MAX;
    assert!(
        report
            .macro_allowances()
            .allows(site, &KnownFeature::CoreIntrinsics.name())
            .is_err()
    );
    report.invocations[0].definition = definition;
    let mut session = report.macro_allowances();
    let feature = KnownFeature::CoreIntrinsics.name();
    assert!(session.allows(site, &feature)?);
    let context = *session
        .memo
        .as_ref()
        .ok_or("memo")?
        .frames
        .keys()
        .next()
        .ok_or("frame")?;
    session
        .memo
        .as_mut()
        .ok_or("memo")?
        .frames
        .get_mut(&context)
        .ok_or("frame")?
        .original
        .start += 1;
    assert!(session.allows(site, &feature).is_err());
    let mut session = report.macro_allowances();
    assert!(session.allows(site, &feature)?);
    let owner: NodeKey = *session
        .memo
        .as_ref()
        .ok_or("memo")?
        .headers
        .keys()
        .next()
        .ok_or("owner")?;
    session.memo.as_mut().ok_or("memo")?.headers.remove(&owner);
    assert!(session.allows(site, &feature).is_err());
    Ok(())
}

#[test]
fn exhausted_global_work_and_storage_deny_warmed_positive_and_negative_queries() -> TestResult {
    for (features, storage) in [
        ("panic_internals", false),
        ("", false),
        ("panic_internals", true),
        ("", true),
    ] {
        let report = allowed(features)?;
        let site = sites(&report, "probe")?[0];
        let mut session = report.macro_allowances();
        let feature = KnownFeature::PanicInternals.name();
        assert_eq!(session.allows(site, &feature)?, !features.is_empty());
        let before = owner_usage(&report)?;
        let contracts = &report.package.source_core().ok_or("provider")?.macros;
        assert!(
            if storage {
                contracts.charge(0, usize::MAX)
            } else {
                contracts.charge(usize::MAX, 0)
            }
            .is_err()
        );
        assert!(session.allows(site, &feature).is_err());
        let after = owner_usage(&report)?;
        assert!(after.0 >= before.0 && after.1 >= before.1);
    }
    Ok(())
}
