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
    row.unstable = row.bound.first().map(|bound| bound.effect);
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
#[test]
fn feature_name_validation_prepays_attempted_bytes_before_retention() -> TestResult {
    let name = "a".repeat(64);
    let source = format!(
        "#![feature(staged_api)]#![stable(feature=\"base\",since=\"1.0.0\")]#[unstable(feature=\"sample\",issue=\"none\")]#[unstable_feature_bound({name})]fn f(){{}}"
    );
    let report = crate::test_source::resolve(&source, true)?;
    let producer = report.lint_producer().ok_or("source annotation producer")?;
    let (&key, row) = producer
        .ineffective_headers
        .as_ref()
        .ok_or("annotation column")?
        .rows
        .iter()
        .next()
        .ok_or("source annotation")?;
    let owner = producer
        .record(row.tail, key)
        .map_err(crate::resolve::package::Error::from)?;
    let unit = report.package.unit(key.file);
    let witness = owner
        .parsed
        .witnesses
        .iter()
        .find(|witness| {
            let capture = &unit.configured.ast.attributes[witness.capture as usize];
            witness.attribute.name(&capture.forest, unit.source.text())
                == Some("unstable_feature_bound")
        })
        .ok_or("actual bound effect")?;
    let forest = &unit.configured.ast.attributes[witness.capture as usize].forest;
    let mut work = Work {
        steps: 10_000_000 - 32,
        ..Work::default()
    };
    let mut bounds = Vec::new();
    let error = feature_bounds(unit, key, witness.attribute, forest, &mut bounds, &mut work)
        .err()
        .ok_or("unpaid name retained")?;
    assert!(error.message.contains("work budget"));
    assert!(work.steps > 10_000_000 && bounds.is_empty());
    let before = work.steps;
    assert!(feature_bounds(unit, key, witness.attribute, forest, &mut bounds, &mut work).is_err());
    assert!(work.steps > before && bounds.is_empty());
    Ok(())
}
#[test]
fn combined_bounds_authenticate_each_effect_and_its_name_partition() -> TestResult {
    let source =
        crate::test_source::fixture("pa40/tests/spec/ineffective-unstable-impl/bound-combined.rs")?;
    let mut report = crate::test_source::resolve(&source, true)?;
    let mut producer = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.policy.take())
        .ok_or("actual producer")?;
    let key = *producer
        .ineffective_headers
        .as_ref()
        .ok_or("annotation column")?
        .rows
        .keys()
        .next()
        .ok_or("combined row")?;
    let row = &producer.ineffective_headers.as_ref().ok_or("column")?.rows[&key];
    let effects = row.bound.clone();
    let names = row.bounds.clone();
    assert_eq!(effects.len(), 2);
    assert_eq!(size_of::<BoundEffect>(), 12);
    assert_eq!(
        (
            effects[0].start,
            effects[0].end,
            effects[1].start,
            effects[1].end
        ),
        (0, 1, 1, 2)
    );
    let owner = producer
        .record(row.tail, key)
        .map_err(crate::resolve::package::Error::from)?;
    let first = attribute(
        &producer,
        report.package.unit(key.file),
        owner,
        effects[0].effect,
        "unstable_feature_bound",
    )
    .map_err(crate::resolve::package::Error::from)?;
    let view = producer
        .impl_annotation(&report.package, key)
        .map_err(crate::resolve::package::Error::from)?
        .ok_or("actual combined view")?;
    assert_eq!(
        view.bounds.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
        vec!["other", "sample"]
    );
    assert_eq!(view.bound.ok_or("first bound")?.span, first.span);
    let cold = producer.work();
    producer
        .declaration_annotation(&report.package, key)
        .map_err(crate::resolve::package::Error::from)?;
    assert!(producer.work().steps > cold.steps);
    assert_eq!(producer.work().bytes, cold.bytes);
    for corrupt in 0..4 {
        let row = producer
            .ineffective_headers
            .as_mut()
            .ok_or("column")?
            .rows
            .get_mut(&key)
            .ok_or("actual row")?;
        match corrupt {
            0 => row.bound[0].effect = effects[1].effect,
            1 => {
                row.bound[0].end += 1;
                row.bound[1].start += 1;
            }
            2 => {
                row.bound.pop();
                row.bounds.truncate(usize::try_from(effects[0].end)?);
            }
            _ => row.bound.swap(0, 1),
        }
        let before = producer.work();
        assert!(
            producer
                .declaration_annotation(&report.package, key)
                .is_err()
        );
        assert!(producer.work().steps > before.steps);
        let row = producer
            .ineffective_headers
            .as_mut()
            .ok_or("column")?
            .rows
            .get_mut(&key)
            .ok_or("actual row")?;
        row.bound.clone_from(&effects);
        row.bounds.clone_from(&names);
        producer
            .declaration_annotation(&report.package, key)
            .map_err(crate::resolve::package::Error::from)?;
    }
    Ok(())
}
#[test]
fn effect_record_growth_is_paid_before_names_or_effects_are_retained() -> TestResult {
    let source =
        crate::test_source::fixture("pa40/tests/spec/ineffective-unstable-impl/bound-combined.rs")?;
    let report = crate::test_source::resolve(&source, true)?;
    let producer = report.lint_producer().ok_or("actual producer")?;
    let (&key, row) = producer
        .ineffective_headers
        .as_ref()
        .ok_or("column")?
        .rows
        .iter()
        .next()
        .ok_or("combined row")?;
    let owner = producer
        .record(row.tail, key)
        .map_err(crate::resolve::package::Error::from)?;
    let witness = owner
        .parsed
        .witnesses
        .iter()
        .find(|witness| witness.effect == row.bound[1].effect)
        .ok_or("second actual effect")?;
    let mut work = Work {
        bytes: 64 * 1024 * 1024,
        ..Work::default()
    };
    let mut effects = Vec::new();
    let mut names = Vec::new();
    for _ in 0..2 {
        let before = (work.steps, work.bytes);
        assert!(
            capture_bound(
                report.package.unit(key.file),
                key,
                witness,
                &mut effects,
                &mut names,
                &mut work
            )
            .is_err()
        );
        assert!(effects.is_empty() && names.is_empty());
        assert!(work.steps > before.0 && work.bytes > before.1);
    }
    Ok(())
}
#[test]
fn repeated_effect_query_work_tracks_the_actual_retained_attribute_count() -> TestResult {
    use std::fmt::Write as _;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source = String::from(
            "#![feature(staged_api)]#![stable(feature=\"base\",since=\"1.0.0\")]#[unstable(feature=\"sample\",issue=\"none\")]",
        );
        for index in 0..width {
            write!(source, "#[unstable_feature_bound(feature_{index})]")?;
        }
        source.push_str("fn f(){}\n");
        let report = crate::test_source::resolve(&source, true)?;
        let producer = report.lint_producer().ok_or("actual producer")?;
        let (&key, row) = producer
            .ineffective_headers
            .as_ref()
            .ok_or("column")?
            .rows
            .iter()
            .next()
            .ok_or("actual row")?;
        assert_eq!(row.bound.len(), width);
        let before = producer.work();
        let view = producer
            .declaration_annotation(&report.package, key)
            .map_err(crate::resolve::package::Error::from)?
            .ok_or("actual view")?;
        assert_eq!(view.bounds.len(), width);
        let cold = producer.work();
        producer
            .declaration_annotation(&report.package, key)
            .map_err(crate::resolve::package::Error::from)?;
        let warm = producer.work();
        assert_eq!(warm.bytes, before.bytes);
        observations.push((cold.steps - before.steps, warm.steps - cold.steps));
    }
    for axis in 0..2 {
        let values: Vec<_> = observations
            .iter()
            .map(|&(cold, warm)| if axis == 0 { cold } else { warm })
            .collect();
        let first = values[1]
            .checked_sub(values[0])
            .ok_or("nonmonotone first interval")?;
        let second = values[2]
            .checked_sub(values[1])
            .ok_or("nonmonotone second interval")?;
        assert!(first > 0 && second <= 6 * first, "{values:?}");
    }
    Ok(())
}
