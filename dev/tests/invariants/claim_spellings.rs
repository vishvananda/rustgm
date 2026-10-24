//! Same-shaped trees and coincident source IDs cannot replace cached claim names.
use super::{TestResult, actual};
use crate::{
    expand::builtin::Builtin,
    resolve::{bindings::panic_contracts::existing, unit::NodeKey},
    support::source::SourceFile,
};

#[test]
fn warm_claim_witness_rejects_equal_width_source_name_changes() -> TestResult {
    for (before, after) in [
        ("edition_panic", "another_panic"),
        ("core_panic", "fake_panic"),
    ] {
        assert_eq!(before.len(), after.len());
        let mut report = actual()?;
        let (key, index, tree, declaration_count) = {
            let state = report
                .package
                .source_core()
                .ok_or("provider")?
                .macros
                .0
                .borrow();
            let (entry, index, tree) = state
                .declarations
                .values()
                .find_map(|entry| {
                    entry.witnesses.iter().find_map(|witness| {
                        witness.nodes.iter().find_map(|&(index, tree, spelling)| {
                            spelling
                                .filter(|spelling| spelling.name() == before)
                                .map(|_| (entry, index, tree))
                        })
                    })
                })
                .ok_or("actual claim identifier witness")?;
            (
                entry.declaration.node,
                index,
                tree,
                state.declarations.len(),
            )
        };
        assert!(existing(&report.package, key)?.is_some());
        let unit = &mut report.package.units[key.file.0 as usize];
        let mut changed = unit.source.text().to_owned();
        let span = tree.span.start as usize..tree.span.end as usize;
        assert_eq!(changed.get(span.clone()), Some(before));
        changed.replace_range(span, after);
        let saved = std::mem::replace(&mut unit.source, SourceFile::new(changed.into_bytes())?);
        let forest = unit
            .configured
            .ast
            .attributes
            .iter()
            .find(|capture| {
                capture.forest.tree(index) == Some(tree)
                    && capture.forest.name(unit.source.text(), index) == Some(after)
            })
            .ok_or("same actual tree with changed source spelling")?;
        assert_eq!(forest.forest.tree(index), Some(tree));
        assert!(existing(&report.package, key).is_err());
        assert_eq!(
            report
                .package
                .source_core()
                .ok_or("provider")?
                .macros
                .0
                .borrow()
                .declarations
                .len(),
            declaration_count
        );
        report.package.units[key.file.0 as usize].source = saved;
        assert!(existing(&report.package, key)?.is_some());
    }
    Ok(())
}

fn role_source(role: &str) -> String {
    format!(
        r#"#![feature(no_core,lang_items,rustc_attrs)]#![no_core]
#[lang="pointee_sized"]trait PointeeSized{{}}
#[lang="meta_sized"]trait MetaSized:PointeeSized{{}}
#[lang="sized"]trait Sized:MetaSized{{}}
#[rustc_builtin_macro({role})]macro_rules! route{{()=>{{7u32}};}}
pub fn answer()->u32{{7u32}}
"#
    )
}
fn role_key(report: &crate::resolve::expansion::Report) -> TestResult<NodeKey> {
    Ok(report
        .definitions
        .iter()
        .find_map(|definition| {
            matches!(definition.body, crate::resolve::expansion::Body::Builtin(_))
                .then_some(definition.node)
                .flatten()
        })
        .ok_or("actual claimed definition")?)
}

#[test]
fn helper_attribute_word_retains_its_exact_warm_claim_identity() -> TestResult {
    for word in ["attributes", "r#attributes"] {
        let source = role_source(&format!("r#line,{word}()"));
        let mut report = crate::test_source::source_provider(&source, "helper_word")?;
        let key = role_key(&report)?;
        assert_eq!(
            existing(&report.package, key)?
                .ok_or("actual claim")?
                .builtin,
            Some(Builtin::Line)
        );
        let changed = source.replacen("attributes", "attribvtes", 1);
        assert_eq!(source.len(), changed.len());
        assert!(crate::test_source::source_provider(&changed, "helper_word").is_err());
        // Only source bytes change: all configured trees, spans and owner IDs remain.
        let unit = &mut report.package.units[key.file.0 as usize];
        let saved = std::mem::replace(&mut unit.source, SourceFile::new(changed.into_bytes())?);
        assert!(existing(&report.package, key).is_err());
        report.package.units[key.file.0 as usize].source = saved;
        assert_eq!(
            existing(&report.package, key)?
                .ok_or("restored claim")?
                .builtin,
            Some(Builtin::Line)
        );
    }
    Ok(())
}

#[test]
fn same_numbered_cross_report_claim_owners_cannot_exchange_source_authority() -> TestResult {
    assert_eq!("core_panic".len(), "option_env".len());
    let mut left = crate::test_source::source_provider(&role_source("core_panic"), "claim_owners")?;
    let mut right =
        crate::test_source::source_provider(&role_source("option_env"), "claim_owners")?;
    let key = role_key(&left)?;
    assert_eq!(key, role_key(&right)?);
    assert_eq!(
        existing(&left.package, key)?.ok_or("left claim")?.builtin,
        Some(Builtin::CorePanic)
    );
    assert_eq!(
        existing(&right.package, key)?.ok_or("right claim")?.builtin,
        Some(Builtin::OptionEnv)
    );
    let left_shapes = &left.package.units[0].configured.ast.attributes;
    let right_shapes = &right.package.units[0].configured.ast.attributes;
    assert_eq!(left_shapes.len(), right_shapes.len());
    for (left, right) in left_shapes.iter().zip(right_shapes) {
        assert_eq!(left.forest.nodes(), right.forest.nodes());
    }
    std::mem::swap(&mut left.package.units[0], &mut right.package.units[0]);
    assert!(existing(&left.package, key).is_err());
    assert!(existing(&right.package, key).is_err());
    std::mem::swap(&mut left.package.units[0], &mut right.package.units[0]);
    assert_eq!(
        existing(&left.package, key)?
            .ok_or("restored left")?
            .builtin,
        Some(Builtin::CorePanic)
    );
    assert_eq!(
        existing(&right.package, key)?
            .ok_or("restored right")?
            .builtin,
        Some(Builtin::OptionEnv)
    );
    eprintln!(
        "claim witness sizes row={} spelling={} witness={} source_core={} definition={}",
        size_of::<super::super::WitnessNode>(),
        size_of::<crate::resolve::bindings::claim_tokens::Spelling>(),
        size_of::<super::super::Witness>(),
        size_of::<crate::resolve::bindings::SourceCore>(),
        size_of::<crate::resolve::expansion::Definition>()
    );
    Ok(())
}
