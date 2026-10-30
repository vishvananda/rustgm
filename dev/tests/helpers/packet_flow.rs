//! Shared actual provider/codec client proof path; no fabricated declarations.
use crate::{
    lower, pipeline, resolve,
    support::{
        bindings::Mode,
        crates::{self, CrateIdentity, Dependencies, Dependency},
    },
};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl resolve::load::Sources for NoFiles {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> std::result::Result<Option<resolve::load::InputFile>, String> {
        Ok(None)
    }
}
pub(crate) fn exported(source: &str) -> Result<crates::Interface> {
    let report = crate::test_source::source_provider(source, "source_transaction")?;
    let identity = report
        .package
        .source_core()
        .ok_or("provider policy")?
        .provider
        .clone();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let (_, interface) =
        lower::instances::build_library_prepared(&report, &mut facts, identity, prepared)?;
    Ok(interface)
}
pub(crate) fn client_analysis(
    interface: crates::Interface,
    source: &str,
) -> Result<(
    resolve::expansion::Report,
    pipeline::Analysis,
    CrateIdentity,
)> {
    client_analysis_with(interface, Vec::new(), source)
}
pub(crate) fn client_analysis_with(
    interface: crates::Interface,
    extra: Vec<crates::Interface>,
    source: &str,
) -> Result<(
    resolve::expansion::Report,
    pipeline::Analysis,
    CrateIdentity,
)> {
    let (report, current) = client_report_with(interface, extra, source)?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis, current))
}
pub(crate) fn client_report(
    interface: crates::Interface,
    source: &str,
) -> Result<resolve::expansion::Report> {
    Ok(client_report_with(interface, Vec::new(), source)?.0)
}
fn client_report_with(
    interface: crates::Interface,
    extra: Vec<crates::Interface>,
    source: &str,
) -> Result<(resolve::expansion::Report, CrateIdentity)> {
    let provider = interface.identity().clone();
    let mut aliases = vec![Dependency::new("base".into(), provider.clone())?];
    for (index, artifact) in extra.iter().enumerate() {
        aliases.push(Dependency::new(
            format!("dependency_{index}"),
            artifact.identity().clone(),
        )?);
    }
    let mut artifacts = vec![interface];
    artifacts.extend(extra);
    let dependencies = Arc::new(Dependencies::new(artifacts, aliases)?);
    let current = CrateIdentity::new("client", "source_transaction")?;
    let context = resolve::bindings::Context {
        crate_name: Some("client".into()),
        current: Some(current.clone()),
        dependencies: Some(dependencies),
        mode: Mode::SourceClient { provider },
    };
    let expanded = resolve::expansion::analyze_with_context(
        resolve::load::InputFile {
            identity: "client.rs".into(),
            path: "client.rs".into(),
            source: crate::support::source::SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &crate::expand::config::Config::default(),
        &context,
    )?;
    let report = resolve::analyze_expanded(expanded)?;
    assert_eq!(
        report.package.units.len(),
        1,
        "provider syntax leaked into fresh client"
    );
    let imported = report
        .resolution
        .crates
        .as_ref()
        .ok_or("actual source inventory")?;
    for definition in imported.generic_definitions() {
        assert!(
            report.resolution.definitions[definition?.0 as usize]
                .node
                .is_none()
        );
    }
    Ok((report, current))
}

pub(crate) fn client(
    interface: crates::Interface,
    source: &str,
) -> Result<crate::lowir::program::Program> {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
        current,
    ) = client_analysis(interface, source)?;
    let (program, _) =
        lower::instances::build_library_prepared(&report, &mut facts, current, prepared)?;
    Ok(program)
}
