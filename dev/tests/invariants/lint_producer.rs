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
fn report(source: &str) -> TestResult<expansion::Report> {
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
    let effect = producer.owners[0].parsed.witnesses[0].effect;
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
