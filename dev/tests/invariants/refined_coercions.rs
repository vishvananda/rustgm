//! A `CoerceUnsized` impl between two pattern types with one identical
//! pattern relates their bases by an ordinary `CoerceUnsized` obligation in
//! the impl's environment, as rustc 1.98.0 decides core's non-null raw
//! pointer impl; the coercion itself is not executed.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/pattern-coercions";
const RELATION: &str =
    "source CoerceUnsized implementation does not prove its exact field relation";
const BASE: &str = "non-null pattern base must be a raw pointer or type parameter";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 13] = [
    ("core-pattern-coercion", None),
    ("mutable-to-const", None),
    ("generic-base", None),
    ("nominal-pattern-field", None),
    ("dispatch-same-range", None),
    (
        "different-patterns",
        Some("source CoerceUnsized pattern types require one identical pattern"),
    ),
    ("integer-base", Some(RELATION)),
    (
        "pattern-to-pointer",
        Some("source CoerceUnsized implementation does not retain a permitted pointer coercion"),
    ),
    ("mutability-mismatch", Some(RELATION)),
    ("missing-unsize", Some(RELATION)),
    ("missing-pointer-impl", Some(RELATION)),
    ("nominal-without-pattern-impl", Some(RELATION)),
    (
        "dispatch-different-ranges",
        Some(
            "DispatchFromDyn impl must preserve its reference lifetime, pointer mutability or nominal owner",
        ),
    ),
];

/// Declared controls rustc 1.98.0 accepts and this analysis rejects by name:
/// non-null patterns over a reference or over another pattern type, and a
/// body that executes the pattern coercion. The assertion fails when one
/// changes.
const STRICTER: [(&str, &str); 3] = [
    ("reference-base", BASE),
    ("nested-pattern", BASE),
    ("executed-coercion", "incompatible types"),
];

#[test]
fn pattern_coercions_relate_bases_by_coerce_unsized() -> TestResult {
    let stricter = STRICTER.map(|(name, message)| (name, Some(message)));
    let mut mismatches = Vec::new();
    for (name, expected) in CASES.into_iter().chain(stricter) {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "pattern_coercion")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) if error.message == message => {}
            (_, Err(error)) => mismatches.push(format!("{name}: rejected: {}", error.message)),
            (_, Ok(_)) => mismatches.push(format!("{name}: accepted")),
        }
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(mismatches.join("\n").into())
    }
}

/// The exact core prefix and impls, then `impls` distinct non-null pattern
/// impls from a mutable pointer to an array to one to its slice, apart from
/// core's `*const` impl, and `consumers` NonNull-shaped nominal impls whose
/// changed field selects core's pattern impl.
fn scaling(impls: usize, consumers: usize) -> Result<String, Box<dyn std::error::Error>> {
    use std::fmt::Write as _;
    let mut source = crate::test_source::fixture(&format!("{CONTROLS}/nominal-pattern-field.rs"))?;
    for index in 0..impls {
        write!(
            source,
            "pub struct Family{index};\n\
             impl CoerceUnsized<pattern_type!(*mut [Family{index}] is !null)> \
             for pattern_type!(*mut [Family{index}; 4] is !null) {{}}\n"
        )?;
    }
    for index in 0..consumers {
        write!(
            source,
            "#[repr(transparent)]\n\
             pub struct Handle{index}<T: PointeeSized> {{ pointer: pattern_type!(*const T is !null) }}\n\
             impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<Handle{index}<U>> for Handle{index}<T> \
             where T: Unsize<U> {{}}\n"
        )?;
    }
    Ok(source)
}

fn steps(impls: usize, consumers: usize) -> Result<usize, Box<dyn std::error::Error>> {
    let report =
        crate::test_source::source_provider(&scaling(impls, consumers)?, "pattern_scaling")?;
    let analysis = crate::pipeline::analyze_source(&report)
        .map_err(|error| format!("{impls} {consumers}: {}", error.message))?;
    Ok(analysis.facts.body_work.canonical_steps)
}

/// Each pattern impl proves one base obligation and each nominal impl one
/// field obligation; the selection work grows linearly with either count.
/// A consumer's field goal also visits every impl that shares the coarse
/// non-null pattern head, as raw pointer impls share their mutability's
/// head, so growing both counts together adds that product.
#[test]
fn pattern_coercion_work_is_linear_in_impls() -> TestResult {
    let mut observed = Vec::new();
    for (impls, consumers) in [(8, 1), (32, 1), (1, 8), (1, 32)] {
        observed.push(steps(impls, consumers)?);
    }
    println!("pattern coercion canonical steps for 8 and 32 impls, then consumers: {observed:?}");
    for &[small, large] in observed.as_chunks::<2>().0 {
        assert!(large > small, "each impl pays its proof: {observed:?}");
        // Quadrupling the impls at most quintuples the work.
        assert!(large <= small * 5, "{observed:?}");
    }
    Ok(())
}
