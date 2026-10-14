use rustgm::types::{
    ItemId, Kind, Primitive, Store, TypeId,
    aggregate::Definitions,
    arguments::Argument,
    copy::Copies,
    parameters::{
        Parameter, ParameterId, ParameterKind,
        environment::{Abilities, Environment},
        predicates::{Bound, Predicate},
        shape::Shapes,
    },
};
use std::{collections::HashMap, sync::Arc};

fn parameter(types: &mut Store, owner: u32) -> Result<(ParameterId, TypeId), &'static str> {
    let id = types.intern_parameter(Parameter {
        owner: ItemId(owner),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    Ok((id, types.intern(Kind::Parameter(id))?))
}

#[test]
fn composite_assumptions_do_not_imply_member_bounds() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let tuple = types.intern_list(&[ty])?;
    let tuple = types.intern(Kind::Tuple(tuple))?;
    for bound in [Bound::Copy, Bound::Clone, Bound::Sized] {
        let environment = Arc::new(
            Environment::new(&types, &[(id, Abilities::default())])?
                .with_predicates(&types, &[Predicate { ty: tuple, bound }])?,
        );
        let mut copies = Copies::in_environment(Arc::clone(&environment));
        let mut shapes = Shapes::new(environment);
        let properties = copies.demand(&types, &Definitions::default(), &HashMap::new(), tuple)?;
        assert_eq!(properties.copy, bound == Bound::Copy);
        assert_eq!(properties.clone_, bound != Bound::Sized);
        assert!(shapes.demand(&types, tuple)?.sized);
        assert!(!shapes.demand(&types, ty)?.sized);
        assert!(shapes.demand(&types, ty)?.well_formed);
        let member = copies.demand(&types, &Definitions::default(), &HashMap::new(), ty)?;
        assert!(!member.copy && !member.clone_);
    }
    Ok(())
}

#[test]
fn sized_supertraits_do_not_mutate_canonical_shapes() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let array = types.intern(Kind::Array {
        element: ty,
        length: 2,
    })?;
    for abilities in [
        Abilities {
            copy: true,
            clone_: false,
        },
        Abilities {
            copy: false,
            clone_: true,
        },
    ] {
        let environment = Environment::new(&types, &[(id, abilities)])?;
        assert!(environment.has_shape_assumptions());
        let mut shapes = Shapes::new(Arc::new(environment));
        assert!(shapes.demand(&types, array)?.well_formed);
        assert!(shapes.demand(&types, array)?.sized);
    }
    let environment = Environment::new(&types, &[(id, Abilities::default())])?;
    assert!(!environment.has_shape_assumptions());
    let mut unbounded = Shapes::new(Arc::new(environment));
    assert!(!unbounded.demand(&types, array)?.well_formed);
    assert!(!unbounded.demand(&types, array)?.sized);
    assert_eq!(types.is_sized(ty), Some(false));
    assert_eq!(types.is_well_formed(array), Some(false));
    Ok(())
}

#[test]
fn assumed_sizedness_cannot_hide_malformed_structure() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let array = types.intern(Kind::Array {
        element: ty,
        length: 0,
    })?;
    let tuple = types.intern_list(&[ty, TypeId::UNIT])?;
    let tuple = types.intern(Kind::Tuple(tuple))?;
    for root in [array, tuple] {
        let environment = Environment::new(&types, &[(id, Abilities::default())])?
            .with_predicates(
                &types,
                &[Predicate {
                    ty: root,
                    bound: Bound::Sized,
                }],
            )?;
        let mut shapes = Shapes::new(Arc::new(environment));
        let shape = shapes.demand(&types, root)?;
        assert!(shape.sized);
        assert!(!shape.well_formed);
    }
    Ok(())
}

#[test]
fn foreign_parameters_are_rejected_before_assumptions_can_hide_them() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let (_, foreign) = parameter(&mut types, 1)?;
    let list = types.intern_list(&[foreign])?;
    let root = types.intern(Kind::Tuple(list))?;
    for subject in [foreign, root, TypeId(u32::MAX)] {
        assert!(
            Environment::new(&types, &[(id, Abilities::default())])?
                .with_predicates(
                    &types,
                    &[Predicate {
                        ty: subject,
                        bound: Bound::Copy
                    }]
                )
                .is_err()
        );
    }
    let environment = Environment::new(&types, &[(id, Abilities::default())])?;
    let mut shapes = Shapes::new(Arc::new(environment));
    for _ in 0..2 {
        assert!(shapes.demand(&types, root).is_err());
    }
    assert!(shapes.demand(&types, ty)?.well_formed);
    assert!(
        shapes
            .demand(&types, TypeId::primitive(Primitive::U8))?
            .sized
    );
    Ok(())
}

#[test]
fn an_explicit_nominal_copy_assumption_needs_no_field_inspection() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let arguments = types.intern_arguments(&[Argument::Type(ty)])?;
    let nominal = types.intern(Kind::Adt(ItemId(1), arguments))?;
    let environment = Environment::new(&types, &[(id, Abilities::default())])?.with_predicates(
        &types,
        &[Predicate {
            ty: nominal,
            bound: Bound::Copy,
        }],
    )?;
    let mut copies = Copies::in_environment(Arc::new(environment));
    let properties = copies.demand(&types, &Definitions::default(), &HashMap::new(), nominal)?;
    assert!(properties.copy && properties.clone_);
    assert!(
        !copies
            .demand(&types, &Definitions::default(), &HashMap::new(), ty)?
            .copy
    );
    Ok(())
}

#[test]
fn shared_shape_graphs_use_iterative_cached_queries() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let environment = Environment::new(&types, &[(id, Abilities::default())])?.with_predicates(
        &types,
        &[Predicate {
            ty,
            bound: Bound::Sized,
        }],
    )?;
    let mut root = ty;
    for _ in 0..4096 {
        let list = types.intern_list(&[root, root])?;
        root = types.intern(Kind::Tuple(list))?;
    }
    let mut shapes = Shapes::new(Arc::new(environment));
    for _ in 0..4096 {
        let shape = shapes.demand(&types, root)?;
        assert!(shape.sized && shape.well_formed);
    }
    Ok(())
}
