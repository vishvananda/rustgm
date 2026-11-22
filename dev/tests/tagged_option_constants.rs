//! A repr(Rust) enum keeps a tag where its payload has no niche. For `Option`
//! that tag is one byte, as rustc 1.98.0 lays out `Option<u8>` (tag at 0,
//! payload at 1) and `Option<u32>` (tag at 0, payload at 4). A constant read of
//! such an enum's bytes decodes that tag: core's `MaybeUninit::zeroed` makes
//! alloc's `vec/is_zero.rs` read a zeroed `Option<u8>` as `None`.
//!
//! The `tagged-option-constants` packet transmutes integers to and from these
//! options in constants and fills one with `write_bytes`; C observers print
//! the strictly evaluated results.
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

/// Both valid programs lower, and an invalid tag is rejected.
#[test]
fn tagged_option_constants_decode_their_one_byte_tag() -> Result {
    program("transmuted")?;
    match program("invalid-tag") {
        Ok(_) => return Err("an invalid Option tag was accepted".into()),
        Err(error) if error.to_string().contains(INVALID) => {}
        Err(error) => return Err(format!("invalid-tag: {error}").into()),
    }
    program("zeroed")?;
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
    crate::observed_program::execute(
        &root.join("obj/parallel/tagged-option-constants/zeroed"),
        &program("zeroed")?,
        &root.join(PACKET).join("zeroed-observe.c"),
        ZEROED.as_bytes(),
    )
}
