//! Capture input observation is sparse and excludes nested callable binders.
use super::*;
use crate::types::{Kind, Primitive, callable::Convention};
type Result<T = ()> = std::result::Result<T, &'static str>;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
fn reference(types: &mut Store, region: RegionId) -> Result<TypeId> {
    types.intern(Kind::Reference {
        mutable: false,
        region,
        referent: BYTE,
    })
}
fn function(types: &mut Store, parameter: TypeId, result: TypeId) -> Result<TypeId> {
    let parameters = types.intern_list(&[parameter])?;
    types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    })
}
#[test]
fn no_capture_and_scalar_fields_allocate_no_observer_or_region_index() -> Result {
    let mut types = Store::default();
    let empty = Inputs::collect(&mut types, &[])?;
    assert!(empty.regions.is_none());
    assert_eq!(empty.steps, 0);
    let ordinary = Inputs::collect(&mut types, &[BYTE; 128])?;
    assert!(ordinary.regions.is_none());
    assert_eq!(ordinary.steps, 128);
    for length in 0..4096 {
        types.intern(Kind::Array {
            element: BYTE,
            length,
        })?;
    }
    let unrelated = Inputs::collect(&mut types, &[BYTE; 128])?;
    assert!(unrelated.regions.is_none());
    assert_eq!(unrelated.steps, ordinary.steps);
    Ok(())
}
#[test]
fn nested_callable_local_names_are_excluded_but_captured_outer_names_survive() -> Result {
    let mut types = Store::default();
    let local = reference(&mut types, RegionId::bound(0, 0)?)?;
    let outer = reference(&mut types, RegionId::bound(1, 3)?)?;
    let pure = function(&mut types, local, local)?;
    let separate = Inputs::collect(&mut types, &[pure])?;
    assert!(separate.regions.is_none());
    assert_eq!(separate.steps, 1);
    let mixed = function(&mut types, local, outer)?;
    let incoming = Inputs::collect(&mut types, &[mixed])?;
    assert!(incoming.contains(RegionId::bound(0, 3)?));
    assert!(!incoming.contains(RegionId::bound(0, 0)?));
    assert_eq!(incoming.regions.as_ref().map(HashSet::len), Some(1));
    Ok(())
}
#[test]
fn repeated_fields_share_one_linear_diamond_observation() -> Result {
    for depth in [8, 32, 128] {
        let mut types = Store::default();
        let region = RegionId::bound(0, 0)?;
        let mut ty = reference(&mut types, region)?;
        for _ in 0..depth {
            let list = types.intern_list(&[ty, ty])?;
            ty = types.intern(Kind::Tuple(list))?;
        }
        for repeat in [8, 32, 128] {
            let before = types.len();
            let incoming = Inputs::collect(&mut types, &vec![ty; repeat])?;
            assert!(incoming.contains(region));
            assert_eq!(incoming.regions.as_ref().map(HashSet::len), Some(1));
            assert_eq!(types.len(), before);
            assert!(incoming.steps <= 32 * (depth + repeat) + 100);
            eprintln!(
                "capture input depth={depth} repeat={repeat} work={}",
                incoming.steps
            );
        }
    }
    Ok(())
}
#[test]
fn exhaustion_does_not_poison_another_body_observer() -> Result {
    let mut exhausted = Inputs {
        steps: 999_999,
        ..Inputs::default()
    };
    assert!(exhausted.charge(2).is_err());
    let ordinary = Inputs::collect(&mut Store::default(), &[])?;
    assert_eq!(ordinary.steps, 0);
    assert!(ordinary.regions.is_none());
    Ok(())
}
