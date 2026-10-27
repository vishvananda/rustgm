//! Actual enclosing sizes and the explicit two-entry ordinary canonical cost.
use super::*;
use crate::{
    literal::{
        storage::StoredValue,
        value::{Float, FloatBits, FloatType},
    },
    sema::{
        model::{NodeFacts, Scalar},
        values::{Constant, View},
    },
};

#[test]
fn two_new_primitives_preserve_existing_ids_and_enclosing_hot_layouts() {
    let store = Store::default();
    let mut old_kinds = vec![Kind::Never, Kind::Tuple(ListId(0))];
    old_kinds.extend(Primitive::ALL.into_iter().take(17).map(Kind::Primitive));
    let old_index: HashMap<Kind, TypeId> = old_kinds
        .iter()
        .copied()
        .zip(0_u32..)
        .map(|(kind, id)| (kind, TypeId(id)))
        .collect();
    let old_properties: Vec<_> = old_kinds
        .iter()
        .map(|kind| {
            let mut properties = Properties::default();
            properties.set(Property::Sized, *kind != Kind::Primitive(Primitive::Str));
            properties
        })
        .collect();
    assert_eq!(store.len(), 21);
    for (index, kind) in old_kinds.iter().enumerate() {
        assert_eq!(store.kinds[index], *kind);
    }
    assert_eq!(TypeId::primitive(Primitive::F32), TypeId(17));
    assert_eq!(TypeId::primitive(Primitive::F64), TypeId(18));
    assert_eq!(TypeId::primitive(Primitive::F16), TypeId(19));
    assert_eq!(TypeId::primitive(Primitive::F128), TypeId(20));
    assert_eq!(store.properties.len() - old_properties.len(), 2);
    assert_eq!(store.index.len() - old_index.len(), 2);
    assert_eq!(size_of::<FloatType>(), 1);
    assert_eq!(size_of::<Option<FloatType>>(), 1);
    assert_eq!(size_of::<FloatBits>(), 24);
    assert_eq!(align_of::<FloatBits>(), 8);
    assert_eq!(size_of::<Scalar>(), 32);
    assert_eq!(size_of::<NodeFacts>(), 80);
    assert_eq!(size_of::<Constant>(), 32);
    assert_eq!(size_of::<crate::body::ir::Rvalue>(), 20);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::lowir::program::Value>(), 12);
    assert_eq!(size_of::<crate::support::crates::Scalar>(), 1);
    println!(
        "old/new canonical kinds={}/{} capacities={}/{} properties={}/{} capacities={}/{} indexentries={}/{} indexcapacities={}/{} Kind={} Properties={} Float={} StoredValue={} FloatBits={} Scalar={} Constant={} View={} Operation={} NodeFacts={} Store={} Primitive={}",
        old_kinds.len(),
        store.kinds.len(),
        old_kinds.capacity(),
        store.kinds.capacity(),
        old_properties.len(),
        store.properties.len(),
        old_properties.capacity(),
        store.properties.capacity(),
        old_index.len(),
        store.index.len(),
        old_index.capacity(),
        store.index.capacity(),
        size_of::<Kind>(),
        size_of::<Properties>(),
        size_of::<Float<'_>>(),
        size_of::<StoredValue>(),
        size_of::<FloatBits>(),
        size_of::<Scalar>(),
        size_of::<Constant>(),
        size_of::<View>(),
        size_of::<crate::sema::model::Operation>(),
        size_of::<NodeFacts>(),
        size_of::<Store>(),
        size_of::<Primitive>()
    );
}
