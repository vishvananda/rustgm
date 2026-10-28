//! Source owner absence, retained incarnation and failed-query work are explicit.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        expansion,
        load::{InputFile, Sources},
        package::Error,
    },
    support::source::SourceFile,
};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
pub(super) fn report(source: &str) -> TestResult<expansion::Report> {
    Ok(expansion::analyze(
        InputFile {
            identity: "policy.rs".into(),
            path: "policy.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)
}
#[test]
fn no_attributes_allocate_no_effects_but_nonlint_attributes_have_positive_empty_records()
-> TestResult {
    assert!(report("fn f(){}")?.attribute_effects.is_none());
    let report = report("#[doc=\"retained\"]fn f(){}")?;
    let producer = report
        .lint_producer()
        .ok_or("missing positive-empty producer")?;
    assert_eq!(producer.owners.len(), 1);
    assert!(producer.owners[0].parsed.changes.is_empty());
    assert_eq!(
        producer.validate(&report.package).map_err(Error::from)?,
        (0, 0, false)
    );
    assert!(producer.frames.is_empty() && producer.calls.is_empty());
    println!(
        "lint policy layouts Effects={} Producer={} Owner={} Witness={} Frame={} Call={} Site={} Tail={} Policy={} Work={}",
        size_of::<expansion::attribute_effects::Effects>(),
        size_of::<Producer>(),
        size_of::<Owner>(),
        size_of::<Witness>(),
        size_of::<Frame>(),
        size_of::<Call>(),
        size_of::<Site>(),
        size_of::<Option<Tail>>(),
        size_of::<Policy>(),
        size_of::<Work>()
    );
    assert_eq!(size_of::<Frame>(), 8);
    assert_eq!(size_of::<Option<Tail>>(), 4);
    let offsets = [
        ("size.Owner", size_of::<Owner>()),
        ("size.Witness", size_of::<Witness>()),
        ("size.Change", size_of::<Change>()),
        (
            "size.TreePair",
            size_of::<(u32, crate::syntax::token_tree::Tree)>(),
        ),
        ("Work.attributes", std::mem::offset_of!(Work, attributes)),
        ("Work.entries", std::mem::offset_of!(Work, entries)),
        ("Work.owners", std::mem::offset_of!(Work, owners)),
        ("Work.frames", std::mem::offset_of!(Work, frames)),
        ("Work.calls", std::mem::offset_of!(Work, calls)),
        (
            "Work.compositions",
            std::mem::offset_of!(Work, compositions),
        ),
        ("Work.steps", std::mem::offset_of!(Work, steps)),
        ("Work.bytes", std::mem::offset_of!(Work, bytes)),
        ("Producer.owners", std::mem::offset_of!(Producer, owners)),
        ("Producer.index", std::mem::offset_of!(Producer, index)),
        ("Producer.frames", std::mem::offset_of!(Producer, frames)),
        ("Producer.calls", std::mem::offset_of!(Producer, calls)),
        (
            "Producer.composed",
            std::mem::offset_of!(Producer, composed),
        ),
        ("Producer.work", std::mem::offset_of!(Producer, work)),
        ("Producer.reads", std::mem::offset_of!(Producer, reads)),
        ("Owner.parsed", std::mem::offset_of!(Owner, parsed)),
        ("Parsed.changes", std::mem::offset_of!(Parsed, changes)),
        ("Parsed.witnesses", std::mem::offset_of!(Parsed, witnesses)),
        ("Parsed.trees", std::mem::offset_of!(Parsed, trees)),
    ];
    println!("policy-observer-layout={offsets:?}");
    Ok(())
}
#[test]
fn stale_effect_source_image_and_unseen_owner_are_rejected_with_charged_failed_reads() -> TestResult
{
    let mut report = report("#[allow(unused_must_use)]fn f(){} #[doc=\"other\"]fn g(){}")?;
    let producer = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.policy.take())
        .ok_or("missing actual producer")?;
    producer.validate(&report.package).map_err(Error::from)?;
    let key = producer.owners[0].key;
    let index = report.package.unit(key.file).owner_indices[key.node.get() as usize]
        .ok_or("owner index")?
        .get() as usize
        - 1;
    let range = report.package.units[key.file.0 as usize].configured.owners[index]
        .attributes
        .clone();
    report.package.units[key.file.0 as usize].configured.owners[index].attributes =
        range.start..range.end + 1;
    let before = producer.work().steps;
    assert!(producer.validate(&report.package).is_err());
    assert!(producer.work().steps > before);
    report.package.units[key.file.0 as usize].configured.owners[index].attributes = range;
    producer.validate(&report.package).map_err(Error::from)?;
    let capture = producer.owners[0].parsed.witnesses[0].capture as usize;
    let forest = report.package.units[key.file.0 as usize]
        .configured
        .ast
        .attributes[capture]
        .forest
        .clone();
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .attributes[capture]
        .forest = crate::syntax::token_tree::Forest::parse("#[warn(unused_must_use)]")
        .map_err(|error| error.message)?;
    assert!(producer.validate(&report.package).is_err());
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .attributes[capture]
        .forest = forest;
    producer.validate(&report.package).map_err(Error::from)?;
    let effect = producer.owners[0].parsed.witnesses[0].effect as usize;
    let original = report.package.units[key.file.0 as usize].configured.effects[effect].capture;
    report.package.units[key.file.0 as usize].configured.effects[effect].capture = u32::MAX;
    assert!(producer.validate(&report.package).is_err());
    report.package.units[key.file.0 as usize].configured.effects[effect].capture = original;
    let copy = report.package.unit(key.file).source.physical_copy();
    report.package.units[key.file.0 as usize].source = copy;
    assert!(producer.validate(&report.package).is_err());
    let empty = Producer {
        sealed: true,
        ..Producer::new(0, 0)
    };
    assert!(empty.validate(&report.package).is_err());
    Ok(())
}

#[test]
fn combined_limits_report_actual_storage_then_work_and_keep_failed_attempts() -> TestResult {
    let report = report("fn f(){}")?;
    let key = NodeKey {
        file: crate::resolve::unit::FileId(0),
        node: report.package.modules[0].root,
    };
    let mut producer = Producer::new(0, 0);
    producer
        .reserve_diagnostic(key, 64 * 1024 * 1024)
        .map_err(Error::from)?;
    producer
        .attribute_read(key, 9_999_999)
        .map_err(Error::from)?;
    assert!(!producer.diagnostic_budget_exhausted());
    let error = producer
        .diagnostic_payload(key, 0, 1)
        .err()
        .ok_or("combined storage limit was not enforced")?;
    assert_eq!(error.message, "source lint policy storage budget exceeded");
    assert_eq!(producer.work().bytes, 64 * 1024 * 1024 + 1);
    assert_eq!(producer.work().steps, 10_000_000);
    let error = producer
        .attribute_read(key, 1)
        .err()
        .ok_or("failed storage payer resumed after work exhaustion")?;
    assert_eq!(
        error.message,
        "source lint policy work and storage budgets exceeded"
    );
    assert_eq!(producer.work().steps, 10_000_001);
    let error = producer
        .validate(&report.package)
        .err()
        .ok_or("combined validation lost the exhausted counters")?;
    assert_eq!(
        error.message,
        "source lint policy work and storage budgets exceeded"
    );
    assert!(producer.diagnostic_budget_exhausted());
    Ok(())
}

#[test]
fn failed_frame_growth_keeps_real_tail_storage_and_extent_unpublished() -> TestResult {
    let report = report("#[allow(unused_must_use)]fn f(){}")?;
    let key = report.lint_producer().ok_or("actual source owner")?.owners[0].key;
    let mut producer = Producer::new(0, 0);
    let tail = producer
        .owner(&report.package, key, None)
        .map_err(Error::from)?;
    assert!(tail.is_some());
    producer.set_frame(0, tail, key).map_err(Error::from)?;
    let capacity = producer.frames.capacity();
    for index in 1..capacity {
        producer
            .set_frame(u32::try_from(index)?, tail, key)
            .map_err(Error::from)?;
    }
    let bytes = (64 * 1024 * 1024_usize)
        .checked_sub(producer.work().bytes)
        .ok_or("unexpected initial storage usage")?;
    producer
        .reserve_diagnostic(key, bytes)
        .map_err(Error::from)?;
    let next = producer.frame_extent;
    let steps = producer.work().steps;
    let error = producer
        .set_frame(next, tail, key)
        .err()
        .ok_or("unpaid frame growth published a tail")?;
    assert_eq!(error.message, "source lint policy storage budget exceeded");
    assert_eq!(producer.frame_extent, next);
    assert_eq!(producer.frames.len(), capacity);
    assert_eq!(producer.frames.capacity(), capacity);
    assert!(producer.work().steps > steps);
    let failed = producer.work().steps;
    assert!(producer.set_frame(next, None, key).is_err());
    assert!(producer.work().steps > failed);
    assert_eq!(producer.frame_extent, next);
    Ok(())
}

#[test]
fn expectation_owner_index_growth_is_paid_before_publication() -> TestResult {
    let report = report(
        "#[expect(deprecated)]fn f0(){}#[expect(deprecated)]fn f1(){}#[expect(deprecated)]fn f2(){}#[expect(deprecated)]fn f3(){}#[expect(deprecated)]fn f4(){}",
    )?;
    let source = report.lint_producer().ok_or("producer")?;
    let root = NodeKey {
        file: crate::resolve::unit::FileId(0),
        node: report.package.modules[0].root,
    };
    let keys: Vec<_> = source
        .expectation_owners
        .iter()
        .map(|tail| {
            source
                .record(*tail, root)
                .map(|owner| owner.key)
                .map_err(Error::from)
        })
        .collect::<Result<_, _>>()?;
    let mut observed = Producer::new(0, 0);
    for &key in &keys[..4] {
        observed
            .owner(&report.package, key, None)
            .map_err(Error::from)?;
    }
    let growth = crate::support::retained::Growth::vector::<Tail>(
        observed.expectation_owners.len(),
        observed.expectation_owners.capacity(),
        1,
    );
    assert!(growth.bytes > 0);
    let before = observed.work().bytes;
    observed
        .owner(&report.package, keys[4], None)
        .map_err(Error::from)?;
    let delta = observed.work().bytes - before;
    let mut limited = Producer::new(0, 0);
    for &key in &keys[..4] {
        limited
            .owner(&report.package, key, None)
            .map_err(Error::from)?;
    }
    limited.work.bytes = (64 * 1024 * 1024_usize)
        .checked_sub(delta - growth.bytes)
        .ok_or("budget extent")?;
    let steps = limited.work().steps;
    let error = limited
        .owner(&report.package, keys[4], None)
        .err()
        .ok_or("unpaid index was published")?;
    assert_eq!(error.message, "source lint policy storage budget exceeded");
    assert_eq!(limited.expectation_owners.len(), 4);
    assert_eq!(limited.owners.len(), 4);
    assert_eq!(limited.index.len(), 4);
    assert_eq!(limited.work().bytes, 64 * 1024 * 1024 + growth.bytes);
    assert!(limited.work().steps > steps);
    assert!(limited.owner(&report.package, keys[4], None).is_err());
    assert_eq!(limited.expectation_owners.len(), 4);
    Ok(())
}
