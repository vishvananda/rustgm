//! Parenthesized callable sugar quantifies its impl condition even when the
//! binder binds no region. Lowered out of that empty binder, the condition
//! shares the trial's inference, so an impl parameter that only its `Output`
//! equality determines resolves.
use crate::types::{
    Kind, Primitive, RegionId, Store, TypeId,
    parameters::{
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
    substitute,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/fn-sugar-impl-parameters";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 10] = [
    ("from-fn-item", None),
    ("const-call-adapter", None),
    ("elided-lifetime", None),
    ("map-adapter", None),
    ("inline-bound", None),
    ("caller-premise", None),
    ("explicit-binding", None),
    ("impl-lifetime-output", None),
    ("caller-mismatch", Some("incompatible types")),
    (
        "output-not-option",
        Some("associated type requires an unambiguous implementation"),
    ),
];

/// Declared controls whose rustc outcome this analysis does not reach yet,
/// each with the outcome it has instead. The assertion fails when one changes.
/// `impl-lifetime-mismatch`: rustc's region error belongs to body region
/// checking, which this analysis does not run for a generic body.
/// `elided-lifetime` and `map-adapter` left this list when a region-binding
/// condition determined its parameter (`spec/region-sugar-impl-parameters/`)
/// and override bounds were compared in normal form
/// (`spec/override-projection-bounds/`).
const GAPS: [(&str, Option<&str>); 1] = [("impl-lifetime-mismatch", None)];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    let report = crate::test_source::source_provider(&source, "fn_sugar")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn sugar_determined_impl_parameters_match_independent_outcomes() -> TestResult {
    for (name, expected) in CASES {
        match (expected, analyze(name)?) {
            (None, Ok(())) => {}
            (Some(message), Err(error)) => assert_eq!(error, message, "{name}"),
            (None, Err(error)) => return Err(format!("{name}: rejected: {error}").into()),
            (Some(message), Ok(())) => {
                return Err(format!("{name}: accepted, not {message}").into());
            }
        }
    }
    for (name, expected) in GAPS {
        let expected = expected.map_or(Ok(()), |message| Err(message.to_owned()));
        assert_eq!(analyze(name)?, expected, "{name}");
    }
    Ok(())
}

fn reference(store: &mut Store, region: RegionId) -> TestResult<TypeId> {
    Ok(store.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::primitive(Primitive::U8),
    })?)
}
fn sized(store: &mut Store, region: RegionId) -> TestResult<Predicate> {
    Ok(Predicate {
        ty: reference(store, region)?,
        bound: Bound::Sized,
    })
}
fn quantified(store: &mut Store, bound_regions: u32, region: RegionId) -> TestResult<Clause> {
    let predicate = sized(store, region)?;
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        bound_regions,
        predicate,
    })?))
}

/// The impl's first captured lifetime is `bound(0, 0)` in its header and
/// `bound(1, 0)` inside the clause binder. Lowering and then substituting the
/// capture gives the same predicate as substituting beneath the binder and
/// then lowering, and the same as the ordinary clause written without sugar.
#[test]
fn an_empty_sugar_binder_lowers_and_commutes_with_impl_substitution() -> TestResult {
    let mut store = Store::default();
    let fresh = RegionId(9);
    let sugar = quantified(&mut store, 0, RegionId::bound(1, 0)?)?;
    let ordinary = sized(&mut store, RegionId::bound(0, 0)?)?;
    let mut lowering = None;
    let Clause::Predicate(lowered) = super::vacuous(&mut store, &mut lowering, sugar)? else {
        return Err("an empty binder stayed quantified".into());
    };
    assert_eq!(lowered, ordinary);
    let mut fold = substitute::Bound::parameters_captured(&store, &[], vec![fresh])?;
    let substituted = fold.predicate(&mut store, lowered)?;
    assert_eq!(substituted, sized(&mut store, fresh)?);
    let Clause::ForAll(beneath) = fold.clause(&mut store, sugar)? else {
        return Err("substitution opened the binder".into());
    };
    let body = store
        .quantified(beneath)
        .ok_or("missing substituted binder")?
        .predicate;
    let commuted = substitute::Bound::lower(1).predicate(&mut store, body)?;
    assert_eq!(commuted, substituted);
    assert!(lowering.is_some_and(|lowering| lowering.steps > 0));
    Ok(())
}

/// A binder that binds a region keeps its scoped proof. An empty binder whose
/// body names its own binder never interns, so lowering never meets one.
#[test]
fn region_binders_stay_scoped_and_malformed_empty_binders_never_intern() -> TestResult {
    let mut store = Store::default();
    let mut lowering = None;
    let scoped = quantified(&mut store, 1, RegionId::bound(0, 0)?)?;
    assert_eq!(super::vacuous(&mut store, &mut lowering, scoped)?, scoped);
    let ordinary = Clause::Predicate(sized(&mut store, RegionId::STATIC)?);
    assert_eq!(
        super::vacuous(&mut store, &mut lowering, ordinary)?,
        ordinary
    );
    assert!(lowering.is_none());
    assert!(quantified(&mut store, 0, RegionId::bound(0, 0)?).is_err());
    Ok(())
}
