//! The original `_mm_stream_ps` and authored asm observers run natively
//! through the real CLI and the pinned object backend, and match pinned Rust
//! on the same C observer: every bit, both sentinels and a real `movntps`.
use std::{
    path::{Path, PathBuf},
    process::Command,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SPEC: &str = "pa41/tests/spec/asm-native";
/// Declared in the fixture contract: one template per invocation in each
/// emitted body, five with `nostack, preserves_flags` and one default.
const TEMPLATES: usize = 6;
const PRESERVING: &str = " intel readwrite 1 24 0 2 {";
const DEFAULT: &str = " intel readwrite 13 24 7 2 {";
const STORE: &str = "movntps XMMWORD PTR [";
fn root() -> Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?
        .to_owned())
}
fn checked(folder: &Path, name: &str, command: &mut Command) -> Result<Vec<u8>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{name}.stderr")), &output.stderr)?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!(
            "{name}: {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}
fn compile_c(folder: &Path, fixture: &Path, name: &str) -> Result<PathBuf> {
    let object = folder.join(format!("{name}.o"));
    checked(
        folder,
        name,
        Command::new("cc")
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-c"])
            .arg(fixture.join(format!("{name}.c")))
            .arg("-o")
            .arg(&object),
    )?;
    Ok(object)
}
/// Intel-syntax `movntps` stores with a memory destination and xmm source.
fn stores(folder: &Path, object: &Path, name: &str) -> Result<usize> {
    let text = String::from_utf8(checked(
        folder,
        name,
        Command::new("objdump")
            .args(["-d", "-M", "intel", "--no-show-raw-insn"])
            .arg(object),
    )?)?;
    std::fs::write(folder.join(format!("{name}.disassembly")), &text)?;
    Ok(text
        .lines()
        .filter(|line| line.contains(STORE) && line.contains("],xmm"))
        .count())
}
#[test]
#[ignore = "requires pinned CPPGM backend, pinned independent Rust and GNU objdump"]
fn original_stream_matches_independent_rust_bits_sentinels_and_movntps() -> Result {
    let root = root()?;
    let fixture = root.join(SPEC);
    let folder = root.join(format!(
        "obj/parallel/stream-assembly/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let expected = std::fs::read(fixture.join("expected.stdout"))?;
    let observer = compile_c(&folder, &fixture, "observer")?;
    let entry = compile_c(&folder, &fixture, "entry")?;
    let source = fixture.join("native.rs");
    let ir = folder.join("native.lowir");
    checked(
        &folder,
        "frontend",
        Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-lowir",
                "--cap-lints=allow",
                "--source-core",
                "probe@asm_native",
                "--crate-name",
                "probe",
                "--crate-disambiguator",
                "asm_native",
            ])
            .arg(&source)
            .arg("-o")
            .arg(&ir),
    )?;
    let lowir = std::fs::read_to_string(&ir)?;
    let lines = |prefix: &str| {
        lowir
            .lines()
            .filter(|line| line.starts_with(prefix))
            .count()
    };
    assert_eq!(lines("assembly @asm"), TEMPLATES);
    assert_eq!(lines("    asm @asm"), TEMPLATES);
    assert_eq!(lowir.matches(PRESERVING).count(), TEMPLATES - 1);
    assert_eq!(lowir.matches(DEFAULT).count(), 1);
    let native = folder.join("native.exe");
    checked(
        &folder,
        "backend",
        Command::new("python3")
            .arg(root.join("scripts/backend.py"))
            .arg(&ir)
            .arg("--native-object")
            .arg(&observer)
            .arg("-o")
            .arg(&native),
    )?;
    assert_eq!(
        checked(&folder, "native-execute", &mut Command::new(&native))?,
        expected
    );
    assert_eq!(
        stores(
            &folder,
            &native.with_extension("unit-0.o"),
            "native-objdump"
        )?,
        TEMPLATES
    );
    assert_eq!(independent(&folder, &source, &observer, &entry)?, expected);
    Ok(())
}
/// Pinned Rust builds the same source as a library object with at least one
/// `movntps`; `entry.c` supplies the C `main`. Returns the linked output.
fn independent(folder: &Path, source: &Path, observer: &Path, entry: &Path) -> Result<Vec<u8>> {
    let object = folder.join("independent.o");
    checked(
        folder,
        "independent",
        Command::new("rustc")
            .env("RUSTC_BOOTSTRAP", "1")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--emit=obj",
                "--cap-lints=allow",
                "-Cpanic=abort",
                "-Copt-level=0",
            ])
            .arg(source)
            .arg("-o")
            .arg(&object),
    )?;
    assert!(stores(folder, &object, "independent-objdump")? >= 1);
    let linked = folder.join("independent.exe");
    checked(
        folder,
        "independent-link",
        Command::new("cc")
            .arg(&object)
            .arg(observer)
            .arg(entry)
            .arg("-o")
            .arg(&linked),
    )?;
    checked(folder, "independent-execute", &mut Command::new(&linked))
}
#[test]
#[ignore = "requires pinned independent Rust"]
fn pinned_rust_accepts_every_form_outside_the_initial_profile() -> Result {
    let root = root()?;
    let fixture = root.join(SPEC);
    let folder = root.join(format!(
        "obj/parallel/stream-assembly/forms-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let cases = std::fs::read_to_string(fixture.join("opponents.tsv"))?;
    let mut count = 0;
    for line in cases.lines().filter(|line| !line.starts_with('#')) {
        let fields: Vec<_> = line.split('\t').collect();
        let &[path, status, _] = &fields[..] else {
            return Err(format!("opponent row: {line}").into());
        };
        let output = Command::new("rustc")
            .env("RUSTC_BOOTSTRAP", "1")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--target=x86_64-unknown-linux-gnu",
                "--crate-type=lib",
                "--crate-name=opponent",
                "--emit=metadata",
                "-Cpanic=abort",
                "-Copt-level=0",
            ])
            .arg(fixture.join(path))
            .arg("--out-dir")
            .arg(&folder)
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(status.parse::<i32>()?),
            "{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        count += 1;
    }
    // B1's 16 forms, then the two comment forms of A's comment fence.
    assert_eq!(count, 18);
    Ok(())
}
