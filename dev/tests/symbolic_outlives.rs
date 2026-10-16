use rustgm::{
    borrow::regions::{constraints::Constraints, graph::Error},
    types::{
        ItemId, Kind, RegionId, Store, TypeId,
        callable::Templates,
        outlives::{Bounds, LiveRegions, Queries},
        parameters::{Parameter, ParameterId, ParameterKind},
        variance::Variances,
    },
};

fn parameter(types: &mut Store, owner: u32) -> Result<(ParameterId, TypeId), &'static str> {
    let id = types.intern_parameter(Parameter {
        owner: ItemId(owner),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, types.intern(Kind::Parameter(id))?))
}
fn reference(
    types: &mut Store,
    region: RegionId,
    referent: TypeId,
) -> Result<TypeId, &'static str> {
    types.intern(Kind::Reference {
        mutable: false,
        region,
        referent,
    })
}

#[test]
fn symbolic_components_do_not_depend_on_the_contains_regions_flag() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let (other, second) = parameter(&mut types, 1)?;
    assert_eq!(types.contains_regions(ty), Some(false));
    let empty = types.intern(Kind::Array {
        element: ty,
        length: 0,
    })?;
    let list = types.intern_list(&[ty, second, ty])?;
    let tuple = types.intern(Kind::Tuple(list))?;
    let a = RegionId::bound(0, 0)?;
    let borrowed = reference(&mut types, a, tuple)?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let mut queries = Queries::new(&types, &bounds, &templates);
    for root in [ty, empty] {
        let parts = queries.components(root)?;
        assert!(parts.scalars.regions.is_empty());
        assert_eq!(parts.scalars.parameters, &[id]);
    }
    let parts = queries.components(tuple)?;
    assert_eq!(parts.scalars.parameters, &[id, other]);
    let parts = queries.components(borrowed)?;
    assert_eq!(parts.scalars.regions, &[a]);
    assert_eq!(parts.scalars.parameters, &[id, other]);
    let requirements = queries.symbolic_requirements(borrowed)?;
    assert!(requirements.scalars.regions.is_empty());
    assert_eq!(requirements.scalars.parameters, &[(id, a), (other, a)]);
    // Closed and live-value consumers must not silently erase a generic proof.
    assert!(queries.requirements(borrowed).is_err());
    assert!(LiveRegions::default().get(&types, borrowed).is_err());
    Ok(())
}

#[test]
fn nested_callable_scopes_export_only_free_requirements() -> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, ty) = parameter(&mut types, 0)?;
    let free = reference(&mut types, RegionId::bound(1, 0)?, ty)?;
    let local = reference(&mut types, RegionId::bound(0, 0)?, ty)?;
    let inputs = types.intern_list(&[free, local])?;
    let function = types.intern(Kind::Function {
        bound_regions: 1,
        parameters: inputs,
        result: TypeId::UNIT,
        unsafe_: false,
    })?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let mut queries = Queries::new(&types, &bounds, &templates);
    let requirements = queries.symbolic_requirements(function)?;
    assert_eq!(
        requirements.scalars.parameters,
        &[(id, RegionId::bound(0, 0)?)]
    );
    let components = queries.components(function)?;
    assert_eq!(components.scalars.regions, &[RegionId::bound(0, 0)?]);
    assert_eq!(components.scalars.parameters, &[id]);
    Ok(())
}

#[test]
fn rigid_parameter_proofs_keep_scope_direction_and_negative_results_isolated()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let (id, _) = parameter(&mut types, 0)?;
    let (foreign, _) = parameter(&mut types, 1)?;
    let variances = Variances::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    for (assume, forward) in [(false, true), (true, false), (true, true)] {
        let mut constraints = Constraints::new(&types, &variances, 2, 10_000)?;
        constraints.parameter(id)?;
        assert_eq!(constraints.parameter(id), Err(Error::Identity));
        assert_eq!(
            constraints.assume_parameter(foreign, a),
            Err(Error::Identity)
        );
        if assume {
            constraints.assume_parameter(id, a)?;
        }
        constraints.assume(if forward { a } else { b }, if forward { b } else { a })?;
        constraints.parameter_outlives(id, b, 37)?;
        let result = constraints.prove();
        assert_eq!(result.is_ok(), assume && forward);
        if let Ok(proof) = result {
            assert_eq!(proof.identities.parameters().len(), 1);
            assert_eq!(proof.identities.universals().len(), 2);
        } else if let Err(Error::Unsatisfied { source, .. }) = result {
            assert_eq!(source, 37);
        } else {
            return Err("unexpected proof error".into());
        }
    }
    Ok(())
}

#[test]
fn obligations_cannot_turn_one_parameter_into_an_assumption_for_another()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let (a, _) = parameter(&mut types, 0)?;
    let (b, _) = parameter(&mut types, 1)?;
    let variances = Variances::default();
    let mut constraints = Constraints::new(&types, &variances, 0, 10_000)?;
    constraints.parameter(a)?;
    constraints.parameter(b)?;
    constraints.assume_parameter(a, RegionId::STATIC)?;
    let inferred = RegionId(1);
    constraints.variable(inferred)?;
    constraints.parameter_outlives(b, inferred, 1)?;
    constraints.outlives(inferred, RegionId::STATIC, 2)?;
    assert!(matches!(
        constraints.solve(),
        Err(Error::Unsatisfied { .. })
    ));
    Ok(())
}

#[test]
fn shared_generic_graphs_are_iterative_cached_and_recover_after_invalid_queries()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let (id, mut root) = parameter(&mut types, 0)?;
    let a = RegionId::bound(0, 0)?;
    let borrowed = reference(&mut types, a, root)?;
    root = borrowed;
    for _ in 0..10_000 {
        let list = types.intern_list(&[root, root])?;
        root = types.intern(Kind::Tuple(list))?;
    }
    let bounds = Bounds::default();
    let templates = Templates::default();
    let mut queries = Queries::new(&types, &bounds, &templates);
    for _ in 0..100 {
        assert_eq!(
            queries.symbolic_requirements(root)?.scalars.parameters,
            &[(id, a)]
        );
    }
    assert!(queries.symbolic_requirements(TypeId(u32::MAX)).is_err());
    assert_eq!(
        queries.symbolic_requirements(borrowed)?.scalars.parameters,
        &[(id, a)]
    );
    assert!(queries.components(TypeId(u32::MAX)).is_err());
    assert!(
        queries
            .components(TypeId::UNIT)?
            .scalars
            .parameters
            .is_empty()
    );
    Ok(())
}
