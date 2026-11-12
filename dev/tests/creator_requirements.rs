//! A closure's requirements on regions and type parameters that its creator
//! supplies are the creator's to prove, as rustc propagates closure
//! requirements: core's `is_sorted_by` closure `|[a, b]| compare(a, b)` passes
//! callee-region parameters to a captured `FnMut(&'a T, &'a T)`.
use crate::native_references as native;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/closure-creator-requirements";
const CALL: &str = "pub fn h<'a, T, F: FnMut(&'a T, &'a T) -> bool>(x: &'a T, mut compare: F) -> bool {\n    take1(x, |a| compare(a, a))\n}\n";
const OUTLIVES: &str = "required lifetime does not outlive its target";
/// Prepared lowering rejects the two creators that cannot prove a closure's
/// requirement (E0521, E0310); typing rejects the unimplied where clause (E0309).
const REJECTED: [(&str, &str, bool); 3] = [
    ("static-region.rs", OUTLIVES, true),
    ("static-type.rs", OUTLIVES, true),
    (
        "where-unimplied.rs",
        "trait argument lifetime requirement is not satisfied",
        false,
    ),
];

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "creators")?;
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
            "minicore@creators",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "creators",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[test]
fn closure_requirements_reach_their_creator_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("creator columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid creator outcome".into()),
        };
        let reason = REJECTED
            .iter()
            .find(|(name, ..)| path.ends_with(&format!("/{name}")))
            .map(|&(_, reason, types)| (reason, types));
        let (types, stderr) = typed(&root.join(path))?;
        let outcome = if types {
            program(&crate::resolve::fixture(path)?)
                .err()
                .map(|error| error.to_string())
        } else {
            Some(stderr)
        };
        let matches = match (&outcome, reason) {
            (None, None) => accepted,
            (Some(message), Some((reason, typing))) => {
                !accepted && types == typing && message.contains(reason)
            }
            _ => false,
        };
        if !matches {
            failures.push(format!("{path}: types={types} {outcome:?}"));
        }
        count += 1;
    }
    assert_eq!(count, 11);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `calls` creators, each passing a closure that calls its captured `FnMut`.
fn width(prefix: &str, calls: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "pub fn h{index}<'a, T, F: FnMut(&'a T, &'a T) -> bool>(x: &'a T, mut compare: F) -> bool {{ take1(x, |a| compare(a, a)) }}"
        )?;
    }
    Ok(text)
}

/// Typing work stays linear, and every width completes prepared lowering.
#[test]
fn creator_requirement_work_tracks_closures() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/captured-call.rs"))?;
    let prefix = source.strip_suffix(CALL).ok_or("creator call suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for calls in [8, 32, 128] {
        let text = width(prefix, calls)?;
        let report = crate::resolve::source_provider(&text, "creators")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("creator requirements calls={calls} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "calls={calls}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        program(&text)?;
    }
    Ok(())
}

/// Each closure calls its creator's comparison with the creator's own values.
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn creator_requirements_run_their_captured_calls() -> Result {
    const TAKE: &str = "fn take1<'a,T,G:FnMut(&'a T)->bool>(x:&'a T,mut f:G)->bool{f(x)}";
    let cases = [
        (
            "captured-fnmut",
            format!("{TAKE}fn h<'a,T,F:FnMut(&'a T,&'a T)->bool>(x:&'a T,mut compare:F)->bool{{take1(x,|a|compare(a,a))}}fn main(){{let v=5u8;check(h(&v,|a,b|*a==*b));check(!h(&v,|a,b|*a!=*b));}}"),
        ),
        (
            "windows",
            "fn all<'a,T,const N:usize,F:FnMut(&'a [T;N])->bool>(w:&'a [[T;N]],mut f:F)->bool{let mut i=0;while i<w.len(){if !f(&w[i]){return false;}i+=1;}true}fn is_sorted_by<'a,T,F:FnMut(&'a T,&'a T)->bool>(w:&'a [[T;2]],mut compare:F)->bool{all(w,|[a,b]|compare(a,b))}fn main(){check(is_sorted_by(&[[1u8,2],[2,3]],|a,b|a<=b));check(!is_sorted_by(&[[1u8,2],[3,2]],|a,b|a<=b));}".to_owned(),
        ),
        (
            "nested",
            format!("{TAKE}fn h<'a,T,F:FnMut(&'a T,&'a T)->bool>(x:&'a T,mut compare:F)->bool{{take1(x,|a|take1(a,|b|compare(b,b)))}}fn main(){{let v=7u8;check(h(&v,|a,b|*a==*b));}}"),
        ),
        (
            "counted",
            format!("{TAKE}fn h<'a,T,F:FnMut(&'a T,&'a T)->bool>(x:&'a T,mut compare:F)->bool{{take1(x,|a|compare(a,a))&&take1(x,|a|compare(a,a))}}fn main(){{let v=1u8;let mut n=0u8;check(h(&v,|a,b|{{n+=1;*a==*b}}));check(n==2);}}"),
        ),
    ];
    let cases: Vec<_> = cases
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect();
    native::check_cases("pa31-native/closure-creator-requirements", &cases)
}
