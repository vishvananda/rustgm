//! `ymm_reg` operands type as rustc's x86 register table admits them under
//! `avx`: integer and float scalars of 32 or 64 bits and 128- or 256-bit
//! vectors. The initial assembly profile still rejects the class before
//! native lowering or export, so no imported body carries one.
use super::Result;
use crate::pipeline;
use std::fmt::Write as _;
const PACKET: &str = "pa41/tests/spec/asm-ymm";
/// rustc's typing rejections, each with this checker's reason.
const REJECTED: [(&str, &str); 6] = [
    ("scalar-u8", "incompatible with register class"),
    ("no-avx", "requires the avx target feature"),
    ("wide", "128 or 256 bits"),
    ("short", "128 or 256 bits"),
    ("modifier-e", "not supported by its class"),
    ("xmm-256", "exactly 128 bits"),
];
/// Accepted forms and the profile text that stops their lowering: an output
/// operand is checked before its class.
const ACCEPTED: [(&str, &str); 10] = [
    ("stream-ps", YMM),
    ("stream-pd", YMM),
    ("stream-si256", YMM),
    ("vector-i32x4", YMM),
    ("vector-i16x16", YMM),
    ("scalar-f64", YMM),
    ("scalar-u32", YMM),
    ("avx2-implies", YMM),
    ("modifier-x", YMM),
    ("output", crate::support::initial_assembly::OUTPUT),
];
const YMM: &str = crate::support::initial_assembly::YMM;

/// A core stream store after `prefix`: its crate attribute and `arch`
/// re-export are dropped, its call names `asm` as `call` does, and its
/// derived vector Clone/Copy becomes explicit.
fn stream_source(path: &str, mut text: String, call: &str) -> Result<String> {
    let input = crate::test_source::fixture(path)?;
    let vector = input
        .lines()
        .find_map(|line| line.strip_prefix("pub struct "))
        .and_then(|rest| rest.split('(').next())
        .ok_or("stream vector declaration")?;
    for line in input
        .lines()
        .filter(|line| !line.starts_with("#!") && !line.starts_with("mod arch "))
    {
        text.push_str(
            &line
                .replace("#[derive(Clone, Copy)]", "")
                .replace("crate::arch::asm!", call),
        );
        text.push('\n');
    }
    write!(
        text,
        "impl Clone for {vector}{{fn clone(&self)->Self{{*self}}}}impl Copy for {vector}{{}}"
    )?;
    Ok(text)
}

/// `_mm256_stream_ps` on the publishable native prelude, whose source-owned
/// `arch` module declares `asm`.
fn export_source() -> Result<String> {
    let mut prefix = super::lowering::native_prefix("#[repr(simd)]")?;
    prefix.push_str("impl<T:Copy,const N:usize>Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize>Copy for [T;N]{}");
    stream_source(
        &format!("{PACKET}/stream-ps.rs"),
        prefix,
        "crate::arch::asm!",
    )
}

fn case_source(name: &str) -> Result<String> {
    let path = format!("{PACKET}/{name}.rs");
    if name.starts_with("stream-") {
        let mut prefix = super::vectors::foundation()?;
        prefix.push_str("#[rustc_builtin_macro(concat)]macro_rules!concat{($($t:tt)*)=>{};}");
        stream_source(&path, prefix, "asm!")
    } else {
        super::vectors::fixture_source(&path)
    }
}

/// The function holding a case's asm: core's stream store or `f`.
fn entry(name: &str) -> String {
    name.strip_prefix("stream-")
        .map_or_else(|| "f".to_owned(), |store| format!("_mm256_stream_{store}"))
}

/// Typing alone, then native lowering of `entry` as the only export, which
/// applies the initial assembly profile.
pub(super) fn check(text: &str, entry: &str) -> Result<(Result, Result)> {
    let report = crate::test_source::source_provider(text, "ymm_asm")?;
    let mut analysis = match pipeline::analyze_source(&report) {
        Ok(analysis) => analysis,
        Err(error) => return Ok((Err(error.to_string().into()), Ok(()))),
    };
    let export = super::lowering::root(&report, &analysis.facts, entry)?;
    let lowered = crate::lower::instances::build_roots_prepared(
        &report,
        &mut analysis.facts,
        crate::lower::Roots {
            entry: None,
            exports: &[export],
        },
        analysis.prepared,
    )
    .map(drop)
    .map_err(|error| error.to_string().into());
    Ok((Ok(()), lowered))
}

#[test]
fn ymm_operands_type_like_rustc_and_stop_at_the_assembly_profile() -> Result {
    let mut failures = Vec::new();
    for (name, reason) in REJECTED {
        match check(&case_source(name)?, &entry(name))? {
            (Err(error), _) if error.to_string().contains(reason) => {}
            (typed, _) => failures.push(format!("{name}: typing {typed:?}")),
        }
    }
    for (name, profile) in ACCEPTED {
        match check(&case_source(name)?, &entry(name))? {
            (Ok(()), Err(error)) if error.to_string().contains(profile) => {}
            outcome => failures.push(format!("{name}: {outcome:?}")),
        }
    }
    // Module-body export applies the same profile, so no published body
    // carries a ymm operand for a consumer to re-prove.
    match crate::module_packets::exported_profile(
        &export_source()?,
        crate::lower::instances::SourcePublication::ModuleBodies,
        "ymm_export",
    ) {
        Err(error) if error.to_string().contains(YMM) => {}
        outcome => failures.push(format!("stream-ps export: {:?}", outcome.err())),
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
