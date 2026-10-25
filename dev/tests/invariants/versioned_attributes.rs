//! Legacy bytes/Unknown and corruption remain separate from new contract presence.
use super::*;

#[test]
fn legacy_unknown_closed_profiles_and_warned_wrong_owners_stay_explicit() -> Result {
    let plain = fixture("plain")?;
    let (legacy, warnings) = exported_profile(&plain, SourcePublication::LegacyFunctions)?;
    assert_eq!(warnings, 0);
    assert_eq!(
        legacy.identity(),
        &crates::CrateIdentity::new("minicore", "attribute_transport")?
    );
    let bytes = crates::encode(&legacy)?;
    let golden = legacy_golden()?;
    assert_eq!(golden.len(), 3628);
    assert_eq!(
        bytes, golden,
        "legacy Functions bytes changed from the qualified pre-v1 producer"
    );
    assert_eq!(bytes.get(13..15), Some([0, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    let (report, analysis, _) =
        source_packets::client_analysis(crates::decode(&bytes)?, &idle("", ""))?;
    let definition = imported_definition(&report, "identity")?;
    let view = analysis
        .facts
        .header_attribute_contracts(&report, definition)?;
    assert!(matches!(view.must_use, ContractKnowledge::Unknown));
    assert!(matches!(view.inline, ContractKnowledge::Unknown));
    assert!(
        view.origin
            .imported()
            .ok_or("legacy canonical origin")?
            .site()
            .is_none()
    );
    assert!(
        analysis
            .facts
            .must_use_contract(&report, definition)
            .is_err()
    );
    for name in ["must-use-reason", "inline"] {
        assert!(source_packets::exported(&fixture(name)?).is_err());
    }
    for name in ["negative-owner", "nominal-plain", "nominal-marked"] {
        assert!(exported(&fixture(name)?).is_err());
    }
    let (interface, warnings) = exported(&fixture("negative-crate-owner")?)?;
    assert_eq!(warnings, 1);
    let rows = interface.source_descriptor().ok_or("descriptor")?.rows();
    assert!(
        rows.header_attributes()
            .ok_or("checked rows")?
            .functions
            .iter()
            .all(|row| row.must_use.is_none())
    );
    let on = roundtrip(&fixture("cfg-on")?)?;
    let off = roundtrip(&fixture("cfg-off")?)?;
    assert!(
        on.source_descriptor()
            .ok_or("on descriptor")?
            .rows()
            .header_attributes()
            .ok_or("on facts")?
            .functions
            .iter()
            .any(|row| row.must_use.is_some())
    );
    assert!(
        off.source_descriptor()
            .ok_or("off descriptor")?
            .rows()
            .header_attributes()
            .ok_or("off facts")?
            .functions
            .iter()
            .all(|row| row.must_use.is_none())
    );
    let mut unknown = crates::encode(&on)?;
    unknown[12] = 11;
    assert!(
        crates::decode(&unknown).is_err(),
        "Provided accepted ordinary contract rows"
    );
    Ok(())
}

fn legacy_golden() -> Result<Vec<u8>> {
    let text = crate::test_source::fixture(
        "pa31/tests/spec/source-attribute-transport/legacy-functions-v0.hex",
    )?;
    let mut bytes = Vec::new();
    for token in text
        .lines()
        .filter(|line| !line.starts_with('#'))
        .flat_map(str::split_ascii_whitespace)
    {
        if token.len() != 2 {
            return Err("legacy Functions golden contains a non-byte hex token".into());
        }
        bytes.push(u8::from_str_radix(token, 16)?);
    }
    Ok(bytes)
}

fn raw(source: &str, interface: &crates::Interface) -> Result<s::SourceRows> {
    let report = crate::test_source::source_provider(source, "attribute_transport")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let mut rows = crate::lower::instances::inspect_source_transport(
        &report,
        &mut facts,
        interface.identity(),
        prepared,
    )?
    .rows;
    let current = interface
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    assert_eq!(rows.declarations.len(), current.declarations.len());
    for (row, actual) in rows.declarations.iter_mut().zip(&current.declarations) {
        row.sites = actual.sites;
    }
    Ok(rows)
}

#[test]
fn malformed_complete_columns_and_wire_tags_reject_before_publication() -> Result {
    let source = fixture("must-use-reason")?;
    let interface = roundtrip(&source)?;
    let rows = raw(&source, &interface)?;
    let attributes = interface
        .source_descriptor()
        .ok_or("descriptor")?
        .rows()
        .header_attributes()
        .ok_or("complete columns")?
        .clone();
    for case in 0..7 {
        let mut changed = attributes.clone();
        match case {
            0 => {
                changed.functions.pop();
            }
            1 => {
                changed.functions.push(changed.functions[0].clone());
            }
            2 => changed.functions.swap(0, 1),
            3 => changed.functions[0].declaration_site = u32::MAX,
            4 => {
                changed
                    .functions
                    .iter_mut()
                    .find_map(|row| row.must_use.as_mut())
                    .ok_or("marked row")?
                    .owner_site = u32::MAX;
            }
            5 => {
                changed.traits[0].inline = Some(s::InlineContract {
                    owner_site: 0,
                    hint: s::InlineHint::Hint,
                    applicability: s::InlineApplicability::DiagnosedNoBody,
                });
            }
            _ => {
                changed
                    .functions
                    .iter_mut()
                    .find_map(|row| row.must_use.as_mut())
                    .ok_or("marked row")?
                    .reason = Some("x".repeat(s::MAX_REASON_BYTES + 1));
            }
        }
        assert!(
            g::Payload::from_source_contracts(
                interface.identity(),
                rows.clone(),
                changed,
                interface.base_retained_bytes(),
                &mut g::Work::default()
            )
            .is_err()
        );
    }
    let bytes = crates::encode(&interface)?;
    for index in [12, 13, 15] {
        let mut changed = bytes.clone();
        changed[index] = 255;
        assert!(crates::decode(&changed).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(crates::decode(&trailing).is_err());
    assert!(crates::decode(&bytes[..bytes.len() - 1]).is_err());
    let mut invalid = bytes.clone();
    let offset = invalid
        .windows("keep result".len())
        .position(|value| value == b"keep result")
        .ok_or("encoded reason")?;
    invalid[offset] = 255;
    assert!(crates::decode(&invalid).is_err());
    assert_transport_reason_limit(&source)?;
    Ok(())
}

fn assert_transport_reason_limit(source: &str) -> Result {
    let long_source = source.replace("keep result", &"x".repeat(s::MAX_REASON_BYTES + 1));
    let report = crate::test_source::source_provider(&long_source, "attribute_transport")?;
    let analysis = pipeline::analyze_source(&report)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, row)| {
            (row.kind == DefKind::Function && report.package.names.get(row.name) == "identity")
                .then_some(DefId(u32::try_from(index).ok()?))
        })
        .ok_or("actual source function")?;
    assert_eq!(
        analysis
            .facts
            .must_use_contract(&report, definition)?
            .ok_or("language contract was rejected")?
            .reason
            .map(str::len),
        Some(s::MAX_REASON_BYTES + 1)
    );
    assert!(
        exported(&long_source)
            .err()
            .ok_or("oversized transport reason published")?
            .to_string()
            .contains("transport profile limit")
    );
    Ok(())
}
