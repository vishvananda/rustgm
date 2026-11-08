//! Consumed original bodies pay their retained rows once, on lowering's own
//! account; shrinking the consumed containers pays work only.
use super::*;
use crate::sema::header_attributes::{lowering_costs, source_contract_costs};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Lowering's steps and bytes so far.
fn lowering(facts: &Declarations) -> Result<(usize, usize)> {
    let [steps, bytes, ..] = lowering_costs(facts).ok_or("open lowering account")?;
    Ok((steps, bytes))
}

#[test]
fn consumed_originals_pay_their_rows_once_and_shrinking_pays_only_work() -> Result {
    let source = super::ledger_tests::inserted(1024, super::ledger_tests::noise)?;
    let report = crate::test_source::source_provider(&source, "needs_drop_work")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    facts.open_lowering_account(&report)?;
    let header = source_contract_costs(&facts);
    let mut program = crate::body::build_concrete(&report, &facts)?;
    let count = program.bodies.len();
    let built = retained::vector(&program.drops.as_ref().ok_or("pending rows")?.bodies);
    let before = lowering(&facts)?;
    let mut originals = Originals::take(&report, &facts, &mut program)?;
    let taken = lowering(&facts)?;
    // Retained growth stays paid: the row owner, the exactly reserved
    // replacement row vector at its actual slots, and the moved one, which no
    // earlier owner charged when it was built.
    let replacement = retained::vector(&program.drops.as_ref().ok_or("replacement")?.bodies);
    assert_eq!(replacement, count * size_of::<Requirements>());
    assert_eq!(
        taken.1 - before.1,
        size_of::<Rows<'_>>() + replacement + built
    );
    originals.reverse(&facts)?;
    let reversed = lowering(&facts)?;
    let (mut copied, mut recharged, mut consumed, mut shrinks) = (0, 0, 0, 0);
    while originals.pop()?.is_some() {
        consumed += 1;
        let rows = originals.requirements.as_ref().ok_or("rows")?.values.len();
        if originals.bodies.len() < originals.bodies.capacity() / 2 {
            copied += originals.bodies.len() + rows;
            shrinks += 1;
            recharged +=
                originals.bodies.len() * size_of::<Body>() + rows * size_of::<Requirements>();
        }
        originals.shrink(&facts)?;
    }
    let shrunk = lowering(&facts)?;
    eprintln!(
        "originals count={count} built={built} take={} copied={copied} recharged={recharged} body={} row={}",
        taken.1 - before.1,
        size_of::<Body>(),
        size_of::<Requirements>()
    );
    assert_eq!(consumed, count);
    assert_eq!(shrunk.1, reversed.1, "shrinking retains no new storage");
    // Each moved body and row pays one unit, and each charge one more to
    // authenticate its source payer.
    assert_eq!(shrunk.0 - reversed.0, copied + shrinks);
    // Before the repair every halving paid its copy again as storage, at
    // least half of the consumed body and row storage a second time.
    assert!(recharged >= (count / 2 - 1) * (size_of::<Body>() + size_of::<Requirements>()));
    // None of it reached the header account.
    assert_eq!(source_contract_costs(&facts), header);
    Ok(())
}
