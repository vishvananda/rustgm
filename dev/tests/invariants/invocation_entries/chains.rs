//! Nested receiver prefixes preserve physical capability and independent binders.
use super::*;
use crate::types::{closure::Adapter, instance::Instance};
fn chain(types: &mut Store, leaf: TypeId, levels: &[(bool, RegionId)]) -> Result<TypeId> {
    let mut ty = leaf;
    for &(mutable, region) in levels.iter().rev() {
        ty = types.intern(Type::Reference {
            referent: ty,
            mutable,
            region,
        })?;
    }
    Ok(ty)
}
#[test]
fn every_reference_level_requires_its_own_blanket_permission() -> Result {
    let (mut types, _, actual) = prefix_fixture(false)?;
    let instance = Instance::closure(&types, actual)?;
    for (levels, requested, accepted) in [
        ([false, false], CallTrait::Fn, true),
        ([false, false], CallTrait::FnMut, true),
        ([true, false], CallTrait::FnOnce, true),
        ([true, true], CallTrait::FnMut, true),
        ([true, false], CallTrait::Fn, false),
        ([false, true], CallTrait::FnOnce, false),
        ([false, true], CallTrait::FnMut, false),
    ] {
        let levels = levels.map(|mutable| (mutable, RegionId(13)));
        let receiver = chain(&mut types, actual, &levels)?;
        assert_eq!(
            Adapter::for_receiver(&types, instance, receiver, requested).is_ok(),
            accepted
        );
    }
    Ok(())
}
#[test]
fn nested_prefixes_preserve_unused_source_slots_and_nested_callable_binders() -> Result {
    let (mut types, formal, actual) = prefix_fixture(false)?;
    let receiver = chain(&mut types, formal, &[(false, RegionId::bound(0, 3)?)])?;
    let signature = function(&mut types, &[receiver], 4)?;
    let canonical = Execution {
        self_ty: formal,
        signature,
        universals: 4,
        receiver: CallKind::Shared,
    };
    let levels = [
        (false, RegionId(7)),
        (false, RegionId(13)),
        (false, RegionId(13)),
        (false, RegionId(11)),
        (false, RegionId::STATIC),
    ];
    let receiver = chain(&mut types, actual, &levels)?;
    let adapter = Adapter::for_receiver(
        &types,
        Instance::closure(&types, actual)?,
        receiver,
        CallTrait::FnOnce,
    )?;
    let matched = borrowed::prepared(&mut types, canonical, 3, adapter, true)?;
    assert_eq!(matched.execution.universals, 5);
    assert!(matched.equalities.is_empty());
    let expected = [
        RegionId::bound(0, 0)?,
        RegionId::bound(0, 4)?,
        RegionId::bound(0, 4)?,
        RegionId::bound(0, 2)?,
        RegionId::STATIC,
    ];
    let mut ty = matched.execution.self_ty;
    for region in expected {
        let Some(Type::Reference {
            referent,
            mutable: false,
            region: actual,
        }) = types.kind(ty)
        else {
            return Err("nested reference shape changed");
        };
        assert_eq!(actual, region);
        ty = referent;
    }
    assert_eq!(ty, formal);
    Ok(())
}
#[test]
fn wide_reference_prefixes_allocate_only_distinct_new_images() -> Result {
    for width in [8u16, 32, 128] {
        let (types, formal, actual) = prefix_fixture(true)?;
        let mut regions = Vec::new();
        for index in 0..width {
            let region = RegionId(100 + u32::from(index));
            regions.extend([region, region]);
        }
        let bindings = matching::bind_instance(&types, 3, formal, actual)?;
        let debts = bindings.equalities.clone();
        let prefix = bindings.prefixes(&regions)?;
        assert_eq!(prefix.universals, 3 + u32::from(width));
        assert_eq!(prefix.equalities, debts);
        assert_eq!(prefix.regions.len(), usize::from(width) * 2);
        for (index, pair) in prefix.regions.as_chunks::<2>().0.iter().enumerate() {
            assert_eq!(
                *pair,
                [RegionId::bound(0, 3 + u16::try_from(index).map_err(|_| "prefix index")?)?; 2]
            );
        }
    }
    Ok(())
}
