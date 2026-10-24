//! Privacy remains caller-dependent even when a pure adjustment is cached.
use super::*;
use crate::{
    expand::{
        config::Config,
        marks::{SyntaxContextId, Transparency},
    },
    resolve::{
        load::{InputFile, Sources},
        model::DefKind,
    },
    support::source::SourceFile,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}

#[test]
fn actual_expansion_owns_privacy_and_cached_roots_keep_each_caller() -> TestResult {
    let source = "mod definition { macro_rules! make { () => { pub fn made() {} } } make!(); } mod caller {}";
    let mut report = crate::resolve::expansion::analyze(
        InputFile {
            identity: "privacy.rs".into(),
            path: "privacy.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?;
    let invocation = report.invocations.first().ok_or("missing invocation")?;
    let expansion = invocation.context;
    let definition = report.definitions[invocation.definition].module;
    let caller = report
        .package
        .modules
        .iter()
        .position(|module| report.package.names.get(module.name) == "caller")
        .ok_or("missing caller")?;
    let caller = ModuleId(u32::try_from(caller)?);
    let made = report
        .resolution
        .definitions
        .iter()
        .position(|item| {
            item.kind == DefKind::Function && report.package.names.get(item.name) == "made"
        })
        .ok_or("missing generated item")?;
    let made = DefId(u32::try_from(made)?);
    // Supplied opaque provenance isolates the API while scoped source admission
    // remains disabled. This is not an end-to-end scoped macro acceptance test.
    let syntax = report.package.marks.apply(
        SyntaxContextId::ROOT,
        expansion,
        Transparency::Opaque,
        SyntaxContextId::ROOT,
    )?;
    let context = report.package.marks.item(syntax)?;
    let root = report.resolution.crate_root.ok_or("missing crate root")?;
    for site in [caller, ModuleId(0), definition] {
        let adjusted = report.adjust_item(root, context, site)?;
        assert_eq!(adjusted.context, ItemContext::ROOT);
        assert_eq!(adjusted.privacy, definition);
        let unchanged = report.adjust_item(made, context, site)?;
        assert_eq!(unchanged.context, context);
        assert_eq!(unchanged.privacy, site);
        let captured = report.adjust_item(made, ItemContext::ROOT, site)?;
        assert_eq!(captured.context, ItemContext::ROOT);
        assert_eq!(captured.privacy, site);
    }
    assert!(
        report
            .adjust_item(DefId(u32::MAX), context, caller)
            .is_err()
    );
    assert!(
        report
            .adjust_item(root, context, ModuleId(u32::MAX))
            .is_err()
    );
    report.invocations[0].definition = usize::MAX;
    assert!(report.adjust_item(root, context, caller).is_err());
    Ok(())
}
