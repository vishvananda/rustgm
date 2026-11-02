//! Optimization preferences cannot supply body, safety, const or transport authority.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/source-optimize/{name}.rs"
    ))?)
}
fn analyze(source: &str) -> TestResult<(Report, Declarations)> {
    let report = test_source::resolve(source, true)?;
    let facts = pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
fn fully_checked(source: &str) -> TestResult {
    let report = test_source::resolve(source, true)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}
fn function(report: &Report, name: &str) -> TestResult<DefId> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(i, row)| {
            (row.kind == DefKind::Function && report.package.names.get(row.name) == name)
                .then_some(i)
        })
        .map(|i| u32::try_from(i).map(DefId))
        .transpose()?
        .ok_or_else(|| "missing actual function".into())
}
#[test]
fn source_forms_targets_features_and_inline_conflicts_match_independent_rust() -> TestResult {
    for name in [
        "cfg-removed",
        "closure",
        "default-method",
        "impl-method",
        "inherent-method",
        "macro-allowed-meta",
        "none-inline-never-0",
        "none-inline-never-1",
        "none",
        "raw-word",
        "size-inline-always-0",
        "size-inline-always-1",
        "size",
        "speed-inline-0",
        "speed-inline-1",
        "speed",
        "trailing",
    ] {
        analyze(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "alias",
        "borrow-denied",
        "closure-missing-expression-gate",
        "const-denied",
        "delimiter",
        "duplicate-size",
        "duplicate-speed",
        "empty",
        "foreign",
        "impl",
        "key-value",
        "literal",
        "macro-forwarded-item",
        "macro-no-allowance",
        "missing-gate",
        "module",
        "multiple",
        "nested",
        "none-inline-0",
        "none-inline-1",
        "none-inline-always-0",
        "none-inline-always-1",
        "qualified",
        "required-method",
        "struct",
        "unknown",
        "unsafe-denied",
        "unsafe-wrapper",
        "value",
        "word",
    ] {
        let error = fully_checked(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(!error.contains("not in path scope"), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn source_intent_preserves_all_preferences_and_absence_needs_a_completed_producer() -> TestResult {
    for (name, preference) in [
        ("size", Optimization::Size),
        ("speed", Optimization::Speed),
        ("none", Optimization::Disabled),
    ] {
        let (report, facts) = analyze(&fixture(name)?)?;
        let id = function(&report, "f")?;
        let ContractKnowledge::KnownPresent(view) = facts.source_optimization(&report, id)? else {
            return Err("missing actual optimization preference".into());
        };
        assert_eq!(view.preference, preference);
        assert_eq!(
            Some(view.source.declaration),
            report.resolution.definitions[id.0 as usize].node
        );
    }
    let input =
        String::from("#![feature(optimize_attribute)]#[optimize(size)]fn marked(){}fn plain(){}");
    let (mut report, facts) = analyze(&input)?;
    let plain = function(&report, "plain")?;
    assert!(matches!(
        facts.source_optimization(&report, plain)?,
        ContractKnowledge::Unknown
    ));
    report.resolution.definitions[plain.0 as usize].node = None;
    assert!(matches!(
        facts.source_optimization(&report, plain)?,
        ContractKnowledge::Unknown
    ));
    let source = String::from("#![feature(optimize_attribute)]")
        + &test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
        + "fn plain(){}";
    let report = test_source::source_provider(&source, "optimization_absence")?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let id = function(&report, "plain")?;
    assert!(matches!(
        facts.source_optimization(&report, id)?,
        ContractKnowledge::KnownAbsent
    ));
    facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("contracts")?
        .headers
        .as_deref_mut()
        .ok_or("ledger")?
        .local
        .comptime_complete = false;
    assert!(matches!(
        facts.source_optimization(&report, id)?,
        ContractKnowledge::Unknown
    ));
    Ok(())
}
#[test]
fn changed_owner_image_item_and_attribute_are_rejected() -> TestResult {
    let input = fixture("size")?;
    let (report, facts) = analyze(&input)?;
    let id = function(&report, "f")?;
    let (fresh, _) = analyze(&input)?;
    assert!(facts.source_optimization(&fresh, id).is_err());
    let (mut report, facts) = analyze(&input)?;
    let key = report.resolution.definitions[id.0 as usize]
        .node
        .ok_or("source node")?;
    let Kind::Item(index) = report
        .package
        .unit(key.file)
        .configured
        .ast
        .node(key.node)
        .kind
    else {
        return Err("item".into());
    };
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .items[index as usize]
        .name = None;
    assert!(facts.source_optimization(&report, id).is_err());
    let (mut report, facts) = analyze(&input)?;
    let owner = header(&facts, key).ok_or("source header")?.owners[0].key;
    let slot = report.package.unit(key.file).owner_indices[owner.node.get() as usize]
        .ok_or("owner slot")?
        .get() as usize
        - 1;
    report.package.units[key.file.0 as usize].configured.owners[slot].enabled = false;
    assert!(facts.source_optimization(&report, id).is_err());
    Ok(())
}
#[test]
fn unsupported_metadata_profiles_preserve_the_optimizer_boundary() -> TestResult {
    let (report, mut facts) = analyze(&fixture("size")?)?;
    let error = crate::sema::crates::export(
        &report,
        &mut facts,
        crate::support::crates::CrateIdentity::new("optimize", "owned")?,
    )
    .err()
    .ok_or("optimization intent erased by metadata")?
    .to_string();
    assert!(
        error.contains("optimization intent requires authenticated metadata transport"),
        "{error}"
    );
    assert!(facts.require_optimization_transport(&report).is_err());
    Ok(())
}
#[test]
fn retention_is_linear_queries_retain_nothing_and_budgets_are_sticky() -> TestResult {
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        use std::fmt::Write;
        let mut source = String::from("#![feature(optimize_attribute)]");
        for index in 0..width {
            write!(source, "#[optimize(size)]fn f{index}()->u32{{7}}")?;
        }
        let (report, facts) = analyze(&source)?;
        let before = super::super::source_contract_costs(&facts);
        for index in 0..width {
            let id = function(&report, &format!("f{index}"))?;
            assert!(matches!(
                facts.source_optimization(&report, id)?,
                ContractKnowledge::KnownPresent(_)
            ));
        }
        let after = super::super::source_contract_costs(&facts);
        assert_eq!(after.1, before.1);
        observations.push(before);
        eprintln!(
            "optimize width={width} work={} bytes={}",
            before.0, before.1
        );
    }
    assert_eq!(
        observations[2].0 - observations[1].0,
        4 * (observations[1].0 - observations[0].0)
    );
    assert_eq!(
        observations[2].1 - observations[1].1,
        4 * (observations[1].1 - observations[0].1)
    );
    let (report, facts) = analyze(&fixture("size")?)?;
    let id = function(&report, "f")?;
    facts
        .must_use_contracts
        .as_deref()
        .ok_or("contracts")?
        .headers
        .as_deref()
        .ok_or("ledger")?
        .local
        .steps
        .set(crate::support::crates::MAX_COMPONENTS);
    assert!(facts.source_optimization(&report, id).is_err());
    assert!(facts.source_optimization(&report, id).is_err());
    assert_eq!(size_of::<Header>(), 80);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn annotated_ordinary_bodies_lower_without_exporting_unencoded_policy() -> TestResult {
    let input = fixture("native-provider")?;
    let report = test_source::resolve(&input, true)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/source-optimize-v1/candidate-native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
