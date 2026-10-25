//! Real source contracts retain bounded failed work before message allocation.
use super::{BYTES, Declarations, Requirement};
use crate::resolve::{expansion::Report, model::DefId, unit::NodeKey};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn facts() -> TestResult<(Report, Declarations, DefId, NodeKey)> {
    let report =
        crate::test_source::resolve("#[must_use=\"PAYLOAD\"]fn marked()->u32{7u32}", true)?;
    let facts = Declarations::new(&report)?;
    let table = facts
        .must_use_contracts
        .as_deref()
        .ok_or("source contract table")?;
    let (&definition, &index) = table
        .definitions
        .iter()
        .next()
        .ok_or("bound source contract")?;
    let key = table.sources[index].declaration;
    Ok((report, facts, definition, key))
}
#[test]
fn actual_selected_reason_is_charged_before_formatting_and_failed_repeats_keep_work() -> TestResult
{
    let (report, mut facts, definition, key) = facts()?;
    let text = facts.must_use_message(&report, key, Requirement::Declaration(definition))?;
    assert_eq!(text, "unused must_use result: PAYLOAD");
    assert_eq!(facts.must_use_work().message_bytes, text.len());
    assert_eq!(facts.must_use_work().message_steps, 1 + text.len());
    facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("contract table")?
        .work
        .message_bytes = BYTES;
    for _ in 0..2 {
        let before = facts.must_use_work();
        let error = facts
            .must_use_message(&report, key, Requirement::Declaration(definition))
            .err()
            .ok_or("over-limit message allocated")?;
        assert!(error.message.contains("message work or storage budget"));
        assert!(facts.must_use_work().message_steps > before.message_steps);
        assert!(facts.must_use_work().message_bytes > before.message_bytes);
        assert_eq!(facts.warnings().count(), 0);
    }
    assert_eq!(
        facts
            .must_use_contract(&report, definition)?
            .ok_or("retained contract")?
            .reason,
        Some("PAYLOAD")
    );
    Ok(())
}
#[test]
fn source_decoding_preflight_keeps_the_original_contract_on_failed_reservation() -> TestResult {
    let (report, mut facts, definition, key) = facts()?;
    let contract = facts
        .must_use_contract(&report, definition)?
        .ok_or("actual contract")?;
    let source = contract
        .origin
        .source()
        .ok_or("actual source contract origin")?;
    let owner = source.owner;
    let span = source.span;
    facts
        .must_use_contracts
        .as_deref_mut()
        .ok_or("contract table")?
        .work
        .contract_input_bytes = BYTES;
    let before = facts.must_use_work();
    let error = facts
        .reserve_must_use_source(owner, span)
        .err()
        .ok_or("over-limit reason decoded")?;
    assert!(
        error
            .message
            .contains("source contract work or storage budget")
    );
    assert!(facts.must_use_work().contract_input_steps > before.contract_input_steps);
    assert!(facts.must_use_work().contract_input_bytes > before.contract_input_bytes);
    assert_eq!(
        facts
            .must_use_contract(&report, definition)?
            .ok_or("retained contract")?
            .origin
            .source()
            .ok_or("retained local contract origin")?
            .declaration,
        key
    );
    Ok(())
}
