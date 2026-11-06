//! Intermediate qualifiers require the refreshed actual state, even when empty.
use super::*;
use crate::{body, borrow};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<(Declarations, body::templates::Template, Contract)> {
    let text = crate::test_source::fixture("pa31/tests/spec/source-const-cleanup/taglif.rs")?;
    let report = crate::test_source::source_provider(&text, "qualified_state_owner")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "probe"
        })
        .copied()
        .ok_or("probe")?;
    let template = body::templates::Prepared::default().function(&report, &facts, &function)?;
    let mut input = body::regions::Collector::default()
        .function(&facts, &function)?
        .clone();
    body::copy_requirements::prepare(&mut facts, &template.body, &mut input)?;
    facts.types.arena_identity();
    facts.aggregates.arena_identity();
    Ok((facts, template, input))
}
fn refresh(
    facts: &mut Declarations,
    body: &Body,
    input: &Contract,
) -> Result<borrow::regions::Body> {
    input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    Ok(borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        input,
        body,
    )?)
}
#[test]
fn empty_and_nonempty_answers_reject_stale_state_and_accept_actual_refresh() -> Result {
    for questions in [Vec::new(), vec![TypeId::UNIT]] {
        let (mut facts, template, mut input) = fixture()?;
        let before = refresh(&mut facts, &template.body, &input)?;
        let answers = qualify(&mut facts, &template.body, &mut input, &questions)?;
        assert!(
            answers
                .values(
                    &template.body,
                    &facts.types,
                    &facts.aggregates,
                    before.builtins.as_deref()
                )
                .is_err()
        );
        let current = refresh(&mut facts, &template.body, &input)?;
        assert_eq!(
            answers
                .values(
                    &template.body,
                    &facts.types,
                    &facts.aggregates,
                    current.builtins.as_deref()
                )?
                .len(),
            questions.len()
        );
        assert!(
            answers
                .values(
                    &template.body.clone(),
                    &facts.types,
                    &facts.aggregates,
                    current.builtins.as_deref()
                )
                .is_err()
        );
    }
    assert_eq!(size_of::<Qualification<'_>>(), 56);
    Ok(())
}
#[test]
fn empty_answers_do_not_skip_a_genuine_unpaid_lifetime_debt() -> Result {
    let (mut facts, template, mut input) = fixture()?;
    assert!(input.universals > 0);
    let before = refresh(&mut facts, &template.body, &input)?;
    let answers = qualify(&mut facts, &template.body, &mut input, &[])?;
    let first = body::dispatch::copy_frontier(&facts, &input)?;
    let mut debt = crate::types::lifetime_proof::Obligations::new(first);
    debt.outlives.push((
        crate::types::RegionId::bound(0, 0)?,
        crate::types::RegionId::STATIC,
    ));
    input
        .proofs
        .push(body::regions::Proof::new(template.body.source, debt));
    let error = refresh(&mut facts, &template.body, &input)
        .err()
        .ok_or("unpaid universal/static debt accepted")?;
    assert!(error.to_string().contains("lifetime"), "{error}");
    assert!(
        answers
            .values(
                &template.body,
                &facts.types,
                &facts.aggregates,
                before.builtins.as_deref()
            )
            .is_err()
    );
    Ok(())
}
