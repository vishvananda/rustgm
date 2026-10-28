//! Source/parsed/resolved regular variant ownership is independent of execution permission.
use super::*;
use crate::resolve::{
    expansion::Report,
    model::{DefId, Namespace},
    stability::{Level, Policy},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn report(name: &str) -> Result<Report> {
    let source = crate::test_source::fixture(&format!(
        "pa31/tests/spec/library-stability/variants/{name}.rs"
    ))?;
    Ok(crate::test_source::source_provider(
        &source,
        "variant_availability",
    )?)
}
#[test]
fn actual_regular_and_alias_variants_retain_current_policy_rows() -> Result {
    for name in ["regular", "alias"] {
        let report = report(name)?;
        let inventory = report
            .package
            .library_policy()
            .and_then(crate::resolve::stability::Library::inventory)
            .ok_or("inventory")?;
        let variants = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .filter(|(_, d)| d.kind == DefKind::Variant)
            .collect::<Vec<_>>();
        assert_eq!(variants.len(), 4);
        for (index, definition) in variants {
            let policy = inventory
                .declaration(DefId(u32::try_from(index)?))
                .ok_or("variant policy")?;
            assert_eq!(Some(policy.origin), definition.node);
            assert!(matches!(
                policy.feature.as_ref(),
                "variant_api" | "variant_next"
            ));
        }
    }
    Ok(())
}
#[test]
fn const_and_unsupported_owner_annotations_remain_rejected() -> Result {
    for name in ["const-variant", "wrong-owner"] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/library-stability/variants/{name}.rs"
        ))?;
        assert!(
            crate::test_source::source_provider(&source, "variant_availability").is_err(),
            "{name}"
        );
    }
    Ok(())
}
fn seeded() -> Result<(Report, usize, Draft)> {
    let report = report("regular")?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::Variant && d.namespace == Namespace::Value)
        .ok_or("variant")?;
    let policy = report
        .package
        .library_policy()
        .and_then(crate::resolve::stability::Library::inventory)
        .and_then(|i| i.declaration(DefId(u32::try_from(index).ok()?)))
        .ok_or("actual policy")?;
    let Level::Stable(since) = policy.level else {
        return Err("stable fixture policy".into());
    };
    let mut draft = Draft::default();
    draft.policies.push(Policy {
        kind: policy.kind,
        feature: policy.feature.clone(),
        level: Level::Stable(since),
        origin: policy.origin,
    });
    draft.attributes.insert(
        policy.origin,
        std::num::NonZeroUsize::new(1).ok_or("policy id")?,
    );
    Ok((report, index, draft))
}
#[test]
fn detached_wrong_kind_and_current_source_policies_cannot_gain_variant_authority() -> Result {
    for case in 0..3 {
        let (mut report, index, mut draft) = seeded()?;
        if case == 0 {
            report.resolution.definitions[index].node = None;
        }
        if case == 1 {
            report.resolution.definitions[index].kind = DefKind::Const;
        }
        if case == 2 {
            let other = report
                .resolution
                .definitions
                .iter()
                .find(|d| d.kind == DefKind::Enum)
                .and_then(|d| d.node)
                .ok_or("other source")?;
            draft.policies[0].origin = other;
        }
        let mut used = HashSet::default();
        let result = consume(
            &report.package,
            &report.resolution,
            &report.resolution.definitions[index],
            &mut draft,
            &mut used,
        );
        if case == 1 {
            assert!(!result?);
        } else {
            assert!(result.is_err(), "{case}");
        }
        assert!(used.is_empty());
    }
    Ok(())
}
