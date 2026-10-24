//! Admission rejects unauthenticated source profiles before synthetic lookup changes.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{bindings::policy, names::Names},
    support::{bindings::Mode, crates::Interface, source::SourceFile},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const HEADER: &str = "#![feature(no_core)]#![no_core]";

fn context() -> Result<Context> {
    Ok(Context {
        crate_name: Some("client".into()),
        current: Some(CrateIdentity::new("client", "client_v1")?),
        dependencies: None,
        mode: Mode::SourceClient {
            provider: CrateIdentity::new("minicore", "provider_v1")?,
        },
    })
}
fn rejected(source: &str, edition: Edition, context: &Context, message: &str) -> Result {
    let unit = Unit::parse(
        SourceFile::new(source.as_bytes().to_vec())?,
        &Config::in_edition(edition),
        &mut Names::default(),
    )
    .map_err(|error| format!("{error:?}"))?;
    let Err(error) = policy::prepare(&unit, context) else {
        return Err("unauthenticated source client admitted".into());
    };
    assert_eq!(error.message, message);
    Ok(())
}

#[test]
fn client_identity_and_source_gate_precede_dependency_admission() -> Result {
    let mut request = context()?;
    for header in [
        HEADER,
        "#![feature(no_core,decl_macro,rustc_attrs)]#![no_core]",
    ] {
        rejected(
            header,
            Edition::Rust2024,
            &request,
            "source client requires an authenticated dependency inventory",
        )?;
    }
    request.current = None;
    rejected(
        HEADER,
        Edition::Rust2024,
        &request,
        "source client requires complete current crate identity",
    )?;
    request = context()?;
    request.current = Some(CrateIdentity::new("minicore", "provider_v1")?);
    request.crate_name = Some("minicore".into());
    rejected(
        HEADER,
        Edition::Rust2024,
        &request,
        "source client requires its distinct exact current crate identity",
    )?;
    request = context()?;
    request.crate_name = Some("different".into());
    rejected(
        HEADER,
        Edition::Rust2024,
        &request,
        "source client requires its distinct exact current crate identity",
    )?;
    request = context()?;
    rejected(
        HEADER,
        Edition::Rust2021,
        &request,
        "source client requires the Rust 2024 source profile",
    )?;
    for source in ["", "#![no_core]", "#![feature(no_core)]"] {
        rejected(
            source,
            Edition::Rust2024,
            &request,
            "source client requires no_core and its explicit source feature",
        )?;
    }
    Ok(())
}

#[test]
fn empty_legacy_and_wrong_identity_artifacts_never_authenticate_source_client() -> Result {
    let mut request = context()?;
    for artifacts in [
        vec![],
        vec![Interface::new(
            CrateIdentity::new("minicore", "wrong_version")?,
            vec![],
            vec![],
        )?],
    ] {
        request.dependencies = Some(Arc::new(Dependencies::new(artifacts, vec![])?));
        rejected(
            HEADER,
            Edition::Rust2024,
            &request,
            "source client provider metadata is missing",
        )?;
    }
    request.dependencies = Some(Arc::new(Dependencies::new(
        vec![Interface::new(
            CrateIdentity::new("minicore", "provider_v1")?,
            vec![],
            vec![],
        )?],
        vec![],
    )?));
    rejected(
        HEADER,
        Edition::Rust2024,
        &request,
        "source client requires a complete supported source provider payload",
    )?;
    Ok(())
}

#[test]
fn source_modes_keep_one_pointer_and_bootstrap_keeps_no_source_policy() -> Result {
    let unit = Unit::parse(
        SourceFile::new(b"fn f(){}".to_vec())?,
        &Config::default(),
        &mut Names::default(),
    )
    .map_err(|error| format!("{error:?}"))?;
    let prepared = policy::prepare(&unit, &Context::default())?;
    assert!(prepared.source.is_none());
    assert!(prepared.library.is_none());
    // The shared macro mark arena adds one lazy pointer; source modes still
    // share their original single pointer and allocate no bootstrap policy.
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<Option<Box<Source>>>(), size_of::<usize>());
    Ok(())
}
