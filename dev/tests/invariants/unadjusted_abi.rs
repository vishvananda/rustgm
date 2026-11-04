//! Source ABI identity and permissions are distinct from physical backend support.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{edition::Edition, source::SourceFile},
    types::{
        Kind,
        callable::{Abi, Convention},
    },
};
use std::{fmt::Write as _, path::Path};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
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
            "unadjusted_abi",
            config,
        )?);
    }
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "abi.rs".into(),
            path: "abi.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        config,
    )?)?)
}
#[test]
fn typed_unadjusted_sources_follow_independent_gates_and_convention_distinctions() -> Result {
    let cases = crate::test_source::fixture("pa31/tests/spec/unadjusted-request/cases-v4.tsv")?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, path, status, provider] = columns.as_slice() else {
            return Err("malformed unadjusted case".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let actual = (|| -> Result {
                let report = report(&source, &Config::in_edition(edition), *provider == "1")?;
                let facts = crate::sema::declarations::analyze(&report)?;
                crate::pipeline::analyze(&report, facts)?;
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
    assert_eq!(count, 30);
    Ok(())
}
#[test]
fn unadjusted_identity_does_not_alias_other_signatures_or_grow_hot_conventions() -> Result {
    let report = report(
        "#![feature(abi_unadjusted)]extern \"unadjusted\" fn raw(){}extern \"C\" fn foreign(){}fn ordinary(){}",
        &Config::default(),
        false,
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let mut signatures = Vec::new();
    for function in &facts.functions {
        let Some(Kind::Function { convention, .. }) = facts.types.kind(function.signature) else {
            return Err("missing function signature".into());
        };
        signatures.push((function.signature, convention.abi));
    }
    assert_eq!(signatures.len(), 3);
    for abi in [Abi::Rust, Abi::C, Abi::Unadjusted] {
        assert_eq!(
            signatures
                .iter()
                .filter(|(_, actual)| *actual == abi)
                .count(),
            1
        );
    }
    for (index, (ty, _)) in signatures.iter().enumerate() {
        assert!(signatures[index + 1..].iter().all(|(other, _)| other != ty));
    }
    assert_eq!(size_of::<Abi>(), 1);
    assert_eq!(size_of::<Convention>(), 2);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::syntax::item::Abi>(), 8);
    Ok(())
}
#[test]
fn repeated_unadjusted_macro_headers_share_one_paid_allowance_session() -> Result {
    fn work(width: usize, calls: usize) -> Result<usize> {
        let features = std::iter::once("abi_unadjusted".to_owned())
            .chain((1..width).map(|i| format!("fixture_feature_{i}")))
            .collect::<Vec<_>>()
            .join(",");
        let mut source = format!(
            "#![feature(allow_internal_unstable)]#[allow_internal_unstable({features})]macro_rules! make{{($name:ident)=>{{extern \"unadjusted\" fn $name(){{}}}}}}"
        );
        for i in 0..calls {
            write!(source, "make!(f{i});")?;
        }
        let report = report(&source, &Config::default(), false)?;
        let before = report
            .package
            .macro_context()?
            .ok_or("actual macro context")?
            .contracts()
            .usage()
            .0;
        crate::sema::declarations::analyze(&report)?;
        let after = report
            .package
            .macro_context()?
            .ok_or("actual macro context")?
            .contracts()
            .usage()
            .0;
        Ok(after.checked_sub(before).ok_or("allowance work order")?)
    }
    let base = work(8, 1)?;
    let repeated = work(8, 32)?.checked_sub(base).ok_or("repeated work")?;
    for width in [32, 128] {
        let first = work(width, 1)?;
        let growth = work(width, 32)?.checked_sub(first).ok_or("growing work")?;
        assert!(
            growth <= repeated + 256 * 31,
            "width{width}: {growth}/{repeated}"
        );
    }
    Ok(())
}
