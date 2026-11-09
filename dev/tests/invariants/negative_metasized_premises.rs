//! rustc 1.98 negates one impl's predicates under the other impl's
//! `param_env`. A negative impl's `T: MetaSized` condition, the default of a
//! `T: ?Sized` parameter, is proved from the other header's own `MetaSized`
//! premises: its Sized parameters, its Sized, Copy and Clone subjects and its
//! `MetaSized` clauses. A header's own premises never pay a debt of the clause
//! being negated, and a `PointeeSized` premise pays none.
use crate::sema::model::Declarations;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/negative-metasized";
const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";
const COUNTS: [usize; 3] = [8, 16, 32];

/// The checked rustc outcomes of `outcomes.json`. Supertrait-implied premises
/// are elaborated, and keep the side of the impl whose generics own them.
/// Amended at the premise-sides batch: `sized-premise-side`, the recorded
/// accepts-invalid, is rejected as rustc rejects it, since only the client's
/// empty Sized premises decide the blanket's `V: Sized`.
const CASES: [(&str, Option<&str>); 13] = [
    ("metasized-param", None),
    ("metasized-written", None),
    ("metasized-strict", None),
    ("sized-param", None),
    ("pointee-param", Some(OVERLAP)),
    ("sized-negative", Some(OVERLAP)),
    ("metasized-stable", Some(OVERLAP)),
    ("supertrait-implied", None),
    ("supertrait-premise-side", Some(OVERLAP)),
    ("edge-pointee", None),
    ("edge-specialized", None),
    ("premise-side", Some(OVERLAP)),
    ("sized-premise-side", Some(OVERLAP)),
];

fn analyze(source: &str) -> TestResult<Result<Declarations, String>> {
    let report = crate::test_source::source_provider(source, "negative_metasized")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|analysis| analysis.facts)
        .map_err(|error| error.message.clone()))
}

#[test]
fn negative_metasized_conditions_assume_only_the_other_headers_premises() -> TestResult {
    for (name, expected) in CASES {
        let source = crate::test_source::fixture(&format!("{CONTROLS}/{name}.rs"))?;
        match (expected, analyze(&source)?) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error, message, "{name}"),
            (None, Err(error)) => return Err(format!("{name}: rejected: {error}").into()),
            (Some(message), Ok(_)) => {
                return Err(format!("{name}: accepted, not {message}").into());
            }
        }
    }
    Ok(())
}

/// One pair whose blanket has `count - 1` conditions that hold and a last
/// `Seal` condition, against a client with `count` parameters bounded by
/// `bound`. Every condition is searched; only the last one decides. With
/// `?Sized`, the negative impl is `?Sized` and its `MetaSized` debt is paid
/// from the client's `count` premises. The `PointeeSized` control has the same
/// shape and conditions but no `MetaSized` premise or debt.
fn scaled(prefix: &str, count: usize, bound: &str) -> TestResult<String> {
    use std::fmt::Write;
    let mut text = format!(
        "{prefix}pub trait Seal {{}}\nimpl<T: {bound}> !Seal for &T {{}}\npub trait Open {{}}\nimpl<T: ?Sized> Open for T {{}}\npub trait Holds<X> {{}}\nimpl<"
    );
    for index in 0..count {
        let condition = if index + 1 == count { "Seal" } else { "Open" };
        write!(text, "T{index:03}: {condition}, ")?;
    }
    text.push_str("S> Holds<(");
    for index in 0..count {
        write!(text, "T{index:03}, ")?;
    }
    text.push_str(")> for S {}\npub struct Local;\nimpl<");
    for index in 0..count {
        write!(text, "U{index:03}: {bound}, ")?;
    }
    text.push_str("> Holds<(");
    for index in 0..count {
        write!(text, "&U{index:03}, ")?;
    }
    text.push_str(")> for Local {}\n");
    Ok(text)
}

/// The coherence steps of the pair of `Holds` impls.
fn pair_steps(facts: &mut Declarations, holds: u32) -> TestResult<usize> {
    let mut headers = Vec::new();
    for header in facts
        .generics
        .implementations
        .as_ref()
        .ok_or("registry")?
        .headers()
    {
        let Some((application, _)) = header.predicate.bound.trait_parts() else {
            continue;
        };
        let item = facts
            .types
            .trait_application(application)
            .ok_or("trait application")?
            .item;
        if item.0 == holds {
            headers.push((header.predicate, header.owner));
        }
    }
    let [left, right] = <[_; 2]>::try_from(headers).map_err(|_| "one pair of Holds impls")?;
    let (possible, steps) = super::super::possible(facts, left, right)?;
    assert!(!possible, "the negative impl proves the pair disjoint");
    Ok(steps)
}

/// The `Holds` pair's coherence steps for `count` parameters bounded by `bound`.
fn measured(prefix: &str, count: usize, bound: &str) -> TestResult<usize> {
    let report = crate::test_source::source_provider(&scaled(prefix, count, bound)?, "scaled")?;
    let holds = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "Holds")
        .map(u32::try_from)
        .transpose()?
        .ok_or("Holds")?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|error| format!("{count} {bound}: {}", error.message))?
        .facts;
    pair_steps(&mut facts, holds)
}

/// The premises' own work, over the control's, grows by at most a constant
/// per premise. The steps of both shapes are observed together.
#[test]
fn negative_metasized_premises_cost_linear_pair_work() -> TestResult {
    let fixture = crate::test_source::fixture(&format!("{CONTROLS}/metasized-param.rs"))?;
    let prefix = fixture
        .split("pub trait Seal")
        .next()
        .ok_or("metasized-param prefix")?;
    let mut premised = Vec::with_capacity(COUNTS.len());
    let mut control = Vec::with_capacity(COUNTS.len());
    for count in COUNTS {
        premised.push(measured(prefix, count, "?Sized")?);
        control.push(measured(prefix, count, "PointeeSized")?);
    }
    eprintln!("MetaSized premise pair steps {premised:?}, PointeeSized control {control:?}");
    let added: Vec<_> = premised
        .iter()
        .zip(&control)
        .map(|(premised, control)| premised.saturating_sub(*control))
        .collect();
    let [small, middle, large] = <[usize; 3]>::try_from(added).map_err(|_| "counts")?;
    let (first, second) = (middle - small, large - middle);
    assert!(
        first > 0,
        "premise work did not grow: {premised:?} {control:?}"
    );
    assert!(
        second <= 2 * first + 64,
        "premise work: {premised:?} over {control:?}"
    );
    Ok(())
}
