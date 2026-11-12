//! A real source producer owns each size query; prior observation is not credit.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn facts() -> Result<Declarations> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-transmute/generic-borrowed-const.rs")?;
    let report = crate::test_source::source_provider(&source, "transmute_query_budget")?;
    Ok(crate::sema::declarations::analyze(&report)?)
}
#[test]
fn transmute_queries_keep_local_work_and_sticky_source_payment() -> Result {
    let mut expected = None;
    for prior in [0, LIMIT, LIMIT.saturating_mul(2)] {
        let mut facts = facts()?;
        facts.body_work.canonical_steps = prior;
        let source = TypeId::primitive(crate::types::Primitive::U32);
        let destination = TypeId::primitive(crate::types::Primitive::I32);
        let before = crate::sema::header_attributes::source_contract_costs(&facts).0;
        require_equal(&mut facts, None, source, destination)?;
        let observed = (
            facts.body_work.canonical_steps - prior,
            crate::sema::header_attributes::source_contract_costs(&facts).0 - before,
        );
        assert!(observed.0 > 0);
        assert_eq!(observed.0, observed.1);
        if let Some(expected) = expected {
            assert_eq!(observed, expected);
        }
        expected = Some(observed);
        let byte = TypeId::primitive(crate::types::Primitive::U8);
        assert!(
            require_equal(&mut facts, None, byte, destination)
                .is_err_and(|error| error.contains("equal source and destination sizes"))
        );
        assert!(
            crate::sema::header_attributes::charge_source_contract_query(&facts, usize::MAX)
                .is_err()
        );
        let steps = facts.layouts.steps;
        let cold = TypeId::primitive(crate::types::Primitive::U128);
        assert!(facts.layouts.get(cold).is_none());
        for _ in 0..2 {
            facts.body_work.canonical_steps = 0;
            assert_eq!(
                require_equal(&mut facts, None, source, cold),
                Err("source transmute query work exhausted".to_owned())
            );
            assert_eq!(facts.layouts.steps, steps);
            assert!(facts.layouts.get(cold).is_none());
        }
    }
    Ok(())
}
#[test]
fn local_transmute_limit_still_rejects_before_layout_mutation() -> Result {
    let mut facts = facts()?;
    let ty = TypeId::primitive(crate::types::Primitive::U128);
    let mut work = LIMIT;
    let steps = facts.layouts.steps;
    assert_eq!(
        size(&mut facts, ty, &mut work, &mut 0),
        Err("source transmute layout work budget exceeded".to_owned())
    );
    assert!(facts.layouts.get(ty).is_none());
    assert_eq!(facts.layouts.steps, steps);
    Ok(())
}
