//! Canonical const products retain typed identity and sharing independently of CTFE storage.
use super::*;
use crate::types::{GenericArgs, ItemId, Primitive, arguments::Constant};
use std::cell::Cell;
type Result<T = ()> = std::result::Result<T, &'static str>;
fn boolean(store: &mut Store, value: bool) -> Result<ConstId> {
    store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Bool),
        bits: u128::from(value),
    })
}
#[test]
fn canonical_products_keep_nominal_variant_type_and_scalar_identities_distinct() -> Result {
    let mut store = Store::default();
    let no = boolean(&mut store, false)?;
    let yes = boolean(&mut store, true)?;
    let bool_ty = TypeId::primitive(Primitive::Bool);
    let fields = store.intern_list(&[bool_ty, bool_ty])?;
    let tuple = store.intern(Kind::Tuple(fields))?;
    let a = store.intern_aggregate_constant(tuple, None, &[no, yes], |_, _| Ok(()))?;
    assert_eq!(
        store.intern_aggregate_constant(tuple, None, &[no, yes], |_, _| Ok(()))?,
        a
    );
    assert_ne!(
        store.intern_aggregate_constant(tuple, None, &[yes, no], |_, _| Ok(()))?,
        a
    );
    assert_eq!(store.const_type(a), Some(tuple));
    assert!(store.constant(a).is_none());
    assert!(store.const_is_value(a));
    let nominal = store.intern(Kind::Adt(ItemId(7), GenericArgs::EMPTY))?;
    let other = store.intern(Kind::Adt(ItemId(8), GenericArgs::EMPTY))?;
    let product = store.intern_aggregate_constant(nominal, None, &[no, yes], |_, _| Ok(()))?;
    assert_ne!(a, product);
    assert_ne!(
        product,
        store.intern_aggregate_constant(other, None, &[no, yes], |_, _| Ok(()))?
    );
    let first = store.intern_aggregate_constant(nominal, Some(0), &[no, yes], |_, _| Ok(()))?;
    assert_ne!(product, first);
    assert_ne!(
        first,
        store.intern_aggregate_constant(nominal, Some(1), &[no, yes], |_, _| Ok(()))?
    );
    let ConstKind::Aggregate(id) = store.const_kind(first).ok_or("aggregate")? else {
        return Err("scalar product");
    };
    let Aggregate::Fields {
        ty,
        variant,
        values,
    } = store.aggregate_constant(id).ok_or("node")?
    else {
        return Err("repeated nominal");
    };
    assert_eq!(
        (ty, variant, values),
        (nominal, Some(0), [no, yes].as_slice())
    );
    assert_eq!(size_of::<ConstKind>(), size_of::<Option<ConstKind>>());
    assert_eq!(size_of::<Constant>(), 32);
    Ok(())
}
#[test]
fn explicit_uniform_and_compressed_arrays_have_one_value_identity() -> Result {
    let mut store = Store::default();
    let no = boolean(&mut store, false)?;
    let yes = boolean(&mut store, true)?;
    for length in [1, 8, 64] {
        let ty = store.intern(Kind::Array {
            element: TypeId::primitive(Primitive::Bool),
            length,
        })?;
        let repeated = store.intern_repeated_constant(ty, yes, |_, _| Ok(()))?;
        let values = vec![yes; usize::try_from(length).map_err(|_| "length")?];
        assert_eq!(
            store.intern_aggregate_constant(ty, None, &values, |_, _| Ok(()))?,
            repeated
        );
        assert_ne!(
            store.intern_repeated_constant(ty, no, |_, _| Ok(()))?,
            repeated
        );
    }
    let empty = store.intern(Kind::Array {
        element: TypeId::primitive(Primitive::Bool),
        length: 0,
    })?;
    assert_eq!(
        store.intern_repeated_constant(empty, no, |_, _| Ok(()))?,
        store.intern_repeated_constant(empty, yes, |_, _| Ok(()))?
    );
    let huge = store.intern(Kind::Array {
        element: TypeId::primitive(Primitive::Bool),
        length: 1_000_000_000,
    })?;
    let value = store.intern_repeated_constant(huge, yes, |_, _| Ok(()))?;
    let ConstKind::Aggregate(id) = store.const_kind(value).ok_or("repeat")? else {
        return Err("scalar repeat");
    };
    assert!(
        matches!(store.aggregate_constant(id),Some(Aggregate::Repeat{element,length:1_000_000_000,..}) if element==yes)
    );
    assert!(
        store
            .arguments
            .as_deref()
            .and_then(|a| a.aggregates.as_deref())
            .ok_or("storage")?
            .fields
            .edge_count()
            < 8
    );
    Ok(())
}
#[test]
fn canonical_const_dag_work_and_storage_follow_unique_edges_and_warm_queries() -> Result {
    let mut store = Store::default();
    let mut root = boolean(&mut store, true)?;
    let mut ty = TypeId::primitive(Primitive::Bool);
    let work = Cell::new(0usize);
    let bytes = Cell::new(0usize);
    let previous_extent = store.argument_extent();
    let unrelated_trait_storage = store.trait_input_bytes();
    let pay = |steps: usize, growth: usize| {
        work.set(work.get().saturating_add(steps));
        bytes.set(bytes.get().saturating_add(growth));
        Ok(())
    };
    for _ in 0..128 {
        let fields = store.intern_list(&[ty, ty])?;
        ty = store.intern(Kind::Tuple(fields))?;
        root = store.intern_aggregate_constant(ty, None, &[root, root], pay)?;
    }
    let storage = store
        .arguments
        .as_deref()
        .and_then(|a| a.aggregates.as_deref())
        .ok_or("storage")?;
    assert_eq!(storage.nodes.len(), 128);
    assert_eq!(storage.fields.edge_count(), 256);
    assert_eq!(store.argument_extent() - previous_extent, 128);
    assert_eq!(store.trait_input_bytes(), unrelated_trait_storage);
    let allocation = storage.allocation();
    assert!(allocation.bytes < 128 * 1024);
    assert!(work.get() < 32_768);
    let ConstKind::Aggregate(id) = store.const_kind(root).ok_or("root")? else {
        return Err("scalar root");
    };
    let Aggregate::Fields { values, .. } = store.aggregate_constant(id).ok_or("root node")? else {
        return Err("repeated tuple");
    };
    let children = [values[0], values[1]];
    let before = (work.get(), bytes.get());
    for _ in 0..128 {
        assert_eq!(
            store.intern_aggregate_constant(ty, None, &children, pay)?,
            root
        );
    }
    assert!(work.get() - before.0 <= 128 * 16);
    assert_eq!(bytes.get(), before.1);
    assert!(
        store
            .intern_aggregate_constant(ty, None, &children, |_, _| Err("exhausted"))
            .is_err()
    );
    assert_eq!(
        store
            .arguments
            .as_deref()
            .and_then(|a| a.aggregates.as_deref())
            .ok_or("storage")?
            .allocation()
            .bytes,
        allocation.bytes
    );
    Ok(())
}
#[test]
fn malformed_or_unpaid_products_do_not_publish_canonical_value_nodes() -> Result {
    let mut store = Store::default();
    let yes = boolean(&mut store, true)?;
    assert!(
        store
            .arguments
            .as_deref()
            .ok_or("scalar storage")?
            .aggregates
            .is_none()
    );
    let fields = store.intern_list(&[TypeId::primitive(Primitive::Bool)])?;
    let tuple = store.intern(Kind::Tuple(fields))?;
    for children in [&[][..], &[ConstId(u32::MAX)][..], &[yes, yes][..]] {
        assert!(
            store
                .intern_aggregate_constant(tuple, None, children, |_, _| Ok(()))
                .is_err()
        );
    }
    assert!(
        store
            .intern_aggregate_constant(tuple, Some(0), &[yes], |_, _| Ok(()))
            .is_err()
    );
    assert!(
        store
            .intern_aggregate_constant(tuple, None, &[yes], |_, _| Err("unpaid"))
            .is_err()
    );
    assert!(
        store
            .intern_repeated_constant(tuple, yes, |_, _| Ok(()))
            .is_err()
    );
    assert!(
        store
            .arguments
            .as_deref()
            .ok_or("scalar storage")?
            .aggregates
            .is_none()
    );
    assert!(!store.const_is_value(ConstId(u32::MAX)));
    Ok(())
}
