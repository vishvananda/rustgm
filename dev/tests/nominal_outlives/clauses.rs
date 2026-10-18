//! Nominal implication filters whole quantified obligations before decomposing types.
use super::{
    Argument, Bound, Bounds, Fixture, ItemId, Kind, Predicate, RegionId, TestResult, TypeId,
};
use rustgm::types::{
    GenericArgs,
    copy::Properties,
    outlives::Projection,
    parameters::clauses::{Clause, Quantified, QuantifiedId},
    projection::{Application, Schema},
    traits,
};

fn clause(
    fixture: &mut Fixture,
    count: u32,
    ty: TypeId,
    bound: Bound,
) -> Result<Clause, &'static str> {
    Ok(Clause::ForAll(fixture.types.intern_quantified(
        Quantified {
            bound_regions: count,
            predicate: Predicate { ty, bound },
        },
    )?))
}
fn projected(fixture: &mut Fixture, subject: TypeId) -> Result<TypeId, &'static str> {
    fixture.types.declare_projection(
        ItemId(100),
        Schema {
            trait_item: ItemId(101),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let trait_application = fixture
        .types
        .intern_trait_application(traits::Application {
            item: ItemId(101),
            arguments: GenericArgs::EMPTY,
        })?;
    fixture.types.intern(Kind::Projection(Application {
        item: ItemId(100),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))
}

#[test]
fn local_subjects_and_targets_supply_no_partial_nominal_implications() -> TestResult {
    let mut fixture = Fixture::default();
    let (_, t) = fixture.parameter(0, 0)?;
    let local = RegionId::bound(0, 0)?;
    let captured = RegionId::bound(1, 1)?;
    let reference = fixture.reference(local, t)?;
    let fields = fixture.types.intern_list(&[reference, t])?;
    let tuple = fixture.types.intern(Kind::Tuple(fields))?;
    fixture.define(0, &[t])?;
    for (ty, shorter) in [
        (t, local),
        (reference, local),
        (reference, RegionId::STATIC),
        (reference, captured),
        (tuple, local),
        (tuple, RegionId::STATIC),
    ] {
        let value = clause(&mut fixture, 1, ty, Bound::Outlives(shorter))?;
        let bounds = Bounds::infer_with_clauses(
            &mut fixture.types,
            &fixture.definitions,
            &[],
            [(ItemId(0), value)],
        )?;
        assert_eq!(bounds.get(ItemId(0)), Some([].as_slice()));
        assert_eq!(bounds.parameters(ItemId(0)), Some([].as_slice()));
        assert!(bounds.projections(ItemId(0)).is_empty());
    }
    Ok(())
}

#[test]
fn empty_and_unused_binders_lift_the_whole_independent_predicate() -> TestResult {
    for count in [0, 1, 7] {
        let mut fixture = Fixture::default();
        let (id, t) = fixture.parameter(0, 2)?;
        fixture.define(0, &[t])?;
        let a = RegionId::bound(1, 0)?;
        let b = RegionId::bound(1, 1)?;
        let subject = fixture.reference(a, t)?;
        let value = clause(&mut fixture, count, subject, Bound::Outlives(b))?;
        let bounds = Bounds::infer_with_clauses(
            &mut fixture.types,
            &fixture.definitions,
            &[],
            [(ItemId(0), value)],
        )?;
        let a = RegionId::bound(0, 0)?;
        let b = RegionId::bound(0, 1)?;
        assert_eq!(bounds.get(ItemId(0)), Some([(a, b)].as_slice()));
        assert_eq!(bounds.parameters(ItemId(0)), Some([(id, b)].as_slice()));
    }
    Ok(())
}

#[test]
fn quantified_trait_requirements_do_not_block_independent_field_lifetimes() -> TestResult {
    let mut fixture = Fixture::default();
    let (id, t) = fixture.parameter(0, 1)?;
    let a = RegionId::bound(0, 0)?;
    let field = fixture.reference(a, t)?;
    fixture.define(0, &[field])?;
    let arguments = fixture.types.intern_arguments(&[Argument::Lifetime(a)])?;
    let application = fixture
        .types
        .intern_trait_application(traits::Application {
            item: ItemId(99),
            arguments,
        })?;
    let value = clause(&mut fixture, 1, t, Bound::Trait(application))?;
    let bounds = Bounds::infer_with_clauses(
        &mut fixture.types,
        &fixture.definitions,
        &[],
        [(ItemId(0), value)],
    )?;
    assert_eq!(bounds.parameters(ItemId(0)), Some([(id, a)].as_slice()));
    assert!(bounds.projections(ItemId(0)).is_empty());
    Ok(())
}

#[test]
fn lifted_nominal_contracts_propagate_through_complete_argument_substitutions() -> TestResult {
    let mut fixture = Fixture::default();
    let (_, t) = fixture.parameter(0, 1)?;
    fixture.define(0, &[t])?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let (id, u) = fixture.parameter(1, 2)?;
    let argument = fixture.reference(b, u)?;
    let nested = fixture.nominal(0, &[Argument::Lifetime(a), Argument::Type(argument)])?;
    fixture.define(1, &[nested])?;
    let value = clause(&mut fixture, 1, t, Bound::Outlives(RegionId::bound(1, 0)?))?;
    let bounds = Bounds::infer_with_clauses(
        &mut fixture.types,
        &fixture.definitions,
        &[],
        [(ItemId(0), value)],
    )?;
    assert_eq!(bounds.get(ItemId(1)), Some([(b, a)].as_slice()));
    let actual = bounds.parameters(ItemId(1)).ok_or("missing nominal")?;
    assert_eq!(actual.len(), 2);
    assert!(actual.contains(&(id, a)));
    assert!(actual.contains(&(id, b)));
    assert_eq!(fixture.definitions.instantiation_steps(), 0);
    Ok(())
}

#[test]
fn independent_projections_stay_opaque_and_local_projections_do_not_imply_input_bounds()
-> TestResult {
    for dependent in [false, true] {
        let mut fixture = Fixture::default();
        let (_, t) = fixture.parameter(0, 1)?;
        fixture.define(0, &[t])?;
        let subject = fixture.reference(RegionId::bound(u16::from(!dependent), 0)?, t)?;
        let projection = projected(&mut fixture, subject)?;
        let value = clause(
            &mut fixture,
            1,
            projection,
            Bound::Outlives(RegionId::STATIC),
        )?;
        let bounds = Bounds::infer_with_clauses(
            &mut fixture.types,
            &fixture.definitions,
            &[],
            [(ItemId(0), value)],
        )?;
        assert_eq!(bounds.get(ItemId(0)), Some([].as_slice()));
        assert_eq!(bounds.parameters(ItemId(0)), Some([].as_slice()));
        if dependent {
            assert!(bounds.projections(ItemId(0)).is_empty());
        } else {
            let subject = fixture.reference(RegionId::bound(0, 0)?, t)?;
            let ty = projected(&mut fixture, subject)?;
            assert_eq!(
                bounds.projections(ItemId(0)),
                [(Projection { ty, depth: 0 }, RegionId::STATIC)]
            );
        }
    }
    Ok(())
}

#[test]
fn nested_callable_locals_survive_removing_an_unused_outer_quantifier() -> TestResult {
    let mut fixture = Fixture::default();
    fixture.define(0, &[])?;
    let local = fixture.reference(RegionId::bound(0, 0)?, TypeId::UNIT)?;
    let capture = fixture.reference(RegionId::bound(2, 0)?, TypeId::UNIT)?;
    let parameters = fixture.types.intern_list(&[local, capture])?;
    let function = fixture.types.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
    })?;
    let projection = projected(&mut fixture, function)?;
    let value = clause(
        &mut fixture,
        1,
        projection,
        Bound::Outlives(RegionId::STATIC),
    )?;
    let bounds = Bounds::infer_with_clauses(
        &mut fixture.types,
        &fixture.definitions,
        &[],
        [(ItemId(0), value)],
    )?;
    let capture = fixture.reference(RegionId::bound(1, 0)?, TypeId::UNIT)?;
    let parameters = fixture.types.intern_list(&[local, capture])?;
    let function = fixture.types.intern(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
    })?;
    let ty = projected(&mut fixture, function)?;
    assert_eq!(
        bounds.projections(ItemId(0)),
        [(Projection { ty, depth: 0 }, RegionId::STATIC)]
    );
    Ok(())
}

#[test]
fn malformed_quantifier_id_fails_instead_of_omitting_a_requirement() -> TestResult {
    let mut fixture = Fixture::default();
    fixture.define(0, &[])?;
    assert!(
        Bounds::infer_with_clauses(
            &mut fixture.types,
            &fixture.definitions,
            &[],
            [(ItemId(0), Clause::ForAll(QuantifiedId(42)))],
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn implied_projection_lifting_shares_captured_tails_and_repeated_clauses() -> TestResult {
    let mut observations = Vec::new();
    for width in [16, 32, 64, 128, 256, 512, 1024] {
        let mut fixture = Fixture::default();
        fixture.define(0, &[])?;
        let mut tail = TypeId::UNIT;
        let mut clauses = Vec::new();
        for _ in 0..width {
            tail = fixture.reference(RegionId::bound(1, 0)?, tail)?;
            let projection = projected(&mut fixture, tail)?;
            let value = clause(
                &mut fixture,
                1,
                projection,
                Bound::Outlives(RegionId::STATIC),
            )?;
            clauses.extend([(ItemId(0), value); 2]);
        }
        let before = fixture.types.len();
        let bounds =
            Bounds::infer_with_clauses(&mut fixture.types, &fixture.definitions, &[], clauses)?;
        let growth = fixture.types.len() - before;
        assert_eq!(bounds.projections(ItemId(0)).len(), width);
        assert_eq!(growth, 2 * width);
        assert!(bounds.steps <= 80 * width + 32, "{}", bounds.steps);
        assert_eq!(fixture.definitions.instantiation_steps(), 0);
        println!(
            "implied-clause width={width} steps={} types={growth}",
            bounds.steps
        );
        observations.push((width, bounds.steps));
    }
    for pair in observations.windows(2) {
        assert!(pair[1].1 <= 2 * pair[0].1 + 32, "{pair:?}");
    }
    Ok(())
}
