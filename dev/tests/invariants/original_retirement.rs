//! Original rows restore exact pre-retirement flags and compose old macrocall overwrites.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        expansion::{self, Report},
        load::{InputFile, Sources},
    },
};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn expanded(source: &str) -> TestResult<Report> {
    Ok(expansion::analyze(
        InputFile {
            identity: "original.rs".into(),
            path: "original.rs".into(),
            source: crate::support::source::SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)
}
#[test]
fn original_test_flags_keep_disabled_owners_and_first_generated_overwrite() -> TestResult {
    let source = "#[test]fn gone(){#[cfg(ub_checks)]fn nested(){}}";
    let mut report = expanded(source)?;
    let unit = &mut report.package.units[0];
    let copy = unit.fresh_instance().map_err(|error| error.message)?;
    assert!(copy.configured.owners.iter().any(|owner| owner.enabled));
    let disabled = copy
        .configured
        .owners
        .iter()
        .find(|owner| !owner.enabled)
        .ok_or("original disabled owner")?;
    assert!(!copy.configured.active[disabled.node.get() as usize]);
    assert!(!std::sync::Arc::ptr_eq(
        unit.source.original_image(),
        copy.source.original_image()
    ));
    assert_eq!(
        unit.source.original().as_ptr(),
        copy.source.original().as_ptr()
    );
    println!(
        "retirement Saved={} Original={} Unit={} journal_rows={}",
        size_of::<Saved>(),
        size_of::<Original>(),
        size_of::<Unit>(),
        unit.original.overwritten.len()
    );
    let saved = unit
        .original
        .overwritten
        .values_mut()
        .find(|saved| saved.owner.is_some())
        .ok_or("actual saved attributed owner")?;
    saved.owner = NonZeroU32::new(u32::MAX);
    let error = unit
        .fresh_instance()
        .err()
        .ok_or("generated-only original owner accepted")?;
    assert!(
        error
            .message
            .contains("invalid saved original attribute owner")
    );

    let generated = expanded(
        "macro_rules!make{()=>{#[test]fn gone(){#[cfg(ub_checks)]fn nested(){}}}}make!();",
    )?;
    let unit = &generated.package.units[0];
    let (&id, saved) = unit
        .original
        .overwritten
        .iter()
        .find(|(_, saved)| matches!(saved.node.kind, crate::syntax::ast::Kind::Macro { .. }))
        .ok_or("first original macrocall row")?;
    assert!(saved.owner.is_none());
    assert!(saved.flags.active());
    let fresh = unit.fresh_instance().map_err(|error| error.message)?;
    assert_eq!(fresh.configured.ast.node(id), saved.node);
    assert!(fresh.configured.active[id.get() as usize]);
    assert!(fresh.owner_indices[id.get() as usize].is_none());
    Ok(())
}
