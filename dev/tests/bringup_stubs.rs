//! The bring-up profile stubs exactly its listed functions and stays explicit.
use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    process::{Command, Output},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const SOURCE: &str = "\
fn kept(x: i32) -> i32 {
    x + 1
}
fn skipped(x: i32) -> i32 {
    x * 2
}
fn main() {
    let a = kept(1);
    let b = skipped(2);
    let _ = a + b;
}
";

/// Two functions whose ownership only the borrow stage rejects, and one it accepts.
const REJECTED: &str = "\
struct Token;
fn consume(_: Token) {}
fn moved_twice() {
    let token = Token;
    consume(token);
    consume(token);
}
fn stolen(token: &Token) {
    consume(*token);
}
fn accepted(x: i32) -> i32 {
    x & 3
}
fn main() {
    moved_twice();
    stolen(&Token);
    let _ = accepted(1);
}
";

/// Three functions that fail only a lifetime, loan or const-cleanup proof:
/// each body still means what it says without that proof.
const UNPROVEN: &str = "\
struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {}
}
fn shortened<'a, 'b>(first: &'a i32, _second: &'b i32) -> &'b i32 {
    first
}
fn conflicted(mut value: i32) -> i32 {
    let held = &value;
    value |= 4;
    *held
}
const fn discarded() {
    let _guard = Guard;
}
fn main() {
    let value = conflicted(1);
    let _ = shortened(&value, &value);
    discarded();
}
";

/// A source-core library whose publication meets one waivable fence.
const FENCED: &str = "\
#![feature(no_core,lang_items,fundamental)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang=\"pointee_sized\"]pub trait PointeeSized{}
#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}
#[lang=\"sized\"]pub trait Sized:MetaSized{}
#[lang=\"legacy_receiver\"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang=\"clone\"]pub trait Clone:Sized{#[lang=\"clone_fn\"]fn clone(&self)->Self;}
#[lang=\"copy\"]pub trait Copy:Clone{}
#[fundamental]pub struct Wrap;
pub fn kept(x: u8) -> u8 { x }
";
const FENCE: &str =
    "general fundamental policy requires authenticated trait/nominal metadata transport";

/// A constant that cannot be evaluated, demanded by exactly one function.
const DEMANDED: &str = "\
fn demanded() -> usize {
    const { 1usize - 2 }
}
fn kept(x: i32) -> i32 {
    x & 3
}
fn main() {
    let _ = kept(1);
    let _ = demanded();
}
";
const GUARD: &str =
    "compiler assertion requires checked source panic signature, caller and runtime routing";

/// Compile `source` as a source-core library named by `identity`; with
/// `publish`, in the source profile that writes `probe.rgm` into the folder.
fn library(
    name: &str,
    source: &str,
    identity: &str,
    list: Option<&str>,
    publish: bool,
) -> Result<(Output, PathBuf)> {
    let folder = folder_with(name, source)?;
    let provider = format!("probe@{identity}");
    let metadata = folder.join("probe.rgm");
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args(["--emit-lowir", "--cap-lints=allow", "--edition=2024"])
        .args(["--crate-type=lib", "--crate-name", "probe"])
        .args([
            "--crate-disambiguator",
            identity,
            "--source-core",
            &provider,
        ]);
    if publish {
        command
            .args(["--source-profile", "module-body-operations-v14"])
            .arg("--metadata-output")
            .arg(&metadata);
    }
    if let Some(list) = list {
        let path = folder.join("skip.tsv");
        std::fs::write(&path, list)?;
        command.arg("--bringup-skip-list").arg(path);
    }
    Ok((command.arg(folder.join("main.rs")).output()?, metadata))
}
fn waived(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter_map(|line| line.strip_prefix("bringup-waived\t"))
        .collect()
}

fn folder(name: &str) -> Result<PathBuf> {
    folder_with(name, SOURCE)
}
fn folder_with(name: &str, source: &str) -> Result<PathBuf> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/bringup-stubs")
        .join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("main.rs"), source)?;
    Ok(folder)
}
fn run(folder: &Path, lists: &[&str], extra: &[&str]) -> Result<Output> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command.arg("--emit-lowir").args(extra);
    for (index, list) in lists.iter().enumerate() {
        let path = folder.join(format!("skip{index}.tsv"));
        std::fs::write(&path, list)?;
        command.arg("--bringup-skip-list").arg(path);
    }
    Ok(command.arg(folder.join("main.rs")).output()?)
}
fn text(output: &Output) -> Result<&str> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(std::str::from_utf8(&output.stdout)?)
}

#[test]
fn a_listed_function_traps_and_other_lists_change_nothing() -> Result {
    let folder = folder("trap")?;
    let strict = run(&folder, &[], &[])?;
    let strict = text(&strict)?;
    assert!(strict.contains("binary mul") && strict.contains("binary add"));
    let offset = SOURCE
        .find("fn skipped")
        .ok_or("fixture has no skipped function")?;
    for unchanged in [
        String::new(),
        "# only a note\n\n".into(),
        format!("main.rs\t{}\tnothing\n", offset + 1),
        format!("other.rs\t{offset}\tskipped\n"),
    ] {
        assert_eq!(text(&run(&folder, &[&unchanged], &[])?)?, strict);
    }
    let list = format!("# note\nmain.rs\t{offset}\tskipped\tsome recorded message\n");
    let stubbed = run(&folder, &[&list], &[])?;
    let stubbed = text(&stubbed)?;
    // Only the listed body is replaced; its symbol, callers and siblings remain.
    assert!(!stubbed.contains("binary mul") && stubbed.contains("binary add"));
    assert_eq!(
        strict.matches("function ").count(),
        stubbed.matches("function ").count()
    );
    Ok(())
}

#[test]
fn malformed_lists_and_unmarked_bring_up_libraries_are_usage_errors() -> Result {
    let folder = folder("usage")?;
    for (list, message) in [
        ("main.rs\n", "expected PATH<TAB>OFFSET"),
        ("main.rs\tx\n", "expected PATH<TAB>OFFSET"),
        ("\t3\n", "expected PATH<TAB>OFFSET"),
        ("main.rs\t3\nmain.rs\t3\n", "repeated entry"),
    ] {
        let output = run(&folder, &[list], &[])?;
        assert_eq!(output.status.code(), Some(2), "{list:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(message));
    }
    let twice = run(&folder, &["", ""], &[])?;
    assert_eq!(twice.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&twice.stderr).contains("duplicate --bringup-skip-list"));
    let library = [
        "--crate-type=lib",
        "--crate-name",
        "probe",
        "--crate-disambiguator",
    ];
    let unmarked = run(&folder, &[""], &[&library[..], &["probe"]].concat())?;
    assert_eq!(unmarked.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unmarked.stderr).contains("ending in _bringup"));
    Ok(())
}

#[test]
fn rejected_functions_are_named_until_the_list_compiles_the_rest() -> Result {
    let folder = folder_with("named", REJECTED)?;
    // Strict compilation fails without naming anything for a skip list.
    let strict = run(&folder, &[], &[])?;
    assert_eq!(strict.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&strict.stderr).contains("bringup-"));
    // Grow the list as the census driver does: every failing run names a new function.
    let mut list = String::new();
    let mut names = Vec::new();
    let stubbed = loop {
        let output = run(&folder, &[&list], &[])?;
        if output.status.success() {
            break String::from_utf8(output.stdout)?;
        }
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8(output.stderr)?;
        assert!(!stderr.contains("bringup-unattributed"), "{stderr}");
        let before = names.len();
        for row in stderr
            .lines()
            .filter_map(|line| line.strip_prefix("bringup-blocker\t"))
        {
            let fields = row.split('\t').collect::<Vec<_>>();
            let &[path, offset, name, message] = fields.as_slice() else {
                return Err(format!("malformed blocker row: {row}").into());
            };
            assert_eq!(path, "main.rs");
            assert_eq!(
                REJECTED.find(&format!("fn {name}")),
                Some(offset.parse::<usize>()?)
            );
            assert!(!message.is_empty() && stderr.contains("ERROR: "));
            assert!(!names.contains(&name.to_owned()), "{name} failed again");
            names.push(name.to_owned());
            writeln!(list, "{path}\t{offset}\t{name}\t{message}")?;
        }
        assert!(names.len() > before, "no new function: {stderr}");
    };
    names.sort_unstable();
    assert_eq!(names, ["moved_twice", "stolen"]);
    // The accepted function keeps its body; both rejected ones trap instead.
    assert!(stubbed.contains("binary and"));
    assert_eq!(stubbed.matches("call void @abort()").count(), 2);
    Ok(())
}

#[test]
fn failed_soundness_proofs_keep_their_bodies_and_are_named() -> Result {
    let folder = folder_with("unproven", UNPROVEN)?;
    // Strict compilation still rejects the first of them and names nothing.
    let strict = run(&folder, &[], &[])?;
    assert_eq!(strict.status.code(), Some(1));
    let stderr = String::from_utf8(strict.stderr)?;
    assert!(stderr.contains("required lifetime does not outlive its target"));
    assert!(!stderr.contains("bringup-"), "{stderr}");
    // Bring-up compiles all three with an empty list and reports each once.
    let passed = run(&folder, &[""], &[])?;
    let stderr = String::from_utf8_lossy(&passed.stderr).into_owned();
    let lowered = text(&passed)?;
    let mut rows = stderr
        .lines()
        .filter_map(|line| line.strip_prefix("bringup-unproven\t"))
        .collect::<Vec<_>>();
    rows.sort_unstable();
    let row = |start: &str, name: &str, message: &str| -> Result<String> {
        let offset = UNPROVEN.find(start).ok_or("fixture lost a function")?;
        Ok(format!("main.rs\t{offset}\t{name}\t{message}"))
    };
    assert_eq!(
        rows,
        [
            row(
                "fn conflicted",
                "conflicted",
                "access conflicts with a live reference loan"
            )?,
            row(
                "const fn discarded",
                "discarded",
                "constant evaluation cannot run destructors"
            )?,
            row(
                "fn shortened",
                "shortened",
                "required lifetime does not outlive its target"
            )?,
        ]
    );
    assert!(!stderr.contains("bringup-blocker"), "{stderr}");
    // No body became a trapping stub: the loan conflict still writes its value.
    assert!(lowered.contains("binary or"), "{lowered}");
    assert!(!lowered.contains("call void @abort()"), "{lowered}");
    Ok(())
}

#[test]
fn publication_fences_stop_strict_libraries_and_are_named_when_waived() -> Result {
    let (strict, _) = library("fence-strict", FENCED, "fenced", None, true)?;
    let stderr = String::from_utf8(strict.stderr)?;
    assert_eq!(strict.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains(FENCE) && !stderr.contains("bringup-"));
    let (passed, metadata) = library("fence-waived", FENCED, "fenced_bringup", Some(""), true)?;
    let stderr = String::from_utf8(passed.stderr)?;
    assert!(passed.status.success(), "{stderr}");
    assert_eq!(waived(&stderr), [FENCE]);
    // The waived fence still publishes the rest of the interface.
    assert!(std::fs::metadata(metadata)?.len() > 0);
    Ok(())
}

#[test]
fn waived_guards_keep_the_course_abort_and_withheld_metadata_is_named() -> Result {
    // This source installs its own panic hooks and reaches a bounds guard.
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/assertion-panics/bounds-route-v2.rs");
    let source = std::fs::read_to_string(fixture)?;
    let (strict, _) = library("guard-strict", &source, "guards", None, false)?;
    assert_eq!(strict.status.code(), Some(1));
    assert!(!String::from_utf8(strict.stderr)?.contains("bringup-"));
    let (passed, _) = library("guard-waived", &source, "guards_bringup", Some(""), false)?;
    let stderr = String::from_utf8(passed.stderr)?;
    assert!(passed.status.success(), "{stderr}");
    let fences = waived(&stderr);
    assert!(fences.contains(&GUARD), "{stderr}");
    assert!(
        fences
            .iter()
            .any(|fence| fence.starts_with("source metadata is withheld: ")),
        "{stderr}"
    );
    assert!(std::str::from_utf8(&passed.stdout)?.contains("call void @abort()"));
    Ok(())
}

#[test]
fn an_unevaluable_constant_stubs_the_function_that_demands_it() -> Result {
    let folder = folder_with("constant", DEMANDED)?;
    assert_eq!(run(&folder, &[], &[])?.status.code(), Some(1));
    let named = run(&folder, &[""], &[])?;
    assert_eq!(named.status.code(), Some(1));
    let stderr = String::from_utf8(named.stderr)?;
    let rows = stderr
        .lines()
        .filter_map(|line| line.strip_prefix("bringup-blocker\t"))
        .collect::<Vec<_>>();
    let &[row] = rows.as_slice() else {
        return Err(format!("expected one blocker: {stderr}").into());
    };
    assert!(row.starts_with("main.rs\t0\tdemanded\t"), "{row}");
    let stubbed = run(&folder, &[&format!("{row}\n")], &[])?;
    let stubbed = text(&stubbed)?;
    // The function that only uses evaluable values keeps its body.
    assert!(stubbed.contains("binary and"));
    assert_eq!(stubbed.matches("call void @abort()").count(), 1);
    Ok(())
}

#[test]
fn unlowered_intrinsics_trap_or_run_their_fallback_bodies_and_are_named() -> Result {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/type-reflection/setup-v3/actual-pair.rs");
    let base = std::fs::read_to_string(fixture)?
        .replace("no_core,lang_items,", "no_core,lang_items,intrinsics,");
    let user =
        "#[unsafe(no_mangle)]pub fn reflected_identity(value:Identity)->u8{type_of(value).kind}";
    for (name, declaration, fence, trapped) in [
        (
            "fallback",
            "#[rustc_intrinsic] const fn type_of(_:Identity)->Reflected{Reflected{kind:7}}",
            "source intrinsic runs its fallback body: type_of",
            false,
        ),
        (
            "bodiless",
            "#[rustc_intrinsic] const fn type_of(_:Identity)->Reflected;",
            "source intrinsic executable lowering is not implemented: type_of",
            true,
        ),
    ] {
        let source = format!("{base}\n{declaration}\n{user}\n");
        let (strict, _) = library(
            &format!("intrinsic-{name}-strict"),
            &source,
            "traps",
            None,
            false,
        )?;
        assert_eq!(strict.status.code(), Some(1));
        assert!(!String::from_utf8(strict.stderr)?.contains("bringup-"));
        let folder = format!("intrinsic-{name}");
        let (passed, _) = library(&folder, &source, "traps_bringup", Some(""), false)?;
        let stderr = String::from_utf8(passed.stderr)?;
        assert!(passed.status.success(), "{stderr}");
        assert!(waived(&stderr).contains(&fence), "{stderr}");
        // Only the bodiless declaration becomes a shim that aborts when reached;
        // its caller keeps its own body either way.
        let lowered = std::str::from_utf8(&passed.stdout)?;
        assert_eq!(lowered.contains("rg.trap.type_of"), trapped, "{lowered}");
        assert_eq!(lowered.contains("call void @abort()"), trapped, "{lowered}");
    }
    Ok(())
}

/// The head of a source-core library that may declare intrinsics.
const HEAD: &str = "\
#![feature(no_core,lang_items,intrinsics,rustc_attrs)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang=\"pointee_sized\"]pub trait PointeeSized{}
#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}
#[lang=\"sized\"]pub trait Sized:MetaSized{}
#[lang=\"legacy_receiver\"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang=\"clone\"]pub trait Clone:Sized{#[lang=\"clone_fn\"]fn clone(&self)->Self;}
#[lang=\"copy\"]pub trait Copy:Clone{}
";
/// Bodiless intrinsics that the bring-up profile lowers in place, each called
/// with types the lowering handles.
const LOWERED: &str = "\
#[rustc_intrinsic] const fn ctpop<T>(x:T)->u32;
#[rustc_intrinsic] const fn saturating_add<T>(a:T,b:T)->T;
#[rustc_intrinsic] const unsafe fn offset<P,D>(dst:P,offset:D)->P;
#[rustc_intrinsic] const unsafe fn read_via_copy<T>(ptr:*const T)->T;
#[unsafe(no_mangle)]pub fn ones(x:u32)->u32{ctpop(x)}
#[unsafe(no_mangle)]pub fn clamped(a:u8,b:u8)->u8{saturating_add(a,b)}
#[unsafe(no_mangle)]pub unsafe fn third(p:*const u32)->u32{unsafe{read_via_copy(offset(p,3isize))}}
";
/// A call with a type that the lowering of the same intrinsic does not handle.
const UNHANDLED: &str = "#[unsafe(no_mangle)]pub fn truth(x:bool)->u32{ctpop(x)}\n";
/// Atomic accesses of a narrow integer, a machine word and a pointer.
const ATOMICS: &str = "\
#[rustc_intrinsic] unsafe fn atomic_load<T,const ORD:bool>(src:*const T)->T;
#[rustc_intrinsic] unsafe fn atomic_xadd<T,U,const ORD:bool>(dst:*mut T,src:U)->T;
#[rustc_intrinsic] unsafe fn atomic_cxchg<T,const S:bool,const F:bool>(dst:*mut T,old:T,src:T)->(T,bool);
#[rustc_intrinsic] unsafe fn atomic_umax<T,const ORD:bool>(dst:*mut T,src:T)->T;
#[rustc_intrinsic] unsafe fn atomic_fence<const ORD:bool>();
#[unsafe(no_mangle)]pub unsafe fn bump(p:*mut u16)->u16{unsafe{atomic_fence::<true>();atomic_xadd::<_,_,true>(p,3u16)}}
#[unsafe(no_mangle)]pub unsafe fn raise(p:*mut i64,v:i64)->i64{unsafe{atomic_umax::<_,true>(p,v)}}
#[unsafe(no_mangle)]pub unsafe fn peek(p:*const *const u8)->*const u8{unsafe{atomic_load::<_,true>(p)}}
#[unsafe(no_mangle)]pub unsafe fn claim(p:*mut *const u8,old:*const u8,new:*const u8)->bool{
    unsafe{atomic_cxchg::<_,true,false>(p,old,new).1}
}
";
/// Counted transfers and comparisons, a volatile read, a slice's size, a
/// validity assertion that holds, a float's magnitude and a forgotten value.
const MEMORY: &str = "\
#[rustc_intrinsic] const unsafe fn copy_nonoverlapping<T>(src:*const T,dst:*mut T,count:usize);
#[rustc_intrinsic] const unsafe fn write_bytes<T>(dst:*mut T,val:u8,count:usize);
#[rustc_intrinsic] const unsafe fn compare_bytes(left:*const u8,right:*const u8,bytes:usize)->i32;
#[rustc_intrinsic] const unsafe fn raw_eq<T>(a:&T,b:&T)->bool;
#[rustc_intrinsic] unsafe fn volatile_load<T>(src:*const T)->T;
#[rustc_intrinsic] const unsafe fn size_of_val<T:?Sized>(ptr:*const T)->usize;
#[rustc_intrinsic] const fn assert_zero_valid<T>();
#[rustc_intrinsic] const fn fabs<T>(x:T)->T;
#[rustc_intrinsic] const fn forget<T>(x:T);
#[unsafe(no_mangle)]pub unsafe fn moved(s:*const u32,d:*mut u32){unsafe{copy_nonoverlapping(s,d,4usize)}}
#[unsafe(no_mangle)]pub unsafe fn filled(d:*mut u16){unsafe{write_bytes(d,7u8,3usize)}}
#[unsafe(no_mangle)]pub unsafe fn ordered(l:*const u8,r:*const u8)->i32{unsafe{compare_bytes(l,r,5usize)}}
#[unsafe(no_mangle)]pub fn same(a:&[u8;6],b:&[u8;6])->bool{unsafe{raw_eq(a,b)}}
#[unsafe(no_mangle)]pub unsafe fn sensed(p:*const u32)->u32{unsafe{volatile_load(p)}}
#[unsafe(no_mangle)]pub unsafe fn extent(p:*const [u16])->usize{unsafe{size_of_val(p)}}
#[unsafe(no_mangle)]pub fn zeroable(){assert_zero_valid::<(u32,[bool;2])>()}
#[unsafe(no_mangle)]pub fn magnitude(x:f64)->f64{fabs(x)}
#[unsafe(no_mangle)]pub fn kept(x:[u64;3]){forget(x)}
";
/// An assertion that the lowering cannot decide for the asserted type.
const UNDECIDED: &str = "#[unsafe(no_mangle)]pub fn refused(){assert_zero_valid::<&u8>()}\n";
const UNQUALIFIED: &str = "source intrinsic uses unqualified lowering: ";
const UNLOWERED: &str = "source intrinsic executable lowering is not implemented: ";
/// Where a strict library with such declarations has always stopped.
const DECLARED: &str = "source intrinsic declaration semantics require a wider metadata profile";
const PARAMETERS: &str = "source const parameters are outside the initial source profile";

/// One library: its source after `HEAD`, the strict profile's fence, the
/// intrinsics lowered in place, pieces of their `LowIR` and the one intrinsic
/// whose call keeps its trap.
struct Lowering {
    name: &'static str,
    source: [&'static str; 2],
    fence: &'static str,
    lowered: &'static [&'static str],
    emitted: &'static [&'static str],
    trapped: Option<&'static str>,
}
const PLAIN: &[&str] = &["ctpop", "saturating_add", "offset", "read_via_copy"];
const SCALAR: &[&str] = &["binary ushr", "index u8"];
const STORED: &[&str] = &[
    "copy_nonoverlapping",
    "write_bytes",
    "compare_bytes",
    "raw_eq",
    "volatile_load",
    "size_of_val",
    "assert_zero_valid",
    "fabs",
    "forget",
];
/// Transfers and comparisons call one shim each; the rest is emitted in place.
const SHIMS: &[&str] = &[
    "rg.unqualified.copy_nonoverlapping.t",
    "rg.unqualified.write_bytes.t",
    "rg.unqualified.compare_bytes.t",
    "rg.unqualified.raw_eq.t",
    "load volatile u32",
    "binary mul i64",
    "binary and i64",
];
const LOWERINGS: &[Lowering] = &[
    Lowering {
        name: "lowered",
        source: [LOWERED, ""],
        fence: DECLARED,
        lowered: PLAIN,
        emitted: SCALAR,
        trapped: None,
    },
    Lowering {
        name: "unhandled",
        source: [LOWERED, UNHANDLED],
        fence: DECLARED,
        lowered: PLAIN,
        emitted: SCALAR,
        trapped: Some("ctpop"),
    },
    // Atomics use the word's own width; a pointer is carried as a machine word.
    Lowering {
        name: "atomics",
        source: [ATOMICS, ""],
        fence: PARAMETERS,
        lowered: &[
            "atomic_load",
            "atomic_xadd",
            "atomic_cxchg",
            "atomic_umax",
            "atomic_fence",
        ],
        emitted: &[
            "atomic_add_fetch u16",
            "atomic_load i64",
            "atomic_compare_exchange i64",
        ],
        trapped: None,
    },
    Lowering {
        name: "memory",
        source: [MEMORY, ""],
        fence: DECLARED,
        lowered: STORED,
        emitted: SHIMS,
        trapped: None,
    },
    Lowering {
        name: "undecided",
        source: [MEMORY, UNDECIDED],
        fence: DECLARED,
        lowered: STORED,
        emitted: SHIMS,
        trapped: Some("assert_zero_valid"),
    },
];

#[test]
fn intrinsics_with_an_unqualified_lowering_are_named_and_stay_fenced_when_strict() -> Result {
    for case in LOWERINGS {
        let source = format!("{HEAD}{}", case.source.concat());
        // The strict profile stops at its own fence, lowers and names nothing.
        let strict = format!("{}-strict", case.name);
        let (strict, _) = library(&strict, &source, "lowers", None, false)?;
        let stderr = String::from_utf8(strict.stderr)?;
        assert_eq!(strict.status.code(), Some(1), "{stderr}");
        assert!(
            stderr.contains(case.fence) && !stderr.contains("bringup-") && strict.stdout.is_empty(),
            "{stderr}"
        );
        let (passed, _) = library(case.name, &source, "lowers_bringup", Some(""), false)?;
        let stderr = String::from_utf8(passed.stderr)?;
        assert!(passed.status.success(), "{stderr}");
        let fences = waived(&stderr);
        for intrinsic in case.lowered {
            let fence = format!("{UNQUALIFIED}{intrinsic}");
            assert!(fences.contains(&fence.as_str()), "{stderr}");
        }
        // Lowered calls need no trap; only the call left alone keeps one.
        let kept: Vec<_> = fences
            .iter()
            .filter_map(|fence| fence.strip_prefix(UNLOWERED))
            .collect();
        assert_eq!(kept, Vec::from_iter(case.trapped), "{stderr}");
        let text = std::str::from_utf8(&passed.stdout)?;
        let traps = case.trapped.is_some();
        assert_eq!(text.contains("rg.trap."), traps, "{text}");
        assert_eq!(text.contains("call void @abort()"), traps, "{text}");
        assert!(
            case.emitted.iter().all(|piece| text.contains(piece)),
            "{text}"
        );
    }
    Ok(())
}

/// The x86 spin-wait hint that `core::hint::spin_loop` reaches through stdarch.
const PAUSE: &str = "\
#![allow(internal_features)]
#![feature(link_llvm_intrinsics)]
unsafe extern \"C\"{#[link_name=\"llvm.x86.sse2.pause\"]fn pause();}
fn main(){unsafe{pause()}}
";

#[test]
fn the_spin_wait_hint_returns_at_once_and_stays_fenced_when_strict() -> Result {
    let fence = "selected LLVM intrinsic operation is not implemented";
    let folder = folder_with("pause", PAUSE)?;
    let strict = run(&folder, &[], &[])?;
    let stderr = String::from_utf8(strict.stderr)?;
    assert_eq!(strict.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains(fence) && !stderr.contains("bringup-") && strict.stdout.is_empty(),
        "{stderr}"
    );
    let passed = run(&folder, &[""], &[])?;
    let stderr = String::from_utf8(passed.stderr)?;
    assert!(passed.status.success(), "{stderr}");
    let waiver = "selected LLVM intrinsic uses unqualified lowering: x86.sse2.pause";
    assert_eq!(waived(&stderr), [waiver], "{stderr}");
    let text = std::str::from_utf8(&passed.stdout)?;
    assert!(
        text.contains("rg.llvm.pause.t") && !text.contains("@abort") && !text.contains("llvm.x86"),
        "{text}"
    );
    // A declaration with any other shape is not the hint and keeps the fence.
    let other = PAUSE.replace("fn pause();", "fn pause(x:u32);");
    let other = folder_with("pause-shape", &other.replace("pause()}", "pause(1u32)}"))?;
    let failed = run(&other, &[""], &[])?;
    let stderr = String::from_utf8(failed.stderr)?;
    assert_eq!(failed.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains(fence) && waived(&stderr).is_empty(),
        "{stderr}"
    );
    Ok(())
}
