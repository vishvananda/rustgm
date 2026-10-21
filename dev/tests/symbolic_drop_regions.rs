use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    aggregate::Definitions,
    arguments::ConstKind,
    destruction::regions::Regions,
    parameters::{
        Parameter, ParameterId, ParameterKind,
        environment::{Abilities, Environment},
        predicates::{Bound, Predicate},
    },
};
use std::sync::Arc;

fn parameter(types: &mut Store, owner: u32) -> Result<(ParameterId, TypeId), &'static str> {
    let id = types.intern_parameter(Parameter {
        owner: ItemId(owner),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, types.intern(Kind::Parameter(id))?))
}
#[test]
fn owned_components_obey_copy_assumptions_without_reverse_inference() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (a, first) = parameter(&mut types, 0)?;
    let (b, second) = parameter(&mut types, 1)?;
    let fields = types.intern_list(&[first, second])?;
    let tuple = types.intern(Kind::Tuple(fields))?;
    let parameters = [(a, Abilities::default()), (b, Abilities::default())];
    let base = Regions::new([])?;
    let definitions = Definitions::default();
    let mut plain = base.scoped(Arc::new(Environment::new(&types, &parameters)?));
    assert_eq!(
        plain.symbolic(&types, &definitions, tuple)?.parameters,
        &[a, b]
    );
    assert!(plain.get(&types, &definitions, tuple).is_err());
    let environment = Environment::new(&types, &parameters)?.with_predicates(
        &types,
        &[Predicate {
            ty: tuple,
            bound: Bound::Copy,
        }],
    )?;
    let mut copy_tuple = base.scoped(Arc::new(environment));
    assert!(copy_tuple.symbolic(&types, &definitions, tuple)?.is_empty());
    assert_eq!(
        copy_tuple.symbolic(&types, &definitions, first)?.parameters,
        &[a]
    );
    let environment = Environment::new(
        &types,
        &[(
            a,
            Abilities {
                copy: true,
                clone_: true,
            },
        )],
    )?;
    let mut copy = base.scoped(Arc::new(environment));
    assert!(copy.symbolic(&types, &definitions, first)?.is_empty());
    assert!(copy.symbolic(&types, &definitions, second).is_err());
    let steps = copy.steps;
    assert!(copy.symbolic(&types, &definitions, second).is_err());
    assert_eq!(copy.steps, steps);
    assert!(copy.symbolic(&types, &definitions, first)?.is_empty());
    Ok(())
}
#[test]
fn array_bounds_and_nonowning_edges_do_not_expand_drop_components() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let const_parameter = types.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let length = types.intern_const(ConstKind::Parameter(const_parameter))?;
    let unknown = types.intern(Kind::ArrayTerm {
        element: ty,
        length,
    })?;
    let huge = types.intern(Kind::Array {
        element: ty,
        length: u64::MAX,
    })?;
    let empty = types.intern(Kind::Array {
        element: ty,
        length: 0,
    })?;
    let reference = types.intern(Kind::Reference {
        region: RegionId(1),
        mutable: true,
        referent: ty,
    })?;
    let pointer = types.intern(Kind::Pointer {
        mutable: true,
        pointee: ty,
    })?;
    let list = types.intern_list(&[ty])?;
    let function = types.intern(Kind::Function {
        bound_regions: 0,
        parameters: list,
        result: ty,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let definitions = Definitions::default();
    let mut base = Regions::new([])?;
    assert!(base.symbolic(&types, &definitions, unknown).is_err());
    let mut scoped = base.scoped(Arc::new(Environment::new(
        &types,
        &[(id, Abilities::default())],
    )?));
    for root in [unknown, huge, ty] {
        assert_eq!(
            scoped.symbolic(&types, &definitions, root)?.parameters,
            &[id]
        );
    }
    for root in [empty, reference, pointer, function] {
        assert!(scoped.symbolic(&types, &definitions, root)?.is_empty());
    }
    assert!(scoped.steps < 80);
    Ok(())
}
#[test]
fn deep_shared_drop_shapes_are_iterative_and_cached() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, mut root) = parameter(&mut types, 0)?;
    for _ in 0..20_000 {
        let fields = types.intern_list(&[root, root])?;
        root = types.intern(Kind::Tuple(fields))?;
    }
    let definitions = Definitions::default();
    let mut query = Regions::new([])?.scoped(Arc::new(Environment::new(
        &types,
        &[(id, Abilities::default())],
    )?));
    assert_eq!(
        query.symbolic(&types, &definitions, root)?.parameters,
        &[id]
    );
    assert!(query.steps < 400_000);
    let steps = query.steps;
    assert_eq!(
        query.symbolic(&types, &definitions, root)?.parameters,
        &[id]
    );
    assert_eq!(query.steps, steps);
    Ok(())
}
