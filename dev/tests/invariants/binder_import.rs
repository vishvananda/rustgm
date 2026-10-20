//! Inferred object metadata is context-local even when its numeric IDs collide.
use super::*;
use crate::types::{ItemId, Primitive, RegionId, TypeId, dynamic};

#[test]
fn dynamic_context_transfers_preserve_binders_and_inference_holes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let shape = store.intern_dynamic_shape(dynamic::Bounds {
        principal: Some(ItemId(0)),
        principal_arguments: 2,
        bindings: &[],
        auto_traits: &[],
    })?;
    for occupied in [0, 1, 8] {
        let mut source = Context::default();
        let mut target = Context::default();
        let lifetimes = source.intern_object_lifetimes(1, RegionId(17))?;
        for count in 0..occupied {
            target.intern_object_lifetimes(count, RegionId::STATIC)?;
        }
        let hole = source.variable(super::super::Domain::Any)?;
        let local = RegionId::bound(0, 0)?;
        let mut children = Vec::new();
        let arguments = source.argument_layout(
            &[Argument::Lifetime(local), Argument::Type(hole)],
            &mut children,
        )?;
        let root = source.construct(
            Constructor::Dynamic {
                shape,
                arguments,
                lifetimes,
            },
            &children,
        )?;
        let imported = target.import(&store, &mut source, root)?;
        let Constructor::Dynamic {
            lifetimes,
            arguments,
            ..
        } = target.head(&store, imported)?.ok_or("object head")?
        else {
            return Err("object constructor lost during import".into());
        };
        let metadata = target.object_lifetimes(lifetimes)?;
        assert_eq!(metadata.bound_regions, 1);
        assert_eq!(metadata.region, RegionId(17));
        let actual = target.application_arguments(imported, arguments, 0)?;
        let [Argument::Lifetime(region), Argument::Type(child)] = actual.as_slice() else {
            return Err("object argument order changed".into());
        };
        assert_eq!(*region, local);
        let byte = target.known(&store, TypeId::primitive(Primitive::U8))?;
        target.equate(&store, *child, byte)?;
        assert!(source.head(&store, hole)?.is_none());
        let ty = target.resolved(&mut store, imported)?;
        assert!(matches!(
            store.kind(ty),
            Some(crate::types::Kind::Dynamic {
                bound_regions: 1,
                region: RegionId(17),
                ..
            })
        ));
        let returned = source.import(&store, &mut target, imported)?;
        assert_eq!(source.resolved(&mut store, returned)?, ty);
    }
    Ok(())
}
