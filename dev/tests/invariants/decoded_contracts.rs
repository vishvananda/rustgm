//! Real encoded family opponents preserve owner/site/namespace and budget fences.
use super::*;
use crate::support::crates::generics::{self as g, source as s};
type Mutation =
    fn(&mut s::SourceRows, &mut s::HeaderAttributeColumns, &mut s::ProvidedContractRows);

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
    ordering_accounting(&artifact)?;
    let opponents: [Mutation; 6] = [
        |_, _, c| {
            c.modifiers.functions.pop();
        },
        |_, _, c| {
            c.modifiers.functions.swap(0, 1);
        },
        |_, _, c| {
            c.modifiers.functions[0].site = u32::MAX;
        },
        |r, _, c| {
            c.namespace.root = r.functions[0].declaration;
        },
        |_, _, c| {
            c.namespace.bindings.push(c.namespace.bindings[0].clone());
        },
        |_, a, _| {
            a.functions.pop();
        },
    ];
    for change in opponents {
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
        "provided cold ValidatedRows={} data-owner={} data-identity={} completion={} admission={} Payload={}",
        size_of::<s::ValidatedRows>(),
        size_of::<crate::resolve::bindings::consumer_policy::HeaderDataOwner>(),
        size_of::<crate::resolve::bindings::consumer_policy::HeaderDataIdentity>(),
        size_of::<crate::sema::crates::authored_export::HeaderCompletion>(),
        size_of::<crate::resolve::bindings::consumer_policy::ProviderAdmission>(),
        size_of::<g::Payload>()
    );
    eprintln!(
        "provided enclosing Header/Headers/ProvidedOwners/ProofSession/HeaderDraft/SelectedConsumer={:?}; StagedConsumer/Source={:?}",
        crate::sema::crates::provided_layouts(),
        crate::resolve::bindings::consumer_policy::layouts()
    );
    Ok(())
}

fn ordering_accounting(artifact: &crates::Interface) -> Result {
    let mut canonical_work = g::Work::default();
    let canonical = check(
        artifact,
        |_, _, _| {},
        artifact.base_retained_bytes(),
        &mut canonical_work,
    )?;
    let mut reordered_work = g::Work::default();
    let reordered = check(
        artifact,
        |_, _, columns| columns.namespace.bindings.reverse(),
        artifact.base_retained_bytes(),
        &mut reordered_work,
    )?;
    assert_eq!(canonical, reordered);
    assert_ne!(canonical_work.steps, reordered_work.steps);
    for (payload, work) in [(&canonical, &canonical_work), (&reordered, &reordered_work)] {
        assert!(work.steps > 0 && work.steps <= crates::MAX_COMPONENTS);
        assert_eq!(payload.budget().0, work.steps);
        assert!(payload.retained_bytes() > 0);
        assert!(payload.retained_bytes() <= payload.peak_bytes());
        assert!(
            payload
                .peak_bytes()
                .saturating_add(artifact.base_retained_bytes())
                <= crates::MAX_BYTES
        );
    }
    // The sealed cost includes the legacy predicate preflight, and excludes
    // only work paid by the caller before this particular publication began.
    let mut seeded_work = g::Work {
        steps: 37,
        ..g::Work::default()
    };
    let seeded = check(
        artifact,
        |_, _, _| {},
        artifact.base_retained_bytes(),
        &mut seeded_work,
    )?;
    assert_eq!(seeded.budget().0, seeded_work.steps - 37);
    assert_eq!(seeded, canonical);
    eprintln!(
        "namespace canonical/reordered actual work={}/{} retained={}/{} peak={}/{}",
        canonical_work.steps,
        reordered_work.steps,
        canonical.retained_bytes(),
        reordered.retained_bytes(),
        canonical.peak_bytes(),
        reordered.peak_bytes()
    );
    let base = || {
        crates::Interface::new(
            artifact.identity().clone(),
            artifact.dependencies().to_vec(),
            Vec::new(),
        )
    };
    let canonical = base()?.with_generic_payload(canonical)?;
    let reordered = base()?.with_generic_payload(reordered)?;
    assert_eq!(canonical, reordered);
    assert_eq!(crates::encode(&canonical)?, crates::encode(&reordered)?);
    Ok(())
}
