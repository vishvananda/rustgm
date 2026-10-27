//! Local allowance ownership keeps ordinary lookup and actual token provenance.
use super::*;
use crate::{
    expand::stream::Location,
    resolve::{
        bindings::FeatureName,
        expansion::Report,
        model::{BuiltinTrait, DefKind},
    },
    support::source::SourceFile,
    syntax::ast::Kind,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn local(definition: &str, expression: &str) -> TestResult<Report> {
    Ok(crate::test_source::resolve(
        &format!(
            "#![feature(allow_internal_unstable)]fn probe()->u32{{7}}\
             {definition}pub fn answer()->u32{{{expression}}}"
        ),
        true,
    )?)
}
fn sites(report: &Report) -> TestResult<Vec<Location>> {
    let mut sites = Vec::new();
    for (file, unit) in report.package.units.iter().enumerate() {
        for (index, node) in unit.configured.ast.nodes.iter().enumerate() {
            if let Kind::Name(span, _) = node.kind
                && unit.configured.active[index]
                && unit
                    .source
                    .text()
                    .get(span.start as usize..span.end as usize)
                    == Some("probe")
            {
                sites.push(Location {
                    file: FileId(u32::try_from(file)?),
                    span,
                });
            }
        }
    }
    if sites.is_empty() {
        return Err("missing actual local use token".into());
    }
    Ok(sites)
}

fn demanded_prelude_lookup() -> TestResult {
    let body = "fn copied<T:Copy>(value:T)->T{value}\
                pub fn demanded()->u32{copied(7u32)}";
    for requested in [false, true] {
        let source = if requested {
            format!("#![feature(allow_internal_unstable)]{body}")
        } else {
            body.to_owned()
        };
        let report = crate::test_source::resolve(&source, true)?;
        assert_eq!(report.package.macro_context()?.is_some(), requested);
        assert!(report.package.modules[0].implicit_prelude);
        let copy = report
            .resolution
            .builtin_traits
            .get(BuiltinTrait::Copy)
            .ok_or("ordinary prelude Copy was not resolved")?;
        let declaration = &report.resolution.definitions[copy.0 as usize];
        assert_eq!(declaration.kind, DefKind::BuiltinTrait);
        assert_eq!(report.package.names.get(declaration.name), "Copy");
        assert!(declaration.node.is_none());
        crate::pipeline::analyze_source(&report)?;
    }
    Ok(())
}

#[test]
fn local_owner_keeps_ordinary_prelude_and_no_source_roles_or_features() -> TestResult {
    let ordinary = crate::test_source::resolve("fn f()->u32{7}", true)?;
    assert!(ordinary.package.source.is_none());
    assert!(ordinary.package.macro_context()?.is_none());
    let unused = local("", "probe()")?;
    assert_eq!(
        unused
            .package
            .macro_context()?
            .ok_or("local owner")?
            .contracts()
            .usage()
            .2,
        0
    );
    let report = local("macro_rules! route{()=>{probe()};}", "route!()")?;
    assert!(matches!(
        report.package.source.as_deref(),
        Some(Source::LocalMacros(_))
    ));
    assert!(!report.package.uses_source_bindings());
    assert!(report.package.modules[0].implicit_prelude);
    // Bootstrap metadata is lazy: neither input has demanded a trait name yet.
    assert!(ordinary.resolution.builtin_traits.bootstrap_absent());
    assert_eq!(
        report.resolution.builtin_traits.bootstrap_absent(),
        ordinary.resolution.builtin_traits.bootstrap_absent()
    );
    demanded_prelude_lookup()?;
    assert!(report.package.source_core().is_none());
    assert!(report.package.source_client().is_none());
    assert!(report.package.source_features().is_none());
    assert!(report.resolution.source_bindings.is_none());
    assert!(!report.package.permits_source_negative_impls());
    assert!(!report.package.permits_source_rust_call());
    assert_eq!(size_of::<Option<Box<Source>>>(), size_of::<usize>());
    assert_eq!(size_of::<Package>(), 256);
    assert_eq!(size_of::<policy::Features>(), 8);
    assert_eq!(size_of::<policy::MacroFeatures>(), 1);
    eprintln!(
        "cfg cold layouts Features={} CfgGate={} CfgUse={} ConfiguredUse={}",
        size_of::<policy::Features>(),
        size_of::<crate::expand::config::CfgGate>(),
        size_of::<crate::expand::config::CfgUse>(),
        size_of::<crate::expand::configured::ConfiguredUse>()
    );
    assert!(size_of::<LocalMacros>() <= size_of::<SourceCore>());
    eprintln!(
        "local macro layouts Source={} Local={} Contracts={} Core={} Client={} Package={} Report={} Definition={} Session={} Context={}",
        size_of::<Source>(),
        size_of::<LocalMacros>(),
        size_of::<panic_contracts::Contracts>(),
        size_of::<SourceCore>(),
        size_of::<super::super::SourceClient>(),
        size_of::<Package>(),
        size_of::<Report>(),
        size_of::<crate::resolve::expansion::Definition>(),
        size_of::<crate::resolve::expansion::MacroAllowanceSession<'_>>(),
        size_of::<MacroContext<'_>>()
    );
    Ok(())
}

#[test]
fn ordinary_literals_captures_plain_frames_and_failed_warm_reads_stay_distinct() -> TestResult {
    let feature = FeatureName::new("fixture_gate")?;
    for (definition, expression, expected) in [
        (
            "#[allow_internal_unstable(fixture_gate)]macro_rules! route{()=>{probe()};}",
            "route!()",
            true,
        ),
        (
            "#[allow_internal_unstable(fixture_gate)]macro_rules! route{($e:expr)=>{$e};}",
            "route!(probe())",
            false,
        ),
        (
            "macro_rules! inner{()=>{probe()};}#[allow_internal_unstable(fixture_gate)]macro_rules! outer{()=>{inner!()};}",
            "outer!()",
            false,
        ),
        (
            "#[allow_internal_unstable(fixture_gate)]macro_rules! inner{()=>{probe()};}macro_rules! outer{()=>{inner!()};}",
            "outer!()",
            true,
        ),
    ] {
        let report = local(definition, expression)?;
        let context = report.package.macro_context()?.ok_or("local owner")?;
        let mut session = report.macro_allowances();
        let uses = sites(&report)?;
        for &site in &uses {
            assert_eq!(session.allows(site, &feature)?, expected);
            let before = context.contracts().usage().0;
            for _ in 0..32 {
                assert_eq!(session.allows(site, &feature)?, expected);
            }
            assert!(context.contracts().usage().0 - before <= 128 * 32);
        }
        assert!(context.contracts().charge(1_000_001, 0).is_err());
        for site in uses {
            assert!(session.allows(site, &feature).is_err());
        }
    }
    Ok(())
}

#[test]
fn ordinary_root_incarnation_and_provider_only_role_guards_remain_required() -> TestResult {
    let claimed = "#![feature(allow_internal_unstable,rustc_attrs)]#[rustc_builtin_macro]macro_rules!line{()=>{}}";
    let diagnostic = crate::test_source::resolve(claimed, true)
        .err()
        .ok_or("local role admitted")?;
    assert!(diagnostic.contains("explicit source-core provider"));
    assert!(
        crate::test_source::resolve(
            "#[allow_internal_unstable(fixture_gate)]macro_rules!route{()=>{7}}",
            true
        )
        .is_err()
    );
    let mut report = local(
        "#[allow_internal_unstable(fixture_gate)]macro_rules!route{()=>{probe()};}",
        "route!()",
    )?;
    let feature = FeatureName::new("fixture_gate")?;
    let uses = sites(&report)?;
    for &site in &uses {
        assert!(report.macro_allowances().allows(site, &feature)?);
    }
    let unit = &mut report.package.units[0];
    let text = unit.source.text().to_owned();
    let physical = unit.source.physical_len();
    let mut replacement = SourceFile::new(unit.source.original().as_bytes().to_vec())?;
    if text.len() > physical {
        replacement.append(&text[physical + 1..])?;
    }
    assert_eq!(replacement.text(), text);
    unit.source = replacement;
    assert!(report.package.macro_context().is_err());
    for site in uses {
        assert!(report.macro_allowances().allows(site, &feature).is_err());
    }
    Ok(())
}
