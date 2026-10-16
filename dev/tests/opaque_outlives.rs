//! Scoped associated bounds prove outputs without granting their inputs permissions.
use rustgm::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    callable::Templates,
    copy::Properties,
    outlives::{Bounds, Queries},
    parameters::{Parameter, ParameterId, ParameterKind},
    projection::{Application, Schema},
    relations::{Constraints, ProofScope},
    traits,
    variance::Variances,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn parameter(types: &mut Store, index: u32) -> Result<(ParameterId, TypeId)> {
    let id = types.intern_parameter(Parameter {
        owner: ItemId(100),
        index,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, types.intern(Kind::Parameter(id))?))
}
fn projection(
    types: &mut Store,
    member: u32,
    subject: TypeId,
    args: &[Argument],
) -> Result<TypeId> {
    types.declare_projection(
        ItemId(member),
        Schema {
            trait_item: ItemId(10),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let arguments = types.intern_arguments(args)?;
    let trait_application = types.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments,
    })?;
    Ok(types.intern(Kind::Projection(Application {
        item: ItemId(member),
        subject,
        trait_application,
        arguments: GenericArgs::EMPTY,
    }))?)
}
fn reference(types: &mut Store, region: RegionId, referent: TypeId) -> Result<TypeId> {
    Ok(types.intern(Kind::Reference {
        region,
        referent,
        mutable: false,
    })?)
}
#[test]
fn result_bound_or_every_input_suffices_without_inverting_projection() -> Result {
    let mut types = Store::default();
    let (t, ty) = parameter(&mut types, 0)?;
    let (u, other) = parameter(&mut types, 1)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let output = projection(
        &mut types,
        11,
        ty,
        &[Argument::Lifetime(a), Argument::Type(other)],
    )?;
    let sibling = projection(
        &mut types,
        12,
        ty,
        &[Argument::Lifetime(a), Argument::Type(other)],
    )?;
    let borrowed = reference(&mut types, b, output)?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let variances = Variances::default();
    for input_mask in 0..8 {
        for assumption in 0..4 {
            let mut queries = Queries::new(&types, &bounds, &templates);
            let mut proof = Constraints::new(&types, &variances, 2, 100_000)?;
            proof.parameter(t)?;
            proof.parameter(u)?;
            if input_mask & 1 != 0 {
                proof.assume_parameter(t, b)?;
            }
            if input_mask & 2 != 0 {
                proof.assume_parameter(u, b)?;
            }
            if input_mask & 4 != 0 {
                proof.assume(a, b)?;
            }
            match assumption {
                1 => proof.assume_type_outlives(&mut queries, output, b)?,
                2 => proof.assume_well_formed(&mut queries, borrowed)?,
                3 => proof.assume_type_outlives(&mut queries, sibling, b)?,
                _ => {}
            }
            proof.require_type(&mut queries, output, b, ProofScope::ROOT, 29)?;
            assert_eq!(
                proof.solve().is_ok(),
                input_mask == 7 || assumption == 1 || assumption == 2,
                "inputs={input_mask}, assumption={assumption}"
            );
        }
    }
    for input in [t, u] {
        let mut queries = Queries::new(&types, &bounds, &templates);
        let mut proof = Constraints::new(&types, &variances, 2, 100_000)?;
        proof.parameter(t)?;
        proof.parameter(u)?;
        proof.assume_type_outlives(&mut queries, output, RegionId::STATIC)?;
        proof.parameter_outlives(input, RegionId::STATIC, 31)?;
        assert!(proof.solve().is_err());
    }
    Ok(())
}
#[test]
fn input_lifetime_equality_is_a_condition_not_an_inferred_permission() -> Result {
    let mut types = Store::default();
    let (t, ty) = parameter(&mut types, 0)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let left = reference(&mut types, a, TypeId::UNIT)?;
    let right = reference(&mut types, b, TypeId::UNIT)?;
    let left = projection(&mut types, 11, ty, &[Argument::Type(left)])?;
    let right = projection(&mut types, 11, ty, &[Argument::Type(right)])?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let variances = Variances::default();
    for mask in 0..4 {
        let mut queries = Queries::new(&types, &bounds, &templates);
        let mut proof = Constraints::new(&types, &variances, 2, 100_000)?;
        proof.parameter(t)?;
        if mask & 1 != 0 {
            proof.assume(a, b)?;
        }
        if mask & 2 != 0 {
            proof.assume(b, a)?;
        }
        proof.assume_type_outlives(&mut queries, left, RegionId::STATIC)?;
        proof.require_type(&mut queries, right, RegionId::STATIC, ProofScope::ROOT, 43)?;
        assert_eq!(proof.solve().is_ok(), mask == 3);
    }
    Ok(())
}
#[test]
fn same_numeric_slot_in_a_younger_scope_cannot_reuse_an_outer_bound() -> Result {
    let mut types = Store::default();
    let (t, ty) = parameter(&mut types, 0)?;
    let outer = projection(
        &mut types,
        11,
        ty,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let captured = projection(
        &mut types,
        11,
        ty,
        &[Argument::Lifetime(RegionId::bound(1, 0)?)],
    )?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let variances = Variances::default();
    for (required, valid) in [(outer, false), (captured, true)] {
        let mut queries = Queries::new(&types, &bounds, &templates);
        let mut proof = Constraints::new(&types, &variances, 1, 100_000)?;
        proof.parameter(t)?;
        proof.assume_type_outlives(&mut queries, outer, RegionId::STATIC)?;
        let inner = proof.universal_scope(ProofScope::ROOT, 1)?;
        proof.require_type(&mut queries, required, RegionId::STATIC, inner, 53)?;
        assert_eq!(proof.solve().is_ok(), valid);
    }
    Ok(())
}
#[test]
fn captured_projection_components_preserve_skipped_callable_binders() -> Result {
    let mut types = Store::default();
    let (t, ty) = parameter(&mut types, 0)?;
    let outer = projection(
        &mut types,
        11,
        ty,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let captured = projection(
        &mut types,
        11,
        ty,
        &[Argument::Lifetime(RegionId::bound(1, 0)?)],
    )?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let variances = Variances::default();
    for (result, valid) in [(outer, false), (captured, true)] {
        let parameters = types.intern_list(&[])?;
        let function = types.intern(Kind::Function {
            bound_regions: 1,
            parameters,
            result,
            unsafe_: false,
        })?;
        let mut queries = Queries::new(&types, &bounds, &templates);
        let mut proof = Constraints::new(&types, &variances, 1, 100_000)?;
        proof.parameter(t)?;
        proof.assume_type_outlives(&mut queries, outer, RegionId::STATIC)?;
        proof.require_type(
            &mut queries,
            function,
            RegionId::STATIC,
            ProofScope::ROOT,
            59,
        )?;
        assert_eq!(proof.solve().is_ok(), valid);
    }
    Ok(())
}
#[test]
fn conditional_input_equality_must_also_hold_at_runtime_points() -> Result {
    use rustgm::borrow::regions::points::Point;
    use rustgm::borrow::regions::{graph::Region, points::infer};
    let mut types = Store::default();
    let (t, ty) = parameter(&mut types, 0)?;
    let a = RegionId(1);
    let b = RegionId(2);
    let left = projection(&mut types, 11, ty, &[Argument::Lifetime(a)])?;
    let right = projection(&mut types, 11, ty, &[Argument::Lifetime(b)])?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let variances = Variances::default();
    let mut queries = Queries::new(&types, &bounds, &templates);
    let mut proof = Constraints::new(&types, &variances, 0, 100_000)?;
    proof.parameter(t)?;
    proof.variable(a)?;
    proof.variable(b)?;
    proof.assume_type_outlives(&mut queries, left, RegionId::STATIC)?;
    proof.require_type(&mut queries, right, RegionId::STATIC, ProofScope::ROOT, 61)?;
    let proof = proof.prove()?;
    let a = proof.identities.get(a).ok_or("missing a")?;
    let b = proof.identities.get(b).ok_or("missing b")?;
    assert!(
        infer(
            &proof.solution,
            2,
            [(a, Point(0)), (b, Point(1))],
            [Region::STATIC],
            100_000
        )
        .is_err()
    );
    infer(
        &proof.solution,
        2,
        [(a, Point(0)), (b, Point(0))],
        [Region::STATIC],
        100_000,
    )?;
    Ok(())
}

#[test]
fn a_bound_on_an_escaping_result_does_not_grant_its_inputs_a_bound() -> Result {
    let mut types = Store::default();
    let (t, ty) = parameter(&mut types, 0)?;
    let output = projection(
        &mut types,
        11,
        ty,
        &[Argument::Lifetime(RegionId::bound(0, 0)?)],
    )?;
    let parameters = types.intern_list(&[])?;
    let function = types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: output,
        unsafe_: false,
    })?;
    let bounds = Bounds::default();
    let templates = Templates::default();
    let variances = Variances::default();
    let mut queries = Queries::new(&types, &bounds, &templates);
    let mut proof = Constraints::new(&types, &variances, 0, 100_000)?;
    proof.parameter(t)?;
    proof.assume_type_outlives(&mut queries, function, RegionId::STATIC)?;
    proof.parameter_outlives(t, RegionId::STATIC, 71)?;
    assert!(proof.solve().is_err());
    Ok(())
}
