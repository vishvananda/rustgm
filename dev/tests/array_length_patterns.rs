//! An array pattern without a rest fixes an inferred array length, as rustc's
//! `check_array_pat_len` does (core's `array_windows().all(|[a, b]| ...)`).
//! A rest, a generic length or a disagreeing known length still rejects.
use crate::native_references as native;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/array-length-patterns";
const CALL: &str = "pub fn g() -> usize {\n    call(|[a, b]| *b)\n}\n";
const UNKNOWN: &str = "array pattern requires a known fixed-array type";
const DISAGREES: &str = "array pattern length disagrees with its type";
/// Typing's reason for each rustc rejection (E0730, E0527 and E0308).
const REJECTED: [(&str, &str); 4] = [
    ("rest-unknown.rs", UNKNOWN),
    ("known-mismatch.rs", DISAGREES),
    ("match-two-lengths.rs", DISAGREES),
    ("generic-len.rs", UNKNOWN),
];
/// rustc accepts these, and they type here, but prepared lowering stops at
/// the closure's call of a captured `FnMut(&'a T, &'a T)` with its parameter,
/// whose region comes from the callee's lifetime, with or without length
/// inference (the base fails the same way without an array pattern).
const LOWERING_GAPS: [&str; 2] = ["core-form.rs", "core-form-known.rs"];
const CAPTURED_CALL: &str = "required lifetime does not outlive its target";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "patterns")?;
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
            "minicore@patterns",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "patterns",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[test]
fn array_patterns_fix_inferred_lengths_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("pattern columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid pattern outcome".into()),
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
        let gap = LOWERING_GAPS
            .iter()
            .any(|name| path.ends_with(&format!("/{name}")));
        let matches = match (&outcome, reason) {
            (None, None) => accepted && !gap,
            (Some(message), None) => accepted && types && gap && message.contains(CAPTURED_CALL),
            (Some(message), Some(reason)) => !accepted && !types && message.contains(reason),
            (None, Some(_)) => false,
        };
        if !matches {
            failures.push(format!("{path}: types={types} {outcome:?}"));
        }
        count += 1;
    }
    assert_eq!(count, 13);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `calls` functions, each passing a closure whose pattern fixes `N`.
fn width(prefix: &str, calls: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(text, "pub fn g{index}() -> usize {{ call(|[a, b]| *b) }}")?;
    }
    Ok(text)
}

#[test]
fn array_length_pattern_work_tracks_closures() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/infer-len.rs"))?;
    let prefix = source.strip_suffix(CALL).ok_or("array pattern suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for calls in [8, 32, 128] {
        let report = crate::resolve::source_provider(&width(prefix, calls)?, "patterns")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("array patterns calls={calls} work={work:?}");
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

/// Each pattern's element count becomes the callee's `N` at run time.
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn array_patterns_run_with_inferred_lengths() -> Result {
    native::check_cases(
        "pa31-native/array-length-patterns",
        &[
            (
                "closure-length",
                "fn call<const N:usize,F:FnMut(&[u8;N])->u8>(mut f:F,a:&[u8;N])->(u8,usize){(f(a),N)}fn make<const N:usize>()->[u8;N]{let mut a=[0u8;N];let mut i=0;while i<N{a[i]=i as u8+4;i+=1;}a}fn main(){let (s,n)=call(|[a,b]|*a+*b,&make());check(s==9);check(n==2);}",
                false,
            ),
            (
                "let-length",
                "fn make<const N:usize>()->[u8;N]{let mut a=[0u8;N];let mut i=0;while i<N{a[i]=i as u8+1;i+=1;}a}fn main(){let [a,b,c]=make();check(a==1&&b==2&&c==3);}",
                false,
            ),
            (
                "by-value-length",
                "fn len<const N:usize,F:FnMut([u8;N])->u8>(_f:F)->usize{N}fn main(){check(len(|[_a,_b,c]|c)==3);}",
                false,
            ),
            (
                "windows",
                "fn all<const N:usize,F:FnMut(&[u8;N])->bool>(v:&[u8],mut f:F)->bool{let mut i=0;while i+N<=v.len(){let mut w=[0u8;N];let mut j=0;while j<N{w[j]=v[i+j];j+=1;}if !f(&w){return false;}i+=1;}true}fn main(){check(all(&[1u8,2,3],|[a,b]|a<=b));check(!all(&[1u8,3,2],|[a,b]|a<=b));}",
                false,
            ),
        ],
    )
}
