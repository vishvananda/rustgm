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
