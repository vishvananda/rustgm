use rustgm::{
    abi::layout::Layouts,
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
        aggregate::{self, Definitions, Repr, Variant},
        arguments::{Argument, ConstId, Constant},
        bound_uses::Uses,
        copy::Copies,
        destruction::NeedsDrop,
        equality::Equality,
        substitute::Bound,
    },
};
use std::collections::HashMap;

#[test]
fn canonical_arguments_retain_kinds_order_types_and_all_constant_bits() -> Result<(), &'static str>
{
    let mut store = Store::default();
    let u8_ = TypeId::primitive(Primitive::U8);
    let u64_ = TypeId::primitive(Primitive::U64);
    let small = store.intern_constant(Constant { ty: u8_, bits: 17 })?;
    let wide = store.intern_constant(Constant { ty: u64_, bits: 17 })?;
    let maximum = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::U128),
        bits: u128::MAX,
    })?;
    assert_ne!(small, wide);
    let values = [
        Argument::Lifetime(RegionId(9)),
        Argument::Type(u8_),
        Argument::Const(small),
        Argument::Const(maximum),
    ];
    let args = store.intern_arguments(&values)?;
    let nominal = store.intern(Kind::Adt(ItemId(6), args))?;
    let count = store.len();
    for _ in 0..1000 {
        assert_eq!(
            store.intern_constant(Constant { ty: u8_, bits: 17 })?,
            small
        );
        assert_eq!(store.intern_arguments(&values)?, args);
        assert_eq!(store.intern(Kind::Adt(ItemId(6), args))?, nominal);
    }
    assert_eq!(store.len(), count);
    assert_eq!(
        store
            .arguments(args)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>(),
        values
    );
    for replacement in [
        Argument::Type(u64_),
        Argument::Const(wide),
        Argument::Const(small),
    ] {
        let mut changed = values;
        changed[1] = replacement;
        let other = store.intern_arguments(&changed)?;
        assert_ne!(args, other);
        assert_ne!(nominal, store.intern(Kind::Adt(ItemId(6), other))?);
    }
    let mut reversed = values;
    reversed.swap(1, 2);
    assert_ne!(store.intern_arguments(&reversed)?, args);
    assert!(store.region_args(args).is_none());
    assert_eq!(std::mem::size_of::<GenericArgs>(), 4);
    assert_eq!(std::mem::size_of::<Argument>(), 8);
    Ok(())
}

#[test]
fn lifetime_only_arguments_have_one_identity_and_borrowed_iteration() -> Result<(), &'static str> {
    let mut store = Store::default();
    assert_eq!(store.intern_arguments(&[])?, GenericArgs::EMPTY);
    let regions = [RegionId::STATIC, RegionId::bound(0, 1)?, RegionId(19)];
    let args = store.intern_regions(&regions)?;
    let values = regions.map(Argument::Lifetime);
    assert_eq!(store.intern_arguments(&values)?, args);
    assert_eq!(store.region_args(args), Some(regions.as_slice()));
    let mut view = store.arguments(args).ok_or("missing argument view")?;
    assert_eq!(view.len(), 3);
    assert_eq!(view.next(), Some(values[0]));
    assert_eq!(view.size_hint(), (2, Some(2)));
    assert_eq!(view.collect::<Vec<_>>(), values[1..]);
    Ok(())
}

#[test]
fn constants_validate_scalar_domains_without_truncation() -> Result<(), &'static str> {
    let mut store = Store::default();
    for primitive in Primitive::ALL {
        if let Some((_, width)) = primitive.integer() {
            let maximum = u128::MAX >> (128 - width);
            let value = Constant {
                ty: TypeId::primitive(primitive),
                bits: maximum,
            };
            let id = store.intern_constant(value)?;
            assert_eq!(store.constant(id), Some(value));
            if width < 128 {
                assert!(
                    store
                        .intern_constant(Constant {
                            bits: maximum + 1,
                            ..value
                        })
                        .is_err()
                );
            }
        }
    }
    for (primitive, bits, accepted) in [
        (Primitive::Bool, 0, true),
        (Primitive::Bool, 1, true),
        (Primitive::Bool, 2, false),
        (Primitive::Char, 0, true),
        (Primitive::Char, 0xd7ff, true),
        (Primitive::Char, 0xd800, false),
        (Primitive::Char, 0xdfff, false),
        (Primitive::Char, 0xe000, true),
        (Primitive::Char, 0x0010_ffff, true),
        (Primitive::Char, 0x0011_0000, false),
        (Primitive::Char, u128::MAX, false),
        (Primitive::F32, 0, false),
        (Primitive::F64, 0, false),
        (Primitive::Str, 0, false),
    ] {
        assert_eq!(
            store
                .intern_constant(Constant {
                    ty: TypeId::primitive(primitive),
                    bits
                })
                .is_ok(),
            accepted
        );
    }
    assert!(
        store
            .intern_constant(Constant {
                ty: TypeId::UNIT,
                bits: 0
            })
            .is_err()
    );
    assert!(
        store
            .intern_constant(Constant {
                ty: TypeId(u32::MAX),
                bits: 0
            })
            .is_err()
    );
    assert!(
        store
            .intern_arguments(&[Argument::Type(TypeId(u32::MAX))])
            .is_err()
    );
    assert!(
        store
            .intern_arguments(&[Argument::Const(ConstId(u32::MAX))])
            .is_err()
    );
    for args in [
        GenericArgs(u32::MAX),
        GenericArgs(0x8000_0000),
        GenericArgs(0x7fff_ffff),
    ] {
        assert!(store.arguments(args).is_none());
        assert!(store.intern(Kind::Adt(ItemId(0), args)).is_err());
    }
    Ok(())
}

#[test]
fn mixed_argument_substitution_preserves_nested_binders_and_constants() -> Result<(), &'static str>
{
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let local = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: byte,
    })?;
    let outer = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 0)?,
        referent: byte,
    })?;
    let constant = store.intern_constant(Constant { ty: byte, bits: 91 })?;
    let arguments = store.intern_arguments(&[
        Argument::Type(local),
        Argument::Type(outer),
        Argument::Const(constant),
        Argument::Lifetime(RegionId::bound(1, 0)?),
    ])?;
    let nominal = store.intern(Kind::Adt(ItemId(3), arguments))?;
    let parameters = store.intern_list(&[nominal])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: nominal,
        unsafe_: false,
    })?;
    let item = store.intern(Kind::FunctionItem {
        item: ItemId(7),
        arguments: GenericArgs::EMPTY,
        signature: function,
    })?;
    let mut substitution = Bound::new(vec![RegionId::bound(0, 5)?]);
    let result = substitution.apply(&mut store, item)?;
    let shifted = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 5)?,
        referent: byte,
    })?;
    let arguments = store.intern_arguments(&[
        Argument::Type(local),
        Argument::Type(shifted),
        Argument::Const(constant),
        Argument::Lifetime(RegionId::bound(1, 5)?),
    ])?;
    let nominal = store.intern(Kind::Adt(ItemId(3), arguments))?;
    let parameters = store.intern_list(&[nominal])?;
    let signature = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: nominal,
        unsafe_: false,
    })?;
    let expected = store.intern(Kind::FunctionItem {
        item: ItemId(7),
        arguments: GenericArgs::EMPTY,
        signature,
    })?;
    assert_eq!(result, expected);
    let steps = substitution.steps;
    assert_eq!(substitution.apply(&mut store, item)?, expected);
    assert_eq!(substitution.steps, steps);
    let mut uses = Uses::default();
    uses.collect(&store, expected)?;
    assert_eq!(uses.indices, std::collections::HashSet::from([5]));
    Ok(())
}

#[test]
fn repeated_generic_type_edges_are_visited_once_per_binder() -> Result<(), &'static str> {
    let mut store = Store::default();
    let mut root = TypeId::primitive(Primitive::U8);
    for index in 0..4096 {
        let args = store.intern_arguments(&[
            Argument::Type(root),
            Argument::Type(root),
            Argument::Lifetime(RegionId::bound(0, 0)?),
        ])?;
        root = store.intern(Kind::Adt(ItemId(index), args))?;
    }
    let mut substitution = Bound::new(vec![RegionId::STATIC]);
    let result = substitution.apply(&mut store, root)?;
    assert_eq!(store.escaping_binders(result), Some(0));
    assert_eq!(store.has_free_regions(result, 0), Some(true));
    assert!(substitution.steps < 4096 * 12);
    let mut uses = Uses::default();
    uses.collect(&store, root)?;
    assert_eq!(uses.indices, std::collections::HashSet::from([0]));
    assert!(uses.steps < 4096 * 8);
    Ok(())
}

#[test]
fn function_item_arguments_are_outside_the_signature_binder() -> Result<(), &'static str> {
    let mut store = Store::default();
    let region = RegionId::bound(0, 0)?;
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let parameters = store.intern_list(&[reference])?;
    let signature = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: reference,
        unsafe_: false,
    })?;
    let arguments =
        store.intern_arguments(&[Argument::Type(reference), Argument::Lifetime(region)])?;
    let item = store.intern(Kind::FunctionItem {
        item: ItemId(8),
        arguments,
        signature,
    })?;
    let mut substitution = Bound::new(vec![RegionId(21)]);
    let substituted = substitution.apply(&mut store, item)?;
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(21),
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let arguments =
        store.intern_arguments(&[Argument::Type(reference), Argument::Lifetime(RegionId(21))])?;
    assert_eq!(
        store.kind(substituted),
        Some(Kind::FunctionItem {
            item: ItemId(8),
            arguments,
            signature
        })
    );
    assert_eq!(store.escaping_binders(substituted), Some(0));
    Ok(())
}

#[test]
fn wide_generic_arguments_cannot_bypass_traversal_budgets() -> Result<(), &'static str> {
    let mut store = Store::default();
    let mut arguments = vec![Argument::Lifetime(RegionId::bound(0, 0)?); 1_000_001];
    arguments[0] = Argument::Type(TypeId::UNIT);
    let arguments = store.intern_arguments(&arguments)?;
    let ty = store.intern(Kind::Adt(ItemId(0), arguments))?;
    assert!(
        Bound::new(vec![RegionId::STATIC])
            .apply(&mut store, ty)
            .is_err()
    );
    assert!(Uses::default().collect(&store, ty).is_err());
    Ok(())
}

#[test]
fn declaration_fields_cannot_be_reused_as_generic_instance_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let item = ItemId(1);
    let args = store.intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U8))])?;
    let ty = store.intern(Kind::Adt(item, args))?;
    let fields = store.intern_list(&[])?;
    let mut definitions = Definitions::default();
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    assert!(Layouts::default().demand(&store, &definitions, ty).is_err());
    assert!(
        Copies::default()
            .demand(&store, &definitions, &HashMap::new(), ty)
            .is_err()
    );
    assert!(
        Equality::default()
            .demand(&store, &definitions, &HashMap::new(), ty)
            .is_err()
    );
    assert!(
        NeedsDrop::new([item])
            .demand(&store, &definitions, ty)
            .is_err()
    );
    let ordinary = store.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
    assert!(
        Layouts::default()
            .demand(&store, &definitions, ordinary)
            .is_ok()
    );
    Ok(())
}
