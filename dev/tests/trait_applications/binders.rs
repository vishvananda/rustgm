//! Inspect scoped equality facts without depending on the source lifetime numbering.
use rustgm::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    infer::{self, Context, Domain, Instantiation},
    parameters::{
        Parameter, ParameterKind,
        predicates::{Bound, Predicate},
    },
};

pub(super) fn reference(store: &mut Store, depth: u16, slot: u16) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        referent: TypeId::UNIT,
        mutable: false,
        region: RegionId::bound(depth, slot)?,
    })
}

pub(super) fn function(
    store: &mut Store,
    count: u32,
    inputs: &[TypeId],
    result: TypeId,
) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(inputs)?;
    store.intern(Kind::Function {
        parameters,
        result,
        bound_regions: count,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })
}

pub(super) fn predicate(store: &mut Store, argument: TypeId) -> Result<Predicate, &'static str> {
    Ok(Predicate {
        ty: TypeId::UNIT,
        bound: Bound::Trait(super::application(store, 200, &[Argument::Type(argument)])?),
    })
}

fn equate(
    store: &mut Store,
    left: TypeId,
    right: TypeId,
) -> Result<Context, Box<dyn std::error::Error>> {
    let template = predicate(store, left)?;
    let assumption = predicate(store, right)?;
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, store, &[])?;
    let goal = context.instantiate_predicate(store, template, &mut substitution)?;
    let expected = context.equate_trait_predicate(store, &goal, assumption);
    let mut inferred = Context::default();
    let mut substitution = Instantiation::new(&mut inferred, store, &[])?;
    let left = inferred.instantiate_predicate(store, template, &mut substitution)?;
    let right = inferred.instantiate_predicate(store, assumption, &mut substitution)?;
    let actual = inferred.equate_predicates(store, &left, &right);
    assert_eq!(actual, expected);
    actual?;
    assert_eq!(inferred.region_equalities, context.region_equalities);
    Ok(inferred)
}

#[test]
fn alpha_mapping_is_bijective_even_for_identical_child_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let a = reference(&mut store, 0, 0)?;
    let b = reference(&mut store, 0, 1)?;
    let left = function(&mut store, 2, &[a, b], a)?;
    let alpha = function(&mut store, 2, &[b, a], b)?;
    let captured = function(&mut store, 2, &[b, b], b)?;
    let output = function(&mut store, 2, &[b, a], a)?;
    let unused = function(&mut store, 65_536, &[b, a], b)?;
    for right in [alpha, unused] {
        assert!(
            equate(&mut store, left, right)?
                .region_equalities
                .is_empty()
        );
    }
    for right in [captured, output] {
        let error = equate(&mut store, left, right)
            .err()
            .ok_or("expected mismatch")?;
        assert_eq!(
            error.downcast_ref::<infer::Error>(),
            Some(&infer::Error::Mismatch)
        );
    }
    Ok(())
}

#[test]
fn outer_region_equations_are_lowered_across_every_callable_binder()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let a = reference(&mut store, 2, 3)?;
    let b = reference(&mut store, 2, 5)?;
    let local = reference(&mut store, 1, 0)?;
    let left = function(&mut store, 0, &[local, a], a)?;
    let right = function(&mut store, 0, &[local, b], b)?;
    let left = function(&mut store, 1, &[left], TypeId::UNIT)?;
    let right = function(&mut store, 1, &[right], TypeId::UNIT)?;
    let context = equate(&mut store, left, right)?;
    assert!(!context.region_equalities.is_empty());
    let expected = (RegionId::bound(0, 3)?, RegionId::bound(0, 5)?);
    assert!(
        context
            .region_equalities
            .iter()
            .all(|&pair| pair == expected)
    );
    Ok(())
}

#[test]
fn nested_inference_cannot_bind_a_local_slot_but_can_retain_closed_functions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(10),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(parameter))?;
    let local = reference(&mut store, 0, 0)?;
    let closed = function(&mut store, 1, &[local], local)?;
    let template = function(&mut store, 1, &[local, ty, ty], TypeId::UNIT)?;
    let template = predicate(&mut store, template)?;
    for (value, accepted) in [(local, false), (closed, true)] {
        let target = function(&mut store, 1, &[local, value, value], TypeId::UNIT)?;
        let assumption = predicate(&mut store, target)?;
        let mut context = Context::default();
        let variable = context.variable(Domain::Any)?;
        let mut substitution = Instantiation::new(
            &mut context,
            &store,
            &[(parameter, infer::arguments::Argument::Type(variable))],
        )?;
        let goal = context.instantiate_predicate(&mut store, template, &mut substitution)?;
        let result = context.equate_trait_predicate(&mut store, &goal, assumption);
        if accepted {
            result?;
            assert_eq!(context.resolved(&mut store, variable)?, closed);
            assert_eq!(context.resolved_predicate(&mut store, &goal)?, assumption);
            assert!(context.region_equalities.is_empty());
        } else {
            assert_eq!(result, Err(infer::Error::Mismatch));
        }
    }
    Ok(())
}

#[test]
fn repeated_callable_children_share_scope_work_and_charge_the_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let a = reference(&mut store, 0, 0)?;
    let b = reference(&mut store, 0, 1)?;
    let left = function(&mut store, 1, &vec![a; 4096], a)?;
    let right = function(&mut store, 2, &vec![b; 4096], b)?;
    let context = equate(&mut store, left, right)?;
    assert!(context.steps < 20_000, "{}", context.steps);
    assert!(context.region_equalities.is_empty());
    let template = predicate(&mut store, left)?;
    let assumption = predicate(&mut store, right)?;
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let goal = context.instantiate_predicate(&mut store, template, &mut substitution)?;
    context.steps = 999_990;
    assert_eq!(
        context.equate_trait_predicate(&mut store, &goal, assumption),
        Err(infer::Error::Budget)
    );
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &store, &[])?;
    let left = context.instantiate_predicate(&mut store, template, &mut substitution)?;
    let right = context.instantiate_predicate(&mut store, assumption, &mut substitution)?;
    context.steps = 999_990;
    assert_eq!(
        context.equate_predicates(&store, &left, &right),
        Err(infer::Error::Budget)
    );
    Ok(())
}
