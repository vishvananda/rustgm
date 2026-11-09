//! An impl's own completed associated-type bound is a normal form in its
//! environment. A supertrait or body goal whose rewritten subject is a rigid
//! projection matches that premise when they differ only in lifetime inputs,
//! which become region equalities, as rustc 1.98.0 relates a rigid alias's
//! arguments. Elaborated, raw-environment and written-equation premises keep
//! the exact compare.
use std::fmt::Write;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa21/tests/spec/assoc-bounds/certified-premises";
/// A related lifetime input that the premise does not prove is a region error.
const REGION: &str = "trait argument lifetime requirement is not satisfied: required lifetime does not outlive its target";

/// Outcomes declared before rustc 1.98.0 ran: `None` accepts, otherwise the
/// first error this compiler reports for a control that is rejected.
const CASES: [(&str, Option<&str>); 19] = [
    ("bound-positive", None),
    ("where-positive", None),
    ("mixed-positive", None),
    ("hrtb-positive", None),
    ("hrtb-premise", None),
    ("outlives-both-ways", None),
    ("lifetime-trait-positive", None),
    ("body-impl-positive", None),
    ("plain-positive", None),
    ("static-positive", None),
    ("normalizable-region", None),
    ("call-positive", None),
    ("substituted-normalizable", None),
    (
        "bound-missing",
        Some("trait implementation does not satisfy the trait's requirements"),
    ),
    ("hrtb-required", Some(REGION)),
    ("other-lifetime", Some(REGION)),
    ("outlives-one-way", Some(REGION)),
    (
        "gate-different-item",
        Some("trait implementation does not satisfy the trait's requirements"),
    ),
    // rustc accepts; the raw-environment premise is never certified. Stricter.
    (
        "raw-environment-premise",
        Some("associated type requires an unambiguous implementation"),
    ),
];

fn analyze(source: &str) -> TestResult<Result<(), String>> {
    let report = crate::test_source::resolve(source, true)?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn certified_bound_premises_match_declared_outcomes() -> TestResult {
    let mut differences = Vec::new();
    for (name, expected) in CASES {
        let source = crate::test_source::fixture(&format!("{CONTROLS}/{name}.rs"))?;
        let actual = analyze(&source)?;
        if actual.as_ref().err().map(String::as_str) != expected {
            differences.push(format!("{name}: {actual:?}, declared {expected:?}"));
        }
    }
    assert!(differences.is_empty(), "{differences:#?}");
    Ok(())
}

/// `width` lifetime inputs on the member and `premises` distinct bounds.
fn scaled(width: usize, premises: usize) -> TestResult<String> {
    let mut source = String::new();
    let lifetimes = (0..width)
        .map(|i| format!("'x{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let applied = vec!["'a"; width].join(", ");
    writeln!(
        source,
        "trait Pattern: Sized {{ type Searcher<{lifetimes}>; }}"
    )?;
    for index in 0..premises {
        writeln!(source, "trait R{index}<'a> {{}}")?;
    }
    let bounds = (0..premises)
        .map(|i| format!("P::Searcher<{applied}>: R{i}<'a>"))
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(
        source,
        "struct Split<'a, P: Pattern>(P::Searcher<{applied}>);"
    )?;
    writeln!(source, "trait Walk {{}} trait FusedWalk: Walk {{}}")?;
    writeln!(
        source,
        "impl<'a, P> Walk for Split<'a, P> where P: Pattern, {bounds} {{}}"
    )?;
    writeln!(
        source,
        "impl<'a, P> FusedWalk for Split<'a, P> where P: Pattern, {bounds} {{}}"
    )?;
    source.push_str("fn main() {}\n");
    Ok(source)
}

/// Selection rows for `width` × `premises`: each supertrait condition first
/// meets its premise unrewritten, then matches once after one rewrite.
fn requests(width: usize, premises: usize) -> TestResult<(usize, usize)> {
    let source = scaled(width, premises)?;
    let capture = crate::sema::copy_trace::Capture::matching(&[
        "assumption-match ",
        "assumption-needs-normalization ",
    ])?;
    let result = analyze(&source)?;
    let rows = capture.rows()?;
    drop(capture);
    assert_eq!(result, Ok(()), "{width}x{premises}");
    let matched = rows
        .iter()
        .filter(|r| r.starts_with("assumption-match "))
        .count();
    Ok((matched, rows.len() - matched))
}

#[test]
fn certified_premise_requests_do_not_grow_with_member_lifetimes() -> TestResult {
    let mut observed = Vec::new();
    for premises in [1, 4, 16] {
        let base = requests(1, premises)?;
        for width in [4, 16] {
            let wide = requests(width, premises)?;
            assert_eq!(wide, base, "{width}x{premises}");
        }
        observed.push((premises, base));
    }
    let (_, (first_matched, _)) = observed[0];
    for &(premises, (matched, retried)) in &observed {
        // Matches grow by at most four for each added premise, whatever the
        // member's width, and each supertrait condition is retried exactly
        // once, after its one complete rewrite.
        assert!(
            matched - first_matched <= 4 * (premises - 1),
            "{observed:?}"
        );
        assert_eq!(retried, premises, "{observed:?}");
    }
    println!("certified-premise requests={observed:?}");
    Ok(())
}

/// A body copies its environment's trait premises, aliasing each escaping
/// one at its universals. A copy inherits only its own source's
/// certificate, and certifying a premise the copy does not hold fails.
#[test]
fn copied_premises_keep_only_their_own_sources_certificates() -> TestResult {
    use crate::sema::traits::assumptions::Assumptions;
    use crate::types::{
        GenericArgs, ItemId, Kind, RegionId, Store,
        arguments::Argument,
        copy::Properties,
        parameters::{
            Parameter, ParameterKind,
            clauses::Clause,
            environment::{Abilities, Environment},
            predicates::{Bound, Predicate},
        },
        projection, traits,
    };
    let mut store = Store::default();
    store.declare_projection(
        ItemId(2),
        projection::Schema {
            trait_item: ItemId(1),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let pattern = store.intern_trait_application(traits::Application {
        item: ItemId(1),
        arguments: GenericArgs::EMPTY,
    })?;
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(100),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = store.intern(Kind::Parameter(parameter))?;
    // `<P as Pattern>::Searcher<'r>: ReverseSearcher<'r>`.
    let mut premise = |region: RegionId| -> TestResult<Clause> {
        let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
        let ty = store.intern(Kind::Projection(projection::Application {
            item: ItemId(2),
            subject,
            trait_application: pattern,
            arguments,
        }))?;
        let bound = Bound::Trait(store.intern_trait_application(traits::Application {
            item: ItemId(4),
            arguments,
        })?);
        Ok(Clause::Predicate(Predicate { ty, bound }))
    };
    let certified = premise(RegionId::bound(0, 0)?)?;
    let uncertified = premise(RegionId::STATIC)?;
    let alias = premise(RegionId(11))?;
    let ordinary = [certified, uncertified]
        .into_iter()
        .map(Clause::ordinary)
        .collect::<Result<Vec<_>, _>>()?;
    let mut environment = Environment::new(&store, &[(parameter, Abilities::default())])?
        .with_predicates(&store, &ordinary)?;
    environment.certify_normal_premises(&store, &[certified])?;
    let premises = [(certified, alias), (uncertified, uncertified)];
    let mut copied = Assumptions::copied(&store, Some(&environment), &premises)?;
    let index = copied.index.as_ref().ok_or("missing copied index")?;
    assert!(index.normal_premise(alias));
    assert!(!index.normal_premise(uncertified));
    let plain = Assumptions::copied(&store, None, &premises)?;
    let index = plain.index.as_ref().ok_or("missing plain index")?;
    assert!(!index.normal_premise(alias) && !index.normal_premise(uncertified));
    assert!(copied.certify_normal(&[certified]).is_err());
    assert!(Assumptions::default().certify_normal(&[alias]).is_err());
    copied.certify_normal(&[uncertified])?;
    Ok(())
}
