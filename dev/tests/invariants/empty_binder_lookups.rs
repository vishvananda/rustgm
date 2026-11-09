//! Callable sugar such as `F: FnMut(T) -> U` publishes its bound and output
//! equality inside a binder even when it names no lifetime. Method lookup
//! reads such an empty binder, whose body captures nothing outside it, as its
//! body. Census 23's record is such a lookup: core's `drain::Drain::new` in
//! `<[T; N]>::try_map`, an inherent `new` of `impl<'l, 'f, T, U, F: FnMut(T)
//! -> U>` beside the trait method `ZipImpl::new`. A binder that quantifies a
//! lifetime still requires a scoped consumer.
use super::TestResult;
use std::fmt::Write as _;

const CONTROLS: &str = "pa31/tests/spec/quantified-lookup-conditions";
const SCOPED: &str = "quantified clause requires a scoped consumer";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports. rustc accepts the six bound-region controls
/// that still fail with [`SCOPED`], known rejects-valid until lookup opens a
/// quantified lifetime; it rejects the last four (E0277 or E0599).
const CASES: [(&str, Option<&str>); 22] = [
    ("core-drain-shape", None),
    ("nonconst-caller", None),
    ("nonconst-new", None),
    ("safe-new", None),
    ("by-value", None),
    ("method-call", None),
    ("drop-impl", None),
    ("sugar-supertrait", None),
    ("trait-fallback", None),
    ("unique-inherent", None),
    ("plain-bound", None),
    ("sugar-lifetime-unique", None),
    ("explicit-binder", Some(SCOPED)),
    ("sugar-lifetime", Some(SCOPED)),
    ("split-shape", Some(SCOPED)),
    ("trait-fallback-region", Some(SCOPED)),
    ("trait-fallback-leak", Some(SCOPED)),
    ("receiver-sugar-lifetime", Some(SCOPED)),
    ("fail-inferred", Some(super::OBLIGATION)),
    (
        "fail-explicit",
        Some("no applicable trait method for this prefix"),
    ),
    ("fail-unique", Some(super::OBLIGATION)),
    ("fail-region-unique", Some(super::OBLIGATION)),
];

#[test]
fn quantified_lookup_conditions_match_declared_outcomes() -> TestResult {
    super::match_outcomes(CONTROLS, &CASES, "quantified_lookup_condition")
}

/// At most 256 rows are captured per analysis: one lookup has about 7N + 2
/// supertrait-lookup rows and 5N empty-binder rows, so N stays small, and the
/// exact affine fits over three points separate P + M from P·M.
const BOUNDS: [usize; 3] = [1, 4, 8];
/// Leading entries of [`super::counters`]: the method selection counters.
const METHODS: usize = 6;
/// Functions in each source that each perform the lookup once.
const CALLERS: usize = 2;
/// One capture per prefix, each in its own analysis of the same source.
const LOOKUPS: &[&str] = &["supertrait-lookup "];
const READS: &[&str] = &["empty-binder-lookup "];

#[derive(Clone, Copy, Debug)]
enum Form {
    /// `S::new(..)`, which reaches applicability because `ZipImpl::new` exists.
    Qualified,
    /// `h.get()` through the receiver's inherent candidate.
    MethodCall,
}

/// An inherent candidate under N sugar bounds has exactly its flat twin's
/// lookup conditions, in the same order, and the same Methods counters. Each
/// lookup that reads an empty binder pays its scope P once and one step per
/// read M, so L lookups cost L·(P+M), never L·P·M; the flat twin pays nothing.
/// Whole-analysis body work is printed only: the chosen callable's own proof
/// takes the quantified path for sugar and the flat path for the twin.
#[test]
fn sugar_lookup_conditions_equal_the_flat_twin() -> TestResult {
    let foundation = crate::test_source::fixture(super::FOUNDATION)?;
    let mut observed = Vec::new();
    for form in [Form::Qualified, Form::MethodCall] {
        let mut shapes = Vec::new();
        let mut work = Vec::new();
        for count in BOUNDS {
            let label = format!("{count} bounds, {form:?}");
            let sugar = impl_source(&foundation, count, true, form)?;
            let flat = impl_source(&foundation, count, false, form)?;
            let (reads, sugar_lookup) = trace(&sugar, &format!("{label}, sugar"))?;
            let (flat_reads, flat_lookup) = trace(&flat, &format!("{label}, flat"))?;
            assert_eq!(sugar_lookup.kept, flat_lookup.kept, "{label}");
            for lookup in [&sugar_lookup, &flat_lookup] {
                assert_eq!(lookup.rows, lookup.kept.len(), "{label}");
                assert_eq!((lookup.mapped, lookup.charge), (0, 0), "{label}");
            }
            assert!(flat_reads.is_empty(), "{label}: {flat_reads:?}");
            let lookups = reads.len();
            let each = *reads.first().ok_or_else(|| format!("{label}: no read"))?;
            assert!(lookups >= CALLERS, "{label}: {lookups} lookups");
            assert!(reads.iter().all(|r| *r == each), "{label}: {reads:?}");
            assert_eq!(each.scope, sugar_lookup.scope, "{label}");
            assert_eq!(each.charge, each.scope + each.reads, "{label}");
            let total: usize = reads.iter().map(|r| r.charge).sum();
            assert_eq!(total, lookups * (each.scope + each.reads), "{label}");
            assert!(
                count == 1 || total < lookups * each.scope * each.reads,
                "{label}"
            );
            assert_eq!(sugar_lookup.rows % lookups, 0, "{label}");
            let shape = Shape {
                lookups,
                rows: sugar_lookup.rows / lookups,
                read: each,
            };
            shapes.push((count, shape));
            let (sugar_work, flat_work) = (
                super::counters(&sugar).map_err(|error| format!("{label}, sugar: {error}"))?,
                super::counters(&flat).map_err(|error| format!("{label}, flat: {error}"))?,
            );
            assert_eq!(sugar_work[..METHODS], flat_work[..METHODS], "{label}");
            work.push((
                count,
                total,
                sugar_work[METHODS..].to_vec(),
                flat_work[METHODS..].to_vec(),
            ));
        }
        let per_bound = Shape::per_bound(&shapes, form)?;
        observed.push((form, per_bound, work));
    }
    println!("empty-binder-lookup {observed:?}");
    Ok(())
}

/// One lookup's reads of empty-binder conditions: its scope's size P, the
/// conditions read M, those skipped as equal to a flat clause, and the charge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Read {
    scope: usize,
    reads: usize,
    skipped: usize,
    charge: usize,
}

/// One N's lookups: their count, supertrait-lookup rows per lookup and reads.
#[derive(Clone, Copy, Debug)]
struct Shape {
    lookups: usize,
    rows: usize,
    read: Read,
}

impl Shape {
    /// Lookups do not depend on N; rows and the scope grow by a constant per
    /// bound, at least the two written clauses; every read and skip belongs
    /// to a bound. Returns each as `(per bound, constant)`.
    fn per_bound(shapes: &[(usize, Self)], form: Form) -> TestResult<[(usize, usize); 4]> {
        let lookups = shapes.first().map(|(_, shape)| shape.lookups);
        assert!(
            shapes
                .iter()
                .all(|(_, shape)| Some(shape.lookups) == lookups),
            "{form:?}: {shapes:?}"
        );
        let fit = |value: fn(&Self) -> usize| {
            affine(shapes, value).map_err(|error| format!("{form:?}: {error}"))
        };
        let rows = fit(|shape| shape.rows)?;
        let scope = fit(|shape| shape.read.scope)?;
        let reads = fit(|shape| shape.read.reads)?;
        let skipped = fit(|shape| shape.read.skipped)?;
        assert!(rows.0 >= 2, "{form:?}: {shapes:?}");
        assert!(reads.0 >= 2 && reads.1 == 0, "{form:?}: {shapes:?}");
        assert_eq!(skipped.1, 0, "{form:?}: {shapes:?}");
        Ok([rows, scope, reads, skipped])
    }
}

/// `value(N) = a·N + b` through every point; returns `(a, b)`.
fn affine(
    shapes: &[(usize, Shape)],
    value: impl Fn(&Shape) -> usize,
) -> TestResult<(usize, usize)> {
    let [(first, one), (second, two), ..] = shapes else {
        return Err("affine fit needs two points".into());
    };
    let rise = value(two)
        .checked_sub(value(one))
        .ok_or("value decreases with N")?;
    let run = second - first;
    assert_eq!(rise % run, 0, "{shapes:?}");
    let slope = rise / run;
    let constant = value(one)
        .checked_sub(slope * first)
        .ok_or("negative constant")?;
    for (count, shape) in shapes {
        assert_eq!(value(shape), slope * count + constant, "{shapes:?}");
    }
    Ok((slope, constant))
}

/// One source's empty-binder reads, one entry per lookup, and its
/// supertrait-lookup rows, each from its own analysis. Every error names the
/// source by `label`: its N, form and side.
fn trace(source: &str, label: &str) -> TestResult<(Vec<Read>, super::Lookup)> {
    let labelled = |error: Box<dyn std::error::Error>| format!("{label}: {error}");
    let lookup = super::parse_lookup(&super::traced(source, label, LOOKUPS)?).map_err(labelled)?;
    let reads = lookup_reads(&super::traced(source, label, READS)?).map_err(labelled)?;
    Ok((reads, lookup))
}

/// Splits empty-binder rows into lookups. A lookup's first read collects the
/// flat clauses and charges P + 1; every later read charges 1.
fn lookup_reads(rows: &[String]) -> TestResult<Vec<Read>> {
    let mut reads: Vec<Read> = Vec::new();
    for row in rows {
        let mut fields = row.splitn(5, ' ').skip(1);
        let mut field = |name: &str| {
            fields
                .next()
                .and_then(|field| field.strip_prefix(name))
                .ok_or_else(|| format!("malformed row {row}"))
        };
        let scope: usize = field("scope=")?.parse()?;
        let charge: usize = field("charge=")?.parse()?;
        let kept: bool = field("kept=")?.parse()?;
        if charge == scope + 1 {
            reads.push(Read {
                scope,
                ..Read::default()
            });
        } else {
            assert_eq!(charge, 1, "{row}");
        }
        let current = reads
            .last_mut()
            .ok_or_else(|| format!("read before its lookup's flat clauses: {row}"))?;
        assert_eq!(current.scope, scope, "{row}");
        current.reads += 1;
        current.skipped += usize::from(!kept);
        current.charge += charge;
    }
    Ok(reads)
}

/// `S<T, F0, ..>` with one inherent impl over `count` callable bounds, sugar
/// `Fi: FnMut(T) -> Ui` or its flat twin `Fi: FnMut<(T,), Output = Ui>`, and
/// [`CALLERS`] lookups whose callers write the same sugar premises in both
/// sources.
fn impl_source(foundation: &str, count: usize, sugar: bool, form: Form) -> TestResult<String> {
    let mut source = foundation.to_owned();
    source.push_str("pub trait ZipImpl { fn new() -> Self; }\n");
    let [mut parameters, mut fields, mut bounds, mut values]: [String; 4] = Default::default();
    let [mut premises, mut callables, mut arguments, mut passed]: [String; 4] = Default::default();
    for index in 0..count {
        write!(parameters, ", F{index}")?;
        write!(fields, ", f{index}: F{index}")?;
        if sugar {
            write!(bounds, ", U{index}, F{index}: FnMut(T) -> U{index}")?;
        } else {
            write!(
                bounds,
                ", U{index}, F{index}: FnMut<(T,), Output = U{index}>"
            )?;
        }
        write!(values, ", f{index}")?;
        write!(premises, ", R{index}, G{index}: FnMut(T) -> R{index}")?;
        write!(callables, ", G{index}")?;
        write!(arguments, ", g{index}: G{index}")?;
        write!(passed, ", g{index}")?;
    }
    writeln!(source, "pub struct S<T{parameters}> {{ a: T{fields} }}")?;
    writeln!(
        source,
        "impl<T{bounds}> S<T{parameters}> {{\n    \
         pub fn new(a: T{fields}) -> Self {{ S {{ a{values} }} }}\n    \
         pub fn get(&self) -> u8 {{ 0 }}\n}}"
    )?;
    for caller in 0..CALLERS {
        match form {
            Form::Qualified => writeln!(
                source,
                "pub fn c{caller}<T{premises}>(a: T{arguments}) -> u8 {{ let _s = S::new(a{passed}); 0 }}"
            )?,
            Form::MethodCall => writeln!(
                source,
                "pub fn g{caller}<T{premises}>(h: S<T{callables}>) -> u8 {{ h.get() }}"
            )?,
        }
    }
    Ok(source)
}
