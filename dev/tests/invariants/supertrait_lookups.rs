//! rustc 1.98's method lookup ignores constness. A method of a const trait
//! resolves through an ordinary impl or an ordinary premise, and its trait's
//! `[const]` supertraits are required only when the call is in a const
//! context, where an ordinary impl or premise does not satisfy them. Census
//! 22's record, core's `<*const VTable>::cmp` through the non-const
//! `impl Ord for *const T`, is such a lookup.
use std::fmt::Write as _;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/const-supertrait-lookup";
const FOUNDATION: &str = "pa31/tests/spec/function-pointers/foundation.rs";
const OBLIGATION: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
const AMBIGUOUS: &str = "multiple applicable trait methods";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects (E0277 or
/// E0034). `generic-const-fn`'s text was declared unpredicted and is frozen
/// as first observed.
const CASES: [(&str, Option<&str>); 22] = [
    ("ord-qualified", None),
    ("ord-method", None),
    ("generic-qualified", None),
    ("generic-method", None),
    ("plain-ord-qualified", None),
    ("plain-ord-method", None),
    ("simple-qualified", None),
    ("simple-method", None),
    ("const-impls", None),
    ("const-impls-const-fn", None),
    ("maybe-const-generic-method", None),
    ("maybe-const-generic-qualified", None),
    ("binder-plain-method", None),
    ("binder-const-impl", None),
    ("binder-inherent", None),
    ("binder-plain-inherent", None),
    ("ord-const-fn", Some(OBLIGATION)),
    ("ord-const-item", Some(OBLIGATION)),
    ("generic-const-fn", Some(OBLIGATION)),
    ("ambiguous-ord-seq", Some(AMBIGUOUS)),
    (
        "ambiguous-ord-seq-method",
        Some("multiple applicable methods for this receiver"),
    ),
    ("ambiguous-plain-seq", Some(AMBIGUOUS)),
];

#[test]
fn const_supertrait_lookups_match_declared_outcomes() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "const_supertrait_lookup")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}

const DEPTHS: [usize; 3] = [1, 8, 64];
const QUALIFIED: &str = "pub fn f(a: *const V, b: *const V) -> u8 { <*const V>::cmp(&a, &b) }\n";
const METHOD_CALL: &str = "pub fn g(a: *const V, b: *const V) -> u8 { a.cmp(&b) }\n";
const BOTH: [&str; 2] = [QUALIFIED, METHOD_CALL];
/// Positions in [`counters`]: selection probes and steps, canonical steps.
const PROBES: usize = 2;
const STEPS: usize = 3;
const CANONICAL: usize = 13;

/// Lookup work does not depend on the supertraits' constness: a `[const]`
/// hierarchy of each depth pays exactly its non-const twin's probes and steps.
#[test]
fn const_supertrait_lookup_work_equals_its_plain_twin() -> TestResult {
    let foundation = crate::test_source::fixture(FOUNDATION)?;
    let mut observed = Vec::new();
    for depth in DEPTHS {
        let conditional = counters(&hierarchy(&foundation, depth, true, &BOTH)?)?;
        let plain = counters(&hierarchy(&foundation, depth, false, &BOTH)?)?;
        let work = |counters: &[usize]| (counters[PROBES], counters[STEPS]);
        assert_eq!(work(&conditional), work(&plain), "depth {depth}");
        // Whole-analysis canonical steps are an observation only.
        observed.push((
            depth,
            work(&conditional),
            conditional[CANONICAL],
            plain[CANONICAL],
        ));
    }
    println!("supertrait-lookup work={observed:?}");
    Ok(())
}

/// Lookup proves a `[const]` hierarchy's conditions as its plain twin's, in
/// the same order. Removing the elaborated duplicates costs at most P·M
/// comparisons for P scope predicates and M `[const]` conditions, and none
/// without a `[const]` condition.
#[test]
fn const_supertrait_lookup_conditions_equal_the_plain_twin() -> TestResult {
    let foundation = crate::test_source::fixture(FOUNDATION)?;
    let mut charged = Vec::new();
    for depth in DEPTHS {
        for form in BOTH {
            let conditional = conditions(&foundation, depth, true, form)?;
            let plain = conditions(&foundation, depth, false, form)?;
            assert_eq!(conditional.kept, plain.kept, "depth {depth}: {form}");
            assert_eq!(conditional.kept.len(), depth + 2, "depth {depth}: {form}");
            assert_eq!(conditional.mapped, depth, "depth {depth}: {form}");
            assert!(
                conditional.charge <= conditional.scope * conditional.mapped,
                "depth {depth}: {form}: charge {}",
                conditional.charge
            );
            assert_eq!(plain.rows, plain.kept.len(), "depth {depth}: {form}");
            assert_eq!(
                (plain.mapped, plain.charge),
                (0, 0),
                "depth {depth}: {form}"
            );
            charged.push((
                depth,
                conditional.charge,
                conditional.scope * conditional.mapped,
            ));
        }
    }
    println!("supertrait-lookup charges={charged:?}");
    Ok(())
}

/// Every method selection and body work counter of the plain twin, which has
/// no `[const]` condition; the batch compares these lines with the base's.
#[test]
fn plain_supertrait_lookup_counters() -> TestResult {
    let foundation = crate::test_source::fixture(FOUNDATION)?;
    for depth in DEPTHS {
        let source = hierarchy(&foundation, depth, false, &BOTH)?;
        println!(
            "supertrait-lookup plain counters depth={depth} {:?}",
            counters(&source)?
        );
    }
    Ok(())
}

/// One lookup's trace in one hierarchy: its rows, kept mapped conditions in
/// order, the scope's size and `[const]` count, and the charged comparisons.
struct Lookup {
    rows: usize,
    kept: Vec<String>,
    scope: usize,
    mapped: usize,
    charge: usize,
}

fn conditions(foundation: &str, depth: usize, conditional: bool, form: &str) -> TestResult<Lookup> {
    let source = hierarchy(foundation, depth, conditional, &[form])?;
    let report = crate::test_source::source_provider(&source, "supertrait_lookup_conditions")?;
    let capture = crate::sema::copy_trace::Capture::matching(&["supertrait-lookup "])?;
    crate::pipeline::analyze_source(&report)
        .map_err(|error| format!("depth {depth}, conditional {conditional}: {error}"))?;
    let rows = capture.rows()?;
    let mut lookup = Lookup {
        rows: rows.len(),
        kept: Vec::new(),
        scope: 0,
        mapped: 0,
        charge: 0,
    };
    for row in &rows {
        let mut fields = row.splitn(6, ' ').skip(1);
        let mut field = |name: &str| {
            fields
                .next()
                .and_then(|field| field.strip_prefix(name))
                .ok_or_else(|| format!("malformed row {row}"))
        };
        let scope: usize = field("scope=")?.parse()?;
        let mapped: usize = field("conditional=")?.parse()?;
        lookup.charge += field("charge=")?.parse::<usize>()?;
        let kept: bool = field("kept=")?.parse()?;
        let predicate = field("")?;
        // A single call: one row per scope predicate, all with one scope.
        assert_eq!(rows.len(), scope, "{row}");
        assert!(lookup.scope == 0 || lookup.mapped == mapped, "{row}");
        (lookup.scope, lookup.mapped) = (scope, mapped);
        if kept {
            lookup.kept.push(predicate.to_owned());
        }
    }
    Ok(lookup)
}

/// The probe hierarchy through `depth` supertraits with the given lookups.
fn hierarchy(
    foundation: &str,
    depth: usize,
    conditional: bool,
    forms: &[&str],
) -> TestResult<String> {
    let (keyword, effect) = if conditional {
        ("const trait", "[const] ")
    } else {
        ("trait", "")
    };
    let mut source = foundation.to_owned();
    let mut supertraits = String::new();
    for index in 0..depth {
        writeln!(source, "pub {keyword} S{index}: PointeeSized {{}}")?;
        writeln!(source, "impl<T: PointeeSized> S{index} for *const T {{}}")?;
        write!(supertraits, "{effect}S{index} + ")?;
    }
    writeln!(
        source,
        "pub {keyword} Cmp: {supertraits}PointeeSized {{ fn cmp(&self, other: &Self) -> u8; }}"
    )?;
    source.push_str(
        "impl<T: PointeeSized> Cmp for *const T { fn cmp(&self, _other: &*const T) -> u8 { 0 } }\n\
         pub struct V;\n",
    );
    for form in forms {
        source.push_str(form);
    }
    Ok(source)
}

/// The six method selection counters, then the twenty body work counters.
fn counters(source: &str) -> TestResult<Vec<usize>> {
    let report = crate::test_source::source_provider(source, "supertrait_lookup_work")?;
    let analysis = crate::pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    let methods = &analysis.facts.methods;
    let work = &analysis.facts.body_work;
    Ok(vec![
        methods.overlap_pairs,
        methods.overlap_steps,
        methods.selection_probes,
        methods.selection_steps,
        methods.selection_shape_steps,
        methods.trait_visibility_steps,
        work.cfg_functions.get(),
        work.cfg_expressions.get(),
        work.cfg_inline_expressions.get(),
        work.cfg_operand_visits.get(),
        work.unsizing_inputs,
        work.unsizing_fields,
        work.inference_steps,
        work.canonical_steps,
        work.trait_selections,
        work.storage_waits,
        work.storage_definition_waits,
        work.storage_selections,
        work.storage_steps,
        work.projection_waits,
        work.projection_selections,
        work.projection_steps,
        work.steps,
        work.suspensions,
        work.suspended_slots,
        work.peak_suspended_slots,
    ])
}
