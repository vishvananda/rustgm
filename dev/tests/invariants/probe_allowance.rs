//! Source probes pay their producer; PA19's ordinary candidate cap stays fixed.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const INPUT: &str = "pa31/tests/spec/const-supertrait-lookup/plain-ord-method.rs";
#[test]
fn source_method_queries_ignore_prior_observations_and_preserve_actual_selection() -> Result {
    let source = crate::test_source::fixture(INPUT)?;
    let mut expected = None;
    for prior in [0, 1_000_000, 2_000_000] {
        let report = crate::test_source::source_provider(&source, "probe_allowance")?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        facts.methods.selection_steps = prior;
        let paid = crate::sema::header_attributes::source_contract_costs(&facts).0;
        let facts = crate::sema::bodies::analyze(&report, facts)?;
        let mut calls = facts
            .methods
            .calls
            .values()
            .map(|call| {
                (
                    call.method,
                    call.callable,
                    call.receiver,
                    call.dereferences,
                    call.borrow,
                )
            })
            .collect::<Vec<_>>();
        calls.sort_by_key(|call| call.0.0);
        assert!(!calls.is_empty());
        let observed = (
            facts.methods.selection_steps - prior,
            crate::sema::header_attributes::source_contract_costs(&facts).0 - paid,
            calls,
        );
        assert!(observed.0 > 0);
        if let Some(expected) = &expected {
            assert_eq!(&observed, expected);
        }
        expected = Some(observed);
    }
    Ok(())
}
#[test]
fn actual_source_probe_exhaustion_is_sticky_and_ordinary_cap_stays_fixed() -> Result {
    let source = crate::test_source::fixture(INPUT)?;
    let report = crate::test_source::source_provider(&source, "probe_allowance")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let paid = crate::sema::header_attributes::source_contract_costs(&facts).0;
    account_probe(&mut facts, 37)?;
    assert_eq!(
        crate::sema::header_attributes::source_contract_costs(&facts).0 - paid,
        37
    );
    assert!(
        crate::sema::header_attributes::charge_source_contract_query(&facts, usize::MAX).is_err()
    );
    for _ in 0..2 {
        facts.methods.selection_steps = 0;
        assert_eq!(
            account_probe(&mut facts, 1),
            Err("source method selection query work exhausted")
        );
        assert_eq!(facts.methods.selection_steps, 1);
    }
    let report = crate::test_source::resolve(
        "struct A; impl A{fn f(&self){}} fn actual(a:A){a.f();}",
        true,
    )?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    assert!(facts.source_bindings.is_none());
    facts.methods.selection_steps = 1_000_000;
    assert_eq!(
        account_probe(&mut facts, 1),
        Err("method selection work budget exceeded")
    );
    Ok(())
}
