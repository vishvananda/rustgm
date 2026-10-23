//! Source-root extraction is lazy, bounded and independent of ambient premises.
use super::*;
use crate::{body::regions::Contract, types::RegionId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn ordinary_contracts_reuse_existing_payload_without_query_allocation() -> Result {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let facts = analysis.facts;
    let input = crate::body::regions::collect(&facts)?.bodies.remove(0);
    let existing: Arc<[Pair]> = [(RegionId::STATIC, RegionId::STATIC)].into();
    let mut collector = Collector::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let retained = collector
        .source(&input, 0, Some(&existing))?
        .ok_or("payload")?;
    assert!(Arc::ptr_eq(&existing, &retained));
    assert!(collector.queries.is_none());
    assert_eq!(collector.steps(), 0);
    Ok(())
}
#[test]
fn only_current_formal_slots_and_static_are_exportable() -> Result {
    assert!(formal(RegionId::STATIC, 0));
    assert!(formal(RegionId::bound(0, 1)?, 2));
    assert!(!formal(RegionId::bound(0, 2)?, 2));
    assert!(!formal(RegionId::bound(1, 0)?, 2));
    assert!(!formal(RegionId(1), 2));
    Ok(())
}
#[test]
fn work_and_storage_exhaustion_precede_unbounded_publication() {
    let mut work = 999_999;
    assert!(charge(&mut work, 2).is_err());
    assert!(pending(usize::MAX).is_err());
    assert!(pending(8).is_ok());
}
#[test]
fn repeated_roots_share_queries_and_existing_pairs_remain_immutable() -> Result {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa24/tests/spec/captures/shared-reference-truncation.rs"),
    )?;
    let report = crate::test_source::resolve(&source, true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let mut input: Contract = crate::body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|c| {
            c.output
                .as_ref()
                .is_some_and(|link| link.formation.is_some())
        })
        .ok_or("formation")?;
    input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let mut collector = Collector::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let pairs = collector.source(&input, 3, None)?.ok_or("root pair")?;
    assert_eq!(
        pairs.as_ref(),
        &[(RegionId::bound(0, 1)?, RegionId::bound(0, 0)?)]
    );
    let query_steps = collector.queries.as_ref().ok_or("query")?.steps();
    let retained = collector
        .source(&input, 3, Some(&pairs))?
        .ok_or("retained pair")?;
    assert!(Arc::ptr_eq(&pairs, &retained));
    assert_eq!(
        collector.queries.as_ref().ok_or("query")?.steps(),
        query_steps
    );
    input.key = crate::body::ir::BodyKey::Anonymous(crate::test_sites::node(input.source)?);
    assert!(collector.source(&input, 3, None).is_err());
    Ok(())
}

#[test]
fn distinct_capture_roots_and_repeated_publication_have_linear_charged_work() -> Result {
    use std::fmt::Write;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = String::from("struct S<'a>{a:&'static u8,b:&'a u8}fn make<'a,'b>(");
        for i in 0..width {
            write!(source, "x{i}:&'a S<'b>,")?;
        }
        source.push_str(")->impl Fn()->u8+'static{||{let mut n=0u8;");
        for i in 0..width {
            write!(source, "n|=*x{i}.a;")?;
        }
        source.push_str("n}}fn main(){}");
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        let input = crate::body::regions::collect(&facts)?
            .bodies
            .into_iter()
            .find(|c| {
                c.output
                    .as_ref()
                    .is_some_and(|link| link.formation.is_some())
            })
            .ok_or("formation")?;
        input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
        assert_eq!(
            input
                .output
                .as_ref()
                .and_then(|link| link.formation.as_ref())
                .ok_or("roots")?
                .inputs
                .len(),
            width
        );
        let mut collector = Collector::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let pairs = collector
            .source(&input, input.universals - 1, None)?
            .ok_or("pairs")?;
        assert_eq!(pairs.len(), 1);
        let queries = collector.queries.as_ref().ok_or("query")?.steps();
        let first = collector.steps();
        let mut previous_repeat = None;
        for repeat in 0..128 {
            let before = collector.steps();
            let retained = collector
                .source(&input, input.universals - 1, Some(&pairs))?
                .ok_or("retained")?;
            assert!(Arc::ptr_eq(&pairs, &retained));
            let delta = collector.steps() - before;
            if let Some(old) = previous_repeat {
                assert_eq!(delta, old);
            }
            previous_repeat = Some(delta);
            assert_eq!(collector.queries.as_ref().ok_or("query")?.steps(), queries);
            if [7, 31, 127].contains(&repeat) {
                eprintln!(
                    "formation width={width} repeat={} charged={} query={queries}",
                    repeat + 1,
                    collector.steps()
                );
            }
        }
        assert!(first <= width * 8 + 128);
        if let Some(old) = previous {
            assert_eq!(queries, old);
        }
        previous = Some(queries);
    }
    Ok(())
}
