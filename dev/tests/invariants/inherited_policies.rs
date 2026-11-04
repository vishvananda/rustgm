//! Default bodies retain exact policy axes while ordinary local execution stays ordinary.
use super::*;
use crate::{pipeline, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct Case {
    name: String,
    path: String,
    accepted: bool,
    provider: bool,
    external: bool,
}
fn cases() -> TestResult<Vec<Case>> {
    test_source::fixture("pa31/tests/spec/default-body-stability/candidate-cases-v1.tsv")?
        .lines()
        .map(|line| {
            let [name, path, status, provider, external]: [_; 5] = line
                .split('\t')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| "five case columns")?;
            Ok(Case {
                name: name.to_owned(),
                path: path.to_owned(),
                accepted: status == "0",
                provider: provider == "True",
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
fn analyze(case: &Case) -> TestResult<(crate::resolve::expansion::Report, pipeline::Analysis)> {
    let text = test_source::fixture(&case.path)?;
    let report = if case.provider {
        test_source::source_provider(&text, "default_bodies")?
    } else {
        test_source::resolve(&text, true)?
    };
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
#[test]
fn local_default_body_sources_match_independent_declaration_and_execution_outcomes() -> TestResult {
    let cases = cases()?;
    assert_eq!(cases.len(), 48);
    let mut count = 0;
    for case in cases.into_iter().filter(|case| !case.external) {
        count += 1;
        let result = analyze(&case);
        assert_eq!(
            result.is_ok(),
            case.accepted,
            "{}: {:?}",
            case.name,
            result.err().map(|e| e.to_string())
        );
    }
    assert_eq!(count, 43);
    Ok(())
}
#[test]
fn default_api_and_const_policy_axes_are_direct_distinct_and_never_inherited_as_permission()
-> TestResult {
    for name in [
        "declaration",
        "both-axes",
        "target-module",
        "target-struct",
        "target-field",
        "target-variant",
        "target-root",
        "target-generic-default",
    ] {
        let report = test_source::resolve(&source(name)?, true)?;
        let library = report.package.library_policy().ok_or("library")?;
        let inventory = library.inventory().ok_or("inventory")?;
        let bodies: Vec<_> = inventory
            .policies
            .iter()
            .filter(|p| p.kind == PolicyKind::DefaultBody)
            .collect();
        assert_eq!(bodies.len(), 1, "{name}");
        assert_eq!(bodies[0].feature.as_ref(), "default_api");
        assert!(bodies[0].level.unstable());
        assert!(inventory.has_default_body_policies());
        assert!(library.predicate_requests()?.is_empty());
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            let id = DefId(u32::try_from(index)?);
            if let Some(policy) = inventory.declaration(id) {
                assert_eq!(policy.kind, PolicyKind::Regular);
            }
            if let Some(policy) = inventory.const_declaration(id) {
                assert_eq!(policy.kind, PolicyKind::Const);
            }
            if name == "declaration" && report.package.names.get(definition.name) == "value" {
                assert_eq!(
                    inventory
                        .declaration(id)
                        .ok_or("method API")?
                        .feature
                        .as_ref(),
                    "method_api"
                );
                assert!(inventory.const_declaration(id).is_none());
            }
        }
        if name != "both-axes" {
            assert!(inventory.consts.is_none(), "{name}");
        }
    }
    assert_eq!(size_of::<Library>(), 160);
    assert_eq!(size_of::<Inventory>(), 120);
    assert_eq!(size_of::<Policy>(), 56);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn provided_and_overridden_bodies_lower_real_generic_calls_without_replacing_the_body() -> TestResult
{
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
    for name in ["observed_default", "observed_override"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some())
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/default-bodies-v1/native");
    std::fs::create_dir_all(&folder)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provided-bodies.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn old_metadata_profiles_keep_unencoded_default_body_stability_closed() -> TestResult {
    let source = source("source-provider")?;
    let error = crate::module_packets::exported_profile(
        &source,
        crate::lower::instances::SourcePublication::ModuleFeaturePredicates,
        "default_body_policy",
    )
    .err()
    .ok_or("default-body policy silently omitted")?
    .to_string();
    assert!(
        error.contains("default-body stability requires complete policy transport"),
        "{error}"
    );
    Ok(())
}
#[test]
fn policy_axis_storage_and_inventory_work_scale_with_declarations() -> TestResult {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for n in [16, 64, 256] {
        let mut text = String::from(
            "#![feature(staged_api)]#![stable(feature=\"root_api\",since=\"1.0.0\")]\n",
        );
        for i in 0..n {
            writeln!(
                text,
                "#[stable(feature=\"trait_{i}\",since=\"1.0.0\")]pub trait Service{i}{{#[stable(feature=\"method_{i}\",since=\"1.0.0\")]#[rustc_default_body_unstable(feature=\"body_{i}\",issue=\"none\")]fn value(&self)->u8{{7}}}}"
            )?;
        }
        let report = test_source::resolve(&text, true)?;
        let inventory = report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or("inventory")?;
        assert_eq!(
            inventory
                .policies
                .iter()
                .filter(|p| p.kind == PolicyKind::DefaultBody)
                .count(),
            n
        );
        assert!(inventory.consts.is_none());
        assert!(
            inventory.steps() < 384 * n + 4096,
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
fn each_attribute_axis_publishes_once_and_failed_body_capture_retains_no_policy() -> TestResult {
    let report = test_source::resolve(&source("both-axes")?, true)?;
    let unit = &report.package.units[0];
    let owner = unit
        .configured
        .owners
        .iter()
        .find(|owner| {
            unit.retained_attributes(owner.attributes.clone())
                .any(|(a, f)| a.name(f, unit.source.text()) == Some("rustc_default_body_unstable"))
        })
        .ok_or("default body owner")?;
    let mut node = owner.node;
    while let crate::syntax::ast::Kind::Attributed { value, .. } =
        unit.configured.ast.node(node).kind
    {
        node = value;
    }
    let key = NodeKey {
        file: FileId(0),
        node,
    };
    let attrs: Vec<_> = unit
        .retained_attributes(owner.attributes.clone())
        .filter(|(a, f)| {
            matches!(
                a.name(f, unit.source.text()),
                Some("stable" | "rustc_const_unstable" | "rustc_default_body_unstable")
            )
        })
        .collect();
    assert_eq!(attrs.len(), 3);
    let mut draft = None;
    for &(attribute, forest) in &attrs {
        assert!(collect(
            report.package.library_policy(),
            &mut draft,
            unit,
            key,
            attribute,
            forest,
            None
        )?);
    }
    let state = draft.as_ref().ok_or("draft")?;
    assert_eq!(state.policies.len(), 3);
    for kind in [
        PolicyKind::Regular,
        PolicyKind::Const,
        PolicyKind::DefaultBody,
    ] {
        let id = state.attribute(key, kind).ok_or("distinct actual axis")?;
        assert_eq!(state.policies[id.get() - 1].kind, kind);
    }
    let &(attribute, forest) = attrs.last().ok_or("body attribute")?;
    assert!(
        collect(
            report.package.library_policy(),
            &mut draft,
            unit,
            key,
            attribute,
            forest,
            None
        )
        .is_err()
    );
    assert_eq!(draft.as_ref().ok_or("draft")?.policies.len(), 3);
    let mut failed = Some(Draft {
        bytes: 64 * 1024 * 1024,
        ..Draft::default()
    });
    for _ in 0..2 {
        assert!(
            collect(
                report.package.library_policy(),
                &mut failed,
                unit,
                key,
                attribute,
                forest,
                None
            )
            .is_err()
        );
        let state = failed.as_ref().ok_or("failed state")?;
        assert!(
            state.policies.is_empty()
                && state.attributes.is_empty()
                && state.relationships.is_empty()
        );
    }
    Ok(())
}
