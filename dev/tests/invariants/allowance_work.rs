//! Header authentication and origin traversal have separate measured dimensions.
use super::{ROOT, TestResult, owner_usage, provider, sites};
use crate::{
    expand::stream::Location,
    resolve::{bindings::KnownFeature, unit::FileId},
    syntax::ast::Kind,
};

#[test]
fn list_width_and_repeated_uses_pay_full_authentication_once() -> TestResult {
    for length in [8_usize, 32, 128] {
        let list = std::iter::once("panic_internals".to_owned())
            .chain((1..length).map(|index| format!("fixture_feature_{index}")))
            .collect::<Vec<_>>()
            .join(",");
        for uses in [8_usize, 32, 128] {
            let report = provider(
                &format!("#[allow_internal_unstable({list})]macro_rules!route{{()=>{{probe()}};}}"),
                "route!()",
            )?;
            let site = sites(&report, "probe")?[0];
            let mut session = report.macro_allowances();
            let feature = KnownFeature::PanicInternals.name();
            let before = owner_usage(&report)?.0;
            assert!(session.allows(site, &feature)?);
            let warm = owner_usage(&report)?.0;
            for _ in 0..uses {
                assert!(session.allows(site, &feature)?);
            }
            let after = owner_usage(&report)?.0;
            let memo = session.memo.as_ref().ok_or("memo")?;
            assert_eq!(memo.headers.len(), 1);
            assert_eq!(memo.frames.len(), 1);
            assert!(
                after - warm <= 128 * uses,
                "{length}/{uses}: {}",
                after - warm
            );
            eprintln!(
                "allowance list={length} repeated={uses} cold={} warm={} payload={}",
                warm - before,
                after - warm,
                memo.payload()
            );
        }
    }
    Ok(())
}

#[test]
fn repeated_empty_attributes_keep_authenticated_negative_rows() -> TestResult {
    for length in [8_usize, 32, 128] {
        let attributes = "#[allow_internal_unstable()]".repeat(length);
        for uses in [8_usize, 32, 128] {
            let report = provider(
                &format!("{attributes}macro_rules!route{{()=>{{probe()}};}}"),
                "route!()",
            )?;
            let site = sites(&report, "probe")?[0];
            let mut session = report.macro_allowances();
            let feature = KnownFeature::PanicInternals.name();
            let before = owner_usage(&report)?.0;
            assert!(!session.allows(site, &feature)?);
            let warm = owner_usage(&report)?.0;
            for _ in 0..uses {
                assert!(!session.allows(site, &feature)?);
            }
            let after = owner_usage(&report)?.0;
            let memo = session.memo.as_ref().ok_or("negative memo")?;
            assert_eq!(memo.headers.len(), 1);
            assert!(memo.headers.values().next().ok_or("header")?.is_some());
            assert!(
                after - warm <= 128 * uses,
                "{length}/{uses}: {}",
                after - warm
            );
            eprintln!(
                "allowance empty-attributes={length} repeated={uses} cold={} warm={}",
                warm - before,
                after - warm
            );
        }
    }
    Ok(())
}

#[test]
fn unannotated_macro_frames_retain_authenticated_plain_headers() -> TestResult {
    let report = provider("macro_rules!route{()=>{probe()};}", "route!()")?;
    let site = sites(&report, "probe")?[0];
    let mut session = report.macro_allowances();
    let feature = KnownFeature::PanicInternals.name();
    assert!(!session.allows(site, &feature)?);
    let warm = owner_usage(&report)?.0;
    for _ in 0..128 {
        assert!(!session.allows(site, &feature)?);
    }
    let after = owner_usage(&report)?.0;
    let memo = session.memo.as_ref().ok_or("plain memo")?;
    assert_eq!(memo.headers.len(), 1);
    assert_eq!(memo.frames.len(), 1);
    assert!(
        memo.headers
            .values()
            .next()
            .ok_or("plain header")?
            .is_none()
    );
    assert!(after - warm <= 128 * 128);
    Ok(())
}

fn metadata_report(width: usize) -> TestResult<crate::resolve::expansion::Report> {
    use std::fmt::Write;
    let mut source = ROOT.to_owned();
    source
        .push_str("mod panic{macro_rules!chosen{()=>{7u32};}pub(crate)use chosen as panic_2021;}");
    for index in 0..width {
        write!(
            source,
            "#[rustc_builtin_macro(core_panic,attributes())]macro_rules!route{index}{{()=>{{9u32}};}}"
        )?;
    }
    source.push_str("pub fn answer()->u32{");
    for index in 0..width {
        write!(
            source,
            "route{index}!(){}",
            if index + 1 == width { "" } else { ";" }
        )?;
    }
    source.push('}');
    Ok(crate::test_source::source_provider(
        &source,
        "metadata_queries",
    )?)
}
fn metadata_sites(report: &crate::resolve::expansion::Report) -> TestResult<Vec<Location>> {
    let mut found = Vec::new();
    for (file, unit) in report.package.units.iter().enumerate() {
        for node in &unit.configured.ast.nodes {
            if let Kind::Name(span, _) = node.kind
                && unit
                    .source
                    .text()
                    .get(span.start as usize..span.end as usize)
                    == Some("panic_2021")
            {
                let file = FileId(u32::try_from(file)?);
                let raw = report.package.syntax_context(file, span.start)?;
                let Some(context) = report.package.marks.outer(raw)? else {
                    continue;
                };
                let invocation = report
                    .invocations
                    .get(context.0 as usize - 1)
                    .filter(|invocation| invocation.context == context)
                    .ok_or("actual delegated invocation")?;
                if matches!(
                    report.definitions[invocation.definition].body,
                    crate::resolve::expansion::Body::Builtin(
                        crate::expand::builtin::Builtin::CorePanic
                    )
                ) {
                    // Only a real builtin-delegated token is a metadata use;
                    // the physical import alias has no such invocation owner.
                    found.push(Location { file, span });
                }
            }
        }
    }
    if found.is_empty() {
        return Err("missing actual delegated metadata sites".into());
    }
    Ok(found)
}

#[test]
fn builtin_metadata_only_frames_memoize_negative_answers_across_width_and_uses() -> TestResult {
    for width in [8_usize, 32, 128] {
        let report = metadata_report(width)?;
        let sites = metadata_sites(&report)?;
        assert_eq!(sites.len(), width);
        let mut session = report.macro_allowances();
        let feature = KnownFeature::PanicInternals.name();
        for site in &sites {
            assert!(!session.allows(*site, &feature)?);
        }
        let warm = owner_usage(&report)?.0;
        for _ in 0..8 {
            for site in &sites {
                assert!(!session.allows(*site, &feature)?);
            }
        }
        let after = owner_usage(&report)?.0;
        let memo = session.memo.as_ref().ok_or("metadata memo")?;
        assert_eq!(memo.headers.len(), width);
        assert_eq!(memo.frames.len(), width);
        assert!(after - warm <= width * 8 * 128);
        eprintln!(
            "allowance metadata-width={width} repeated=8 warm={} payload={}",
            after - warm,
            memo.payload()
        );
    }
    Ok(())
}

#[test]
fn constant_grammar_forwarding_counts_each_actual_retained_origin_link() -> TestResult {
    use std::fmt::Write;
    for depth in [8_usize, 32, 128] {
        let mut definitions = String::new();
        for layer in 0..depth {
            // A wrapper creates a genuine new expression capture instead of
            // merely forwarding one persistent opaque handle unchanged.
            let expression = if layer == 0 {
                "$e".to_owned()
            } else {
                format!("pass{}!(($e))", layer - 1)
            };
            write!(
                definitions,
                "macro_rules!pass{layer}{{($e:expr)=>{{{expression}}};}}"
            )?;
        }
        write!(
            definitions,
            "#[allow_internal_unstable(panic_internals)]macro_rules!start{{()=>{{pass{}!(probe())}};}}",
            depth - 1
        )?;
        let report = provider(&definitions, "start!()")?;
        let site = sites(&report, "probe")?[0];
        let mut session = report.macro_allowances();
        let feature = KnownFeature::PanicInternals.name();
        assert!(session.allows(site, &feature)?);
        let before = owner_usage(&report)?.0;
        let links_before = session.memo.as_ref().ok_or("memo")?.origin_links;
        let lookups_before = session.memo.as_ref().ok_or("memo")?.origin_lookups;
        for _ in 0..32 {
            assert!(session.allows(site, &feature)?);
        }
        let after = owner_usage(&report)?.0;
        let memo = session.memo.as_ref().ok_or("memo")?;
        assert_eq!(memo.headers.len(), 1);
        assert_eq!(memo.frames.len(), 1);
        assert!(memo.origin_links - links_before >= 32 * depth);
        eprintln!(
            "allowance constant-grammar-origin-depth={depth} repeated=32 charged-use-work={} links={} lookups={} headers={} frames={}",
            after - before,
            memo.origin_links - links_before,
            memo.origin_lookups - lookups_before,
            memo.headers.len(),
            memo.frames.len()
        );
        assert!(after - before >= 32);
    }
    Ok(())
}
