//! Actual proved ordinary source rows exercise the draft codec without authority.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

// Fail early during test/static compilation if common source records grow.
const _: [(); 24] = [(); size_of::<crate::body::ir::Value>()];
const _: [(); 12] = [(); size_of::<crate::types::parameters::predicates::Predicate>()];
const _: [(); 32] = [(); size_of::<crate::sema::associated::Owner>()];
const _: [(); 20] = [(); size_of::<crate::sema::associated::OwnerKind>()];

fn actual() -> Result<(crates::CrateIdentity, s::SourceRows)> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    inspect(&source)
}
fn inspect(source: &str) -> Result<(crates::CrateIdentity, s::SourceRows)> {
    let report = crate::test_source::source_provider(source, "source_function_bytes")?;
    let owner = report
        .package
        .source_core()
        .ok_or("actual source provider")?
        .provider
        .clone();
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let inspected = crate::lower::instances::inspect_source_transport(
        &report,
        &mut facts,
        &owner,
        analysis.prepared,
    )?;
    Ok((owner, inspected.rows))
}

#[test]
fn source_function_columns_accept_renamed_reordered_multiple_complete_originals() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let source = source.replace(
        "pub fn identity<T>(x: T) -> T { x }",
        "pub fn second<U>(other: U) -> U { other }\npub fn first<T>(value: T) -> T { value }",
    );
    let (owner, rows) = inspect(&source)?;
    assert_eq!(rows.originals.len(), 2);
    assert!(
        !rows
            .declarations
            .iter()
            .any(|row| row.name.as_deref() == Some("identity"))
    );
    let frozen = s::ValidatedRows::stage(&owner, rows.clone(), 0, &mut g::Work::default())?;
    let encoded = bytes(&frozen)?;
    let mut reader = Reader::new(&encoded);
    let decoded = decode(&mut reader, 0)?;
    assert!(reader.done());
    assert_eq!(decoded, rows);
    let decoded = s::ValidatedRows::stage(&owner, decoded, 0, &mut g::Work::default())?;
    assert_eq!(bytes(&decoded)?, encoded);
    Ok(())
}

#[test]
fn source_column_staging_rejects_late_corruption_without_a_partial_payload() -> Result {
    let (owner, rows) = actual()?;
    let mut changed = rows.clone();
    let last = changed
        .originals
        .last_mut()
        .ok_or("actual provided original")?;
    last.declaration.0 = u32::MAX;
    let mut work = g::Work::default();
    let error = s::ValidatedRows::stage(&owner, changed, 0, &mut work)
        .err()
        .ok_or("corrupted original was staged")?;
    assert!(work.steps > 0);
    eprintln!("source-column rollback: {error}; charged={}", work.steps);
    let mut bounded = g::Work::default();
    assert!(
        s::ValidatedRows::stage(&owner, rows.clone(), crates::MAX_BYTES, &mut bounded).is_err()
    );
    assert!(bounded.steps > 0);
    // A failed stage cannot poison a later independent completion of the same
    // immutable raw provider. Descriptor/Interface publication remains disabled.
    let complete = s::ValidatedRows::stage(&owner, rows, 0, &mut g::Work::default())?;
    assert!(!bytes(&complete)?.is_empty());
    Ok(())
}
fn bytes(rows: &s::ValidatedRows) -> Result<Vec<u8>> {
    let mut out = Writer::default();
    encode(&mut out, rows.descriptor().rows())?;
    Ok(out.finish())
}
#[test]
fn ordinary_source_columns_round_trip_complete_real_headers_and_originals() -> Result {
    let (owner, rows) = actual()?;
    let mut work = g::Work::default();
    let frozen = s::ValidatedRows::stage(&owner, rows.clone(), 0, &mut work)?;
    let encoded = bytes(&frozen)?;
    let mut reader = Reader::new(&encoded);
    let decoded = decode(&mut reader, 0)?;
    assert!(reader.done());
    assert_eq!(decoded, rows);
    let mut decoded_work = g::Work {
        steps: reader.used_atoms(),
        ..g::Work::default()
    };
    let decoded = s::ValidatedRows::stage(&owner, decoded, 0, &mut decoded_work)?;
    assert_eq!(decoded, frozen);
    assert_eq!(bytes(&decoded)?, encoded);
    assert!(g::Payload::from_source(&owner, rows, 0, &mut g::Work::default()).is_ok());
    println!(
        "source-columns bytes={} source-work={} decoded-work={} Payload={} Columns={} ValidatedRows={} View={} Owner-OwnerKind-TraitHeader-Implementation={:?}",
        encoded.len(),
        work.steps,
        decoded_work.steps,
        size_of::<g::Payload>(),
        size_of::<s::Columns>(),
        size_of::<s::ValidatedRows>(),
        size_of::<s::ValidatedView<'_>>(),
        crate::sema::model::Declarations::metadata_model_layouts()
    );
    Ok(())
}
#[test]
fn ordinary_source_columns_reject_truncation_and_aggregate_decode_budget() -> Result {
    let (owner, rows) = actual()?;
    let frozen = s::ValidatedRows::stage(&owner, rows, 0, &mut g::Work::default())?;
    let encoded = bytes(&frozen)?;
    for end in 0..encoded.len() {
        assert!(
            decode(&mut Reader::new(&encoded[..end]), 0).is_err(),
            "truncation end={end}"
        );
    }
    assert!(decode(&mut Reader::new(&encoded), crates::MAX_BYTES).is_err());
    // Policy is provider u32 + role range; first counted column is declarations.
    let mut corrupt = encoded.clone();
    corrupt[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(decode(&mut Reader::new(&corrupt), 0).is_err());
    Ok(())
}
