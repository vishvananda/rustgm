//! Whole typed calls include argument formation, unlike the isolated Freeze query.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn observe(source: &str) -> Result<(usize, usize)> {
    let report = crate::test_source::source_provider(source, "freeze_perf_v1")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let functions = facts.functions.clone();
    let mut checker = Checker::new(&report, &mut facts);
    let mut observed = None;
    for function in functions {
        let first = (
            checker.facts.body_work.inference_steps,
            checker.facts.body_work.canonical_steps,
        );
        checker.function(function)?;
        if report
            .package
            .names
            .get(report.resolution.definitions[function.definition.0 as usize].name)
            == "observe"
        {
            // Ready sibling roots share one query's formation DAG. Its storage
            // dies with that query; these calls need no cross-query cache owner.
            assert!(
                checker.state.canonical_trait_proofs.is_none(),
                "conjunction retained cross-query storage: {:?}",
                checker
                    .state
                    .canonical_trait_proofs
                    .as_deref()
                    .map(selection::Reuse::observations)
            );
            observed = Some((
                checker.facts.body_work.inference_steps - first.0,
                checker.facts.body_work.canonical_steps - first.1,
            ));
        }
    }
    observed.ok_or_else(|| "observe".into())
}
#[test]
fn original_full_call_alias_depths_reuse_sizing_without_unrelated_nominal_growth() -> Result {
    let mut previous: Option<(usize, usize, usize)> = None;
    for depth in [8, 32, 128] {
        let mut measured = None;
        for noise in [0, 128] {
            let source = crate::test_source::fixture(&format!(
                "pa31/tests/spec/source-freeze/formation/typed-depth-{depth}-noise-{noise}.rs"
            ))?;
            let current = observe(&source)?;
            println!(
                "typed_depth={depth} noise={noise} inference={} canonical={}",
                current.0, current.1
            );
            assert!(current.0 > 0);
            assert!(current.1 > 0);
            if let Some(old) = measured {
                assert_eq!(
                    old, current,
                    "unrelated nominal definitions changed the demanded proof scope"
                );
            } else {
                measured = Some(current);
            }
        }
        let (inference, canonical) = measured.ok_or("measurement")?;
        if let Some((old_depth, old_inference, old_canonical)) = previous {
            let ratio = depth / old_depth;
            assert!(
                inference <= ratio * old_inference + 4096,
                "typed inference prefix work is not bounded"
            );
            assert!(
                canonical <= ratio * old_canonical + 4096,
                "canonical proof prefix work is not bounded"
            );
        }
        previous = Some((depth, inference, canonical));
    }
    Ok(())
}
#[test]
fn ordinary_scalar_bodies_do_not_allocate_the_canonical_reuse_owner() -> Result {
    let report = crate::test_source::resolve("fn main(){let x=7u8;let y=x;}", true)?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let functions = facts.functions.clone();
    let mut checker = Checker::new(&report, &mut facts);
    for function in functions {
        checker.function(function)?;
        assert!(checker.state.canonical_trait_proofs.is_none());
    }
    Ok(())
}
