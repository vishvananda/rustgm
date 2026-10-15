//! Impl headers and caller goals can both contain inference variables.
use super::binders::{function, predicate, reference};
use rustgm::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, ConstKind, Constant},
    infer::{self, Context, Domain, Instantiation},
    parameters::{Parameter, ParameterId, ParameterKind, predicates::Predicate},
};

fn parameter(store: &mut Store, owner: u32) -> Result<(ParameterId, TypeId), &'static str> {
    let id = store.intern_parameter(Parameter {
        owner: ItemId(owner),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    Ok((id, store.intern(Kind::Parameter(id))?))
}

fn instantiate(
    context: &mut Context,
    store: &mut Store,
    template: Predicate,
    bindings: &[(ParameterId, infer::arguments::Argument)],
) -> Result<infer::predicates::Predicate, infer::Error> {
    let mut substitution = Instantiation::new(context, store, bindings)?;
    context.instantiate_predicate(store, template, &mut substitution)
}

#[test]
fn both_callable_sides_infer_without_losing_local_correlations()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let (first, ty) = parameter(&mut store, 10)?;
    let (second, other) = parameter(&mut store, 11)?;
    let local = reference(&mut store, 0, 0)?;
    let renamed = reference(&mut store, 0, 1)?;
    let byte = TypeId::primitive(Primitive::U8);
    let left = function(&mut store, 1, &[local, ty, byte], ty)?;
    let right = function(&mut store, 2, &[renamed, byte, other], other)?;
    let left = predicate(&mut store, left)?;
    let right = predicate(&mut store, right)?;
    for reverse in [false, true] {
        let mut context = Context::default();
        let x = context.variable(Domain::Any)?;
        let y = context.variable(Domain::Any)?;
        let left = instantiate(
            &mut context,
            &mut store,
            left,
            &[(first, infer::arguments::Argument::Type(x))],
        )?;
        let right = instantiate(
            &mut context,
            &mut store,
            right,
            &[(second, infer::arguments::Argument::Type(y))],
        )?;
        let (left, right) = if reverse {
            (&right, &left)
        } else {
            (&left, &right)
        };
        context.equate_predicates(&store, left, right)?;
        assert_eq!(context.resolved(&mut store, x)?, byte);
        assert_eq!(context.resolved(&mut store, y)?, byte);
        assert!(context.region_equalities.is_empty());
    }
    Ok(())
}

#[test]
fn either_inferred_side_rejects_capture_and_accepts_closed_nested_functions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let (t, ty) = parameter(&mut store, 10)?;
    let local = reference(&mut store, 0, 0)?;
    let closed = function(&mut store, 1, &[local], local)?;
    let template = function(&mut store, 1, &[local, ty, ty], TypeId::UNIT)?;
    let template = predicate(&mut store, template)?;
    for (value, accepted) in [(local, false), (closed, true)] {
        let target = function(&mut store, 1, &[local, value, value], TypeId::UNIT)?;
        let target = predicate(&mut store, target)?;
        for reverse in [false, true] {
            let mut context = Context::default();
            let variable = context.variable(Domain::Any)?;
            let left = instantiate(
                &mut context,
                &mut store,
                template,
                &[(t, infer::arguments::Argument::Type(variable))],
            )?;
            let right = instantiate(&mut context, &mut store, target, &[])?;
            let (left, right) = if reverse {
                (&right, &left)
            } else {
                (&left, &right)
            };
            let result = context.equate_predicates(&store, left, right);
            if accepted {
                result?;
                assert_eq!(context.resolved(&mut store, variable)?, closed);
                assert!(context.region_equalities.is_empty());
            } else {
                assert_eq!(result, Err(infer::Error::Mismatch));
            }
        }
    }
    Ok(())
}

#[test]
fn inferred_outer_regions_rebase_on_both_sides_and_still_require_proof()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let (t, ty) = parameter(&mut store, 10)?;
    let outer = reference(&mut store, 0, 3)?;
    let nested = reference(&mut store, 1, 3)?;
    let later = reference(&mut store, 0, 5)?;
    let left = function(&mut store, 0, &[ty, ty], TypeId::UNIT)?;
    let right = function(&mut store, 0, &[nested, nested], TypeId::UNIT)?;
    let left = Predicate {
        ty: TypeId::UNIT,
        bound: rustgm::types::parameters::predicates::Bound::Trait(super::application(
            &mut store,
            200,
            &[Argument::Type(left), Argument::Type(ty)],
        )?),
    };
    let right = Predicate {
        ty: TypeId::UNIT,
        bound: rustgm::types::parameters::predicates::Bound::Trait(super::application(
            &mut store,
            200,
            &[Argument::Type(right), Argument::Type(later)],
        )?),
    };
    for reverse in [false, true] {
        let mut context = Context::default();
        let variable = context.variable(Domain::Any)?;
        let left = instantiate(
            &mut context,
            &mut store,
            left,
            &[(t, infer::arguments::Argument::Type(variable))],
        )?;
        let right = instantiate(&mut context, &mut store, right, &[])?;
        let (left, right) = if reverse {
            (&right, &left)
        } else {
            (&left, &right)
        };
        context.equate_predicates(&store, left, right)?;
        assert_eq!(context.resolved(&mut store, variable)?, outer);
        let a = RegionId::bound(0, 3)?;
        let b = RegionId::bound(0, 5)?;
        let pair = if reverse { (b, a) } else { (a, b) };
        assert_eq!(context.region_equalities, [pair]);
    }
    Ok(())
}

#[test]
fn shared_inference_variables_are_reflexive_inside_callable_binders()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let (t, ty) = parameter(&mut store, 10)?;
    let a = reference(&mut store, 0, 0)?;
    let b = reference(&mut store, 0, 1)?;
    let left = function(&mut store, 1, &[a, ty], ty)?;
    let right = function(&mut store, 2, &[b, ty], ty)?;
    let left = predicate(&mut store, left)?;
    let right = predicate(&mut store, right)?;
    let mut context = Context::default();
    let x = context.variable(Domain::Any)?;
    let bindings = [(t, infer::arguments::Argument::Type(x))];
    let left = instantiate(&mut context, &mut store, left, &bindings)?;
    let right = instantiate(&mut context, &mut store, right, &bindings)?;
    context.equate_predicates(&store, &left, &right)?;
    assert!(context.region_equalities.is_empty());
    assert_eq!(
        context.resolved(&mut store, x),
        Err(infer::Error::Unresolved)
    );
    let byte = context.known(&store, TypeId::primitive(Primitive::U8))?;
    context.equate(&store, x, byte)?;
    context.finish_with_predicates(&mut store, [&left, &right].into_iter())?;
    Ok(())
}

#[test]
fn complete_header_equations_share_type_and_const_roots() -> Result<(), Box<dyn std::error::Error>>
{
    let mut store = Store::default();
    let (t, ty) = parameter(&mut store, 10)?;
    let n = store.intern_parameter(Parameter {
        owner: ItemId(10),
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let length = store.intern_const(ConstKind::Parameter(n))?;
    let array = store.intern(Kind::ArrayTerm {
        element: ty,
        length,
    })?;
    let left = Predicate {
        ty: array,
        bound: rustgm::types::parameters::predicates::Bound::Trait(super::application(
            &mut store,
            200,
            &[Argument::Type(ty), Argument::Const(length)],
        )?),
    };
    let byte = TypeId::primitive(Primitive::U8);
    let bytes = store.intern(Kind::Array {
        element: byte,
        length: 3,
    })?;
    for count in [3, 4] {
        let constant = store.intern_constant(Constant {
            ty: TypeId::primitive(Primitive::Usize),
            bits: count,
        })?;
        let right = Predicate {
            ty: bytes,
            bound: rustgm::types::parameters::predicates::Bound::Trait(super::application(
                &mut store,
                200,
                &[Argument::Type(byte), Argument::Const(constant)],
            )?),
        };
        let mut context = Context::default();
        let bindings = [
            (t, context.parameter_variable(&store, t)?),
            (n, context.parameter_variable(&store, n)?),
        ];
        let left = instantiate(&mut context, &mut store, left, &bindings)?;
        let right = instantiate(&mut context, &mut store, right, &[])?;
        let result = context.equate_predicates(&store, &left, &right);
        if count == 3 {
            result?;
            assert_eq!(
                context.resolved_predicate(&mut store, &left)?,
                context.resolved_predicate(&mut store, &right)?,
            );
        } else {
            assert_eq!(result, Err(infer::Error::Mismatch));
        }
    }
    Ok(())
}
