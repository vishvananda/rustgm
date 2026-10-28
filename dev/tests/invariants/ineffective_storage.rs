//! Cold exact provenance, retained work, invalidation and rejected attempts.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn report() -> TestResult<crate::resolve::expansion::Report> {
    Ok(crate::test_source::resolve(
        &crate::test_source::fixture(
            "pa40/tests/spec/ineffective-unstable-impl/bound-matching.rs",
        )?,
        true,
    )?)
}
#[test]
fn cold_annotations_keep_hot_owners_and_validate_current_impl_incarnations() -> TestResult {
    let mut report = report()?;
    assert!(
        report
            .package
            .library_policy()
            .and_then(crate::resolve::stability::Library::inventory)
            .ok_or("completed inventory")?
            .local_declaration(crate::resolve::model::DefId(u32::MAX))
            .is_err()
    );
    let mut producer = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.policy.take())
        .ok_or("producer")?;
    let key = *producer
        .ineffective_headers
        .as_ref()
        .ok_or("annotations")?
        .rows
        .keys()
        .next()
        .ok_or("row")?;
    let cold = producer.work();
    let row = producer
        .impl_annotation(&report.package, key)
        .map_err(crate::resolve::package::Error::from)?
        .ok_or("actual annotation")?;
    assert_eq!(
        row.bounds.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
        vec!["sample"]
    );
    assert!(row.unstable.is_some() && row.bound.is_some());
    let warm = producer.work();
    producer
        .impl_annotation(&report.package, key)
        .map_err(crate::resolve::package::Error::from)?;
    assert_eq!(producer.work().bytes, warm.bytes);
    assert!(warm.steps > cold.steps);
    assert_eq!(size_of::<super::super::producer::Owner>(), 160);
    assert_eq!(size_of::<super::super::Policy>(), 152);
    assert_eq!(size_of::<super::super::Level>(), 1);
    let original = producer
        .ineffective_headers
        .as_ref()
        .ok_or("annotations")?
        .rows[&key]
        .tail;
    let other = producer
        .index
        .values()
        .copied()
        .find(|&tail| tail != original)
        .ok_or("opposing owner")?;
    producer
        .ineffective_headers
        .as_mut()
        .ok_or("annotations")?
        .rows
        .get_mut(&key)
        .ok_or("row")?
        .tail = other;
    assert!(producer.impl_annotation(&report.package, key).is_err());
    producer
        .ineffective_headers
        .as_mut()
        .ok_or("annotations")?
        .rows
        .get_mut(&key)
        .ok_or("row")?
        .tail = original;
    let row = producer
        .ineffective_headers
        .as_mut()
        .ok_or("annotations")?
        .rows
        .get_mut(&key)
        .ok_or("row")?;
    row.item.name = Some(crate::syntax::token_tree::Span { start: 0, end: 0 });
    let invalidated = producer.work();
    for _ in 0..2 {
        assert!(producer.impl_annotation(&report.package, key).is_err());
    }
    assert!(producer.work().steps > invalidated.steps);
    println!(
        "impl annotations cold={} warm={} failed={} charged={} producer={} row={} map={} owner={} policy={}",
        cold.steps,
        warm.steps,
        producer.work().steps,
        producer.work().bytes,
        size_of::<Producer>(),
        size_of::<Annotation>(),
        size_of::<Annotations>(),
        size_of::<super::super::producer::Owner>(),
        size_of::<super::super::Policy>()
    );
    Ok(())
}
#[test]
fn annotation_storage_and_work_follow_actual_impl_width() -> TestResult {
    use std::fmt::Write as _;
    let ordinary = crate::test_source::resolve("#[allow(dead_code)]fn plain(){}", true)?;
    assert!(
        !ordinary
            .lint_producer()
            .ok_or("producer")?
            .has_impl_annotations()
    );
    let mut observations = Vec::new();
    for width in [1, 8, 32, 128] {
        let mut source = String::from(
            "#![feature(staged_api)]#![stable(feature=\"base\",since=\"1.0.0\")]#![allow(ineffective_unstable_trait_impl)]#[stable(feature=\"base\",since=\"1.0.0\")]trait Tr{}",
        );
        for index in 0..width {
            write!(
                source,
                "#[stable(feature=\"base\",since=\"1.0.0\")]struct S{index};#[unstable(feature=\"sample\",issue=\"none\")]impl Tr for S{index}{{}}"
            )?;
        }
        let report = crate::test_source::resolve(&source, true)?;
        let facts = crate::sema::declarations::analyze(&report)?;
        let producer = report.lint_producer().ok_or("producer")?;
        assert_eq!(
            producer
                .ineffective_headers
                .as_ref()
                .ok_or("annotations")?
                .rows
                .len(),
            width
        );
        assert!(facts.warnings().next().is_none());
        assert_eq!(facts.retained_policy_rows().2, 160);
        assert!(producer.work().steps < 30_000 * (width + 1));
        assert!(producer.work().bytes < 30_000 * (width + 1));
        observations.push((producer.work().steps, producer.work().bytes));
        println!(
            "impl width={width} steps={} bytes={} dense_node={} row={}",
            producer.work().steps,
            producer.work().bytes,
            size_of::<u32>(),
            facts.retained_policy_rows().2
        );
    }
    for axis in 0..2 {
        let values = observations
            .iter()
            .map(|&(steps, bytes)| if axis == 0 { steps } else { bytes })
            .collect::<Vec<_>>();
        let first = values[2]
            .checked_sub(values[1])
            .ok_or("nonmonotone first interval")?;
        let second = values[3]
            .checked_sub(values[2])
            .ok_or("nonmonotone second interval")?;
        assert!(
            first > 0 && second <= 6 * first,
            "nonlinear impl axis {axis}: {values:?}"
        );
    }
    Ok(())
}
#[test]
fn exhausted_annotation_observation_keeps_each_failed_attempt_paid() -> TestResult {
    let mut report = report()?;
    let mut producer = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.policy.take())
        .ok_or("producer")?;
    let key = *producer
        .ineffective_headers
        .as_ref()
        .ok_or("annotations")?
        .rows
        .keys()
        .next()
        .ok_or("row")?;
    let row = producer
        .ineffective_headers
        .as_mut()
        .ok_or("annotations")?
        .rows
        .get_mut(&key)
        .ok_or("row")?;
    row.unstable = row.bound;
    let error = producer
        .impl_annotation(&report.package, key)
        .err()
        .ok_or("foreign attribute consumer accepted")?;
    assert_eq!(
        error.message,
        "impl annotation effect belongs to another attribute consumer"
    );
    let mut reads = producer.reads.get();
    reads.steps = 10_000_000;
    producer.reads.set(reads);
    let before = producer.work();
    for _ in 0..2 {
        let error = producer
            .impl_annotation(&report.package, key)
            .err()
            .ok_or("exhausted query succeeded")?;
        assert_eq!(error.message, "source lint policy work budget exceeded");
    }
    assert!(producer.work().steps >= before.steps + 2);
    assert_eq!(producer.work().bytes, before.bytes);
    Ok(())
}
