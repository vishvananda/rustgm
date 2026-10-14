use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::source::SourceFile,
};
use std::path::Path;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
pub fn resolve(source: &str, expanded: bool) -> Result<Report, String> {
    let mut report = resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .map_err(|error| error.to_string())?;
    if expanded {
        resolve::analyze_expanded(report).map_err(|error| error.to_string())
    } else {
        report.resolution = resolve::analyze(&mut report.package).map_err(|e| e.to_string())?;
        Ok(report)
    }
}
