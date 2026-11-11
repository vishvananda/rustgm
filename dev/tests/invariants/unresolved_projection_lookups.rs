//! Unresolved projection properties filter candidates without proving their bounds.
use super::TestResult;
use crate::types::{
    self, ItemId,
    infer::{Constructor, Context, Domain, properties::Query},
    parameters::environment::Environment,
};
use std::{fmt::Write as _, sync::Arc};

const CASES: [(&str, Option<&str>); 13] = [
    ("free-inferred", None),
    ("core-default-method", None),
    ("unique-inherent", None),
    ("explicit-rigid", None),
    ("explicit-concrete", None),
    (
        "explicit-concrete-fail",
        Some("no applicable trait method for this prefix"),
    ),
    ("explicit-concrete-fallback", None),
    ("inferred-concrete", None),
    ("inferred-concrete-fail", Some(super::OBLIGATION)),
    ("free-copy", None),
    ("free-sized", None),
    ("method-rigid", None),
    ("method-unresolved", None),
];

#[test]
fn projection_lookup_conditions_match_declared_outcomes() -> TestResult {
    super::match_outcomes(
        "pa31/tests/spec/projection-lookup-conditions",
        &CASES,
        "projection_lookup_condition",
    )
}

/// The failed resolution visits the projection, its tuple prefix and its hole.
/// Only those visits and the request itself are charged; a repeated query pays
/// one cache lookup. A budget error cannot become an ambiguous candidate.
#[test]
fn projection_property_walks_pay_for_the_reachable_prefix() -> TestResult {
    let mut store = types::Store::default();
    let item = ItemId(700);
    store.declare_projection(
        item,
        types::projection::Schema {
            trait_item: ItemId(701),
            sized: true,
            copies: types::copy::Properties::default(),
        },
    )?;
    let environment = Arc::new(Environment::new(&store, &[])?);
    let definitions = types::aggregate::Definitions::default();
    let selected = std::collections::HashMap::<ItemId, types::copy::Declaration>::new();
    for depth in [0, 4, 16] {
        let mut context = Context::default();
        let mut subject = context.variable(Domain::Any)?;
        for _ in 0..depth {
            subject = context.construct(Constructor::Tuple, &[subject])?;
        }
        let root = context.projected(&store, item, subject, &[], &[])?;
        let mut query = Query::new(Arc::clone(&environment));
        let before = context.steps;
        let value = query.demand(&mut context, &mut store, &definitions, &selected, root)?;
        assert!(value.sized && value.abilities.copy && value.abilities.clone_);
        assert_eq!(context.steps - before, depth + 3);
        let before = context.steps;
        query.demand(&mut context, &mut store, &definitions, &selected, root)?;
        assert_eq!(context.steps - before, 1);
    }
    let mut context = Context::default();
    let subject = context.variable(Domain::Any)?;
    let root = context.projected(&store, item, subject, &[], &[])?;
    context.account_work(999_999 - context.steps)?;
    let mut query = Query::new(environment);
    assert_eq!(
        query
            .demand(&mut context, &mut store, &definitions, &selected, root)
            .err(),
        Some("candidate property work budget exceeded")
    );
    assert!(context.steps > 1_000_000);
    Ok(())
}

/// Two callers retain the same selected owners and every method counter except
/// the charged selection steps. Their extra steps fit a single affine function
/// of the number of projection conditions, in both lookup forms.
#[test]
fn unresolved_projection_lookup_work_is_affine_in_conditions() -> TestResult {
    let foundation = crate::test_source::fixture(super::FOUNDATION)?;
    for method in [false, true] {
        let mut points = Vec::new();
        for count in [1, 4, 8] {
            let projected = source(&foundation, count, true, method)?;
            let plain = source(&foundation, count, false, method)?;
            let (left, left_owners) = observation(&projected)?;
            let (right, right_owners) = observation(&plain)?;
            assert_eq!(left_owners, right_owners, "{count}, method={method}");
            for index in [0, 1, 2, 4, 5] {
                assert_eq!(left[index], right[index], "counter {index}, {count}");
            }
            points.push((count, left[3].checked_sub(right[3]).ok_or("negative work")?));
        }
        let slope = (points[1].1 - points[0].1) / 3;
        let constant = points[0].1.checked_sub(slope).ok_or("negative intercept")?;
        assert!(slope > 0);
        for &(count, work) in &points {
            assert_eq!(
                work,
                slope * count + constant,
                "method={method}: {points:?}"
            );
        }
        println!("projection-lookup method={method} work={points:?}");
    }
    Ok(())
}

fn observation(source: &str) -> TestResult<([usize; 6], Vec<u32>)> {
    let report = crate::test_source::source_provider(source, "projection_lookup_work")?;
    let analysis = crate::pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    let methods = &analysis.facts.methods;
    let mut owners: Vec<_> = methods.calls.values().map(|call| call.method.0).collect();
    owners.extend(
        methods
            .qualified
            .values()
            .map(|entry| entry.implementation.0),
    );
    owners.sort_unstable();
    Ok((
        [
            methods.overlap_pairs,
            methods.overlap_steps,
            methods.selection_probes,
            methods.selection_steps,
            methods.selection_shape_steps,
            methods.trait_visibility_steps,
        ],
        owners,
    ))
}

fn source(foundation: &str, count: usize, projected: bool, method: bool) -> TestResult<String> {
    let mut source = foundation.to_owned();
    source.push_str(
        "pub trait ZipImpl { fn new() -> Self; }\npub trait Make { fn make() -> Self; }\n",
    );
    source.push_str("pub trait Iter {");
    let mut bounds = String::new();
    for index in 0..count {
        write!(source, "type A{index};")?;
        write!(bounds, "I::A{index}: Clone,")?;
    }
    source.push_str("}\npub struct S<I: Iter> { iter: I }\n");
    let conditions = if projected { bounds.as_str() } else { "" };
    writeln!(
        source,
        "impl<I: Iter> S<I> where {conditions} {{\n\
         pub fn new(iter: I) -> Self {{ Self {{ iter }} }}\n\
         pub fn get(&self) -> u8 {{ 1 }}\n}}"
    )?;
    for caller in 0..2 {
        if method {
            writeln!(
                source,
                "pub fn c{caller}<I: Iter + Make>() -> u8 where {bounds} {{\n\
                 let h = S {{ iter: Make::make() }}; let r = h.get();\n\
                 let _y: S<I> = h; r }}"
            )?;
        } else {
            writeln!(
                source,
                "pub fn c{caller}<I: Iter>(i: I) where {bounds} {{ let _h = S::new(i); }}"
            )?;
        }
    }
    Ok(source)
}
