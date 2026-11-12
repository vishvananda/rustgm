//! A pattern matched against a captured place captures it as rustc's closure
//! capture analysis walks the pattern: core's `Duration` `fmt_decimal` closure
//! tests `if let Some(integer_part) = integer_part` on a captured `Option<u64>`.
use crate::native_references as native;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/closure-capture-patterns";
const MATCH: &str = "pub fn g(part: Opt<u8>) -> u8 {\n    let c = || match part {\n        Opt::Yes(x) => x,\n        Opt::No => 0,\n    };\n    c()\n}\n";
const MOVED: &str = "local may be uninitialized or moved at this read";
const LOAN: &str = "access conflicts with a live reference loan";
const PRECISE: &str = "precise capture patterns are not implemented";
/// Prepared lowering rejects the borrow conflicts rustc reports (E0382, E0505,
/// E0502). The recorded gaps reject what rustc accepts: an uncaptured
/// scrutinee has no lowering place, and a capture more precise than the
/// matched place fails closed at typing.
const REJECTED: [(&str, &str, bool); 11] = [
    ("match-move-twice.rs", MOVED, true),
    ("move-then-use.rs", MOVED, true),
    ("shared-conflict.rs", LOAN, true),
    ("ref-mut-conflict.rs", LOAN, true),
    ("discriminant-conflict.rs", LOAN, true),
    ("at-binding-twice.rs", MOVED, true),
    ("let-else-move-then-use.rs", MOVED, true),
    ("legacy-tuple.rs", MOVED, true),
    ("wildcard.rs", "missing match input place", true),
    ("let-tuple-precise.rs", PRECISE, false),
    ("ref-scrutinee.rs", PRECISE, false),
];
const GAPS: [&str; 3] = ["wildcard.rs", "let-tuple-precise.rs", "ref-scrutinee.rs"];

fn edition(name: &str) -> Result<rustgm::support::edition::Edition> {
    use rustgm::support::edition::Edition;
    Ok(match name {
        "2018" => Edition::Rust2018,
        "2024" => Edition::Rust2024,
        _ => return Err("invalid capture pattern edition".into()),
    })
}

fn program(source: &str, edition: &str) -> Result<rustgm::lowir::program::Program> {
    let config = rustgm::expand::config::Config::in_edition(self::edition(edition)?);
    let report = crate::resolve::source_provider_config(source, "captures", &config)?;
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
fn typed(path: &std::path::Path, edition: &str) -> Result<(bool, String)> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-types",
            &format!("--edition={edition}"),
            "--cap-lints=allow",
            "--source-core",
            "minicore@captures",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "captures",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

#[test]
fn matched_captured_places_capture_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let mut columns = line.split('\t');
        let (Some(path), Some(expected), Some(edition), None) = (
            columns.next(),
            columns.next(),
            columns.next(),
            columns.next(),
        ) else {
            return Err("capture pattern columns".into());
        };
        let name = path.rsplit('/').next().ok_or("capture pattern name")?;
        let accepted = match expected {
            "0" => !GAPS.contains(&name),
            "1" => false,
            _ => return Err("invalid capture pattern outcome".into()),
        };
        let reason = REJECTED
            .iter()
            .find(|(case, ..)| *case == name)
            .map(|&(_, reason, types)| (reason, types));
        let (types, stderr) = typed(&root.join(path), edition)?;
        let outcome = if types {
            program(&crate::resolve::fixture(path)?, edition)
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
    assert_eq!(count, 29);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` closures, each matching its creator's captured `Opt<u8>`.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "pub fn g{index}(part: Opt<u8>) -> u8 {{ let c = || match part {{ Opt::Yes(x) => x, Opt::No => 0 }}; c() }}"
        )?;
    }
    Ok(text)
}

/// Typing work stays linear, and every width completes prepared lowering.
#[test]
fn capture_pattern_work_tracks_closures() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/match-copy.rs"))?;
    let prefix = source.strip_suffix(MATCH).ok_or("capture match suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "captures")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("capture patterns functions={functions} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "functions={functions}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        program(&text, "2024")?;
    }
    Ok(())
}

/// Each closure matches a captured place and the program observes the
/// selected arm, the moved payload or the mutation.
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn capture_patterns_run_their_matches() -> Result {
    const OPT: &str = "enum Opt<T>{No,Yes(T)}struct Owned(u8);";
    let cases = [
        (
            "fmt-decimal",
            format!(
                "{OPT}fn emit(part:Opt<u64>,out:&mut u64)->u8{{let mut n=0u8;let mut emit_without_padding=|f:&mut u64|{{if let Opt::Yes(integer_part)=part{{*f=*f*10+integer_part;}}n+=1;}};emit_without_padding(out);emit_without_padding(out);n}}fn main(){{let mut out=0u64;check(emit(Opt::Yes(3),&mut out)==2);check(out==33);let mut none=0u64;check(emit(Opt::No,&mut none)==2);check(none==0);let _=Owned(0);}}"
            ),
        ),
        (
            "move-payload",
            format!(
                "{OPT}fn take(part:Opt<Owned>)->u8{{let c=||match part{{Opt::Yes(x)=>x,Opt::No=>Owned(9)}};let Owned(v)=c();v}}fn main(){{check(take(Opt::Yes(Owned(4)))==4);check(take(Opt::No)==9);}}"
            ),
        ),
        (
            "ref-mut",
            format!(
                "{OPT}fn main(){{let mut part=Opt::Yes(1u8);{{let mut c=||match part{{Opt::Yes(ref mut x)=>*x+=4,Opt::No=>{{}}}};c();c();}}if let Opt::Yes(v)=part{{check(v==9);}}else{{check(false);}}let _:Opt<Owned>=Opt::No;}}"
            ),
        ),
        (
            "let-else-while-let",
            format!(
                "{OPT}fn main(){{let part=Opt::Yes(5u8);let c=||{{let Opt::Yes(x)=part else{{return 0u8;}};let mut n=0u8;while let Opt::Yes(y)=part{{n+=y;if n>9{{break;}}}}x+n}};check(c()==15);let none:Opt<u8>=Opt::No;let d=||{{let Opt::Yes(x)=none else{{return 7u8;}};x}};check(d()==7);let _=Owned(0);}}"
            ),
        ),
        (
            "array-and-at",
            format!(
                "{OPT}fn main(){{let w=[3u8,4];let c=||{{let [a,b]=w;a*b}};check(c()==12);check(c()==12);let part=Opt::Yes(Owned(6));let d=||match part{{whole@Opt::Yes(_)=>whole,Opt::No=>Opt::No}};if let Opt::Yes(Owned(v))=d(){{check(v==6);}}else{{check(false);}}}}"
            ),
        ),
        (
            "guard-or-nested",
            format!(
                "{OPT}fn keep(x:u8)->bool{{x>1}}fn main(){{let part=Opt::Yes(Opt::Yes(3u8));let c=||match part{{Opt::Yes(Opt::Yes(x)) if keep(x)=>x,Opt::Yes(Opt::No)|Opt::No=>1,_=>0}};check(c()==3);let q=Opt::Yes(1u8);let d=||match q{{Opt::Yes(1)|Opt::No=>5u8,_=>0}};check(d()==5);let _=Owned(0);}}"
            ),
        ),
    ];
    let cases: Vec<_> = cases
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect();
    native::check_cases("pa31-native/closure-capture-patterns", &cases)
}
