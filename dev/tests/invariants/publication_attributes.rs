//! Publishing checks source contracts without turning omission into absence.
use super::*;
use crate::{
    resolve::{self, model::DefKind, package::Error},
    support::{
        bindings::Mode,
        crates::{self, CrateIdentity, Dependencies, Dependency},
    },
};
use std::sync::Arc;
#[path = "closed_default_headers.rs"]
mod defaults;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-publication-attributes/{name}.rs"
    ))?)
}
fn plain() -> Result<String> {
    Ok(crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?)
}
fn prepared(text: &str, tag: &str) -> Result<(Report, crate::pipeline::Analysis, CrateIdentity)> {
    let report = crate::test_source::source_provider(text, tag)?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual source provider")?
        .provider
        .clone();
    let analysis = crate::pipeline::analyze_source(&report)?;
    Ok((report, analysis, identity))
}
fn staged(report: &Report, facts: &mut Declarations, identity: &CrateIdentity) -> Result<Source> {
    Ok(Source {
        stage: Stage::begin(report, facts, identity)?,
        openings: HashMap::default(),
        numbering: None,
    })
}
fn publish(text: &str) -> Result<crates::Interface> {
    let (report, mut analysis, identity) = prepared(text, "attribute_publication")?;
    let (_, interface) = crate::lower::instances::build_library_prepared(
        &report,
        &mut analysis.facts,
        identity,
        analysis.prepared,
    )?;
    Ok(interface)
}

#[test]
fn plain_and_configured_absent_headers_publish_with_the_existing_profile() -> Result {
    defaults::reject_untransported_defaultness()?;
    let original = publish(&plain()?)?;
    let disabled = publish(&fixture("cfg-off")?)?;
    assert_eq!(crates::encode(&original)?, crates::encode(&disabled)?);
    assert_eq!(
        original
            .source_descriptor()
            .ok_or("source descriptor")?
            .profile(),
        crates::SourceProfile::Source2024Functions
    );
    let (report, analysis, _) = prepared(&fixture("negative-owner")?, "negative_owner")?;
    assert_eq!(analysis.facts.warnings().count(), 1);
    let module = report
        .resolution
        .definitions
        .iter()
        .position(|definition| {
            definition.kind == DefKind::Module
                && report.package.names.get(definition.name) == "ignored"
        })
        .ok_or("actual ignored module")?;
    assert!(
        analysis
            .facts
            .must_use_contract(&report, DefId(u32::try_from(module)?))?
            .is_none()
    );
    let error = publish(&fixture("negative-owner")?)
        .err()
        .ok_or("nested namespace profile expanded")?;
    assert!(
        error
            .to_string()
            .contains("nested source modules require a wider source namespace profile")
    );
    let (report, mut analysis, identity) =
        prepared(&fixture("negative-crate-owner")?, "negative_crate_owner")?;
    let warnings: Vec<_> = analysis.facts.warnings().collect();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].code.name(), "unused_attributes");
    let root = report.resolution.crate_root.ok_or("actual crate owner")?;
    assert!(analysis.facts.must_use_contract(&report, root)?.is_none());
    staged(&report, &mut analysis.facts, &identity)?
        .require_plain_attributes(&report, &analysis.facts)?;
    publish(&fixture("negative-crate-owner")?)?;
    for name in ["nominal-plain", "nominal-marked"] {
        let error = publish(&fixture(name)?)
            .err()
            .ok_or("unsupported nominal source profile accepted")?;
        assert!(
            error
                .to_string()
                .contains("source declaration kind is outside the initial source profile"),
            "{name}: {error}"
        );
    }
    Ok(())
}

fn marked_site(report: &Report, facts: &Declarations, family: &str) -> Result<(u32, u32)> {
    for (index, actual) in report.resolution.definitions.iter().enumerate() {
        if actual.node.is_none() || !matches!(actual.kind, DefKind::Function | DefKind::Trait) {
            continue;
        }
        let definition = DefId(u32::try_from(index)?);
        if family == "must_use" {
            if let Some(view) = facts.must_use_contract(report, definition)? {
                let source = view.origin.source().ok_or("actual source contract")?;
                return Ok((source.owner.file.0, source.span.start));
            }
        } else if let Some(view) = facts.inline_hint_for_definition(report, definition)? {
            let source = view.origin.source().ok_or("actual source hint")?;
            return Ok((source.owner.file.0, source.span.start));
        }
    }
    Err("missing actual selected semantic contract".into())
}

#[test]
fn all_selected_function_and_trait_contracts_reject_before_payload_publication() -> Result {
    for (name, family) in [
        ("must-use", "must_use"),
        ("must-use-reason", "must_use"),
        ("cfg-on", "must_use"),
        ("unused-marked", "must_use"),
        ("required-member", "must_use"),
        ("trait-owner", "must_use"),
        ("distinct-owner", "must_use"),
        ("inline", "inline"),
        ("inline-always", "inline"),
        ("inline-never", "inline"),
        ("required-inline", "inline"),
    ] {
        let text = fixture(name)?;
        let (report, analysis, _) = prepared(&text, "marked_source")?;
        let (file, offset) = marked_site(&report, &analysis.facts, family)?;
        let error = publish(&text)
            .err()
            .ok_or_else(|| format!("{name}: silently published a contract"))?;
        let typed = error
            .downcast_ref::<Error>()
            .ok_or("publication discarded its typed Error")?;
        assert_eq!(
            typed.to_string(),
            format!(
                "file {file}: {offset}: source {family} attribute contract transport is not implemented"
            ),
            "{name}"
        );
        if name == "required-inline" {
            assert_eq!(typed.warnings().len(), 1);
            assert_eq!(typed.warnings()[0].code.name(), "unused_attributes");
        }
    }
    Ok(())
}

#[test]
fn staged_identity_and_existing_query_failures_are_preserved() -> Result {
    let (mut report, mut analysis, identity) = prepared(&fixture("must-use")?, "authority_source")?;
    let mut source = staged(&report, &mut analysis.facts, &identity)?;
    let (foreign, _, _) = prepared(&plain()?, "another_provider")?;
    let error = source
        .require_plain_attributes(&foreign, &analysis.facts)
        .err()
        .ok_or("foreign provider report accepted")?;
    assert!(error.to_string().contains("another provider report"));
    source.stage.definitions.push(None);
    let error = source
        .require_plain_attributes(&report, &analysis.facts)
        .err()
        .ok_or("different arena accepted")?;
    assert!(error.to_string().contains("another declaration arena"));
    source.stage.definitions.pop();
    let definition = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "identity")
        .ok_or("actual identity")?;
    let row = source.stage.definitions[definition].ok_or("selected identity row")?;
    let original = source.stage.rows.declarations[row.0 as usize]
        .name
        .replace("other".into());
    let error = source
        .require_plain_attributes(&report, &analysis.facts)
        .err()
        .ok_or("wrong staged owner accepted")?;
    assert!(error.to_string().contains("staged declaration"));
    source.stage.rows.declarations[row.0 as usize].name = original;
    let owner = analysis
        .facts
        .must_use_contract(&report, DefId(u32::try_from(definition)?))?
        .ok_or("actual must_use source")?
        .origin
        .source()
        .ok_or("actual local must_use source origin")?
        .owner;
    let unit = &mut report.package.units[owner.file.0 as usize];
    let index = unit.owner_indices[owner.node.get() as usize].ok_or("actual configured owner")?;
    unit.configured.owners[index.get() as usize - 1].enabled = false;
    let error = source
        .require_plain_attributes(&report, &analysis.facts)
        .err()
        .ok_or("changed owner accepted")?;
    assert!(
        error
            .to_string()
            .contains("must_use contract actual declaration owner changed")
    );
    assert!(!error.to_string().contains("transport is not implemented"));
    Ok(())
}

struct NoFiles;
impl resolve::load::Sources for NoFiles {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> std::result::Result<Option<resolve::load::InputFile>, String> {
        Ok(None)
    }
}

#[test]
fn old_functions_metadata_keeps_imported_attribute_state_unknown() -> Result {
    let interface = crates::decode(&crates::encode(&publish(&plain()?)?)?)?;
    let provider = interface.identity().clone();
    let dependencies = Arc::new(Dependencies::new(
        vec![interface],
        vec![Dependency::new("base".into(), provider.clone())?],
    )?);
    let current = CrateIdentity::new("client", "unknown_attributes")?;
    let context = resolve::bindings::Context {
        crate_name: Some("client".into()),
        current: Some(current),
        dependencies: Some(dependencies),
        mode: Mode::SourceClient { provider },
    };
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/runtime-client.rs",
    )?;
    let expanded = resolve::expansion::analyze_with_context(
        resolve::load::InputFile {
            identity: "client.rs".into(),
            path: "client.rs".into(),
            source: crate::support::source::SourceFile::new(source.into_bytes())?,
        },
        &mut NoFiles,
        &crate::expand::config::Config::default(),
        &context,
    )?;
    let report = resolve::analyze_expanded(expanded)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| {
            definition.kind == DefKind::Function
                && definition.node.is_none()
                && report.package.names.get(definition.name) == "identity"
        })
        .ok_or("fresh imported identity")?;
    let definition = DefId(u32::try_from(index)?);
    let error = analysis
        .facts
        .must_use_contract(&report, definition)
        .err()
        .ok_or("old imported absence invented")?;
    assert!(
        error
            .to_string()
            .contains("imported must_use contract transport is not implemented")
    );
    let error = analysis
        .facts
        .inline_hint_for_definition(&report, definition)
        .err()
        .ok_or("old imported inline absence invented")?;
    assert!(
        error
            .to_string()
            .contains("imported inline hint transport is not implemented")
    );
    Ok(())
}
