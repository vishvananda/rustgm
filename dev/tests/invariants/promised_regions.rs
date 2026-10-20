//! Public promises cannot supply reverse input permissions or another identity's bounds.
use crate::types::{
    Kind, RegionId, Store, TypeId, callable, copy, opaque,
    outlives::{Bounds, LiveRegions, Queries},
    relations::{Constraints, ProofScope},
    variance::Variances,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn declaration(
    store: &mut Store,
    captures: &[RegionId],
    promises: &[RegionId],
) -> Result<TypeId, &'static str> {
    let id = store.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties::default(),
    })?;
    let args = store.intern_regions(captures)?;
    let ty = store.intern(Kind::Opaque(id, args))?;
    store.declare_opaque_outlives(ty, promises)?;
    Ok(ty)
}

#[test]
fn a_result_bound_does_not_constrain_its_captures_or_another_declaration() -> TestResult {
    let mut store = Store::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let first = declaration(&mut store, &[a], &[RegionId::STATIC])?;
    let second = declaration(&mut store, &[a], &[])?;
    let bounds = Bounds::default();
    let callables = callable::Templates::default();
    let variances = Variances::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    for (ty, accepted) in [(first, true), (second, false)] {
        let mut proof = Constraints::new(&store, &variances, 2, 10_000)?;
        proof.require_type(&mut queries, ty, RegionId::STATIC, ProofScope::ROOT, 1)?;
        assert_eq!(proof.solve().is_ok(), accepted);
    }
    let mut proof = Constraints::new(&store, &variances, 2, 10_000)?;
    proof.assume_type_outlives(&mut queries, first, b)?;
    proof.outlives(a, b, 2)?;
    assert!(proof.solve().is_err());
    Ok(())
}

#[test]
fn live_bounds_use_actual_capture_positions_and_preserve_distinct_choices() -> TestResult {
    let mut store = Store::default();
    let a = RegionId::bound(0, 3)?;
    let b = RegionId::bound(0, 9)?;
    let first = declaration(&mut store, &[b, a], &[a, a])?;
    let second = declaration(&mut store, &[b, a], &[a, b])?;
    let third = declaration(&mut store, &[b, a], &[a, RegionId::STATIC, b])?;
    let mut live = LiveRegions::default();
    assert_eq!(live.symbolic(&store, first)?.regions, &[a]);
    let mut actual = live.symbolic(&store, second)?.regions.to_vec();
    actual.sort_unstable_by_key(|r| r.0);
    assert_eq!(actual, [a, b]);
    assert!(live.symbolic(&store, third)?.is_empty());
    let Some(Kind::Opaque(id, _)) = store.kind(first) else {
        return Err("missing opaque".into());
    };
    let args = store.intern_regions(&[RegionId(11), RegionId(12)])?;
    let instance = store.intern(Kind::Opaque(id, args))?;
    assert_eq!(live.symbolic(&store, instance)?.regions, &[RegionId(12)]);
    Ok(())
}

#[test]
fn wide_promises_and_repeated_live_queries_share_completed_components() -> TestResult {
    for width in [16_u16, 64, 256, 1024] {
        let mut store = Store::default();
        let captures = (0..width)
            .map(|i| RegionId::bound(0, i))
            .collect::<Result<Vec<_>, _>>()?;
        let ty = declaration(&mut store, &captures, &captures)?;
        let mut live = LiveRegions::default();
        assert_eq!(live.symbolic(&store, ty)?.regions.len(), usize::from(width));
        let first = live.steps();
        for _ in 0..width {
            assert_eq!(live.symbolic(&store, ty)?.regions.len(), usize::from(width));
        }
        assert_eq!(
            live.steps(),
            first,
            "repeated queries walked the published contract"
        );
        assert!(first < usize::from(width) * 4 + 16, "{width}: {first}");
        eprintln!("opaque live bounds width={width} work={first}");
    }
    Ok(())
}

#[test]
fn a_unique_promise_guides_inference_but_competing_permissions_remain_alternatives() -> TestResult {
    let mut store = Store::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let formal = declaration(&mut store, &[a, b], &[a, a])?;
    let Some(Kind::Opaque(id, _)) = store.kind(formal) else {
        return Err("missing opaque identity".into());
    };
    let variable = RegionId(1);
    let arguments = store.intern_regions(&[variable, b])?;
    let application = store.intern(Kind::Opaque(id, arguments))?;
    let bounds = Bounds::default();
    let callables = callable::Templates::default();
    let variances = Variances::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    // The callable input grants an upper bound; the result promise supplies
    // the matching lower bound instead of testing an unconstrained minimum.
    let mut proof = Constraints::new(&store, &variances, 2, 10_000)?;
    proof.variable(variable)?;
    proof.outlives(a, variable, 0)?;
    proof.require_type(&mut queries, application, a, ProofScope::ROOT, 1)?;
    assert!(proof.solve().is_ok());
    // An independent static promise can justify the result without requiring
    // the captured argument to be static. Do not add the unique-declaration edge.
    let mut proof = Constraints::new(&store, &variances, 2, 10_000)?;
    proof.variable(variable)?;
    proof.outlives(a, variable, 0)?;
    proof.assume_type_outlives(&mut queries, application, RegionId::STATIC)?;
    proof.require_type(
        &mut queries,
        application,
        RegionId::STATIC,
        ProofScope::ROOT,
        1,
    )?;
    assert!(proof.solve().is_ok());
    Ok(())
}
