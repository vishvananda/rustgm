//! Existing projection and cold column limits are exercised without allocation hooks.
use super::*;
use std::fmt::Write;
const GENERATED: &str = "trait T{fn f()->u8;}macro_rules! m{()=>{#[cfg(any())]impl<X>T for X{default fn f()->u8{1}}}}m!();fn main(){}";
fn projection_identity() -> TestResult {
    let source = "macro_rules! m{()=>{fn f(){}}}m!();fn main(){}";
    let found = report(source)?;
    let id = definition(&found, DefKind::Function, "f")?;
    let key = found.resolution.definitions[id.0 as usize]
        .node
        .ok_or("real expanded function")?;
    let offset = found
        .package
        .unit(key.file)
        .configured
        .ast
        .node(key.node)
        .span
        .start;
    let ordinary = found.diagnostic(Error::at(key.file, offset, "ordinary diagnostic"));
    let mut steps = 0_usize;
    let mut bytes = 0_usize;
    let charged = found.diagnostic_charged(
        Error::at(key.file, offset, "ordinary diagnostic"),
        |work, payload| {
            steps = steps.saturating_add(work);
            bytes = bytes.saturating_add(payload);
            Ok(())
        },
    )?;
    let expected = format!(
        "file 0: {}: ordinary diagnostic\n  in macro m at file 0: {}; defined at file 0: 0",
        source.find("fn f").ok_or("definition token")?,
        source.find("m!();").ok_or("invocation token")?
    );
    assert_eq!(ordinary.to_string(), expected);
    assert_eq!(charged.to_string(), expected);
    assert!(!charged.expansions.is_empty() && steps > 0 && bytes > 0);
    Ok(())
}
fn budget_and_origin_failures() -> TestResult {
    let found = report(GENERATED)?;
    let entries = found.package.units[0]
        .configured
        .ast
        .gate_sites
        .as_ref()
        .ok_or("generated column")?;
    let row = entries.entries()[0];
    assert_ne!(row.context_key(), Some(std::num::NonZeroU32::MIN));
    let limit = 64 * 1024 * 1024;
    entries.charge(0, limit - entries.storage().3)?;
    let original = Error::at(FileId(0), row.span.start, "primary projection");
    let failure = found
        .diagnostic_charged(original, |steps, bytes| entries.charge(steps, bytes))
        .err()
        .ok_or("exhausted projection retained a partial trace")?;
    assert_eq!(failure.message, "primary projection");
    assert_eq!(failure.span.start, row.span.start);
    assert!(failure.expansions.is_empty());
    let joined = found.retain_expansion_warnings(Error::at(FileId(0), 0, "primary unchanged"));
    assert_eq!(joined.message, "primary unchanged");
    assert!(joined.warnings().is_empty());
    assert_eq!(
        joined.to_string(),
        "file 0: 0: primary unchanged\n  note: retained warning work or storage budget exceeded; further warnings unavailable"
    );
    let before = entries.storage().2;
    let _ = found.pre_expansion_warnings();
    assert!(
        entries.storage().2 > before,
        "failed read-only attempts must remain charged"
    );
    for replace_image in [true, false] {
        let mut found = report(GENERATED)?;
        let row = found.package.units[0]
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("actual row")?
            .entries()[0];
        if replace_image {
            found.package.units[0].source = found.package.units[0].source.physical_copy();
        } else {
            let origin = found.package.units[0]
                .origins
                .iter_mut()
                .find(|origin| {
                    origin.generated.start <= row.span.start
                        && row.span.start < origin.generated.end
                })
                .ok_or("actual keyword origin")?;
            origin.context = Some(crate::expand::marks::SyntaxContextId::ROOT);
        }
        let error = found.retain_expansion_warnings(Error::at(FileId(0), 0, "primary unchanged"));
        assert_eq!(error.message, "primary unchanged");
        assert!(error.warnings().is_empty());
        assert_eq!(
            error.to_string(),
            "file 0: 0: primary unchanged\n  note: retained warning source could not be authenticated; further warnings unavailable"
        );
    }
    Ok(())
}
fn cold_storage_and_enclosing_layouts() -> TestResult {
    let ordinary = report("fn main(){}")?;
    assert!(
        ordinary.package.units[0]
            .configured
            .ast
            .gate_sites
            .is_none()
    );
    assert!(ordinary.attribute_effects.is_none());
    for width in [8, 32, 128] {
        let mut text = String::from("#![feature(min_specialization)]fn main(){}");
        for index in 0..width {
            write!(
                text,
                "struct S{index};impl S{index}{{default fn f()->u8{{1}}}}"
            )?;
        }
        let found = report(&text)?;
        let entries = found.package.units[0]
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("scaled column")?;
        let before = entries.storage();
        assert_eq!(before.0, width);
        assert!(entries.pending_context().is_none());
        pipeline::analyze_source(&found)?;
        pipeline::analyze_source(&found)?;
        let after = entries.storage();
        assert_eq!(before.0, after.0);
        assert_eq!(before.1, after.1);
        assert!(after.2 > before.2 && after.2 - before.2 < width * 1024);
        println!(
            "defaultness width={width} rows={} capacity={} owner={} row={} payload={} steps={} bytes={} replay-step-delta={}",
            after.0,
            after.1,
            size_of::<crate::syntax::gate_sites::Declarations>(),
            size_of::<crate::syntax::gate_sites::Declaration>(),
            found.package.units[0].configured.ast.gate_sites_bytes(),
            after.2,
            after.3,
            after.2 - before.2
        );
    }
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::syntax::item::Item>(), 64);
    assert_eq!(size_of::<crate::syntax::item::ItemKind>(), 28);
    assert_eq!(size_of::<crate::resolve::model::Scope>(), 32);
    assert_eq!(size_of::<crate::resolve::model::Definition>(), 36);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::resolve::lint_policy::Policy>(), 12);
    assert_eq!(size_of::<Report>(), 3040);
    assert_eq!(size_of::<Engine<'_, NoFiles>>(), 4088);
    assert_eq!(size_of::<crate::resolve::expansion::Body>(), 96);
    assert_eq!(size_of::<crate::resolve::expansion::Definition>(), 136);
    assert_eq!(size_of::<crate::resolve::package::ErrorData>(), 128);
    println!(
        "defaultness enclosing Syntax={} Configured={} Unit={} Extent={} Original-Saved={:?} Report={} Engine={}; authenticated prior Syntax304 Configured384 Unit672; prior Original/Extent unmeasured",
        size_of::<Syntax<crate::literal::storage::StoredValue>>(),
        size_of::<
            crate::expand::crate_syntax::ConfiguredSyntax<crate::literal::storage::StoredValue>,
        >(),
        size_of::<Unit>(),
        size_of::<crate::syntax::transfer::Extent>(),
        Unit::original_layout_bytes(),
        size_of::<Report>(),
        size_of::<Engine<'_, NoFiles>>()
    );
    Ok(())
}
#[test]
fn charged_projection_retains_primary_failures_and_cold_storage_is_bounded() -> TestResult {
    projection_identity()?;
    budget_and_origin_failures()?;
    cold_storage_and_enclosing_layouts()?;
    Ok(())
}
