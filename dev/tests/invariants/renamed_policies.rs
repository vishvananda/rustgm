//! Independently qualified former names are diagnostics, never enabling aliases.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Case {
    name: String,
    path: String,
    accepted: bool,
    provider: bool,
    diagnostic: String,
    external: bool,
}
fn cases() -> TestResult<Vec<Case>> {
    test_source::fixture("pa31/tests/spec/renamed-features/candidate-cases-v1.tsv")?
        .lines()
        .map(|line| {
            let [name, path, status, provider, diagnostic, external]: [_; 6] = line
                .split('\t')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| "six case columns")?;
            Ok(Case {
                name: name.to_owned(),
                path: path.to_owned(),
                accepted: status == "0",
                provider: provider == "True",
                diagnostic: diagnostic.to_owned(),
                external: !external.is_empty(),
            })
        })
        .collect()
}
fn source(name: &str) -> TestResult<String> {
    let case = cases()?
        .into_iter()
        .find(|case| case.name == name)
        .ok_or("source case")?;
    Ok(test_source::fixture(&case.path)?)
}
#[test]
fn independently_observed_requests_keep_canonical_first_policy_and_collision_rules() -> TestResult {
    let cases = cases()?;
    assert_eq!(cases.len(), 44);
    let mut observed = 0;
    for case in cases.into_iter().filter(|case| !case.external) {
        observed += 1;
        let text = test_source::fixture(&case.path)?;
        let result = if case.provider {
            test_source::source_provider(&text, "renamed_features")
        } else {
            test_source::resolve(&text, true)
        };
        if case.accepted {
            let report = result.map_err(|error| format!("{}: {error}", case.name))?;
            pipeline::analyze_source(&report)?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("invalid {} accepted", case.name))?;
            if case.diagnostic == "renamed" {
                assert!(error.contains("renamed"), "{}: {error}", case.name);
            }
            if case.diagnostic == "unknown feature" {
                assert!(
                    error.contains("undefined library feature"),
                    "{}: {error}",
                    case.name
                );
            }
        }
    }
    assert_eq!(observed, 41);
    Ok(())
}
#[test]
fn exact_relationships_preserve_regular_const_owners_and_unrestricted_old_strings() -> TestResult {
    for (name, kind, count) in [
        ("declaration", PolicyKind::Regular, 1),
        ("constant-declaration", PolicyKind::Const, 1),
        ("alias-empty", PolicyKind::Regular, 1),
        ("alias-nonidentifier", PolicyKind::Regular, 1),
        ("alias-unicode", PolicyKind::Regular, 1),
        ("both-relations", PolicyKind::Regular, 2),
    ] {
        let report = test_source::resolve(&source(name)?, true)?;
        let library = report.package.library_policy().ok_or("library")?;
        let inventory = library.inventory().ok_or("inventory")?;
        assert_eq!(inventory.relationships.len(), count, "{name}");
        for edge in &inventory.relationships {
            let policy = &inventory.policies[edge.policy.get() - 1];
            assert_eq!(policy.kind, kind);
            if edge.kind == relationships::Kind::OldName {
                assert!(
                    !inventory
                        .policies
                        .iter()
                        .any(|p| p.feature.as_ref() == edge.name.as_ref())
                );
            }
        }
        assert!(library.predicate_requests()?.is_empty());
    }
    let old = test_source::fixture("pa31/tests/spec/stability-implications/renamed-required.rs")?;
    let report = test_source::resolve(&old, true)?;
    pipeline::analyze_source(&report)?;
    assert_eq!(
        report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or("existing required source")?
            .relationships
            .len(),
        1
    );
    assert_eq!(size_of::<Library>(), 152);
    assert_eq!(size_of::<Inventory>(), 112);
    assert_eq!(size_of::<Policy>(), 56);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    Ok(())
}
#[test]
fn ordinary_body_lowers_actual_native_value_and_old_metadata_fences_every_relationship()
-> TestResult {
    let report = test_source::resolve(&source("ordinary-native")?, true)?;
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
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
    );
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/renamed-features-v1/native");
    std::fs::create_dir_all(&folder)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("ordinary.lowir"))?,
    )?;
    for name in ["source-provider", "both-relations"] {
        let provider = if name == "source-provider" {
            source(name)?
        } else {
            test_source::fixture("pa31/tests/spec/feature-predicate-metadata-v4/provider.rs")?
                + "\n#[unstable(feature=\"implied_child\",issue=\"none\",implied_by=\"identity_api\",old_name=\"old_child\")]pub fn child()->u8{7}\n"
        };
        let error = crate::module_packets::exported_profile(
            &provider,
            crate::lower::instances::SourcePublication::ModuleFeaturePredicates,
            "renamed_policy",
        )
        .err()
        .ok_or("metadata silently dropped a feature relationship")?
        .to_string();
        assert!(
            error.contains("complete wider policy transport"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn growing_alias_and_request_sets_pay_linear_collection_and_no_new_hot_storage() -> TestResult {
    let mut observations = Vec::new();
    for n in [16, 64, 256] {
        use std::fmt::Write;
        let mut source = String::from(
            "#![feature(staged_api)]#![stable(feature=\"root_api\",since=\"1.0.0\")]#![feature(",
        );
        for i in 0..n {
            write!(source, "current_{i},")?;
        }
        source.push_str(")]\n");
        for i in 0..n {
            writeln!(
                source,
                "#[unstable(feature=\"current_{i}\",issue=\"none\",old_name=\"former_{i}\")]pub fn f{i}()->u8{{7}}"
            )?;
        }
        let report = test_source::resolve(&source, true)?;
        let inventory = report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or("inventory")?;
        assert_eq!(inventory.relationships.len(), n);
        assert!(
            inventory.steps() < 256 * n + 4096,
            "{n}: {}",
            inventory.steps()
        );
        observations.push(inventory.steps());
    }
    let first = observations[1]
        .checked_sub(observations[0])
        .ok_or("first interval")?;
    let second = observations[2]
        .checked_sub(observations[1])
        .ok_or("second interval")?;
    assert!(first > 0 && second <= 5 * first, "{observations:?}");
    Ok(())
}
#[test]
fn failed_relationship_growth_keeps_both_link_kinds_and_policies_unpublished() {
    let mut draft = Draft {
        bytes: 64 * 1024 * 1024,
        ..Draft::default()
    };
    for _ in 0..2 {
        assert!(draft.prepay_relationships(2).is_err());
        assert!(
            draft.relationships.is_empty()
                && draft.policies.is_empty()
                && draft.attributes.is_empty()
        );
        assert!(draft.bytes > 64 * 1024 * 1024);
    }
}
