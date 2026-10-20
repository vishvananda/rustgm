//! Typed variance and lexical binders feed the shared hidden lifetime solver.
use super::*;
use crate::types::{Kind, Primitive, Store, TypeId, variance::Variances};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn reference(
    store: &mut Store,
    region: RegionId,
    referent: TypeId,
    mutable: bool,
) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        region,
        referent,
        mutable,
    })
}

#[test]
fn return_coercions_choose_permitted_lifetimes_using_declared_order() -> TestResult {
    for ordered in [false, true] {
        let mut store = Store::default();
        let a = RegionId::bound(0, 0)?;
        let b = RegionId::bound(0, 1)?;
        let hidden = RegionId(1);
        let byte = store.intern(Kind::Primitive(Primitive::U8))?;
        let first = reference(&mut store, a, byte, false)?;
        let second = reference(&mut store, b, byte, false)?;
        let result = reference(&mut store, hidden, byte, false)?;
        let variances = Variances::default();
        let mut constraints = Constraints::new(&store, &variances, 2, 10_000)?;
        constraints.variable(hidden)?;
        if ordered {
            constraints.assume(a, b)?;
        }
        constraints.coerce(first, result, false, 1)?;
        constraints.coerce(second, result, false, 2)?;
        let choices = constraints.member_choices(&[RegionId::STATIC, a, b])?;
        let member = constraints.member(hidden, choices, 3)?;
        if ordered {
            assert_eq!(constraints.prove()?.member_choice(member)?, b);
        } else {
            assert!(matches!(
                constraints.prove(),
                Err(Error::Membership { source: 3 })
            ));
        }
    }
    Ok(())
}

#[test]
fn invariant_referents_cannot_shorten_distinct_captures() -> TestResult {
    for mutable in [false, true] {
        let mut store = Store::default();
        let a = RegionId::bound(0, 0)?;
        let b = RegionId::bound(0, 1)?;
        let hidden = RegionId(1);
        let byte = store.intern(Kind::Primitive(Primitive::U8))?;
        let mut inputs = Vec::new();
        for region in [a, b, hidden] {
            let referent = reference(&mut store, region, byte, false)?;
            inputs.push(reference(&mut store, RegionId::STATIC, referent, mutable)?);
        }
        let variances = Variances::default();
        let mut constraints = Constraints::new(&store, &variances, 2, 10_000)?;
        constraints.variable(hidden)?;
        constraints.assume(a, b)?;
        constraints.coerce(inputs[0], inputs[2], false, 1)?;
        constraints.coerce(inputs[1], inputs[2], false, 2)?;
        let choices = constraints.member_choices(&[RegionId::STATIC, a, b])?;
        let member = constraints.member(hidden, choices, 3)?;
        if mutable {
            assert!(matches!(
                constraints.prove(),
                Err(Error::Unsatisfied { .. })
            ));
        } else {
            assert_eq!(constraints.prove()?.member_choice(member)?, b);
        }
    }
    Ok(())
}

#[test]
fn pointer_input_contravariance_sets_a_hidden_lifetime_lower_bound() -> TestResult {
    let mut store = Store::default();
    let captured = RegionId::bound(0, 0)?;
    let inside = RegionId::bound(1, 0)?;
    let hidden = RegionId(1);
    let byte = store.intern(Kind::Primitive(Primitive::U8))?;
    let unit = store.intern_list(&[])?;
    let unit = store.intern(Kind::Tuple(unit))?;
    let mut functions = Vec::new();
    for region in [inside, hidden] {
        let parameter = reference(&mut store, region, byte, false)?;
        let parameters = store.intern_list(&[parameter])?;
        functions.push(store.intern(Kind::Function {
            parameters,
            result: unit,
            bound_regions: 0,
            unsafe_: false,
        })?);
    }
    let variances = Variances::default();
    let mut constraints = Constraints::new(&store, &variances, 1, 10_000)?;
    constraints.variable(hidden)?;
    constraints.coerce(functions[0], functions[1], false, 1)?;
    let choices = constraints.member_choices(&[RegionId::STATIC, captured])?;
    let member = constraints.member(hidden, choices, 2)?;
    assert_eq!(constraints.prove()?.member_choice(member)?, captured);
    Ok(())
}
