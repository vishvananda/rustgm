//! Foreign-interface checks use the ordinary typed frontend and checked demand pipeline.
use rustgm::{
    expand::config::Config,
    resolve::{self, load::InputFile},
    sema,
    support::source::SourceFile,
};
struct NoFiles;
impl resolve::load::Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
pub fn analyze(
    source: &str,
) -> Result<(resolve::expansion::Report, rustgm::pipeline::Analysis), String> {
    let input = InputFile {
        identity: "lib.rs".into(),
        path: "lib.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    let expanded = resolve::expansion::analyze(input, &mut NoFiles, &Config::default())
        .map_err(|e| e.to_string())?;
    let report = resolve::analyze_expanded(expanded).map_err(|e| e.to_string())?;
    let analysis = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    Ok((report, analysis))
}
pub fn checked(source: &str) -> Result<rustgm::lowir::program::Program, String> {
    let (
        mut report,
        rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(source)?;
    let entry = sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
        .map_err(|e| e.to_string())
}
