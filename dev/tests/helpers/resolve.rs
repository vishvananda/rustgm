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

/// Test-only actual-provider context, without bootstrap declarations.
#[allow(
    dead_code,
    reason = "shared helper is compiled by suites without source providers"
)]
pub fn source_provider(source: &str, tag: &str) -> Result<Report, String> {
    use rustgm::{
        resolve::bindings::Context,
        support::{bindings::Mode, crates::CrateIdentity},
    };
    let identity = CrateIdentity::new("minicore", tag).map_err(str::to_owned)?;
    let options = Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    let input = InputFile {
        identity: "source-provider.rs".into(),
        path: "source-provider.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    let expanded =
        resolve::expansion::analyze_with_context(input, &mut NoFiles, &Config::default(), &options)
            .map_err(|e| e.to_string())?;
    resolve::analyze_expanded(expanded).map_err(|e| e.to_string())
}
