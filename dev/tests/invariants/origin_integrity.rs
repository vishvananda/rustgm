//! Applicability and incarnation checks use real configured providers/imports.
use super::*;
use crate::support::crates::owner_catalog::DeclarationRef;
use std::fmt::Write as _;

#[test]
fn inline_and_trait_contracts_retain_complete_actual_owner_applicability() -> Result {
    for (name, hint) in [
        ("inline", inlining::Hint::Hint),
        ("inline-always", inlining::Hint::Always),
        ("inline-never", inlining::Hint::Never),
    ] {
        let interface = roundtrip(&fixture(name)?)?;
        let (report, analysis, _) = source_packets::client_analysis(interface, &idle("", ""))?;
        let definition = imported_definition(&report, "identity")?;
        let view = analysis
            .facts
            .inline_hint_for_definition(&report, definition)?
            .ok_or("imported advisory hint")?;
        assert_eq!(view.hint, hint);
        assert_eq!(view.applicability, s::InlineApplicability::FunctionBody);
        assert!(view.origin.source().is_none());
    }
    let (interface, warnings) = exported(&fixture("required-inline")?)?;
    assert_eq!(warnings, 1);
    let (report, analysis, _) = source_packets::client_analysis(
        crates::decode(&crates::encode(&interface)?)?,
        &idle("", ""),
    )?;
    assert_eq!(
        analysis.facts.warnings().count(),
        0,
        "provider declaration warning was replayed"
    );
    let clone = imported_definition(&report, "clone")?;
    assert_eq!(
        analysis
            .facts
            .inline_hint_for_definition(&report, clone)?
            .ok_or("required member hint")?
            .applicability,
        s::InlineApplicability::DiagnosedNoBody
    );
    let invalid_owner = fixture("plain")?.replace(
        "pub trait Copy: Clone {}",
        "#[inline]pub trait Copy: Clone {}",
    );
    let error = exported(&invalid_owner)
        .err()
        .ok_or("inline on a trait reached publication")?;
    assert!(
        error
            .to_string()
            .contains("inline attribute requires a function or closure")
    );
    for name in ["trait-owner", "required-member", "unused-marked"] {
        let interface = roundtrip(&fixture(name)?)?;
        let rows = interface.source_descriptor().ok_or("descriptor")?.rows();
        let columns = rows.header_attributes().ok_or("complete columns")?;
        assert_eq!(columns.functions.len(), rows.functions.len());
        assert_eq!(columns.traits.len(), rows.traits.len());
        let marked = columns
            .functions
            .iter()
            .chain(&columns.traits)
            .find(|row| row.must_use.is_some())
            .ok_or("actual marked header")?
            .declaration;
        let provider = interface.identity().clone();
        let (report, analysis, _) = source_packets::client_analysis(interface, &idle("", ""))?;
        let definition = report
            .resolution
            .crates
            .as_ref()
            .ok_or("inventory")?
            .generic_definition(&provider, marked)?;
        let contract = analysis
            .facts
            .must_use_contract(&report, definition)?
            .ok_or("actual imported contract")?;
        let origin = contract
            .origin
            .imported()
            .ok_or("canonical imported contract")?;
        assert_eq!(origin.coordinate().0, &provider);
        assert_eq!(origin.coordinate().1, marked);
        if report.resolution.definitions[definition.0 as usize].kind == DefKind::Trait {
            assert!(
                origin.external_definition().is_none(),
                "trait acquired a fake linker identity"
            );
        }
    }
    Ok(())
}

#[test]
fn completion_incarnations_and_repeated_import_queries_remain_bounded() -> Result {
    let source = fixture("must-use-reason")?;
    let report = crate::test_source::source_provider(&source, "attribute_transport")?;
    let mut analysis = pipeline::analyze_source(&report)?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find_map(|(index, row)| {
            (report.package.names.get(row.name) == "identity" && row.kind == DefKind::Function)
                .then_some(DefId(u32::try_from(index).ok()?))
        })
        .ok_or("actual source identity")?;
    let other = crate::test_source::source_provider(&source, "attribute_transport")?;
    assert!(
        analysis
            .facts
            .header_attribute_contracts(&other, definition)
            .is_err()
    );
    assert!(
        analysis
            .facts
            .must_use_contract(&other, definition)
            .is_err()
    );
    let retained = analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("cold owner")?
        .headers
        .take();
    assert!(matches!(
        analysis
            .facts
            .header_attribute_contracts(&report, definition)?
            .must_use,
        ContractKnowledge::Unknown
    ));
    analysis
        .facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("cold owner")?
        .headers = retained;
    let bytes = crates::encode(&roundtrip(&source)?)?;
    let (client, imported, _) =
        source_packets::client_analysis(crates::decode(&bytes)?, &idle("", ""))?;
    let definition = imported_definition(&client, "identity")?;
    let (foreign, _, _) = source_packets::client_analysis(crates::decode(&bytes)?, &idle("", ""))?;
    assert!(
        imported
            .facts
            .header_attribute_contracts(&foreign, definition)
            .is_err()
    );
    assert_borrowed_queries(&client, &imported.facts, definition)?;
    assert_header_scaling()?;
    Ok(())
}

fn assert_header_scaling() -> Result {
    for count in [1, 8, 32] {
        let mut source = fixture("plain")?;
        for index in 0..count {
            write!(source, "pub fn extra{index}(x:u8)->u8{{x}}")?;
        }
        let (interface, _) = exported(&source)?;
        let rows = interface.source_descriptor().ok_or("descriptor")?.rows();
        assert_eq!(
            rows.header_attributes()
                .ok_or("complete columns")?
                .functions
                .len(),
            count + 2
        );
        assert!(rows.header_contract(DeclarationRef(u32::MAX)).is_err());
    }
    Ok(())
}

fn assert_borrowed_queries(client: &Report, facts: &Declarations, definition: DefId) -> Result {
    let ledger = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.headers.as_deref())
        .and_then(|ledger| ledger.imported.as_deref())
        .ok_or("retained imported ledger")?;
    let before = ledger.work.get();
    let mut pointer = None;
    for _ in 0..128 {
        let view = facts
            .must_use_contract(client, definition)?
            .ok_or("imported reason")?;
        let actual = view.reason.ok_or("reason")?.as_ptr();
        if let Some(expected) = pointer {
            assert_eq!(actual, expected);
        }
        pointer = Some(actual);
    }
    assert_eq!(
        ledger.work.get() - before,
        128,
        "warm read performed an inventory traversal"
    );
    ledger.work.set(crates::MAX_COMPONENTS);
    for _ in 0..2 {
        assert!(facts.must_use_contract(client, definition).is_err());
    }
    assert_eq!(ledger.work.get(), crates::MAX_COMPONENTS + 2);
    eprintln!(
        "HeaderLedger={} SourceContractOrigin={} ContractView={} InlineView={} AttributeColumns={} ValidatedRows={}",
        size_of::<Ledger>(),
        size_of::<ContractOrigin<'_>>(),
        size_of::<crate::sema::must_use::ContractView<'_>>(),
        size_of::<inlining::View<'_>>(),
        size_of::<s::HeaderAttributeColumns>(),
        size_of::<s::ValidatedRows>()
    );
    Ok(())
}
