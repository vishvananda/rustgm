//! Failed scopes remove only their own writes and preserve nested/preexisting values.
use super::*;
use crate::{
    pipeline,
    sema::model::Scalar,
    test_source,
    types::{Primitive, TypeId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn exact_journals_restore_nested_foreign_and_preexisting_values() -> Result {
    let report = test_source::resolve(
        "const fn f<const N:u8>()->u8{const{N}}const A:u8=f::<2>();",
        true,
    )?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let registry = facts.inline_constants.as_mut().ok_or("registry")?;
    let (instance, result, value) = registry
        .instance_values()
        .next()
        .ok_or("completed inline")?;
    let outer_key = (instance, TypeId::primitive(Primitive::U16));
    let inner_key = (instance, TypeId::primitive(Primitive::U32));
    let outer = registry.begin_journal([(instance, result), outer_key].into_iter())?;
    assert!(registry.current_journal(outer));
    registry.publish_instance(instance, result, value)?;
    let outer_value = Constant::scalar(Scalar::Integer(99));
    registry.publish_instance(outer_key.0, outer_key.1, outer_value)?;
    let inner = registry.begin_journal([inner_key].into_iter())?;
    assert!(registry.finish_journal(outer, false).is_err());
    assert!(registry.current_journal(inner));
    let inner_value = Constant::scalar(Scalar::Integer(17));
    registry.publish_instance(inner_key.0, inner_key.1, inner_value)?;
    registry.finish_journal(inner, true)?;
    registry.finish_journal(outer, false)?;
    assert!(registry.instance_value(instance, result) == Some(value));
    assert!(registry.instance_value(outer_key.0, outer_key.1).is_none());
    assert!(registry.instance_value(inner_key.0, inner_key.1) == Some(inner_value));
    assert!(!registry.current_journal(outer));
    assert!(registry.finish_journal(outer, true).is_err());
    Ok(())
}
#[test]
fn journal_quota_preflights_and_foreign_writes_are_not_removed() -> Result {
    let report = test_source::resolve(
        "const fn f<const N:u8>()->u8{const{N}}const A:u8=f::<2>();",
        true,
    )?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let registry = facts.inline_constants.as_mut().ok_or("registry")?;
    let (instance, _, _) = registry.instance_values().next().ok_or("inline")?;
    let key = (instance, TypeId::primitive(Primitive::U16));
    let lease = registry.begin_journal([key].into_iter())?;
    registry.publish_instance(key.0, key.1, Constant::scalar(Scalar::Integer(1)))?;
    let foreign = Constant::scalar(Scalar::Integer(2));
    registry
        .instances
        .as_mut()
        .ok_or("values")?
        .completed
        .insert(key, foreign);
    assert!(registry.finish_journal(lease, false).is_err());
    assert!(registry.instance_value(key.0, key.1) == Some(foreign));
    registry.retained_bytes = 64 * 1024 * 1024;
    assert!(registry.begin_journal([key].into_iter()).is_err());
    assert!(!registry.current_journal(lease));
    Ok(())
}
