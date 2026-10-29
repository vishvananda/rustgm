//! Capped debt remains source witnessed and cold; dense policy widths do not grow.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn caps_preserve_retained_attribute_debt_and_compact_records() -> Result {
    use crate::{expand::config::Config, support::diagnostics::LintCap};
    let source =
        crate::test_source::fixture("pa40/tests/spec/lint-cap-allow/source-unknown-expect.rs")?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let mut report = crate::test_source::source_provider_config(&source, "cap_storage", &config)?;
    let producer = report.lint_producer().ok_or("producer")?;
    assert_eq!(report.deferred_lint_expectations()?, 1);
    assert!(!producer.has_expectations());
    assert!(
        producer
            .owners
            .iter()
            .all(|owner| owner.parsed.expectations.is_empty())
    );
    assert!(
        producer.owners.iter().any(
            |owner| owner.parsed.witnesses.iter().any(|witness| matches!(
                witness.attribute.input,
                crate::syntax::attribute::Input::Delimited(_)
            ))
        )
    );
    assert_eq!(size_of::<crate::support::diagnostics::LintCap>(), 1);
    assert_eq!(size_of::<Config>(), 80);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::resolve::expansion::Report>(), 3096);
    assert_eq!(size_of::<Producer>(), 368);
    assert_eq!(size_of::<Owner>(), 160);
    assert_eq!(size_of::<Policy>(), 152);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<super::super::expectations::Domain>(), 8);
    let initial = producer.work();
    assert_eq!(report.deferred_lint_expectations()?, 1);
    assert_eq!(producer.work().bytes, initial.bytes);
    assert!(producer.work().steps > initial.steps);
    println!(
        "cap Config={} Package={} Producer={} Domain={} ArcDomainPaid={} Owner={} Policy={} NodeFacts={} charged={}",
        size_of::<Config>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<Producer>(),
        size_of::<super::super::expectations::Domain>(),
        2 * size_of::<usize>() + size_of::<super::super::expectations::Domain>(),
        size_of::<Owner>(),
        size_of::<Policy>(),
        size_of::<crate::sema::model::NodeFacts>(),
        producer.work().bytes
    );
    report.package.lint_cap = LintCap::Uncapped;
    assert!(crate::sema::declarations::analyze(&report).is_err());
    Ok(())
}
