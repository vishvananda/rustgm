//! Declaration abilities describe the result; formation remains the caller's duty.
use rustgm::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    aggregate::Definitions,
    copy::{Copies, Properties},
    destruction::{NeedsDrop, regions::Regions},
    projection::{Application, Schema},
    traits,
};
use std::collections::HashMap;

const BOTH: Properties = Properties {
    copy: true,
    clone_: true,
};
const CLONE: Properties = Properties {
    copy: false,
    clone_: true,
};

fn schema(copies: Properties) -> Schema {
    Schema {
        trait_item: ItemId(10),
        sized: true,
        copies,
    }
}

fn projected(
    store: &mut Store,
    item: u32,
    copies: Properties,
    subject: TypeId,
) -> Result<TypeId, &'static str> {
    store.declare_projection(ItemId(item), schema(copies))?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments: GenericArgs::EMPTY,
    })?;
    store.intern(Kind::Projection(Application {
        item: ItemId(item),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))
}

#[test]
fn only_copy_promises_remove_destruction_requirements() -> Result<(), &'static str> {
    let mut store = Store::default();
    let subject = store.intern(Kind::Reference {
        mutable: true,
        region: RegionId(1),
        referent: TypeId::UNIT,
    })?;
    let unknown = projected(&mut store, 11, Properties::default(), TypeId::UNIT)?;
    let clone = projected(&mut store, 12, CLONE, TypeId::UNIT)?;
    let copy = projected(&mut store, 13, BOTH, subject)?;
    let definitions = Definitions::default();
    let selected = HashMap::new();
    let mut copies = Copies::default();
    let mut drop = NeedsDrop::new([]);
    let mut regions = Regions::new([])?;
    assert!(
        copies
            .demand(&store, &definitions, &selected, unknown)
            .is_err()
    );
    assert_eq!(
        copies.demand(&store, &definitions, &selected, clone)?,
        CLONE
    );
    assert_eq!(copies.demand(&store, &definitions, &selected, copy)?, BOTH);
    assert!(
        !copies
            .demand(&store, &definitions, &selected, subject)?
            .copy
    );
    for ty in [unknown, clone] {
        assert!(drop.demand(&store, &definitions, ty).is_err());
        assert!(regions.symbolic(&store, &definitions, ty).is_err());
    }
    assert!(!drop.demand(&store, &definitions, copy)?);
    assert!(regions.symbolic(&store, &definitions, copy)?.is_empty());
    for (second, expected) in [(copy, BOTH), (clone, CLONE)] {
        let fields = store.intern_list(&[copy, second])?;
        let tuple = store.intern(Kind::Tuple(fields))?;
        assert_eq!(
            copies.demand(&store, &definitions, &selected, tuple)?,
            expected
        );
        assert_eq!(
            drop.demand(&store, &definitions, tuple).is_ok(),
            expected.copy
        );
        assert_eq!(
            regions.symbolic(&store, &definitions, tuple).is_ok(),
            expected.copy
        );
    }
    Ok(())
}

#[test]
fn declarations_cannot_change_under_cached_property_queries() -> Result<(), &'static str> {
    let mut store = Store::default();
    let unknown = projected(&mut store, 11, Properties::default(), TypeId::UNIT)?;
    let mut copies = Copies::default();
    let definitions = Definitions::default();
    let selected = HashMap::new();
    assert!(
        copies
            .demand(&store, &definitions, &selected, unknown)
            .is_err()
    );
    assert!(store.declare_projection(ItemId(11), schema(BOTH)).is_err());
    assert!(
        Copies::default()
            .demand(&store, &definitions, &selected, unknown)
            .is_err()
    );
    for invalid in [
        schema(Properties {
            copy: true,
            clone_: false,
        }),
        Schema {
            sized: false,
            ..schema(CLONE)
        },
        Schema {
            sized: false,
            ..schema(BOTH)
        },
    ] {
        assert!(store.declare_projection(ItemId(12), invalid).is_err());
        assert!(store.projection_schema(ItemId(12)).is_none());
    }
    store.declare_projection(ItemId(12), schema(BOTH))?;
    store.declare_projection(ItemId(12), schema(BOTH))?;
    assert!(store.declare_projection(ItemId(12), schema(CLONE)).is_err());
    assert_eq!(store.projection_schema(ItemId(12)), Some(schema(BOTH)));
    Ok(())
}
