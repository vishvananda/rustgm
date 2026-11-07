//! Distinct closed own bounds pay flat work per bound; bounds with one value
//! still trial-unify and stay ambiguous when two of them match.
use super::super::Assumptions;
use super::Discriminants;
use crate::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::{Argument, Constant},
    infer::Context,
    parameters::{
        Parameter, ParameterKind,
        predicates::{Bound, Predicate},
    },
    traits::Application,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// `subject: B<region, value>` for one trait `B`.
fn bound(
    store: &mut Store,
    subject: TypeId,
    region: Option<RegionId>,
    value: u32,
) -> Result<Predicate, &'static str> {
    let constant = store.intern_constant(Constant {
        ty: TypeId::primitive(Primitive::Usize),
        bits: value.into(),
    })?;
    let mut arguments: Vec<_> = region.map(Argument::Lifetime).into_iter().collect();
    arguments.push(Argument::Const(constant));
    let arguments = store.intern_arguments(&arguments)?;
    let application = store.intern_trait_application(Application {
        item: ItemId(1),
        arguments,
    })?;
    Ok(Predicate {
        ty: subject,
        bound: Bound::Trait(application),
    })
}

fn subject(store: &mut Store) -> Result<TypeId, &'static str> {
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(2),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    store.intern(Kind::Parameter(parameter))
}

/// Total inference steps to check `count` distinct own bounds of one subject.
fn own_bound_steps(count: u32) -> Result<usize, Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let ty = subject(&mut store)?;
    let mut assumptions = Assumptions::default();
    let mut predicates = Vec::new();
    for value in 0..count {
        let predicate = bound(&mut store, ty, None, value)?;
        assumptions.insert(&store, predicate)?;
        predicates.push(predicate);
    }
    let mut context = Context::default();
    let mut discriminants = Discriminants::default();
    for predicate in predicates {
        assert!(assumptions.unambiguous_declaration(
            &mut store,
            &mut context,
            predicate,
            &mut discriminants
        )?);
    }
    Ok(context.steps)
}

#[test]
fn distinct_closed_own_bounds_pay_flat_work_per_bound() -> TestResult {
    let base = own_bound_steps(64)?;
    for count in [64, 256, 1024] {
        let steps = own_bound_steps(count)?;
        eprintln!("own-bounds={count} steps={steps}");
        // Per-bound work at 256 and 1,024 stays within 5/4 of the 64-bound rate.
        assert!(
            steps * 64 * 4 <= base * usize::try_from(count)? * 5,
            "{count}: {steps}"
        );
    }
    Ok(())
}

#[test]
fn own_bounds_with_one_value_and_unifying_lifetimes_remain_ambiguous() -> TestResult {
    let mut store = Store::default();
    let ty = subject(&mut store)?;
    let first = RegionId::bound(0, 0)?;
    let second = RegionId::bound(0, 1)?;
    let mut assumptions = Assumptions::default();
    let ambiguous = bound(&mut store, ty, Some(first), 0)?;
    assumptions.insert(&store, ambiguous)?;
    let other = bound(&mut store, ty, Some(second), 0)?;
    assumptions.insert(&store, other)?;
    let mut distinct = Vec::new();
    for value in 1..256 {
        let predicate = bound(&mut store, ty, Some(first), value)?;
        assumptions.insert(&store, predicate)?;
        distinct.push(predicate);
    }
    let mut context = Context::default();
    let mut discriminants = Discriminants::default();
    assert!(!assumptions.unambiguous_declaration(
        &mut store,
        &mut context,
        ambiguous,
        &mut discriminants
    )?);
    for predicate in distinct {
        assert!(assumptions.unambiguous_declaration(
            &mut store,
            &mut context,
            predicate,
            &mut discriminants
        )?);
    }
    Ok(())
}

/// One function whose projected subject has `count` distinct own bounds; with
/// `ambiguous`, two of them differ only by a lifetime that unifies.
fn family(count: u32, ambiguous: bool) -> Result<String, std::fmt::Error> {
    use std::fmt::Write as _;
    let mut source = String::from(if ambiguous {
        "trait A{type Out;}trait B<'a,const N:usize>{}fn f<'a,'b,T:A<Out:B<'b,0>"
    } else {
        "trait A{type Out;}trait B<const N:usize>{}impl<const N:usize>B<N>for u8{}\
         struct S;impl A for S{type Out=u8;}fn f<T:A<Out:B<0>"
    });
    for value in 1..count {
        if ambiguous {
            write!(source, "+B<'a,{}>", value - 1)?;
        } else {
            write!(source, "+B<{value}>")?;
        }
    }
    source.push_str(if ambiguous {
        ">>(){}fn main(){}"
    } else {
        ">>(){}fn main(){f::<S>();}"
    });
    Ok(source)
}

#[test]
fn declaration_bound_families_are_checked_and_ambiguity_is_still_rejected() -> TestResult {
    for count in [64, 256, 1024] {
        let report = crate::test_source::resolve(&family(count, false)?, true)?;
        crate::pipeline::analyze_source(&report)?;
    }
    let report = crate::test_source::resolve(&family(256, true)?, true)?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("ambiguous own bounds were accepted")?;
    assert_eq!(error.message, "ambiguous trait assumptions in declaration");
    Ok(())
}
