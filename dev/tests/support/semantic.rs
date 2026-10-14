//! Shared in-memory resolution and semantic analysis for maintainer invariants.
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{self, model::Declarations},
    support::source::SourceFile,
};
use std::path::Path;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
pub fn expand(source: &str) -> Result<Report, String> {
    resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .map_err(|e| e.to_string())
}
pub fn resolve(source: &str) -> Result<Report, String> {
    resolve::analyze_expanded(expand(source)?).map_err(|e| e.to_string())
}
pub fn analyze(source: &str) -> Result<(Report, Declarations), String> {
    let report = resolve(source)?;
    let facts = sema::declarations::analyze(&report)
        .and_then(|f| sema::bodies::analyze(&report, f))
        .map_err(|e| e.to_string())?;
    Ok((report, facts))
}
