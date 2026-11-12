//! Branches share one reference type as rustc's `CoerceMany` does: each branch
//! coerces to a known slice expectation, and without one an earlier array
//! referent unsizes to a later slice (core's `Split::as_slice`, `next_back`).
use crate::native_references as native;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/branch-unsizing";
const LATER: &str = "pub fn f<'a, T>(c: bool, v: &'a [T]) -> &'a [T] {\n    let r = if c { &[] } else { v };\n    r\n}\n";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "branches")?;
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
            "minicore@branches",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "branches",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[test]
fn branch_references_unsize_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("branch columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid branch outcome".into()),
        };
        let (types, stderr) = typed(&root.join(path))?;
        let matches = if types {
            // Body construction applies each branch's recorded unsizing.
            let lowered = program(&crate::resolve::fixture(path)?);
            if let Err(error) = &lowered {
                failures.push(format!("{path}: lowering: {error}"));
            }
            accepted && lowered.is_ok()
        } else {
            !accepted && stderr.contains("incompatible types")
        };
        if !matches {
            failures.push(format!("{path}: types={types} {stderr}"));
        }
        count += 1;
    }
    assert_eq!(count, 14);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `calls` functions, each joining an empty array branch with a slice.
fn width(prefix: &str, calls: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "pub fn f{index}<'a, T>(c: bool, v: &'a [T]) -> &'a [T] {{ let r = if c {{ &[] }} else {{ v }}; r }}"
        )?;
    }
    Ok(text)
}

#[test]
fn branch_unsizing_work_tracks_joins() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/lub-later.rs"))?;
    let prefix = source.strip_suffix(LATER).ok_or("branch join suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for calls in [8, 32, 128] {
        let report = crate::resolve::source_provider(&width(prefix, calls)?, "branches")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("branch joins calls={calls} work={work:?}");
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

/// Each empty branch carries length 0 at run time; the selected slice keeps its own.
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn branch_unsizing_keeps_each_branch_length() -> Result {
    native::check_cases(
        "pa31-native/branch-unsizing",
        &[
            (
                "expected",
                "fn pick<'a>(c:bool,v:&'a [u8])->&'a [u8]{if c{&[]}else{v}}fn main(){let a=[3u8,5];match pick(true,&a){[]=>(),_=>check(false)}check(pick(false,&a)[1]==5);}",
                false,
            ),
            (
                "later-slice",
                "fn pick<'a>(c:bool,v:&'a [u8])->u8{let r=if c{&[]}else{v};match r{[]=>0,[a,_,_]=>*a,_=>9}}fn main(){let a=[3u8,5,7];check(pick(true,&a)==0);check(pick(false,&a)==3);}",
                false,
            ),
            (
                "match-arms",
                "fn pick<'a>(k:u8,v:&'a [u8])->u8{let r=match k{0=>&[],1=>v,_=>&[9,8]};match r{[]=>0,[a]=>*a,[a,b]=>*a+*b,_=>99}}fn main(){let a=[3u8];check(pick(0,&a)==0);check(pick(1,&a)==3);check(pick(2,&a)==17);}",
                false,
            ),
            (
                "field-deref",
                "struct Split<'a>{v:&'a [u8],finished:bool}fn as_slice<'a>(s:&Split<'a>)->&'a [u8]{if s.finished{&[]}else{&s.v}}fn main(){let a=[3u8,5];match as_slice(&Split{v:&a,finished:true}){[]=>(),_=>check(false)}check(as_slice(&Split{v:&a,finished:false})[1]==5);}",
                false,
            ),
        ],
    )
}
