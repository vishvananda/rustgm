//! `Option` of a signed integer that excludes only zero is that integer, with
//! zero as `None`, as rustc lays out `Option<NonZero<i8>>`. The source writes
//! the range `..=-1 | 1..`. Every packet outcome is frozen in its
//! `signed/contract.json`.
use super::{Result, cli_types, lowered};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const PACKET: &str = "pa31/tests/spec/source-integer-niches/signed";
const WIDTHS: [&str; 6] = ["i8", "i16", "i32", "i64", "i128", "isize"];
const BOUNDARIES: [&str; 10] = [
    "echo",
    "make",
    "get",
    "invoke",
    "size",
    "align",
    "constant_none",
    "constant_some",
    "constant_minus",
    "constant_round",
];
/// Rust rejects the invalid constant of `some-zero`; `partial-range` is valid.
const REJECTED: [(&str, Option<&str>, &str); 2] = [
    (
        "some-zero",
        Some("E0080"),
        "value is outside its scalar pattern range",
    ),
    (
        "partial-range",
        None,
        "scalar pattern enum niches require their complete ABI profile",
    ),
];
/// A unit variant beside the signed carrier niche-fills in either orientation.
const COMPANION: &str = "pub enum Either<T,E>{Left(T),Right(E)}\n\
fn unit_or(x:i8)->Either<(),Nz_i8>{match Nz_i8::new(x){Some(v)=>Either::Right(v),None=>Either::Left(())}}\n\
fn or_unit(x:Either<Nz_i8,()>)->i8{match x{Either::Left(v)=>v.as_inner(),Either::Right(())=>0}}\n\
#[unsafe(no_mangle)]pub extern \"C\" fn bits(x:i8)->i8{unsafe{intrinsics::transmute::<Either<(),Nz_i8>,i8>(unit_or(x))}}\n\
#[unsafe(no_mangle)]pub extern \"C\" fn through(x:i8)->i8{match unit_or(x){Either::Left(())=>0,Either::Right(v)=>or_unit(Either::Left(v))}}\n\
#[unsafe(no_mangle)]pub extern \"C\" fn size_either()->usize{const{size_of::<Either<(),Nz_i8>>()}}\n";
const COMPANION_OBSERVER: &str = "#include <stdint.h>\n#include <stdio.h>\n\
extern int8_t bits(int8_t);extern int8_t through(int8_t);extern uintptr_t size_either(void);\n\
int main(void){printf(\"companion %d %d %d %d %d %d %d %d %lu\\n\",bits(0),bits(-7),bits(-128),bits(127),\
through(0),through(-7),through(-128),through(127),(unsigned long)size_either());return 0;}\n";
const COMPANION_PRINTED: &str = "companion 0 -7 -128 127 0 -7 -128 127 1\n";

fn root() -> Result<&'static Path> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?)
}

fn packet(name: &str) -> Result<PathBuf> {
    Ok(root()?.join(PACKET).join(name))
}

fn evidence(kind: &str) -> Result<PathBuf> {
    let folder = root()?
        .join("obj/parallel/signed-niches")
        .join(kind)
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}

/// The independent oracle: pinned rustc compiles one library to an object.
fn rustc(source: &Path, object: &Path) -> Result<Output> {
    Ok(Command::new("rustc")
        .env("RUSTC_BOOTSTRAP", "1")
        .args(["+1.98.0", "--edition=2024", "--crate-type=lib"])
        .args(["--crate-name", "minicore", "--emit=obj"])
        .args(["--cap-lints=allow", "-Cpanic=abort", "-Copt-level=0"])
        .arg("-o")
        .arg(object)
        .arg(source)
        .output()?)
}

/// What rustc's object prints once linked with the C observer.
fn independent(source: &Path, observer: &Path, folder: &Path) -> Result<String> {
    let object = folder.join("independent.o");
    let oracle = rustc(source, &object)?;
    if !oracle.status.success() {
        return Err(String::from_utf8_lossy(&oracle.stderr).into_owned().into());
    }
    let binary = folder.join("independent");
    let linked = Command::new("cc")
        .arg(observer)
        .arg(object)
        .arg("-o")
        .arg(&binary)
        .output()?;
    if !linked.status.success() {
        return Err(format!("independent link: {linked:?}").into());
    }
    let printed = Command::new(&binary).output()?;
    if !printed.status.success() {
        return Err(format!("independent program: {printed:?}").into());
    }
    Ok(String::from_utf8(printed.stdout)?)
}

fn companion() -> Result<String> {
    Ok(crate::resolve::fixture(&format!("{PACKET}/source-runtime.rs"))? + COMPANION)
}

/// Every width's foreign boundary, layout query and evaluated constant lowers
/// to one function body.
#[test]
fn signed_zero_niches_type_and_lower_every_boundary() -> Result {
    let source = packet("source-runtime.rs")?;
    let oracle = rustc(&source, &evidence("accepted")?.join("runtime.o"))?;
    assert!(
        oracle.status.success(),
        "rustc: {}",
        String::from_utf8_lossy(&oracle.stderr)
    );
    let (accepted, stderr) = cli_types(&source)?;
    assert!(accepted && stderr.is_empty(), "types: {stderr}");
    let program = lowered(&std::fs::read_to_string(&source)?, &[])?;
    let bodies = |program: &rustgm::lowir::program::Program, name: &str| {
        program
            .symbols
            .iter()
            .filter(|symbol| symbol.name == name && symbol.body.is_some())
            .count()
    };
    for width in WIDTHS {
        for boundary in BOUNDARIES {
            let name = format!("{boundary}_{width}");
            assert_eq!(bodies(&program, &name), 1, "{name}");
        }
    }
    for name in ["nested", "mixed", "packet"] {
        assert_eq!(bodies(&program, name), 1, "{name}");
    }
    let program = lowered(&companion()?, &[])?;
    for name in ["bits", "through", "size_either"] {
        assert_eq!(bodies(&program, name), 1, "{name}");
    }
    Ok(())
}

/// A zero payload is never silently `None`, and a range that excludes more
/// than zero keeps its explicit unsupported profile.
#[test]
fn zero_payloads_and_partial_signed_ranges_are_not_niches() -> Result {
    let folder = evidence("rejected")?;
    for (name, code, reason) in REJECTED {
        let source = packet(&format!("{name}.rs"))?;
        let oracle = rustc(&source, &folder.join(format!("{name}.o")))?;
        let stderr = String::from_utf8_lossy(&oracle.stderr);
        match code {
            Some(code) => assert!(
                oracle.status.code() == Some(1) && stderr.contains(code),
                "{name} rustc: {stderr}"
            ),
            None => assert!(oracle.status.success(), "{name} rustc: {stderr}"),
        }
        let error = lowered(&std::fs::read_to_string(&source)?, &[])
            .err()
            .ok_or_else(|| format!("{name}: invalid or unsupported niche admitted"))?
            .to_string();
        assert!(error.contains(reason), "{name}: {error}");
    }
    Ok(())
}

/// rustc's object and the native backend's give each C observer the same bits.
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn signed_zero_niches_run_as_rustc_runs() -> Result {
    let folder = evidence("native")?;
    let expected = crate::resolve::fixture(&format!("{PACKET}/expected.stdout"))?;
    let source = packet("source-runtime.rs")?;
    let observer = packet("observer.c")?;
    assert_eq!(independent(&source, &observer, &folder)?, expected);
    crate::ffi_program::check(
        root()?,
        &folder,
        "signed",
        &lowered(&std::fs::read_to_string(&source)?, &[])?,
        &std::fs::read_to_string(&observer)?,
        &expected,
    )?;
    let (source, observer) = (folder.join("companion.rs"), folder.join("observer.c"));
    std::fs::write(&source, companion()?)?;
    std::fs::write(&observer, COMPANION_OBSERVER)?;
    assert_eq!(independent(&source, &observer, &folder)?, COMPANION_PRINTED);
    crate::ffi_program::check(
        root()?,
        &folder,
        "companion",
        &lowered(&companion()?, &[])?,
        COMPANION_OBSERVER,
        COMPANION_PRINTED,
    )?;
    Ok(())
}
