//! Actual local regular/const policies retain relationships without granting predicates.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/stability-implications/{name}.rs"
    ))?)
}
#[test]
fn local_implications_preserve_actual_regular_const_and_per_policy_identity() -> Result {
    eprintln!(
        "local stability cold Policy={} Inventory={} Draft={}",
        size_of::<Policy>(),
        size_of::<Inventory>(),
        size_of::<Draft>()
    );
    for (name, count) in [
        ("regular", 1),
        ("unstable-parent", 1),
        ("const", 1),
        ("self-edge", 1),
        ("cycle", 2),
        ("per-policy", 2),
    ] {
        let report = crate::test_source::resolve(&fixture(name)?, true)?;
        crate::pipeline::analyze_source(&report)?;
        let library = report.package.library_policy().ok_or("actual library")?;
        let inventory = library.inventory().ok_or("complete actual inventory")?;
        assert_eq!(inventory.implications.len(), count, "{name}");
        assert!(
            inventory
                .implications
                .windows(2)
                .all(|rows| rows[0].policy < rows[1].policy)
        );
        for edge in &inventory.implications {
            let policy = &inventory.policies[edge.policy.get() - 1];
            assert_eq!(
                policy.kind,
                if name == "const" {
                    PolicyKind::Const
                } else {
                    PolicyKind::Regular
                }
            );
            assert!(policy.level.unstable());
            assert!(
                inventory
                    .policies
                    .iter()
                    .any(|p| p.feature.as_ref() == edge.by.as_ref())
            );
        }
        if name == "per-policy" {
            let edges = &inventory.implications;
            assert_eq!(
                inventory.policies[edges[0].policy.get() - 1].feature,
                inventory.policies[edges[1].policy.get() - 1].feature
            );
            assert_ne!(edges[0].by, edges[1].by);
        }
        assert!(library.predicate_requests()?.is_empty());
    }
    Ok(())
}
#[test]
fn malformed_missing_and_untransported_implications_remain_hard_boundaries() -> Result {
    for (name, message) in [
        ("missing-target", "undefined library feature"),
        ("language-only-target", "undefined library feature"),
        ("invalid-name", "identifier"),
        ("duplicate", "duplicate library stability argument"),
        ("stable-argument", "unsupported library stability argument"),
        ("renamed-required", "renamed library features"),
    ] {
        let source = fixture(name)?;
        let result = if name == "language-only-target" {
            crate::test_source::source_provider(&source, "implied_language_target")
        } else {
            crate::test_source::resolve(&source, true)
        };
        let error = result.err().ok_or("invalid policy accepted")?;
        assert!(error.contains(message), "{name}: {error}");
    }
    let source =
        crate::test_source::fixture("pa31/tests/spec/feature-predicate-metadata-v4/provider.rs")?;
    let source = source
        + "\n#[unstable(feature=\"implied_child\",issue=\"none\",implied_by=\"identity_api\")]pub fn implied()->u8{7}\n";
    let error = crate::module_packets::exported_profile(
        &source,
        crate::lower::instances::SourcePublication::ModuleFeaturePredicates,
        "implied_policy",
    )
    .err()
    .ok_or("metadata silently dropped an implied policy")?
    .to_string();
    assert!(error.contains("complete wider policy transport"), "{error}");
    Ok(())
}

#[test]
fn known_advisory_feature_request_defines_no_library_feature_or_semantic_capability() -> Result {
    let report = crate::test_source::resolve(&fixture("lint-feature-request")?, true)?;
    assert!(report.package.library_policy().is_none());
    let error =
        crate::test_source::source_provider(&fixture("lint-as-parent")?, "lint_not_library")
            .err()
            .ok_or("advisory request manufactured a library definition")?;
    assert!(error.contains("undefined library feature"), "{error}");
    Ok(())
}
#[test]
fn implied_attribute_failed_payment_never_publishes_a_policy_or_relationship() -> Result {
    let report = crate::test_source::resolve(&fixture("regular")?, true)?;
    let unit = &report.package.units[0];
    let (owner, attribute, forest) = unit
        .configured
        .owners
        .iter()
        .find_map(|owner| {
            unit.retained_attributes(owner.attributes.clone())
                .find(|(attr, forest)| attr.name(forest, unit.source.text()) == Some("unstable"))
                .map(|(a, f)| (owner, a, f))
        })
        .ok_or("actual implied unstable annotation")?;
    let crate::syntax::ast::Kind::Attributed { value, .. } =
        unit.configured.ast.node(owner.node).kind
    else {
        return Err("actual wrapper".into());
    };
    let key = NodeKey {
        file: FileId(0),
        node: value,
    };
    let mut draft = Some(Draft {
        bytes: 64 * 1024 * 1024,
        ..Draft::default()
    });
    for _ in 0..2 {
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
        let failed = draft.as_ref().ok_or("failed state retained")?;
        assert!(
            failed.policies.is_empty()
                && failed.implications.is_empty()
                && failed.attributes.is_empty()
        );
        assert!(failed.bytes > 64 * 1024 * 1024);
    }
    Ok(())
}
