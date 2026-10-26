//! Parsed keyword ownership and direct feature warnings use genuine source graphs.
use super::*;
use crate::{
    expand::config::Config,
    pipeline,
    resolve::{
        self,
        load::InputFile,
        model::{DefId, DefKind},
        unit::{FileId, Unit},
    },
    support::source::SourceFile,
    syntax::{
        ast::{NodeId, Syntax},
        gate_sites::Kind as DefaultKind,
        item::ItemKind,
    },
};
use std::path::Path;
#[path = "modifier_authority.rs"]
mod authority;
#[path = "specialization_bodies.rs"]
mod callables;
#[path = "modifier_storage.rs"]
mod costs;
#[path = "modifier_transfers.rs"]
mod transfers;
#[path = "syntax_feature_warnings.rs"]
mod warnings;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str) -> Result<Report, Error> {
    let expanded = resolve::expansion::analyze(
        InputFile {
            identity: "defaults.rs".into(),
            path: "defaults.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec()).map_err(Error::unlocated)?,
        },
        &mut NoFiles,
        &Config::default(),
    )?;
    resolve::analyze_expanded(expanded)
}
fn fixture(lane: &str, name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/defaultness/{lane}/{name}.rs"
    ))?)
}
fn resolved(lane: &str, name: &str) -> TestResult<Report> {
    Ok(report(&fixture(lane, name)?)?)
}
fn definition(report: &Report, kind: DefKind, name: &str) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|actual| actual.kind == kind && report.package.names.get(actual.name) == name)
        .ok_or("actual named declaration")?;
    Ok(DefId(u32::try_from(index)?))
}
