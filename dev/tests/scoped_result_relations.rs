//! Opaque results compare complete inputs in actual callable scopes without inversion.
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, Constant},
    copy::Properties,
    projection::{Application, Schema},
    region_graph::Error,
    relations::Constraints,
    traits,
    variance::Variances,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn projection(types: &mut Store, region: RegionId) -> Result<TypeId> {
    application(types, TypeId::UNIT, &[Argument::Lifetime(region)], &[])
}
fn application(
    types: &mut Store,
    subject: TypeId,
    inputs: &[Argument],
    family: &[Argument],
) -> Result<TypeId> {
    types.declare_projection(
        ItemId(11),
        Schema {
            trait_item: ItemId(10),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let arguments = types.intern_arguments(inputs)?;
    let trait_application = types.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments,
    })?;
    let arguments = types.intern_arguments(family)?;
    Ok(types.intern(Kind::Projection(Application {
        item: ItemId(11),
        subject,
        trait_application,
        arguments,
    }))?)
}
fn function(types: &mut Store, count: u32, input: TypeId, result: TypeId) -> Result<TypeId> {
    let parameters = types.intern_list(&[input])?;
    Ok(types.intern(Kind::Function {
        bound_regions: count,
        parameters,
        result,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?)
}
#[test]
fn captured_results_keep_identity_when_sibling_callable_scopes_differ() -> Result {
    let mut types = Store::default();
    let captured = projection(&mut types, RegionId::bound(1, 0)?)?;
    let outer = types.intern(Kind::Reference {
        region: RegionId::bound(1, 0)?,
        mutable: false,
        referent: TypeId::UNIT,
    })?;
    let fresh = types.intern(Kind::Reference {
        region: RegionId(1),
        mutable: false,
        referent: TypeId::UNIT,
    })?;
    for count in [0, 1] {
        let actual = function(&mut types, count, outer, captured)?;
        let expected = function(&mut types, count, fresh, captured)?;
        let variances = Variances::default();
        let mut proof = Constraints::new(&types, &variances, 1, 10_000)?;
        proof.variable(RegionId(1))?;
        proof.coerce(actual, expected, false, 7)?;
        let solution = proof.solve()?;
        assert_eq!(
            solution.verify_regions(|_, _| Err(Error::Identity), 100)?,
            0
        );
    }
    Ok(())
}
#[test]
fn comparing_results_requires_input_equality_without_inferring_it() -> Result {
    let mut types = Store::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let left = projection(&mut types, a)?;
    let right = projection(&mut types, b)?;
    let variable = projection(&mut types, RegionId(1))?;
    let variances = Variances::default();
    for mask in 0..4 {
        let mut proof = Constraints::new(&types, &variances, 2, 10_000)?;
        if mask & 1 != 0 {
            proof.assume(a, b)?;
        }
        if mask & 2 != 0 {
            proof.assume(b, a)?;
        }
        proof.coerce(left, right, false, 13)?;
        let result = proof.solve();
        assert_eq!(result.is_ok(), mask == 3);
        if let Ok(solution) = result {
            assert!(matches!(
                solution.verify_regions(|longer, shorter| Ok(longer == shorter), 100),
                Err(Error::Verification { source: 13 })
            ));
        }
    }
    for constrained in [false, true] {
        let mut proof = Constraints::new(&types, &variances, 2, 10_000)?;
        proof.variable(RegionId(1))?;
        if constrained {
            proof.outlives(a, RegionId(1), 17)?;
            proof.outlives(RegionId(1), a, 17)?;
        }
        proof.coerce(left, variable, false, 19)?;
        assert_eq!(proof.solve().is_ok(), constrained);
    }
    Ok(())
}
#[test]
fn same_numeric_local_slot_cannot_supply_a_captured_input() -> Result {
    let mut types = Store::default();
    let captured = projection(&mut types, RegionId::bound(1, 0)?)?;
    let local = projection(&mut types, RegionId::bound(0, 0)?)?;
    let actual = function(&mut types, 1, TypeId::UNIT, captured)?;
    let expected = function(&mut types, 1, TypeId::UNIT, local)?;
    let variances = Variances::default();
    for (a, b) in [(actual, expected), (expected, actual)] {
        let mut proof = Constraints::new(&types, &variances, 1, 10_000)?;
        proof.coerce(a, b, false, 23)?;
        assert!(matches!(
            proof.solve(),
            Err(Error::Verification { source: 23 })
        ));
    }
    Ok(())
}
#[test]
fn mixed_trait_and_family_inputs_and_nested_local_binders_remain_complete() -> Result {
    let mut types = Store::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let zero = types.intern(Kind::Reference {
        region: a,
        mutable: false,
        referent: TypeId::UNIT,
    })?;
    let one = types.intern(Kind::Reference {
        region: b,
        mutable: false,
        referent: TypeId::UNIT,
    })?;
    let nested = function(&mut types, 2, zero, TypeId::UNIT)?;
    let other = function(&mut types, 2, one, TypeId::UNIT)?;
    let three = types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 3,
    })?;
    let four = types.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: 4,
    })?;
    let inputs = [
        Argument::Lifetime(a),
        Argument::Type(nested),
        Argument::Const(three),
    ];
    let family = [Argument::Type(TypeId::UNIT), Argument::Const(three)];
    let original = application(&mut types, TypeId::UNIT, &inputs, &family)?;
    for (subject, input, family) in [
        (TypeId::NEVER, inputs, family),
        (
            TypeId::UNIT,
            [
                Argument::Lifetime(a),
                Argument::Type(other),
                Argument::Const(three),
            ],
            family,
        ),
        (
            TypeId::UNIT,
            inputs,
            [Argument::Type(TypeId::NEVER), Argument::Const(three)],
        ),
        (
            TypeId::UNIT,
            [
                Argument::Lifetime(a),
                Argument::Type(nested),
                Argument::Const(four),
            ],
            family,
        ),
        (
            TypeId::UNIT,
            inputs,
            [Argument::Type(TypeId::UNIT), Argument::Const(four)],
        ),
    ] {
        let different = application(&mut types, subject, &input, &family)?;
        let variances = Variances::default();
        let mut proof = Constraints::new(&types, &variances, 2, 10_000)?;
        proof.assume(a, b)?;
        proof.assume(b, a)?;
        assert_eq!(
            proof.coerce(original, different, false, 31),
            Err(Error::Identity)
        );
    }
    Ok(())
}
#[test]
fn shared_projection_inputs_have_bounded_nonrecursive_relation_work() -> Result {
    let mut types = Store::default();
    let mut captured = types.intern(Kind::Reference {
        region: RegionId::bound(1, 0)?,
        mutable: false,
        referent: TypeId::UNIT,
    })?;
    for _ in 0..10_000 {
        let children = types.intern_list(&[captured, captured])?;
        captured = types.intern(Kind::Tuple(children))?;
    }
    let result = application(&mut types, captured, &[], &[])?;
    let variable = types.intern(Kind::Reference {
        region: RegionId(1),
        mutable: false,
        referent: TypeId::UNIT,
    })?;
    let actual = function(&mut types, 0, TypeId::NEVER, result)?;
    let expected = function(&mut types, 0, variable, result)?;
    let variances = Variances::default();
    for (limit, accepted) in [(1_000, false), (100_000, true)] {
        let mut proof = Constraints::new(&types, &variances, 1, limit)?;
        proof.variable(RegionId(1))?;
        // Reverse the input relation so Never is the source of its contravariant step.
        let relation = proof.coerce(expected, actual, false, 37);
        if accepted {
            relation?;
            proof.solve()?;
        } else {
            assert_eq!(relation, Err(Error::Budget));
        }
    }
    Ok(())
}
