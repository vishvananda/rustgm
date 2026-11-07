//! Source header allocation follows actual rows and keeps failed payment sticky.
use super::*;
use crate::test_source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn provider(width: usize) -> Result<Report> {
    use std::fmt::Write;
    let mut source =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    for index in 0..width {
        write!(source, "#[cold]pub fn retained{index}(){{}}")?;
    }
    Ok(test_source::source_provider(&source, "header_capacity")?)
}
#[test]
fn large_single_owner_sources_pay_only_real_header_and_binding_rows() -> Result {
    let mut previous = 0;
    for width in [7_000, 28_000] {
        let report = provider(width)?;
        // Exercise the actual declaration-attribute producer before later signature
        // and body phases; this is not an executable or completed type certificate.
        let facts = Declarations::new(&report)?;
        let headers = &facts
            .must_use_contracts
            .as_deref()
            .ok_or("contracts")?
            .headers
            .as_deref()
            .ok_or("local ledger")?
            .local;
        assert!(headers.declarations.len() >= width);
        let (steps, bytes) = (headers.steps.get(), headers.bytes.get());
        assert!(
            steps * 2 <= headers.bound.work && bytes * 2 <= headers.bound.bytes,
            "dense linear headers keep their own input-graph headroom: {steps}/{bytes} of {:?}",
            headers.bound
        );
        assert!(
            headers.bound.work >= previous * 3,
            "the bound follows its input graph"
        );
        previous = headers.bound.work;
        for row in headers.declarations.values() {
            if row.owners.len() == 1 {
                assert_eq!(row.owners.capacity(), 1);
            }
        }
    }
    assert_eq!(size_of::<Header>(), 80);
    assert_eq!(size_of::<Owner>(), 56);
    assert_eq!(size_of::<Headers>(), 136);
    Ok(())
}
#[test]
fn duplicate_binding_does_not_charge_a_phantom_header_or_replace_its_storage() -> Result {
    let report = provider(1)?;
    let mut facts = Declarations::new(&report)?;
    let (index, row) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "retained0")
        .ok_or("actual function")?;
    let id = DefId(u32::try_from(index)?);
    let key = row.node.ok_or("actual source")?;
    let before = source_contract_storage(&facts).ok_or("paid header")?;
    assert!(bind_definition(&report, &mut facts, id, key).is_err());
    let after = source_contract_storage(&facts).ok_or("failed paid header")?;
    assert!(after.0 > before.0);
    assert_eq!(
        (after.1, after.2, after.3, after.4),
        (before.1, before.2, before.3, before.4)
    );
    Ok(())
}
#[test]
fn failed_owner_growth_is_paid_before_the_existing_row_changes() -> Result {
    let report = provider(1)?;
    let mut facts = Declarations::new(&report)?;
    let key = report
        .resolution
        .definitions
        .iter()
        .find(|row| report.package.names.get(row.name) == "retained0")
        .and_then(|row| row.node)
        .ok_or("actual declaration")?;
    let headers = &facts
        .must_use_contracts
        .as_deref()
        .ok_or("contracts")?
        .headers
        .as_deref()
        .ok_or("local ledger")?
        .local;
    let owner = headers
        .declarations
        .get(&key)
        .ok_or("actual header")?
        .owners[0]
        .key;
    headers.bytes.set(crate::support::crates::MAX_BYTES);
    let before = source_contract_storage(&facts).ok_or("paid header")?;
    assert!(observe_owner(&report, &mut facts, owner, key).is_err());
    let after = source_contract_storage(&facts).ok_or("failed header")?;
    assert!(after.0 > before.0 && after.1 > before.1);
    assert_eq!((after.2, after.3, after.4), (before.2, before.3, before.4));
    Ok(())
}
