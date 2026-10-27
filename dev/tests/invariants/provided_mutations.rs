//! Real encoded family opponents preserve owner/site/namespace and budget fences.
use super::*;
use crate::support::crates::generics::{self as g, source as s};

fn check(
    artifact: &crates::Interface,
    change: impl FnOnce(
        &mut s::SourceRows,
        &mut s::HeaderAttributeColumns,
        &mut s::ProvidedContractRows,
    ),
    base: usize,
    work: &mut g::Work,
) -> std::result::Result<g::Payload, &'static str> {
    let descriptor = artifact
        .source_descriptor()
        .ok_or("actual provided descriptor")?;
    let mut rows = module_packets::raw_rows(descriptor.rows());
    let mut attributes = descriptor
        .rows()
        .header_attributes()
        .ok_or("actual advisories")?
        .clone();
    let mut contracts = descriptor.provided_contracts()?.rows().clone();
    change(&mut rows, &mut attributes, &mut contracts);
    g::Payload::from_source_provided_contracts(
        artifact.identity(),
        rows,
        attributes,
        contracts,
        base,
        work,
    )
}
#[test]
fn complete_columns_reject_missing_swapped_sites_namespace_and_failed_storage() -> Result {
    let (_, artifact) = provider("")?;
    for change in [
        (|_: &mut s::SourceRows,
          _: &mut s::HeaderAttributeColumns,
          c: &mut s::ProvidedContractRows| {
            c.modifiers.functions.pop();
        }) as fn(_, _, _),
        |_, _, c| {
            c.modifiers.functions.swap(0, 1);
        },
        |_, _, c| {
            c.modifiers.functions[0].site = u32::MAX;
        },
        |r, _, c| {
            c.namespace.root = r.functions[0].declaration;
        },
        |_, a, _| {
            a.functions.pop();
        },
    ] {
        let mut work = g::Work::default();
        let result = check(&artifact, change, artifact.base_retained_bytes(), &mut work);
        assert!(result.is_err());
        assert!(work.steps > 0, "failed validation lost attempted work");
    }
    let mut work = g::Work::default();
    assert!(check(&artifact, |_, _, _| {}, crates::MAX_BYTES, &mut work).is_err());
    assert!(work.steps > 0);
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.last(), Some(&0));
    let mut unknown = bytes.clone();
    *unknown.last_mut().ok_or("last complete policy")? = 255;
    assert!(crates::decode(&unknown).is_err());
    let mut version = bytes.clone();
    version[13..15].copy_from_slice(&2u16.to_le_bytes());
    assert!(crates::decode(&version).is_err());
    assert!(crates::decode(&bytes[..bytes.len() - 1]).is_err());
    eprintln!(
        "provided cold ValidatedRows={} data-owner={} data-identity={} completion={} admission={}",
        size_of::<s::ValidatedRows>(),
        size_of::<crate::resolve::bindings::consumer_policy::HeaderDataOwner>(),
        size_of::<crate::resolve::bindings::consumer_policy::HeaderDataIdentity>(),
        size_of::<crate::sema::crates::authored_export::HeaderCompletion>(),
        size_of::<crate::resolve::bindings::consumer_policy::ProviderAdmission>()
    );
    Ok(())
}
