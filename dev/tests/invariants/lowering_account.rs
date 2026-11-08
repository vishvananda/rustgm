//! Lowering's account opens once on the crate's own ledger. Header queries
//! and payments made before it opens charge the header account; the same
//! charges made after it opens charge lowering's account only.
use super::super::{charge_source_contract_query, source_contract_costs};
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const BASE: &str = "pa31/tests/spec/needs-drop/work/width8-calls8.rs";

#[test]
fn charges_move_to_lowering_only_once_its_account_opens() -> Result {
    let source = crate::test_source::fixture(BASE)?;
    let report = crate::test_source::source_provider(&source, "needs_drop_work")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    assert!(lowering_costs(&facts).is_none());
    // Before opening, a query and a payment reach the header account.
    let analysis = source_contract_costs(&facts);
    charge_source_contract_query(&facts, 3)?;
    pay_growth(&facts, 5, 7)?;
    let header = source_contract_costs(&facts);
    assert_eq!(header, (analysis.0 + 8, analysis.1 + 7));
    // A different crate's report cannot open this crate's account.
    let other = crate::test_source::source_provider(
        &format!("{source}fn extra(){{}}\n"),
        "needs_drop_work",
    )?;
    assert_eq!(
        open(&other, &mut facts),
        Err("lowering report differs from its declarations' input graph")
    );
    assert!(lowering_costs(&facts).is_none());
    open(&report, &mut facts)?;
    let granted = bound(&report);
    assert_eq!(
        lowering_costs(&facts),
        Some([0, 0, granted.work, granted.bytes])
    );
    // After opening, the same query and payment reach lowering's account.
    charge_source_contract_query(&facts, 3)?;
    pay_growth(&facts, 5, 7)?;
    assert_eq!(source_contract_costs(&facts), header);
    assert_eq!(
        lowering_costs(&facts),
        Some([8, 7, granted.work, granted.bytes])
    );
    // Reopening keeps the account and what it was paid; a mismatched report
    // is still rejected once it is open.
    open(&report, &mut facts)?;
    assert_eq!(
        lowering_costs(&facts),
        Some([8, 7, granted.work, granted.bytes])
    );
    assert!(open(&other, &mut facts).is_err());
    // Exceeding lowering's bound reports lowering's message and leaves the
    // header account untouched.
    assert_eq!(
        pay_growth(&facts, 0, granted.bytes),
        Err("lowering work or retention exceeds its input-graph bound")
    );
    let query = charge_source_contract_query(&facts, 1)
        .err()
        .ok_or("query past lowering's bound")?;
    assert!(
        query.to_string().contains("lowering work or retention"),
        "{query}"
    );
    assert_eq!(source_contract_costs(&facts), header);
    Ok(())
}

#[test]
fn crates_without_a_ledger_open_no_account() -> Result {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    open(&report, &mut facts)?;
    assert!(lowering_costs(&facts).is_none());
    pay_growth(&facts, usize::MAX, usize::MAX)?;
    Ok(())
}
