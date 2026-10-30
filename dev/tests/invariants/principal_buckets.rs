//! Principal identities only exclude candidates; object shapes still require proof.
use super::*;
use crate::types::{auto_traits::AutoTrait, dynamic::Bounds};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn principal_bucket_ignores_object_shape_without_colliding_with_nominal_identity() -> Result {
    let mut store = Store::default();
    let principal = ItemId(17);
    let first = store.intern_dynamic_shape(Bounds {
        principal: Some(principal),
        principal_arguments: 1,
        bindings: &[],
        auto_traits: &[],
    })?;
    let second = store.intern_dynamic_shape(Bounds {
        principal: Some(principal),
        principal_arguments: 1,
        bindings: &[],
        auto_traits: &[AutoTrait::Send],
    })?;
    assert_ne!(first, second);
    let name = NameId(2);
    let key = Bucket::declared(&store, Head::Dynamic(first), name).ok_or("principal bucket")?;
    assert_eq!(
        Bucket::lookup(&store, Head::Dynamic(second), name),
        Some(key)
    );
    assert_ne!(key, Bucket::Nominal(principal, name));
    let marker = store.intern_dynamic_shape(Bounds {
        principal: None,
        principal_arguments: 0,
        bindings: &[],
        auto_traits: &[AutoTrait::Send],
    })?;
    assert!(Bucket::lookup(&store, Head::Dynamic(marker), name).is_none());
    println!(
        "local object bucket={} target={} structural={} Body={} Node={}",
        size_of::<Bucket>(),
        size_of::<Target>(),
        size_of::<Structural>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::syntax::ast::Node>()
    );
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    Ok(())
}

#[test]
fn principal_groups_join_nominal_coherence_while_primitive_groups_remain_demanded() -> Result {
    let mut methods = crate::sema::methods::Methods::default();
    let name = NameId(0);
    let principal = Bucket::Structural(Target::Principal(ItemId(5)), name);
    let primitive = Bucket::Structural(Target::Shape(Head::Pointer(false)), name);
    methods.publish_inherent(principal, ItemContext::ROOT, DefId(1));
    methods.publish_inherent(principal, ItemContext::ROOT, DefId(2));
    methods.publish_inherent(primitive, ItemContext::ROOT, DefId(3));
    let groups: Vec<_> = methods.inherent_groups().collect();
    assert_eq!(groups.len(), 1);
    assert_eq!(
        groups[0],
        (principal, ItemContext::ROOT, &[DefId(1), DefId(2)][..])
    );
    Ok(())
}
