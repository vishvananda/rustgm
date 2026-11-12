//! A closure without an expected signature takes its parameter types from its
//! body and later uses, as core's `partial_compare` does. A region its body
//! gives a parameter is external to the body, as in rustc, so the body's
//! requirements on it reach the enclosing body's lifetime proof.
use crate::native_references as native;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/closure-input-regions";
const LATER: &str = "pub fn g<A: Cmp>(x: &A, y: &A) -> u8 {\n    let c = |a, b| Cmp::cmp2(a, b);\n    take(x, y, c)\n}\n";
/// Typing rejects the non-const impl; prepared lowering's lifetime proof
/// rejects the escaping parameter region.
const REJECTED: [(&str, &str); 2] = [
    (
        "const-nonconst-impl.rs",
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
    (
        "static-bound.rs",
        "required lifetime does not outlive its target",
    ),
];

/// rustc accepts these, and they type here, but const closure lowering stops
/// first with or without parameter annotations (`const-typed-fn`,
/// `const-typed-once`). A `[const] Fn` call by reference in `take` is not yet a
/// const invocation, and a const closure body's own call needs a function owner.
const LOWERING_GAPS: [(&str, &str); 6] = [
    ("const-later-sig.rs", CONST_FN_CALL),
    ("const-later-sig-match.rs", CONST_FN_CALL),
    ("const-concrete.rs", CONST_FN_CALL),
    ("const-typed-fn.rs", CONST_FN_CALL),
    ("const-later-once.rs", CONST_CLOSURE_OWNER),
    ("const-typed-once.rs", CONST_CLOSURE_OWNER),
];
const CONST_FN_CALL: &str = "const invocation differs from its defining call-trait or effect fact";
const CONST_CLOSURE_OWNER: &str = "const call operators require their actual source function owner";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "closures")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}

/// The CLI's typing status and its stderr.
fn typed(path: &std::path::Path) -> Result<(bool, String)> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-types",
            "--edition=2024",
            "--cap-lints=allow",
            "--source-core",
            "minicore@closures",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "closures",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[test]
fn closure_input_regions_match_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("closure columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid closure outcome".into()),
        };
        let reason = REJECTED
            .iter()
            .find(|(name, _)| path.ends_with(&format!("/{name}")))
            .map(|(_, reason)| *reason);
        let (types, stderr) = typed(&root.join(path))?;
        let outcome = if types {
            program(&crate::resolve::fixture(path)?)
                .err()
                .map(|error| error.to_string())
        } else {
            Some(stderr)
        };
        println!("{path}: {outcome:?}");
        let gap = LOWERING_GAPS
            .iter()
            .find(|(name, _)| path.ends_with(&format!("/{name}")))
            .map(|(_, reason)| *reason);
        let matches = match (&outcome, reason, gap) {
            (Some(message), None, Some(gap)) => accepted && types && message.contains(gap),
            (None, None, None) => accepted,
            (Some(message), Some(reason), None) => !accepted && message.contains(reason),
            _ => false,
        };
        if !matches {
            failures.push(format!("{path}: {outcome:?}"));
        }
        count += 1;
    }
    assert_eq!(count, 13);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `calls` functions, each passing an unannotated closure to a later bound.
fn width(prefix: &str, calls: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "pub fn g{index}<A: Cmp>(x: &A, y: &A) -> u8 {{ let c = |a, b| Cmp::cmp2(a, b); take(x, y, c) }}"
        )?;
    }
    Ok(text)
}

#[test]
fn closure_input_region_work_tracks_closures() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/later-sig.rs"))?;
    let prefix = source.strip_suffix(LATER).ok_or("closure suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for calls in [8, 32, 128] {
        let report = crate::resolve::source_provider(&width(prefix, calls)?, "closures")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("closure inputs calls={calls} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "calls={calls}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
    }
    Ok(())
}

/// Closures whose parameter types come from a later bound run as rustc's do.
/// The same closure inside an instantiated generic function is a recorded gap:
/// its input port slots follow the public call binder that instantiation
/// reconstructs ("closure signature lifetime interface differs from its body").
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn unannotated_closures_run_with_inferred_inputs() -> Result {
    native::check_cases(
        "pa31-native/closure-input-regions",
        &[
            (
                "path-call",
                "fn add(a:&u8,b:&u8)->u8{*a+*b}fn take<'l,F:Fn(&'l u8,&'l u8)->u8>(x:&'l u8,y:&'l u8,f:F)->u8{f(x,y)}fn main(){let c=|a,b|add(a,b);check(take(&3,&4,c)==7);}",
                false,
            ),
            (
                "called-twice",
                "fn add(a:&u8,b:&u8)->u8{*a+*b}fn take<'l,F:Fn(&'l u8,&'l u8)->u8>(x:&'l u8,y:&'l u8,f:&F)->u8{f(x,y)}fn main(){let c=|a,b|add(a,b);let (p,q)=(1u8,2u8);check(take(&p,&q,&c)==3);check(take(&q,&q,&c)==4);}",
                false,
            ),
        ],
    )
}
