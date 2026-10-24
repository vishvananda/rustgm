//! Actual token provenance remains separate from complete feature membership.
use super::*;
use crate::{
    resolve::{
        bindings::{KnownFeature, SourceCore},
        unit::FileId,
    },
    syntax::ast::Kind,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = r#"#![recursion_limit="512"]#![feature(no_core,lang_items,rustc_attrs,allow_internal_unstable)]#![no_core]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
fn probe()->u32{7u32}
"#;

#[path = "allowance_identity.rs"]
mod identity;
#[path = "unsafe_allowances.rs"]
mod unsafe_lint;
#[path = "allowance_work.rs"]
mod work;

fn provider(definitions: &str, expression: &str) -> TestResult<Report> {
    Ok(crate::test_source::source_provider(
        &format!("{ROOT}{definitions}pub fn answer()->u32{{{expression}}}"),
        "allowance_queries",
    )?)
}
fn sites(report: &Report, spelling: &str) -> TestResult<Vec<Location>> {
    let mut found = Vec::new();
    for (file, unit) in report.package.units.iter().enumerate() {
        for (index, node) in unit.configured.ast.nodes.iter().enumerate() {
            if let Kind::Name(span, _) = node.kind
                && unit.configured.active[index]
                && unit
                    .source
                    .text()
                    .get(span.start as usize..span.end as usize)
                    == Some(spelling)
            {
                found.push(Location {
                    file: FileId(u32::try_from(file)?),
                    span,
                });
            }
        }
    }
    if found.is_empty() {
        return Err("missing actual source token sites".into());
    }
    Ok(found)
}
fn owner_usage(report: &Report) -> TestResult<(usize, usize, usize)> {
    Ok(report
        .package
        .source_core()
        .ok_or("provider")?
        .macros
        .usage())
}

#[test]
fn actual_literals_and_captured_tokens_keep_distinct_outer_permissions() -> TestResult {
    for (definitions, expression, expected) in [
        (
            "#[allow_internal_unstable(panic_internals)]macro_rules!route{()=>{probe()};}",
            "route!()",
            true,
        ),
        (
            "#[allow_internal_unstable(panic_internals)]macro_rules!route{($e:expr)=>{$e};}",
            "route!(probe())",
            false,
        ),
        (
            "#[allow_internal_unstable(panic_internals)]macro_rules!route{($p:path)=>{$p()};}",
            "route!(probe)",
            false,
        ),
        (
            "#[allow_internal_unstable(panic_internals)]macro_rules!route{($($t:tt)*)=>{$($t)*};}",
            "route!(probe())",
            false,
        ),
        (
            "macro_rules!inner{()=>{probe()};}#[allow_internal_unstable(panic_internals)]macro_rules!outer{()=>{inner!()};}",
            "outer!()",
            false,
        ),
        (
            "#[allow_internal_unstable(panic_internals)]macro_rules!inner{()=>{probe()};}macro_rules!outer{()=>{inner!()};}",
            "outer!()",
            true,
        ),
        (
            "macro_rules!inner{($e:expr)=>{$e};}#[allow_internal_unstable(panic_internals)]macro_rules!outer{()=>{inner!(probe())};}",
            "outer!()",
            true,
        ),
    ] {
        let report = provider(definitions, expression)?;
        let mut session = report.macro_allowances();
        let feature = KnownFeature::PanicInternals.name();
        for site in sites(&report, "probe")? {
            assert_eq!(session.allows(site, &feature)?, expected, "{definitions}");
        }
    }
    Ok(())
}

#[test]
fn complete_empty_duplicate_raw_and_nfc_lists_are_data_not_global_grants() -> TestResult {
    let definitions = "#[allow_internal_unstable(fixture_caf\u{e9},r#fixture_cafe\u{301})]\n\
        #[allow_internal_unstable(fixture_a\u{301}\u{323},fixture_a\u{323}\u{301})]\n\
        #[allow_internal_unstable()]macro_rules!route{()=>{probe()};}";
    let report = provider(definitions, "route!()")?;
    let invocation = report.invocations.first().ok_or("actual invocation")?;
    let header = report.definitions[invocation.definition]
        .header
        .as_ref()
        .ok_or("owned header")?;
    assert_eq!(header.feature_count(), 2);
    assert!(!report.definitions[invocation.definition].edition_panic);
    let mut session = report.macro_allowances();
    for query in [
        "fixture_caf\u{e9}",
        "r#fixture_cafe\u{301}",
        "fixture_a\u{301}\u{323}",
        "fixture_a\u{323}\u{301}",
    ] {
        let feature = FeatureName::new(query)?;
        for site in sites(&report, "probe")? {
            assert!(session.allows(site, &feature)?);
        }
    }
    let ordinary = provider(definitions, "probe()")?;
    let mut plain_session = ordinary.macro_allowances();
    for site in sites(&ordinary, "probe")? {
        assert!(!plain_session.allows(site, &FeatureName::new("fixture_caf\u{e9}")?)?);
    }
    assert!(plain_session.memo.is_none());
    let inert = provider(
        "#[allow_internal_unstable(const_trait_impl)]macro_rules!route{()=>{probe()};}",
        "route!()",
    )?;
    assert!(inert.package.library_policy().is_none());
    assert!(
        !inert
            .package
            .source_features()
            .ok_or("source features")?
            .has(crate::resolve::bindings::policy::Feature::ConstTrait)
    );
    let mut inert_session = inert.macro_allowances();
    let symbol = FeatureName::new("const_trait_impl")?;
    for site in sites(&inert, "probe")? {
        assert!(inert_session.allows(site, &symbol)?);
    }
    Ok(())
}

#[test]
fn public_symbols_validate_once_and_ordinary_sessions_allocate_no_memo() -> TestResult {
    for invalid in ["", "_", "r#", "with space", "7feature", "path::feature"] {
        assert!(FeatureName::new(invalid).is_err());
    }
    assert_eq!(
        FeatureName::new("r#fixture_cafe\u{301}")?.spelling(),
        "fixture_caf\u{e9}"
    );
    let mut report =
        crate::test_source::resolve("fn probe()->u32{7}fn answer()->u32{probe()}", true)?;
    let symbol = report.package.names.intern("fixture_cafe\u{301}")?;
    assert_eq!(report.feature_name(symbol)?.spelling(), "fixture_caf\u{e9}");
    assert!(
        report
            .feature_name(crate::support::names::NameId(u32::MAX))
            .is_err()
    );
    let invalid = report.package.names.intern_exact("not a feature name")?;
    assert!(report.feature_name(invalid).is_err());
    let policy = crate::resolve::stability::Policy {
        feature: "invalid feature".into(),
        level: crate::resolve::stability::Level::Stable(crate::resolve::stability::Since::Current),
        origin: crate::resolve::unit::NodeKey {
            file: FileId(0),
            node: crate::syntax::ast::NodeId::new(0),
        },
    };
    assert!(policy.feature_name().is_err());
    let mut session = report.macro_allowances();
    for site in sites(&report, "probe")? {
        assert!(!session.allows(site, &KnownFeature::PanicInternals.name())?);
    }
    assert!(session.memo.is_none());
    eprintln!(
        "allowance layouts session={} memo={} definition={} source_core={}",
        size_of::<MacroAllowanceSession<'_>>(),
        size_of::<Memo>(),
        size_of::<super::super::Definition>(),
        size_of::<SourceCore>()
    );
    Ok(())
}
