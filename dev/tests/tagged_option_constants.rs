//! A repr(Rust) enum keeps a tag where its payload has no niche. For `Option`
//! that tag is one byte, as rustc 1.98.0 lays out `Option<u8>` (tag at 0,
//! payload at 1) and `Option<u32>` (tag at 0, payload at 4). A constant read of
//! such an enum's bytes decodes that tag: core's `MaybeUninit::zeroed` makes
//! alloc's `vec/is_zero.rs` read a zeroed `Option<u8>` as `None`.
//!
//! The `tagged-option-constants` packet transmutes integers to and from these
//! options in constants, and in `zeroed` fills one with `write_bytes`, which
//! only the bring-up profile executes in constants; C observers print them.
//! Outcomes were declared before either compiler ran; rustc 1.98.0's are
//! recorded in its `outcomes.json`.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const PACKET: &str = "pa31/tests/spec/tagged-option-constants";
/// What rustc's build of `transmuted` prints.
const NATIVE: &str = "none8 200\nsome8 9\nnone32 200\nsome32 7\nbytes 1281\n";
/// Tag byte 2 is no variant of `Option<u8>`; rustc rejects it with E0080.
const INVALID: &str = "constant enum read has an invalid discriminant";
/// What rustc's build of `zeroed` prints.
const ZEROED: &str = "zeroed 200\nones 41\n";
/// The strict profile keeps `write_bytes` fenced in constants.
const FENCED: &str = "source intrinsic constant execution is not implemented";

fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(
        &crate::resolve::fixture(&format!("{PACKET}/{name}.rs"))?,
        "tagged_option_constants",
        &config,
    )?;
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

/// The command-line compiler's `LowIR` for `zeroed`, written in `folder`:
/// strict, or bring-up with an empty skip list.
fn zeroed(folder: &std::path::Path, bringup: bool) -> Result<std::process::Output> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    std::fs::create_dir_all(folder)?;
    let identity = if bringup {
        "zeroed_option_bringup"
    } else {
        "zeroed_option"
    };
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args(["--emit-lowir", "--cap-lints=allow", "--edition=2024"])
        .args(["--crate-type=lib", "--crate-name", "probe"])
        .args(["--crate-disambiguator", identity])
        .args(["--source-core", &format!("probe@{identity}")]);
    if bringup {
        let list = folder.join("skip.tsv");
        std::fs::write(&list, "")?;
        command.arg("--bringup-skip-list").arg(list);
    }
    let output = command.arg(root.join(PACKET).join("zeroed.rs")).output()?;
    if !matches!(output.status.code(), Some(0 | 1)) {
        return Err(format!("CLI infrastructure failure: {}", output.status).into());
    }
    std::fs::write(folder.join("zeroed.lowir"), &output.stdout)?;
    Ok(output)
}

/// The accepted programs lower, the invalid tag is rejected, and `zeroed`
/// stays fenced in the strict profile.
#[test]
fn tagged_option_constants_decode_their_one_byte_tag() -> Result {
    program("transmuted")?;
    match program("invalid-tag") {
        Ok(_) => return Err("an invalid Option tag was accepted".into()),
        Err(error) if error.to_string().contains(INVALID) => {}
        Err(error) => return Err(format!("invalid-tag: {error}").into()),
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/tagged-option-constants/check");
    let strict = zeroed(&folder, false)?;
    let stderr = String::from_utf8(strict.stderr)?;
    assert!(
        !strict.status.success() && stderr.contains(FENCED),
        "{stderr}"
    );
    let bringup = zeroed(&folder, true)?;
    let stderr = String::from_utf8(bringup.stderr)?;
    assert!(bringup.status.success(), "{stderr}");
    Ok(())
}

/// The accepted program prints exactly what rustc's build of it prints.
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn tagged_option_constants_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    crate::observed_program::execute(
        &root.join("obj/parallel/tagged-option-constants/native"),
        &program("transmuted")?,
        &root.join(PACKET).join("observe.c"),
        NATIVE.as_bytes(),
    )?;
    let folder = root.join("obj/parallel/tagged-option-constants/zeroed");
    let output = zeroed(&folder, true)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    crate::observed_program::execute_lowir(
        &folder,
        &folder.join("zeroed.lowir"),
        &root.join(PACKET).join("zeroed-observe.c"),
        ZEROED.as_bytes(),
    )
}
