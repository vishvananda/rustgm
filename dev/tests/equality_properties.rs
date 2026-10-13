use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    aggregate::{self, Definitions, Repr, Variant},
    equality::{Equality, Properties},
};
use std::collections::HashMap;
const BOTH: Properties = Properties {
    partial_eq: true,
    eq: true,
};

#[test]
fn recursive_equality_dependencies_propagate_failures_to_every_parent() -> Result<(), &'static str>
{
    for bad in [false, true] {
        let mut types = Store::default();
        let mut definitions = Definitions::default();
        let a = types.intern(Kind::Adt(ItemId(0)))?;
        let b = types.intern(Kind::Adt(ItemId(1)))?;
        let unsupported = types.intern(Kind::Adt(ItemId(2)))?;
        let a_ref = types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::STATIC,
            referent: a,
        })?;
        let b_ref = types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::STATIC,
            referent: b,
        })?;
        let mut fields = vec![b_ref];
        if bad {
            fields.push(unsupported);
        }
        for (item, fields) in [
            (ItemId(0), fields),
            (ItemId(1), vec![a_ref]),
            (ItemId(2), Vec::new()),
        ] {
            let fields = types.intern_list(&fields)?;
            definitions.insert(
                item,
                aggregate::Kind::Struct,
                Repr::default(),
                &[Variant {
                    fields,
                    discriminant: 0,
                }],
            )?;
        }
        let selected = HashMap::from([(ItemId(0), BOTH), (ItemId(1), BOTH)]);
        for order in [[a, b, a_ref, b_ref], [b, a, b_ref, a_ref]] {
            let mut equality = Equality::default();
            let expected = if bad { Properties::default() } else { BOTH };
            for ty in order {
                assert_eq!(
                    equality.demand(&types, &definitions, &selected, ty)?,
                    expected
                );
            }
            let work = equality.work;
            for _ in 0..1000 {
                assert_eq!(
                    equality.demand(&types, &definitions, &selected, a)?,
                    expected
                );
            }
            assert_eq!(
                equality.work, work,
                "complete results must be cached, including negative results"
            );
        }
    }
    Ok(())
}

#[test]
fn equality_type_obligations_remain_distinct_from_actual_structural_values()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let mut definitions = Definitions::default();
    let mut equality = Equality::default();
    let float = TypeId::primitive(Primitive::F32);
    let zero = types.intern(Kind::Array {
        element: float,
        length: 0,
    })?;
    let empty = types.intern_list(&[])?;
    let function = types.intern(Kind::Function {
        bound_regions: 0,
        parameters: empty,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let item = types.intern(Kind::FunctionItem {
        item: ItemId(99),
        signature: function,
    })?;
    let opaque = types.intern(Kind::Adt(ItemId(0)))?;
    definitions.insert(
        ItemId(0),
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields: empty,
            discriminant: 0,
        }],
    )?;
    let empty_opaque = types.intern(Kind::Array {
        element: opaque,
        length: 0,
    })?;
    let pointer = types.intern(Kind::Pointer {
        mutable: false,
        pointee: opaque,
    })?;
    let fields = types.intern_list(&[float])?;
    let nominal = types.intern(Kind::Adt(ItemId(1)))?;
    definitions.insert(
        ItemId(1),
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let selected = HashMap::from([(ItemId(1), BOTH)]);
    let partial = Properties {
        partial_eq: true,
        eq: false,
    };
    for (ty, expected) in [
        (zero, partial),
        (function, BOTH),
        (item, Properties::default()),
        (empty_opaque, Properties::default()),
        (pointer, BOTH),
        (nominal, partial),
    ] {
        assert_eq!(
            equality.demand(&types, &definitions, &selected, ty)?,
            expected
        );
    }
    Ok(())
}

#[test]
fn deep_equality_graphs_are_iterative_and_repeated_queries_are_constant_work()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let definitions = Definitions::default();
    let selected = HashMap::<ItemId, Properties>::new();
    let mut root = TypeId::primitive(Primitive::U8);
    for _ in 0..20_000 {
        root = types.intern(Kind::Array {
            element: root,
            length: 1,
        })?;
    }
    let mut equality = Equality::default();
    assert_eq!(
        equality.demand(&types, &definitions, &selected, root)?,
        BOTH
    );
    assert!(equality.work < 50_000);
    let work = equality.work;
    for _ in 0..20_000 {
        assert_eq!(
            equality.demand(&types, &definitions, &selected, root)?,
            BOTH
        );
    }
    assert_eq!(equality.work, work);
    Ok(())
}
