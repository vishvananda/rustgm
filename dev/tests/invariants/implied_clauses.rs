//! Whole clauses are filtered before lifting either quantified or callable binders.
use super::Implied;
use crate::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    parameters::{
        Parameter, ParameterKind,
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn reference(store: &mut Store, referent: TypeId, region: RegionId) -> TestResult<TypeId> {
    Ok(store.intern(Kind::Reference {
        referent,
        mutable: false,
        region,
    })?)
}

fn quantified(store: &mut Store, ty: TypeId, shorter: RegionId) -> TestResult<Clause> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions: 1,
        predicate: Predicate {
            ty,
            bound: Bound::Outlives(shorter),
        },
    })?))
}

#[test]
fn dependent_clauses_cannot_grant_independent_components_as_permissions() -> TestResult {
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(parameter))?;
    let local = RegionId::bound(0, 0)?;
    let borrowed = reference(&mut store, TypeId::UNIT, local)?;
    let list = store.intern_list(&[ty, borrowed])?;
    let composite = store.intern(Kind::Tuple(list))?;
    let mut implied = Implied::default();
    for (subject, shorter) in [
        (ty, local),
        (composite, local),
        (composite, RegionId::STATIC),
    ] {
        let clause = quantified(&mut store, subject, shorter)?;
        assert!(implied.outlives(&mut store, clause, 0)?.is_none());
    }
    let expected = Predicate {
        ty,
        bound: Bound::Outlives(RegionId::STATIC),
    };
    let independent = quantified(&mut store, ty, RegionId::STATIC)?;
    for clause in [Clause::Predicate(expected), independent] {
        assert_eq!(implied.outlives(&mut store, clause, 0)?, Some(expected));
    }
    assert!(
        implied
            .outlives(
                &mut store,
                Predicate {
                    ty,
                    bound: Bound::Sized
                }
                .into(),
                0
            )?
            .is_none()
    );
    Ok(())
}

#[test]
fn outer_captures_lift_through_both_binders_without_capturing_nested_locals() -> TestResult {
    let mut store = Store::default();
    let local = reference(&mut store, TypeId::UNIT, RegionId::bound(0, 0)?)?;
    let result = reference(&mut store, TypeId::UNIT, RegionId::bound(3, 1)?)?;
    let parameters = store.intern_list(&[local])?;
    let function = store.intern(Kind::Function {
        parameters,
        result,
        bound_regions: 1,
        unsafe_: false,
    })?;
    let clause = quantified(&mut store, function, RegionId::bound(2, 1)?)?;
    let result = reference(&mut store, TypeId::UNIT, RegionId::bound(1, 1)?)?;
    let expected = store.intern(Kind::Function {
        parameters,
        result,
        bound_regions: 1,
        unsafe_: false,
    })?;
    let mut implied = Implied::default();
    assert_eq!(
        implied.outlives(&mut store, clause, 1)?,
        Some(Predicate {
            ty: expected,
            bound: Bound::Outlives(RegionId::bound(0, 1)?),
        })
    );
    let dependent = quantified(&mut store, function, RegionId::bound(1, 0)?)?;
    assert!(implied.outlives(&mut store, dependent, 1)?.is_none());
    let captured = RegionId::bound(1, 2)?;
    assert_eq!(
        implied.regions((captured, RegionId::STATIC), 1)?,
        Some((RegionId::bound(0, 2)?, RegionId::STATIC))
    );
    for pair in [
        (captured, RegionId::bound(0, 0)?),
        (RegionId::bound(0, 0)?, captured),
    ] {
        assert!(implied.regions(pair, 1)?.is_none());
        assert_eq!(implied.regions(pair, 0)?, Some(pair));
    }
    Ok(())
}

#[test]
fn growing_shared_subjects_lift_with_linear_work_and_storage() -> TestResult {
    for width in [16, 64, 256, 1024] {
        let mut store = Store::default();
        let mut subject = TypeId::UNIT;
        let mut clauses = Vec::new();
        let captured = RegionId::bound(2, 0)?;
        for _ in 0..width {
            subject = reference(&mut store, subject, captured)?;
            clauses.push(quantified(&mut store, subject, captured)?);
        }
        let before = store.len();
        let mut implied = Implied::default();
        let mut results = Vec::new();
        for &clause in &clauses {
            let result = implied
                .outlives(&mut store, clause, 1)?
                .ok_or("missing implied bound")?;
            assert_eq!(result.bound, Bound::Outlives(RegionId::bound(0, 0)?));
            results.push(result);
        }
        let steps = implied.steps();
        let growth = store.len() - before;
        println!("implied width={width} steps={steps} types={growth}");
        assert!(steps <= 64 * width + 64, "{width}: {steps}");
        assert!(growth <= 2 * width + 4, "{width}: {growth}");
        for (&clause, &result) in clauses.iter().zip(&results) {
            assert_eq!(implied.outlives(&mut store, clause, 1)?, Some(result));
        }
        assert!(implied.steps() - steps <= 8 * width);
        assert_eq!(store.len() - before, growth);
    }
    Ok(())
}
