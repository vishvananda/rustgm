//! Wider bytes remain unpaid data while canonical source import stays closed.
use super::*;
use crate::support::crates::{Dependencies, codec};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[path = "encoding_baselines.rs"]
mod encodings;

fn actual(full: bool) -> Result<(crates::CrateIdentity, s::SourceRows)> {
    let fixture = if full {
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs"
    } else {
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs"
    };
    let source = crate::test_source::fixture(fixture)?;
    let report = crate::test_source::source_provider(&source, "supplied_data_codec")?;
    let owner = report
        .package
        .source_core()
        .ok_or("actual source policy")?
        .provider
        .clone();
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let rows = crate::lower::instances::inspect_source_transport_with_numbering(
        &report,
        &mut facts,
        &owner,
        analysis.prepared,
    )?
    .rows;
    Ok((owner, rows))
}
fn interface(owner: crates::CrateIdentity, rows: s::SourceRows) -> Result<crates::Interface> {
    let base = crates::Interface::new(owner, Vec::new(), Vec::new())?;
    let payload = g::Payload::from_source_provided(
        base.identity(),
        rows,
        base.base_retained_bytes(),
        &mut g::Work::default(),
    )?;
    Ok(base.with_generic_payload(payload)?)
}
fn columns(rows: &s::ValidatedRows) -> Result<Vec<u8>> {
    let mut writer = Writer::default();
    encode_provided(&mut writer, rows.descriptor().rows())?;
    Ok(writer.finish())
}

#[test]
fn supplied_full_family_data_round_trips_all_columns_without_import_permission() -> Result {
    let (owner, rows) = actual(true)?;
    assert_eq!(rows.originals.len(), 3);
    assert_eq!(rows.body_units.len(), 3);
    assert_eq!(rows.roles.len(), 9);
    actual_default(&rows)?;
    let original = interface(owner.clone(), rows.clone())?;
    let encoded = codec::encode(&original)?;
    assert_eq!(encoded[12], 11);
    let decoded = codec::decode(&encoded)?;
    assert_eq!(decoded, original);
    assert_eq!(codec::encode(&decoded)?, encoded);
    assert_eq!(
        decoded
            .source_descriptor()
            .ok_or("actual descriptor")?
            .profile(),
        s::Profile::Source2024Provided
    );
    let payload = decoded.generic_payload().ok_or("actual payload")?;
    assert!(payload.legacy_rows().is_err());
    let g::PayloadView::Source(source) = payload.view() else {
        return Err("source payload".into());
    };
    let stored = source.descriptor().rows();
    assert_eq!(stored.body_owners(), rows.body_owners.as_slice());
    assert_eq!(stored.originals, rows.originals);
    assert_eq!(stored.body_inputs, rows.body_inputs);
    assert_eq!(stored.body_units, rows.body_units);
    // This stays unpaid data: only semantic bind may decide whether an empty
    // ledger was actually permitted. The wire must retain the distinction.
    let mut distinct = rows.clone();
    let zero = distinct
        .body_units
        .iter_mut()
        .find(|unit| unit.universals == 0)
        .ok_or("actual zero-universal original")?;
    assert!(zero.aliases.is_none());
    zero.aliases = Some(Vec::new());
    let distinct = interface(owner.clone(), distinct)?;
    let different = codec::encode(&distinct)?;
    assert_ne!(different, encoded);
    assert_eq!(codec::decode(&different)?, distinct);
    let dependencies = Dependencies::new(vec![decoded], Vec::new())?;
    let descriptor = dependencies.artifacts()[0]
        .source_descriptor()
        .ok_or("actual supplied descriptor")?;
    assert_eq!(descriptor.identity(), &owner);
    assert_eq!(descriptor.provider(), &owner);
    assert_eq!(
        descriptor.edition(),
        crate::support::edition::Edition::Rust2024
    );
    assert_eq!(descriptor.profile(), s::Profile::Source2024Provided);
    assert_eq!(
        dependencies.source_provider_index(&owner),
        Err("source client provider profile is unsupported")
    );
    println!(
        "supplied data bytes={} originals=3 source import denied",
        encoded.len()
    );
    Ok(())
}

fn actual_default(rows: &s::SourceRows) -> Result {
    use crate::support::constness::Constness;
    let named = |function: &s::FunctionHeader, name: &str| {
        rows.declarations[function.declaration.0 as usize]
            .name
            .as_deref()
            == Some(name)
    };
    let from = rows
        .functions
        .iter()
        .find(|function| named(function, "clone_from"))
        .ok_or("actual provided default")?;
    let s::FunctionBody::Original(body) = from.body else {
        return Err("actual default body".into());
    };
    assert_eq!(from.constness, Constness::Ordinary);
    assert_eq!(
        rows.originals[body.0 as usize].context,
        s::CallContext::ConstDefinition
    );
    assert!(matches!(
        rows.body_inputs[body.0 as usize].environment,
        s::EnvironmentOrigin::ConstTraitDefault { .. }
    ));
    assert!(
        rows.body_units[body.0 as usize]
            .aliases
            .as_ref()
            .is_some_and(|aliases| aliases.len() >= 2)
    );
    let clone = rows
        .functions
        .iter()
        .find(|function| {
            named(function, "clone") && matches!(function.body, s::FunctionBody::Original(_))
        })
        .ok_or("actual ordinary u8 Clone body")?;
    assert_eq!(clone.constness, Constness::Ordinary);
    let s::FunctionBody::Original(body) = clone.body else {
        return Err("ordinary Clone body".into());
    };
    assert_eq!(
        rows.originals[body.0 as usize].context,
        s::CallContext::Runtime
    );
    assert!(
        rows.body_inputs[body.0 as usize]
            .normalizations
            .iter()
            .any(|proof| !proof.owned.is_empty())
    );
    let freeze = rows
        .roles
        .iter()
        .find(|role| role.role == crate::support::bindings::LangItem::Freeze)
        .ok_or("actual Freeze role")?;
    let flags = rows
        .traits
        .iter()
        .find(|header| header.declaration == freeze.declaration.declaration)
        .ok_or("actual Freeze header")?
        .flags
        .bits();
    assert_eq!(
        flags & (s::TraitFlags::UNSAFE | s::TraitFlags::AUTO),
        s::TraitFlags::UNSAFE | s::TraitFlags::AUTO
    );
    Ok(())
}

#[test]
fn supplied_data_codec_rejects_truncation_versions_extras_and_combined_budget() -> Result {
    let (owner, rows) = actual(true)?;
    let frozen =
        s::ValidatedRows::stage_provided(&owner, rows.clone(), 0, &mut g::Work::default())?;
    let encoded = columns(&frozen)?;
    for end in 0..encoded.len() {
        assert!(
            decode_provided(&mut Reader::new(&encoded[..end]), 0).is_err(),
            "truncated column end={end}"
        );
    }
    assert!(decode_provided(&mut Reader::new(&encoded), crates::MAX_BYTES).is_err());
    let mut forged = encoded.clone();
    forged[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(decode_provided(&mut Reader::new(&forged), 0).is_err());
    let artifact = codec::encode(&interface(owner, rows)?)?;
    for offset in [13, 14, 15] {
        let mut changed = artifact.clone();
        changed[offset] = 255;
        assert!(codec::decode(&changed).is_err());
    }
    let mut changed = artifact.clone();
    changed.push(0);
    assert!(codec::decode(&changed).is_err());
    let mut changed = artifact;
    changed[12] = 10;
    assert!(
        codec::decode(&changed).is_err(),
        "old profile must reject wider vocabulary"
    );
    Ok(())
}

#[test]
fn supplied_data_staging_rejects_wrong_owner_alias_domain_environment_and_call_context() -> Result {
    let (owner, actual) = actual(true)?;
    let default = actual
        .body_inputs
        .iter()
        .position(|input| {
            matches!(
                input.environment,
                s::EnvironmentOrigin::ConstTraitDefault { .. }
            )
        })
        .ok_or("actual default environment")?;
    for case in 0..6 {
        let mut rows = actual.clone();
        match case {
            0 => {
                rows.body_units.pop();
            }
            1 => {
                rows.body_units[default]
                    .aliases
                    .as_mut()
                    .ok_or("aliases")?
                    .pop();
            }
            2 => rows.body_units[default].allocation_order[0] = u32::MAX,
            3 => {
                rows.body_inputs[default].environment =
                    s::EnvironmentOrigin::Declaration(rows.originals[default].declaration);
            }
            4 => rows.originals[default].context = s::CallContext::Runtime,
            5 => rows.originals[default].declaration.0 = u32::MAX,
            _ => unreachable!(),
        }
        let mut work = g::Work::default();
        assert!(
            g::Payload::from_source_provided(&owner, rows, 0, &mut work).is_err(),
            "case {case}"
        );
        assert!(work.steps > 0);
    }
    let mut work = g::Work {
        steps: crates::MAX_COMPONENTS,
        ..g::Work::default()
    };
    assert!(g::Payload::from_source_provided(&owner, actual, 0, &mut work).is_err());
    assert!(work.steps > crates::MAX_COMPONENTS);
    Ok(())
}

fn denied(
    bytes: &[u8],
    check: impl FnOnce(&mut Input<'_, '_>) -> std::result::Result<(), &'static str>,
) {
    let mut reader = Reader::new(bytes);
    let mut input = Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        provided: false,
    };
    assert!(check(&mut input).is_err());
    assert_eq!(
        input.bytes, 0,
        "Functions must deny before allocating wider records"
    );
}
#[test]
fn functions_reader_denies_each_wider_tag_before_allocation() {
    for tag in [4, 5] {
        denied(&[tag], |read| recipe_columns::type_read(read).map(|_| ()));
    }
    denied(&[3], |read| recipe_columns::region_read(read).map(|_| ()));
    for tag in [7, 8] {
        denied(&[tag], |read| contract_columns::role_read(read).map(|_| ()));
    }
    for tag in [1, 2] {
        denied(&[tag], |read| {
            recipe_columns::effect_read(read, "wider effect").map(|_| ())
        });
    }
    denied(&[3], |read| {
        relation_columns::environment_read(read).map(|_| ())
    });
    denied(&[1], |read| operation_columns::place_read(read).map(|_| ()));
    let count = 1_u32.to_le_bytes();
    denied(&count, |read| {
        read.provided_list::<u32>(1, 0, Input::id).map(|_| ())
    });
}

#[test]
fn functions_columns_keep_their_existing_layout_without_unit_suffix() -> Result {
    let (owner, mut raw) = actual(false)?;
    raw.body_units.clear();
    let frozen = s::ValidatedRows::stage(&owner, raw.clone(), 0, &mut g::Work::default())?;
    let mut writer = Writer::default();
    encode(&mut writer, frozen.descriptor().rows())?;
    let old = writer.finish();
    let mut reader = Reader::new(&old);
    let decoded = decode(&mut reader, 0)?;
    assert!(reader.done());
    assert_eq!(decoded, raw);
    let mut writer = Writer::default();
    encode_provided(&mut writer, frozen.descriptor().rows())?;
    let wider = writer.finish();
    assert_eq!(&wider[..old.len()], old);
    assert_eq!(&wider[old.len()..], 0_u32.to_le_bytes());
    let mut reader = Reader::new(&wider);
    decode(&mut reader, 0)?;
    assert!(
        !reader.done(),
        "column reader leaves trailing bytes to envelope"
    );
    let base = crates::Interface::new(owner, Vec::new(), Vec::new())?;
    let payload = g::Payload::from_source(
        base.identity(),
        raw,
        base.base_retained_bytes(),
        &mut g::Work::default(),
    )?;
    let mut artifact = codec::encode(&base.with_generic_payload(payload)?)?;
    assert_eq!(artifact[12], 10);
    artifact.extend_from_slice(&0_u32.to_le_bytes());
    assert!(
        codec::decode(&artifact).is_err(),
        "Functions envelope must reject even an empty extra unit column"
    );
    Ok(())
}
