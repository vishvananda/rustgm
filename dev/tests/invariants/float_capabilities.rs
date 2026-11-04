//! Query permission follows source provenance without changing backend facts.
use crate::{
    expand::config::{CfgGate, Config},
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{edition::Edition, source::SourceFile},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const KEYS: &[&str] = &[
    "target_has_reliable_f16",
    "target_has_reliable_f16_math",
    "target_has_reliable_f128",
    "target_has_reliable_f128_math",
];
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, config: &Config, provider: bool) -> Result<Report> {
    if provider {
        return Ok(crate::test_source::source_provider_config(
            source,
            "float_reliability",
            config,
        )?);
    }
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "floats.rs".into(),
            path: "floats.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        config,
    )?)?)
}
#[test]
fn reliability_queries_preserve_pinned_feature_and_macro_origin_outcomes() -> Result {
    let cases = crate::test_source::fixture("pa41/tests/spec/float-reliability-cfg/cases-v2.tsv")?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, path, status, provider] = columns.as_slice() else {
            return Err("malformed reliability case".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let actual = report(&source, &Config::in_edition(edition), *provider == "1");
            assert_eq!(
                actual.is_ok(),
                *status == "0",
                "{name} {edition:?}: {:?}",
                actual.err()
            );
        }
        count += 1;
    }
    assert_eq!(count, 74);
    Ok(())
}
#[test]
fn reliability_permissions_never_enable_or_forge_backend_capabilities() {
    let mut config = Config::default();
    for &key in KEYS {
        assert_eq!(CfgGate::named(key), Some(CfgGate::FloatReliability));
        assert!(!config.contains(key, None));
        assert!(!config.contains(key, Some("yes")));
        assert!(config.add(key).is_err());
        assert!(config.add(&format!("{key}=\"yes\"")).is_err());
        assert!(!config.contains(key, None));
        assert!(!config.contains(key, Some("yes")));
    }
    assert_eq!(size_of::<CfgGate>(), 1);
    assert_eq!(size_of::<super::KnownFeature>(), 1);
}
