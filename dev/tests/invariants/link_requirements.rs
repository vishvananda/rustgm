//! Conditional native dependencies preserve actual cfg, declaration and metadata authority.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::model::Declarations,
    support::{
        crates::{self, CrateIdentity, Interface},
        diagnostics::LintCap,
        edition::Edition,
        linker_libraries::{Kind, Library, Plan},
        source::SourceFile,
    },
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, config: &Config) -> Result<Report> {
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "native-links.rs".into(),
            path: "native-links.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        config,
    )?)?)
}
fn plan(facts: &Declarations) -> Option<&Plan> {
    facts
        .foreign
        .as_deref()
        .and_then(crate::sema::foreign::Registry::native_links)
}
#[test]
fn independent_cases_keep_conditional_linking_separate_from_visibility_and_unsafe() -> Result {
    let cases =
        crate::test_source::fixture("pa29/tests/spec/conditional-native-links/cases-v2.tsv")?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, path, status] = columns.as_slice() else {
            return Err("invalid conditional link case".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let mut config = Config::in_edition(edition);
            config.set_lint_cap(LintCap::Allow);
            let actual = (|| -> Result {
                let report = report(&source, &config)?;
                crate::pipeline::analyze_source(&report)?;
                Ok(())
            })();
            assert_eq!(
                actual.is_ok(),
                *status == "0",
                "{name} {edition:?}: {:?}",
                actual.err()
            );
        }
        count += 1;
    }
    assert_eq!(count, 47);
    Ok(())
}
#[test]
fn selected_dependencies_use_actual_configuration_and_source_order() -> Result {
    let source = "#![feature(link_cfg)]#[link(name=\"first\",cfg(feature=\"enabled\"))]unsafe extern \"C\"{}\n#[link(name=\"second\",kind=\"static\",cfg(all()))]unsafe extern \"C\"{}";
    for enabled in [false, true] {
        let mut config = Config::default();
        config.set_lint_cap(LintCap::Allow);
        if enabled {
            config
                .add("feature=\"enabled\"")
                .map_err(|error| error.message)?;
        }
        let report = report(source, &config)?;
        let facts = crate::sema::declarations::analyze(&report)?;
        let libraries = plan(&facts)
            .ok_or("missing selected native library")?
            .libraries();
        assert_eq!(
            libraries.iter().map(Library::name).collect::<Vec<_>>(),
            if enabled {
                vec!["first", "second"]
            } else {
                vec!["second"]
            }
        );
        assert_eq!(
            libraries.last().ok_or("missing last dependency")?.kind(),
            Kind::Static
        );
        crate::pipeline::analyze(&report, facts)?;
    }
    Ok(())
}
#[test]
fn metadata_retains_dynamic_dependencies_and_bounds_malformed_envelopes() -> Result {
    let identity = CrateIdentity::new("native_provider", "one")?;
    let plain = crates::encode(&Interface::new(identity.clone(), vec![], vec![])?)?;
    let mut links = Plan::default();
    links.push(Library::new("selected_native", Kind::Dylib)?)?;
    let interface = Interface::new(identity.clone(), vec![], vec![])?.with_native_links(links)?;
    let encoded = crates::encode(&interface)?;
    assert_eq!(crates::decode(&encoded)?, interface);
    assert_eq!(
        crates::encode(&Interface::new(identity.clone(), vec![], vec![])?)?,
        plain
    );
    for length in 0..encoded.len() {
        assert!(crates::decode(&encoded[..length]).is_err());
    }
    let mut trailing = encoded.clone();
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    let mut unknown = encoded.clone();
    unknown[13] = 2;
    assert!(crates::decode(&unknown).is_err());
    let mut oversized = encoded.clone();
    oversized[14..18].copy_from_slice(&1025_u32.to_le_bytes());
    assert!(crates::decode(&oversized).is_err());
    let mut static_links = Plan::default();
    static_links.push(Library::new("selected_native", Kind::Static)?)?;
    let static_interface =
        Interface::new(identity, vec![], vec![])?.with_native_links(static_links)?;
    assert!(
        crates::encode(&static_interface).is_err(),
        "missing bundled archive cannot be silently dropped"
    );
    Ok(())
}
#[test]
fn native_plan_storage_is_bounded_without_deduplicating_source_requests() -> Result {
    let mut plan = Plan::default();
    for _ in 0..1024 {
        plan.push(Library::new("same", Kind::Dylib)?)?;
    }
    assert!(plan.push(Library::new("overflow", Kind::Dylib)?).is_err());
    assert_eq!(plan.libraries().len(), 1024);
    assert!(Library::new("", Kind::Dylib).is_err());
    assert!(Library::new("contains\0nul", Kind::Dylib).is_err());
    assert!(Library::new(&"a".repeat(4097), Kind::Dylib).is_err());
    let mut bytes = Plan::default();
    for _ in 0..16 {
        bytes.push(Library::new(&"a".repeat(4096), Kind::Dylib)?)?;
    }
    assert!(bytes.push(Library::new("overflow", Kind::Dylib)?).is_err());
    Ok(())
}
