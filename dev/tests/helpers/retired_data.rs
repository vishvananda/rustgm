//! The same checked metadata-only provider feeds public and private controls.
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{
        bindings::Mode,
        crates::{CrateIdentity, Dependencies, Dependency, Interface},
        diagnostics::LintCap,
        source::SourceFile,
    },
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
pub type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
pub const PACKET: &str = "../pa34/tests/spec/removed-library-features";
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
pub fn packet() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(PACKET)
}
pub fn fixture(file: &str) -> Result<String> {
    Ok(std::fs::read_to_string(packet().join(file))?)
}
fn report(file: &str, context: &Context) -> Result<Report> {
    let path = packet().join(file);
    let root = InputFile {
        identity: path.clone(),
        path,
        source: SourceFile::new(fixture(file)?.into_bytes())?,
    };
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let expanded = resolve::expansion::analyze_with_context(root, &mut NoFiles, &config, context)?;
    Ok(resolve::analyze_expanded(expanded)?)
}
pub fn source(file: &str) -> Result<Report> {
    let identity = CrateIdentity::new("removed_provider", "retired_controls")?;
    report(
        file,
        &Context {
            crate_name: Some(identity.name().into()),
            current: Some(identity.clone()),
            mode: Mode::Source { provider: identity },
            ..Context::default()
        },
    )
}
pub fn provider() -> Result<Interface> {
    let report = source("provider.rs")?;
    let rows = report
        .package
        .removed_features()?
        .ok_or("checked removed provider records")?;
    Ok(
        Interface::new(rows.owner().clone(), Vec::new(), Vec::new())?
            .with_removed_features(rows)?,
    )
}
pub fn client(artifact: Interface, file: &str) -> Result<Report> {
    let alias = Dependency::new("removed_provider".into(), artifact.identity().clone())?;
    let identity = CrateIdentity::new("removed_client", "retired_controls")?;
    report(
        file,
        &Context {
            crate_name: Some(identity.name().into()),
            current: Some(identity),
            dependencies: Some(Arc::new(Dependencies::new(vec![artifact], vec![alias])?)),
            ..Context::default()
        },
    )
}
