//! The bring-up profile is an explicit driver input and never a qualification.
use rustgm::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::bringup::{Stubs, admit_guests, select, selected},
};
use std::{
    fs,
    io::{self, Write},
};

/// Consume `--bringup-skip-list PATH`, which selects the unqualified profile.
pub(super) fn consume(
    argument: &str,
    args: &mut impl Iterator<Item = String>,
) -> Result<bool, String> {
    if argument != "--bringup-skip-list" {
        return Ok(false);
    }
    let path = args.next().ok_or("--bringup-skip-list requires a path")?;
    let text =
        fs::read_to_string(&path).map_err(|error| format!("bring-up skip list {path}: {error}"))?;
    select(Stubs::parse(&text)?).map_err(|_| "duplicate --bringup-skip-list")?;
    Ok(true)
}

/// Guest crates exist only in the bring-up profile, which takes them here.
/// A bring-up library names the profile in its identity, so that no consumer
/// can bind it where it expects a strict artifact.
pub(super) fn validate(crates: &crate::crates::Options) -> Result<(), String> {
    if selected().is_none() && !crates.guests.is_empty() {
        return Err("--bringup-guest requires the bring-up profile (--bringup-skip-list)".into());
    }
    if selected().is_some()
        && crates.is_library()
        && !crates
            .disambiguator
            .as_deref()
            .is_some_and(|identity| identity.ends_with("_bringup"))
    {
        return Err(
            "a bring-up library requires a --crate-disambiguator ending in _bringup".into(),
        );
    }
    if !crates.guests.is_empty() {
        admit_guests(crates.guests.clone())?;
    }
    Ok(())
}

/// Give each guest crate named by `--bringup-guest-cfg` its own cfg options,
/// once the session's own configuration is complete.
pub(super) fn configure(
    crates: &crate::crates::Options,
    config: &mut rustgm::expand::config::Config,
) -> Result<(), String> {
    for (name, setting) in &crates.guest_cfgs {
        let scope = config.crate_scope(name);
        if let Some(setting) = setting {
            scope
                .add(setting)
                .map_err(|e| format!("--bringup-guest-cfg {name}: {}: {}", e.offset, e.message))?;
        }
    }
    Ok(())
}

/// Name every function that owns a failure, for the driver that maintains the
/// list, and every failure that no list entry can remove. Without `facts`,
/// a failure belongs to a function item that the resolver declared. The last
/// field names the stage that recorded the failure, or `stopped` for the one
/// that stopped the compilation without being recorded.
pub(super) fn attribute(
    report: &Report,
    facts: Option<&Declarations>,
    error: &Error,
) -> io::Result<()> {
    let attribution = rustgm::body::bringup::blockers(report, facts, error);
    let mut stderr = io::stderr().lock();
    for blocker in &attribution.blockers {
        writeln!(
            stderr,
            "bringup-blocker\t{}\t{}\t{}\t{}\t{}",
            blocker.path,
            blocker.offset,
            blocker.name,
            line(&blocker.message),
            stage(blocker)
        )?;
    }
    for message in &attribution.unattributed {
        writeln!(stderr, "bringup-unattributed\t{}", line(message))?;
    }
    Ok(())
}
/// Name every function that keeps its body although a soundness proof of it
/// failed, and the stage whose proof it is.
pub(super) fn unproven(report: &Report, facts: &Declarations) -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    for body in rustgm::body::bringup::unproven_bodies(report, facts) {
        writeln!(
            stderr,
            "bringup-unproven\t{}\t{}\t{}\t{}\t{}",
            body.path,
            body.offset,
            body.name,
            line(&body.message),
            stage(&body)
        )?;
    }
    Ok(())
}
/// Name every publication fence this compilation passed over.
pub(super) fn waivers() -> io::Result<()> {
    let mut stderr = io::stderr().lock();
    for fence in rustgm::support::bringup::waived() {
        writeln!(stderr, "bringup-waived\t{}", line(&fence))?;
    }
    Ok(())
}
fn stage(blocker: &rustgm::body::bringup::Blocker) -> &'static str {
    blocker
        .stage
        .map_or("stopped", rustgm::support::bringup::Stage::name)
}
/// A message as one tab-free field.
fn line(message: &str) -> String {
    message.replace(['\t', '\n', '\r'], " ")
}
