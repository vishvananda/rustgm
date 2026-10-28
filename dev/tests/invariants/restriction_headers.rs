//! Parsed gate demand, literal-origin authority and exact module implementation domains.
use super::*;
use crate::test_source;
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn checked(source: &str) -> Result<(), String> {
    let report = test_source::resolve(source, true)?;
    crate::pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    Ok(())
}
fn fixture(name: &str) -> std::io::Result<String> {
    test_source::fixture(&format!(
        "pa31/tests/spec/impl-restriction-source/{name}.rs"
    ))
}

#[test]
fn parsed_restrictions_keep_feature_origins_disabled_headers_and_ordinary_layouts() -> TestResult {
    for name in [
        "unexpanded-macro-opaque",
        "disabled-call-opaque",
        "literal-origin-allowance",
    ] {
        checked(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "missing-feature-active",
        "missing-feature-disabled-owner",
        "missing-feature-disabled-ancestor",
        "captured-origin-denied",
    ] {
        let error = checked(&fixture(name)?)
            .err()
            .ok_or("ungated restriction accepted")?;
        assert!(
            error.contains("implementation restrictions require"),
            "{name}: {error}"
        );
    }
    for count in [0, 8, 64, 128] {
        let mut source = String::new();
        for i in 0..count {
            use std::fmt::Write;
            writeln!(&mut source, "fn ordinary_{i}(){{}}")?;
        }
        let report = test_source::resolve(&source, true)?;
        let work = headers(&report.package, |_, _| {
            Err(Error::unlocated("ordinary header queried allowance"))
        })?;
        assert_eq!(work.items, count);
        assert_eq!(work.queries, 0);
        assert!(report.package.source.is_none());
        let report =
            test_source::resolve(&format!("#![feature(impl_restriction)] {source}"), true)?;
        assert_eq!(
            headers(&report.package, |_, _| Err(Error::unlocated(
                "global request queried allowance"
            )))?,
            HeaderWork::default()
        );
        assert!(!report.package.uses_source_bindings());
        assert!(report.package.source_core().is_none());
        assert!(report.package.source_client().is_none());
        assert!(report.package.source_features().is_none());
        assert!(report.resolution.source_bindings.is_none());
        let context = report
            .package
            .macro_context()?
            .ok_or("actual cold request owner")?;
        let usage = context.contracts().usage();
        assert_eq!(usage.2, 0);
        assert!(usage.1 >= size_of::<crate::resolve::bindings::Source>());
        assert_eq!(
            headers(&report.package, |_, _| Ok(false))?,
            HeaderWork::default()
        );
        assert_eq!(context.contracts().usage(), usage);
    }
    let mut authenticated =
        test_source::resolve("#![feature(impl_restriction)] fn ordinary(){}", true)?;
    assert!(authenticated.package.requested_impl_restriction()?);
    let text = authenticated.package.units[0]
        .source
        .original()
        .as_bytes()
        .to_vec();
    authenticated.package.units[0].source = crate::support::source::SourceFile::new(text)?;
    assert!(authenticated.package.requested_impl_restriction().is_err());
    let text = fixture("const-unsafe-auto-grammar")?;
    let syntax = crate::syntax::expression::parse_crate(&text)
        .map_err(|error| format!("{}: {}", error.offset, error.message))?;
    assert_eq!(
        syntax
            .items
            .iter()
            .filter(|item| matches!(
                item.kind,
                ItemKind::Trait {
                    restriction: Some(_),
                    ..
                }
            ))
            .count(),
        2
    );
    observe_layouts();
    Ok(())
}

#[test]
fn exact_trait_domains_resolve_aliases_and_charge_actual_ancestry_before_publication() -> TestResult
{
    bootstrap_identity_opponents()?;
    for name in [
        "self-and-descendant",
        "super-domain",
        "crate-domain",
        "forward-ancestor-alias",
    ] {
        checked(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for (name, text) in [
        ("self-outside-denied", "trait cannot be implemented outside"),
        ("child-domain-denied", "must name an ancestor module"),
        ("nonmodule-denied", "must name an actual module"),
    ] {
        let error = checked(&fixture(name)?)
            .err()
            .ok_or("invalid domain accepted")?;
        assert!(error.contains(text), "{name}: {error}");
    }
    checked(
        "#![feature(impl_restriction)] mod owner {pub impl(self) trait T{}} struct S; impl S {fn identity(&self)->&Self{self}}",
    )?;
    assert!(
        test_source::resolve(
            "#![feature(impl_restriction)] mod owner {pub impl(self) trait T{} fn wrong(_:self){}}",
            true
        )
        .is_err()
    );
    assert!(
        test_source::resolve(
            "#![feature(impl_restriction)] pub impl(in Self) trait T{}",
            true
        )
        .is_err()
    );
    let source = "#![feature(impl_restriction)] mod outer { pub impl(self) trait T{} mod child {impl super::T for u8{}} } mod unrelated {}";
    let report = test_source::resolve(source, true)?;
    let root = ModuleId(0);
    let child = report
        .package
        .modules
        .iter()
        .position(|m| report.package.names.get(m.name) == "child")
        .ok_or("child")?;
    let child = ModuleId(u32::try_from(child)?);
    let mut steps = 0;
    assert!(within(&report.package, child, root, || {
        steps += 1;
        Ok(())
    })?);
    assert_eq!(steps, 3);
    let mut failed = 0;
    let error = within(&report.package, child, root, || {
        failed += 1;
        Err("attempt budget exhausted")
    })
    .err()
    .ok_or("failed query granted permission")?;
    assert_eq!(error, "attempt budget exhausted");
    assert_eq!(failed, 1);
    let trait_id = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::Trait && report.package.names.get(d.name) == "T")
        .ok_or("T")?;
    let trait_id = DefId(u32::try_from(trait_id)?);
    let clause = clause(&report, trait_id)?.ok_or("clause")?;
    let Kind::ImplRestriction { path } = report
        .package
        .unit(clause.file)
        .configured
        .ast
        .node(clause.node)
        .kind
    else {
        return Err("typed clause".into());
    };
    let path = NodeKey {
        file: clause.file,
        node: path,
    };
    let analysis = crate::pipeline::analyze_source(&report)?;
    let target = analysis
        .facts
        .node(path)
        .and_then(|facts| facts.reference)
        .ok_or("retained module")?;
    assert!(matches!(target, Target::Definition(_)));
    // Keep crate-inner attributes before all items while relocating the owner graph.
    let copied = test_source::resolve(
        &source.replacen("mod outer", "mod Other {} mod outer", 1),
        true,
    )
    .map_err(|error| format!("relocated owner graph: {error}"))?;
    crate::pipeline::analyze_source(&copied)?;
    // Reaching the same valid graph after a failed independent query remains valid.
    crate::pipeline::analyze_source(&report)?;
    assert!(module(&report.package, &report.resolution, trait_id).is_err());
    Ok(())
}

fn observe_layouts() {
    assert_eq!(
        crate::resolve::bindings::policy::Feature::ImplRestriction as u8,
        18
    );
    assert_eq!(size_of::<crate::syntax::item::ItemKind>(), 28);
    assert_eq!(size_of::<crate::syntax::item::Item>(), 64);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::resolve::bindings::policy::Features>(), 8);
    assert_eq!(
        size_of::<crate::resolve::bindings::policy::MacroFeatures>(),
        1
    );
    assert_eq!(size_of::<crate::resolve::bindings::Source>(), 184);
    assert_eq!(size_of::<Package>(), 256);
    // Inline import work adds one lineage pointer and nine usize counters.
    assert_eq!(size_of::<Report>(), 3096);
    assert_eq!(size_of::<crate::sema::associated::OwnerKind>(), 20);
    assert_eq!(size_of::<crate::sema::associated::Owner>(), 32);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    println!(
        "restriction layouts Syntax={} Configured={} Unit={} Source={} SourceCore={} Package={} Report={} ItemKind={} Item={} Node={}",
        size_of::<crate::syntax::ast::Syntax<crate::literal::storage::StoredValue>>(),
        size_of::<
            crate::expand::crate_syntax::ConfiguredSyntax<crate::literal::storage::StoredValue>,
        >(),
        size_of::<crate::resolve::unit::Unit>(),
        size_of::<crate::resolve::bindings::Source>(),
        size_of::<crate::resolve::bindings::SourceCore>(),
        size_of::<Package>(),
        size_of::<Report>(),
        size_of::<crate::syntax::item::ItemKind>(),
        size_of::<crate::syntax::item::Item>(),
        size_of::<crate::syntax::ast::Node>()
    );
}

fn bootstrap_identity_opponents() -> TestResult {
    // Bootstrap Clone/Copy impls have a separate preexisting unsupported boundary.
    checked("struct S; unsafe impl Send for S{} unsafe impl Sync for S{}")
        .map_err(|error| format!("actual bootstrap Send/Sync owner: {error}"))?;
    let mut counterfeit = test_source::resolve("trait Clone{}", true)
        .map_err(|error| format!("authored shadow trait owner: {error}"))?;
    let fake = counterfeit
        .resolution
        .definitions
        .iter()
        .position(|definition| {
            definition.kind == DefKind::Trait
                && counterfeit.package.names.get(definition.name) == "Clone"
        })
        .ok_or("actual shadow trait")?;
    counterfeit.resolution.definitions[fake].kind = DefKind::BuiltinTrait;
    counterfeit.resolution.definitions[fake].node = None;
    assert!(
        counterfeit
            .resolution
            .builtin_traits
            .identify(DefId(u32::try_from(fake)?))
            .is_none()
    );
    assert!(clause(&counterfeit, DefId(u32::try_from(fake)?)).is_err());
    Ok(())
}
