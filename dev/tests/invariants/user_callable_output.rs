//! Rustc assembles impl candidates for a callable trait beside its language
//! evidence. A user impl for a struct or for `&mut Adapter`, whose referent is
//! not callable, therefore normalizes its own `Self::Output`. Each frame lists
//! its impl candidates once: a declined referent lists them after forwarding,
//! and a pattern marker, which forwards after them, never lists them again.
use super::{Decision, Declarations, Frame, Search};
use crate::{
    sema::traits::{
        assumptions::Assumptions, implementations::Availability, selection::search::tests,
    },
    types::{Constraint, GenericArgs, Kind, Primitive, RegionId, TypeId, arguments::Argument},
};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/user-callable-output";
const NONNULL: &str = "pa31/tests/spec/source-nonnull/source-methods.rs";
const COUNTS: [usize; 3] = [6, 24, 96];
const BYTE: TypeId = TypeId::primitive(Primitive::U8);

/// Independently checked rustc 1.98.0 outcomes: `None` accepts, otherwise the
/// first error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 4] = [
    ("struct-output", None),
    ("reference-output", None),
    ("output-mismatch", Some("incompatible types")),
    (
        "reference-without-impl",
        Some("associated type requires an unambiguous implementation"),
    ),
];

fn analyze(name: &str) -> TestResult<Result<(), String>> {
    let path = format!("{CONTROLS}/{name}.rs");
    let source = crate::test_source::fixture(&path)?;
    let report = crate::test_source::source_provider(&source, "user_callable")?;
    Ok(crate::pipeline::analyze_source(&report)
        .map(|_| ())
        .map_err(|error| error.message.clone()))
}

#[test]
fn user_callable_impls_keep_their_own_output_like_rustc() -> TestResult {
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
    Ok(())
}

/// A provider with `count` unrelated user impls of `role`, each on its own struct.
fn provider(
    base: &str,
    count: usize,
    body: impl Fn(usize) -> String,
) -> TestResult<(crate::resolve::expansion::Report, Declarations)> {
    let mut source = base.to_owned();
    for index in 0..count {
        write!(source, "\npub struct S{index};{}", body(index))?;
    }
    let report = crate::test_source::source_provider(&source, "listed_candidates")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}

/// Selects `ty: role<arguments>` and returns its decision with each candidate list.
fn listed(
    facts: &mut Declarations,
    role: crate::resolve::model::DefId,
    ty: TypeId,
    arguments: GenericArgs,
) -> TestResult<(bool, Vec<usize>)> {
    let predicate = tests::predicate(facts, role, ty, arguments)?;
    let (mut context, goal) = tests::instantiate(facts, predicate)?;
    let assumptions = Assumptions::default();
    let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
    let first = Frame::new(facts, &mut context, &goal, 1)?;
    let decision = search.run(facts, first)?;
    assert!(matches!(decision, Decision::Proven(_) | Decision::Absent));
    Ok((
        matches!(decision, Decision::Proven(_)),
        std::mem::take(&mut search.query.candidate_lists),
    ))
}

#[test]
fn subjects_without_signature_or_referent_list_their_own_candidates_once() -> TestResult {
    let base = crate::test_source::fixture(&format!("{CONTROLS}/struct-output.rs"))?
        + "\npub struct Plain;";
    let mut observed = Vec::new();
    for count in COUNTS {
        let (report, mut facts) = provider(&base, count, |index| {
            format!(
                "impl FnOnce<(u8,)> for S{index} {{ type Output = u8; \
                 extern \"rust-call\" fn call_once(self, args: (u8,)) -> u8 {{ loop {{}} }} }}"
            )
        })?;
        let role = tests::definition(&report, "FnOnce")?;
        let inputs = facts.types.intern_list(&[BYTE])?;
        let inputs = facts.types.intern(Kind::Tuple(inputs))?;
        let arguments = facts.types.intern_arguments(&[Argument::Type(inputs)])?;
        let mut cases = Vec::new();
        for name in ["S0", "Plain"] {
            let subject =
                facts.definitions[tests::definition(&report, name)?.0 as usize].ok_or("subject")?;
            let reference = facts.types.intern(Kind::Reference {
                mutable: true,
                region: RegionId::STATIC,
                referent: subject,
            })?;
            cases.push(listed(&mut facts, role, subject, arguments)?);
            cases.push(listed(&mut facts, role, reference, arguments)?);
        }
        observed.push(cases);
    }
    let [first, ..] = observed.as_slice() else {
        return Err("no observations".into());
    };
    // The struct's own impl proves it; neither reference nor `Plain` is callable.
    let accepted: Vec<bool> = first.iter().map(|(accepted, _)| *accepted).collect();
    assert_eq!(accepted, [true, false, false, false], "{first:?}");
    // A struct lists its candidates once. A declined `&mut` subject lists its
    // own once more after its referent's `FnMut` frame listed that referent's.
    let lists: Vec<usize> = first.iter().map(|(_, lists)| lists.len()).collect();
    assert_eq!(lists, [1, 2, 1, 2], "{first:?}");
    assert!(
        observed.iter().all(|cases| cases == first),
        "candidate lists grew with unrelated impls: {observed:?}"
    );
    Ok(())
}

#[test]
fn a_pattern_marker_without_base_evidence_never_relists_its_candidates() -> TestResult {
    // The provider keeps every `*const T` declaration and drops `*mut T`'s copying.
    let base = crate::test_source::fixture(NONNULL)?;
    let mut base = base.replace(
        "const impl<T:PointeeSized> Clone for *mut T { fn clone(&self)->Self { *self } }\n",
        "",
    );
    base = base.replace("impl<T:PointeeSized> Copy for *mut T {}\n", "");
    assert!(
        !base.contains("for *mut T {"),
        "unexpected raw pointer copying"
    );
    let mut observed = Vec::new();
    for count in COUNTS {
        let (report, mut facts) = provider(&base, count, |index| {
            format!(
                "impl Clone for S{index} {{ fn clone(&self)->Self {{ S{index} }} }} impl Copy for S{index} {{}}"
            )
        })?;
        let role = tests::definition(&report, "Copy")?;
        let mut cases = Vec::new();
        for mutable in [false, true] {
            let pointee = facts.types.intern(Kind::Pointer {
                mutable,
                pointee: BYTE,
            })?;
            let refined = facts.types.intern(Kind::Pattern {
                base: pointee,
                constraint: Constraint::NonNull,
            })?;
            cases.push(listed(&mut facts, role, refined, GenericArgs::EMPTY)?);
        }
        observed.push(cases);
    }
    let [first, ..] = observed.as_slice() else {
        return Err("no observations".into());
    };
    // `*const u8` is Copy through its one impl, whose `u8: PointeeSized`
    // condition lists no impl; `*mut u8` has none. Every frame lists its own
    // candidates once: the marker's (no impl names a pattern type) precede its base's.
    assert_eq!(
        first.as_slice(),
        [(true, vec![0, 1, 0]), (false, vec![0, 0])],
        "{first:?}"
    );
    assert!(
        observed.iter().all(|cases| cases == first),
        "candidate lists grew with unrelated impls: {observed:?}"
    );
    Ok(())
}
