//! Assembly's baseline union still authenticates the actual enclosing body owner.
use super::Checker;
use crate::{
    sema::{declarations, target_features::FeatureSet},
    syntax::ast::Kind,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn assembly_baseline_union_keeps_actual_body_owner_and_bounded_work() -> Result {
    let mut text = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    text.push_str("#[rustc_builtin_macro(asm)]macro_rules!actual{($($t:tt)*)=>{};}#[target_feature(enable=\"avx\")]fn selected(){unsafe{actual!(\"nop\");}}fn other(){}");
    let report = crate::test_source::source_provider(&text, "asm_cpu_context")?;
    let mut facts = declarations::analyze(&report)?;
    let selected = *facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "selected"
        })
        .ok_or("selected body")?;
    let other = *facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "other"
        })
        .ok_or("other body")?;
    let features = facts
        .target_feature_functions
        .get(&selected.definition)
        .ok_or("real row")?
        .features();
    let node = report
        .package
        .unit(selected.node.file)
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(i, n)| matches!(n.kind, Kind::InlineAssembly(_)).then_some(i))
        .ok_or("actual asm")?;
    let node = crate::syntax::ast::NodeId::new(u32::try_from(node)?);
    let mut checker = Checker::new(&report, &mut facts);
    let progress = checker.start_function(selected)?;
    checker.require_complete_body(progress)?;
    let before = report.assembly_work().ok_or("existing payer")?;
    let available = checker.assembly_features(node)?;
    assert!(available.supports_xmm() && available.covers(features));
    assert!(!FeatureSet::default().supports_xmm());
    let baseline = FeatureSet::target_baseline(|_| Ok::<(), &str>(()), |e| e)?;
    assert!(baseline.supports_xmm() && !baseline.covers(features));
    let after = report.assembly_work().ok_or("same payer")?;
    println!(
        "assembly baseline/query delta-work={} charged-bytes-before={} charged-bytes-after={} feature-set={} body={}",
        after.0 - before.0,
        before.1,
        after.1,
        size_of::<FeatureSet>(),
        size_of::<crate::body::ir::Body>()
    );
    assert_eq!(after.1, before.1);
    assert!(after.0 > before.0 && after.0 - before.0 < 1024);
    assert!(FeatureSet::target_baseline(|_| Err::<(), &str>("paid failure"), |e| e).is_err());
    checker.state.body = other.body.ok_or("other source body")?;
    assert!(checker.assembly_features(node).is_err());
    let mut config = crate::expand::config::Config::default();
    assert!(config.add("target_feature=\"avx\"").is_err());
    assert!(config.contains("target_feature", Some("sse2")));
    Ok(())
}
