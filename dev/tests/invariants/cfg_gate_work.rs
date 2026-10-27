//! Engine feature queries reuse one cold header memo as call and list widths grow.
use super::*;
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn usage(width: usize, calls: usize, atomic: bool) -> Result<usize> {
    let (feature, predicate) = if atomic {
        (
            "cfg_target_has_atomic",
            "target_has_atomic_load_store=\"8\"",
        )
    } else {
        ("cfg_ub_checks", "ub_checks")
    };
    let list = std::iter::once(feature.to_owned())
        .chain((1..width).map(|i| format!("fixture_feature_{i}")))
        .collect::<Vec<_>>()
        .join(",");
    let mut source = format!(
        r#"#![feature(no_core,lang_items,rustc_attrs,allow_internal_unstable)]#![no_core]
        #[rustc_builtin_macro]macro_rules!cfg{{($($t:tt)*)=>{{}}}}
        #[lang="pointee_sized"]trait PointeeSized{{}}
        #[lang="meta_sized"]trait MetaSized:PointeeSized{{}}
        #[lang="sized"]trait Sized:MetaSized{{}}
        #[allow_internal_unstable({list})]macro_rules!route{{()=>{{cfg!({predicate})}}}}
        fn answer(){{"#
    );
    for _ in 0..calls {
        write!(source, "let _=route!();")?;
    }
    source.push('}');
    let report = crate::test_source::source_provider(&source, "cfg_gate_work")?;
    assert!(
        report
            .package
            .units
            .iter()
            .all(|u| u.configured.cfg_uses.is_none())
    );
    let observed = report
        .package
        .source_core()
        .ok_or("source macro owner")?
        .macros
        .usage()
        .0;
    println!("cfg! allowance width={width} calls={calls} work={observed}");
    Ok(observed)
}
#[test]
fn cfg_expansion_repeated_queries_do_not_repeat_growing_allowance_header_walks() -> Result {
    for atomic in [false, true] {
        let base = usage(8, 1, atomic)?;
        let repeated = usage(8, 32, atomic)?
            .checked_sub(base)
            .ok_or("work ordering")?;
        for width in [32, 128] {
            let cold = usage(width, 1, atomic)?;
            let growth = usage(width, 32, atomic)?
                .checked_sub(cold)
                .ok_or("work ordering")?;
            assert!(
                growth <= repeated + 256 * 31,
                "width{width}: {growth}/{repeated}"
            );
        }
    }
    Ok(())
}

#[test]
fn demanded_false_self_predicates_survive_disabled_owners_and_fresh_prefix_cloning() -> Result {
    use crate::{
        expand::config::Config,
        resolve::unit::Unit,
        support::{names::Names, source::SourceFile},
    };
    assert_eq!(size_of::<crate::expand::configured::ConfiguredUses>(), 24);
    assert_eq!(
        size_of::<Option<Box<crate::expand::configured::ConfiguredUses>>>(),
        8
    );
    let ordinary = Unit::parse(
        SourceFile::new(b"#[cfg(all())]fn ordinary(){}".to_vec())?,
        &Config::default(),
        &mut Names::default(),
    )
    .map_err(|e| e.message)?;
    assert!(ordinary.configured.cfg_uses.is_none());
    let source = "#[cfg(ub_checks)]fn disabled(){}#[cfg(any())]#[cfg(ub_checks)]fn undemanded(){}";
    let unit = Unit::parse(
        SourceFile::new(source.as_bytes().to_vec())?,
        &Config::default(),
        &mut Names::default(),
    )
    .map_err(|e| e.message)?;
    let uses = unit
        .configured
        .cfg_uses
        .as_ref()
        .ok_or("demanded false predicate")?;
    assert_eq!(uses.len(), 1);
    let original = uses.as_slice()[0];
    let effect = &unit.configured.effects[original.effect as usize];
    assert!(matches!(
        effect.attribute.effect,
        crate::expand::configured::Effect::Cfg(false)
    ));
    let owner = unit
        .configured
        .owners
        .iter()
        .find(|owner| owner.attributes.contains(&(original.effect as usize)))
        .ok_or("actual owner")?;
    assert!(!owner.enabled);
    let copy = unit.fresh_instance().map_err(|e| e.message)?;
    let copied = copy
        .configured
        .cfg_uses
        .as_ref()
        .ok_or("fresh demanded prefix")?;
    assert_eq!(copied.len(), 1);
    assert_eq!(copied.as_slice()[0].effect, original.effect);
    assert_eq!(copied.as_slice()[0].predicate.span, original.predicate.span);
    assert!(!std::sync::Arc::ptr_eq(
        unit.source.original_image(),
        copy.source.original_image()
    ));
    let report_source = format!(
        "#![feature(no_core,lang_items,cfg_ub_checks)]#![no_core]#[lang=\"pointee_sized\"]trait PointeeSized{{}}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{{}}#[lang=\"sized\"]trait Sized:MetaSized{{}}{source}"
    );
    let mut report = crate::test_source::source_provider(&report_source, "cfg_false_demand")?;
    validate_attributes(&report)?;
    let unit = report
        .package
        .units
        .iter_mut()
        .find(|u| u.configured.cfg_uses.is_some())
        .ok_or("retained use")?;
    unit.configured.effects.clear();
    assert!(validate_attributes(&report).is_err());
    Ok(())
}
