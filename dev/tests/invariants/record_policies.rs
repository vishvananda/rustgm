//! Actual configured field owners retain policy without changing field storage.
use super::*;
use crate::{
    pipeline,
    resolve::stability::{Level, Library},
    test_source,
};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn required_fields_match_independent_language_and_real_body_outcomes() -> Result {
    let cases = test_source::fixture("pa31/tests/spec/aggregate-field-policy/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in cases.lines() {
        let (path, status) = line.split_once('\t').ok_or("required field case")?;
        let source = test_source::fixture(path)?;
        let result = (|| -> Result {
            let report = test_source::resolve(&source, true)?;
            let mut analysis = pipeline::analyze_source(&report)?;
            crate::lower::instances::inspect_prepared(
                &report,
                &mut analysis.facts,
                analysis.prepared,
                |_, _| (),
            )?;
            Ok(())
        })();
        if result.is_ok() != (status == "0") {
            failures.push(format!("{path}: {:?}", result.err()));
        }
        count += 1;
    }
    assert_eq!(count, 24);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn completed_fields_preserve_parent_and_direct_inherited_policy_identity() -> Result {
    for (name, features) in [
        ("tuple-direct-stable", vec!["field_api"]),
        ("tuple-inherits-stable", vec!["aggregate_api"]),
        ("enum-inherited-variant", vec!["field_api"]),
        (
            "two-fields-distinct-policy",
            vec!["field_api", "field_next"],
        ),
    ] {
        let source =
            test_source::fixture(&format!("pa31/tests/spec/aggregate-field-policy/{name}.rs"))?;
        let report = test_source::resolve(&source, true)?;
        let analysis = pipeline::analyze_source(&report)?;
        let library = report.package.library_policy().ok_or("actual library")?;
        let inventory = library.inventory().ok_or("actual inventory")?;
        let rows = &inventory.fields.as_deref().ok_or("actual fields")?.rows;
        assert_eq!(rows.len(), features.len());
        for field in &analysis.facts.members.fields {
            let key = field.origin.source().ok_or("actual source field")?;
            let row = rows.get(&key).ok_or("actual retained field")?;
            let policy = inventory
                .field_declaration(key, row.owner)?
                .ok_or("actual field policy")?;
            assert!(features.contains(&policy.feature.as_ref()));
            assert_eq!(
                matches!(policy.level, Level::Unstable { .. }),
                policy.feature.as_ref() == "field_next"
            );
            assert!(inventory.field_declaration(key, key).is_err());
            assert!(inventory.field_declaration(row.owner, row.owner).is_err());
        }
        assert!(library.require_plain_metadata().is_err());
    }
    assert_eq!(size_of::<Inventory>(), 120);
    assert_eq!(size_of::<Library>(), 160);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn field_publication_prepays_failed_growth_and_scales_without_cross_field_queries() -> Result {
    let mut rows = Fields::default();
    let mut draft = Draft::default();
    let report = test_source::resolve("struct S(u8);fn main(){}", true)?;
    assert!(report.package.library_policy().is_none());
    let owner = report
        .resolution
        .definitions
        .iter()
        .find(|d| d.kind == DefKind::Struct)
        .and_then(|d| d.node)
        .ok_or("actual aggregate")?;
    let unit = report.package.unit(owner.file);
    let fields = match unit.configured.ast.node(owner.node).kind {
        Kind::Item(index) => match unit.configured.ast.items[index as usize].kind {
            ItemKind::Struct { fields, .. } => fields,
            _ => return Err("actual struct".into()),
        },
        _ => return Err("actual item".into()),
    };
    let key = NodeKey {
        node: *unit
            .configured
            .ast
            .list(fields)
            .first()
            .ok_or("actual field")?,
        ..owner
    };
    draft.steps = 1_000_000;
    assert!(
        rows.insert(
            key,
            Row {
                owner,
                regular: None
            },
            &mut draft
        )
        .is_err()
    );
    assert!(rows.rows.is_empty());
    assert!(draft.steps > 1_000_000);
    let mut observed = Vec::new();
    for count in [32, 128] {
        let mut source = String::from(
            "#![feature(staged_api)]#![unstable(feature=\"scale\",issue=\"none\")]pub struct S{",
        );
        for index in 0..count {
            write!(source, "pub f{index}:u8,")?;
        }
        source.push_str("}fn main(){}");
        let report = test_source::resolve(&source, true)?;
        let inventory = report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or("inventory")?;
        assert_eq!(
            inventory.fields.as_deref().ok_or("fields")?.rows.len(),
            count
        );
        observed.push(inventory.steps());
    }
    assert!(observed[1] < observed[0] * 6);
    Ok(())
}
