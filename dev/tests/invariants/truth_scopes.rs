//! Only actual source-provider inputs extend cfg capture grammar and pay its work.
use super::*;
use crate::{
    lex::token::LexError,
    syntax::{
        expression,
        interpolation::{EMPTY_INPUTS, Nonterminal, ParsedSyntax},
        token_tree::Span,
    },
};
use std::{cell::Cell, fmt::Write as _};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn checked<T>(result: Result<T, LexError>) -> TestResult<T> {
    result.map_err(|error| format!("{error:?}").into())
}
fn provider() -> TestResult<Report> {
    let source = crate::test_source::fixture("pa31/tests/spec/literal-predicates/prefix.rs")?;
    Ok(crate::test_source::source_provider(&source, "truth_scope")?)
}
fn occurrence(parsed: &ParsedSyntax) -> Splice<'_> {
    Splice {
        fragment: Nonterminal::Syntax {
            kind: expression::fragment::FragmentKind::Literal,
            parsed,
        },
        original_source: "true",
        original: Span { start: 0, end: 4 },
        destination: Span { start: 0, end: 4 },
    }
}
#[test]
fn source_boolean_demand_survives_wrappers_and_default_inputs_stay_closed() -> TestResult {
    let report = provider()?;
    let core = report
        .package
        .source_core()
        .ok_or("actual source provider")?;
    let parsed = checked(ParsedSyntax::own(
        "true",
        checked(expression::parse("true"))?,
    ))?;
    let splice = occurrence(&parsed);
    assert_eq!(checked(EMPTY_INPUTS.cfg_boolean_literal(&splice))?, None);
    let source = SourceInputs {
        inputs: &EMPTY_INPUTS,
        source: Some(core),
    };
    let editions = crate::syntax::edition_context::WithEditions::new(&source, |_| {
        Ok(crate::support::edition::Edition::Rust2021)
    });
    let paid = Cell::new(0);
    let charge = |steps, _| {
        paid.set(paid.get() + steps);
        Ok(())
    };
    let counted = crate::expand::builtin::choice::Counted {
        inputs: &editions,
        charge: &charge,
    };
    let before = core.macros.usage();
    assert_eq!(checked(counted.cfg_boolean_literal(&splice))?, Some(true));
    assert_eq!(core.macros.usage().0, before.0 + 1);
    assert_eq!(paid.get(), 1);
    // The outer policy owner must not inherit a provider from nested inputs.
    let ordinary = SourceInputs {
        inputs: &counted,
        source: None,
    };
    assert_eq!(checked(ordinary.cfg_boolean_literal(&splice))?, None);
    Ok(())
}
#[test]
fn exhausted_source_boolean_payment_fails_before_reading_the_capture() -> TestResult {
    let report = provider()?;
    let core = report
        .package
        .source_core()
        .ok_or("actual source provider")?;
    let parsed = checked(ParsedSyntax::own(
        "true",
        checked(expression::parse("true"))?,
    ))?;
    let source = SourceInputs {
        inputs: &EMPTY_INPUTS,
        source: Some(core),
    };
    assert!(core.macros.charge(usize::MAX, 0).is_err());
    assert!(source.cfg_boolean_literal(&occurrence(&parsed)).is_err());
    assert!(source.cfg_boolean_literal(&occurrence(&parsed)).is_err());
    assert_eq!(parsed.literals.decoded_bytes(), 0);
    Ok(())
}
#[test]
fn repeated_source_booleans_have_bounded_work_and_one_retained_capture() -> TestResult {
    let prefix = crate::test_source::fixture("pa31/tests/spec/literal-predicates/prefix.rs")?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = prefix.clone();
        source.push_str("macro_rules! many{($b:literal)=>{");
        for index in 0..width {
            writeln!(source, "#[cfg($b)]fn truth_{index}(){{}}")?;
        }
        source.push_str("}}many!(true);");
        let report = crate::test_source::source_provider(&source, "truth_scope")?;
        assert_eq!(report.captures.len(), 1);
        assert_eq!(report.invocations.len(), 1);
        assert_eq!(report.invocations[0].parser_calls, 1);
        assert_eq!(
            report
                .resolution
                .definitions
                .iter()
                .filter(|d| report.package.names.get(d.name).starts_with("truth_"))
                .count(),
            width
        );
        let work = report
            .package
            .source_core()
            .ok_or("actual provider")?
            .macros
            .usage()
            .0;
        assert!(work >= width);
        if let Some(old) = previous {
            assert!(work <= 4 * old + 1024, "{width}: {old}->{work}");
        }
        previous = Some(work);
    }
    Ok(())
}
