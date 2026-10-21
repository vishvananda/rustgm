//! Typed tag readiness blocks physical layout until every variant is published.
use super::{Definitions, Kind, Repr, Variant};
use crate::{
    abi::layout::Layouts,
    types::{GenericArgs, ItemId, Store},
};
#[test]
fn enum_layout_never_caches_provisional_discriminants() -> Result<(), String> {
    let mut types = Store::default();
    let fields = types.intern_list(&[])?;
    let item = ItemId(0);
    let ty = types.intern(crate::types::Kind::Adt(item, GenericArgs::EMPTY))?;
    let mut definitions = Definitions::default();
    definitions.insert(
        item,
        Kind::Enum,
        Repr::default(),
        &[
            Variant {
                fields,
                discriminant: 0,
            },
            Variant {
                fields,
                discriminant: 0,
            },
        ],
    )?;
    definitions.mark_discriminants_pending(item)?;
    for (ordinal, bits) in [(0, 3), (1, 7)] {
        let mut layouts = Layouts::default();
        let Err(error) = layouts.demand(&types, &definitions, ty) else {
            return Err("incomplete enum layout was accepted".into());
        };
        assert_eq!(
            error.message,
            "enum layout requires completed discriminants"
        );
        assert!(layouts.is_empty());
        definitions.set_discriminant(item, ordinal, bits)?;
        assert!(definitions.discriminant_ready(item, ordinal));
    }
    let mut layouts = Layouts::default();
    layouts
        .demand(&types, &definitions, ty)
        .map_err(|e| e.to_string())?;
    assert!(!layouts.is_empty());
    assert_eq!(definitions.discriminant_work(), (2, 2));
    definitions.set_discriminant(item, 1, 7)?;
    assert_eq!(definitions.discriminant_work(), (2, 2));
    Ok(())
}
