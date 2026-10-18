//! Shared list summaries retain their binder depth across different owning types.
use super::{Captures, Slots};
use crate::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::Argument,
    parameters::predicates::{Bound, Predicate},
    traits::Application,
};
type Result = std::result::Result<(), &'static str>;

#[test]
fn wide_generic_arguments_are_summarized_once_across_nominals_and_traits() -> Result {
    for width in [16_u16, 64, 256, 1024] {
        let mut types = Store::default();
        let values = (0..width)
            .map(|slot| Ok(Argument::Lifetime(RegionId::bound(0, slot)?)))
            .collect::<std::result::Result<Vec<_>, &'static str>>()?;
        let arguments = types.intern_arguments(&values)?;
        let mut slots = Slots::default();
        for index in 0..width {
            let item = ItemId(u32::from(index));
            let nominal = types.intern(Kind::Adt(item, arguments))?;
            assert_eq!(slots.collect(&types, nominal)?, u32::from(width));
            let application = types.intern_trait_application(Application { item, arguments })?;
            assert_eq!(
                slots.predicate(
                    &types,
                    Predicate {
                        ty: TypeId::UNIT,
                        bound: Bound::Trait(application)
                    }
                )?,
                u32::from(width)
            );
        }
        assert!(slots.steps < 5 * usize::from(width));
        eprintln!("slot groups width={width} steps={}", slots.steps);
    }
    Ok(())
}

#[test]
fn callable_parameter_lists_share_summaries_but_never_scope_coordinates() -> Result {
    let mut types = Store::default();
    let mut parameters = Vec::new();
    for (depth, slot) in [(0, 9), (1, 3), (2, 7)] {
        parameters.push(types.intern(Kind::Reference {
            region: RegionId::bound(depth, slot)?,
            mutable: false,
            referent: TypeId::UNIT,
        })?);
    }
    let parameters = types.intern_list(&parameters)?;
    let mut slots = Slots::default();
    let mut captures = Captures::new(1);
    for primitive in [
        Primitive::U8,
        Primitive::U16,
        Primitive::U32,
        Primitive::U64,
    ] {
        let callable = types.intern(Kind::Function {
            bound_regions: 10,
            parameters,
            result: TypeId::primitive(primitive),
            unsafe_: false,
        })?;
        assert_eq!(slots.collect_inside(&types, callable, 0)?, 4);
        assert_eq!(slots.collect_inside(&types, callable, 1)?, 8);
        assert_eq!(slots.collect_inside(&types, callable, 2)?, 0);
        assert!(captures.collect_inside(&types, callable, 0)?);
        assert!(captures.collect_inside(&types, callable, 1)?);
        assert!(!captures.collect_inside(&types, callable, 2)?);
    }
    assert!(slots.steps < 60);
    Ok(())
}
