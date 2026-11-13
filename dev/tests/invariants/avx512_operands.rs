//! `zmm_reg` and `kreg` operands type as rustc's x86 register table admits
//! them under `avx512f`: `zmm_reg` takes 32- and 64-bit scalars and 128-, 256-
//! or 512-bit vectors, and `kreg` 8- and 16-bit integers, with `avx512bw` for
//! 32 and 64 bits. Vector widths count `f16` lanes. The initial assembly
//! profile still rejects both classes before native lowering or export.
use super::Result;
use crate::support::initial_assembly::{COMMENT, KREG, OUTPUT, YMM, ZMM};
use std::fmt::Write as _;
/// The `pure` option's profile text.
const PURE: &str = "inline assembly pure option is outside the initial assembly profile";
const PACKET: &str = "pa41/tests/spec/asm-avx512";
/// rustc's typing rejections, each with this checker's reason.
const REJECTED: [(&str, &str); 11] = [
    ("zmm-no-avx512", "requires the avx512f target feature"),
    ("kreg-no-avx512", "requires the avx512f target feature"),
    ("kreg-u32-no-bw", "requires the avx512bw target feature"),
    ("kreg-f32", "incompatible with register class"),
    ("kreg-vector", "incompatible with register class"),
    ("kreg-modifier", "not supported by its class"),
    ("zmm-wide", "128, 256 or 512 bits"),
    ("zmm-short", "128, 256 or 512 bits"),
    ("zmm-modifier-e", "not supported by its class"),
    ("zmm-scalar-u8", "incompatible with register class"),
    ("xmm-f16x16", "exactly 128 bits"),
];
/// Accepted forms and the profile text that stops their lowering: options
/// come first, then each operand's output and class in order, and comments
/// last.
const ACCEPTED: [(&str, &str); 15] = [
    ("stream-ps512", ZMM),
    ("mask-load-ss", PURE),
    ("mask-load-sh", PURE),
    ("mask-store-sh", KREG),
    ("zmm-i64x8", ZMM),
    ("zmm-i32x4", ZMM),
    ("zmm-f32x8", ZMM),
    ("zmm-scalar-f64", ZMM),
    ("zmm-modifier-y", ZMM),
    ("zmm-output", OUTPUT),
    ("kreg-u16", KREG),
    ("kreg-u64-bw", KREG),
    ("kreg-fp16-implies", KREG),
    // Its eight `f16` lanes pass the class; its `/* */` reaches the comment fence.
    ("xmm-f16x8", COMMENT),
    ("ymm-f16x16", YMM),
];

/// A std-form case after `prefix`: crate attributes and the `asm` import are
/// dropped, `asm!` is named as `call`, and each derived vector Clone/Copy
/// becomes explicit. A case with `f16` lanes enables `f16` and its Clone/Copy.
fn case_text(name: &str, mut text: String, call: &str) -> Result<String> {
    let input = crate::test_source::fixture(&format!("{PACKET}/{name}.rs"))?;
    if input.contains("f16") {
        if !text.contains("#![feature(") {
            return Err("f16 case prefix has no feature list".into());
        }
        text = text.replacen("#![feature(", "#![feature(f16,", 1);
        text.push_str("impl Clone for f16{fn clone(&self)->Self{*self}}impl Copy for f16{}");
    }
    let mut vectors = Vec::new();
    for line in input
        .lines()
        .filter(|line| !line.starts_with("#!") && !line.starts_with("use core::arch::asm;"))
    {
        if let Some(rest) = line.split("struct ").nth(1) {
            vectors.push(rest.split('(').next().ok_or("vector name")?.to_owned());
        }
        text.push_str(
            &line
                .replace("#[derive(Clone,Copy)]", "")
                .replace("asm!(", call),
        );
        text.push('\n');
    }
    for vector in vectors {
        write!(
            text,
            "impl Clone for {vector}{{fn clone(&self)->Self{{*self}}}}impl Copy for {vector}{{}}"
        )?;
    }
    Ok(text)
}

fn case_source(name: &str) -> Result<String> {
    let mut prefix = super::vectors::foundation()?;
    prefix.push_str("#[rustc_builtin_macro(concat)]macro_rules!concat{($($t:tt)*)=>{};}");
    case_text(name, prefix, "asm!(")
}

/// The function holding a case's asm: core's intrinsic or `f`.
fn entry(name: &str) -> Result<String> {
    let input = crate::test_source::fixture(&format!("{PACKET}/{name}.rs"))?;
    Ok(input
        .split("pub unsafe fn ")
        .nth(1)
        .and_then(|rest| rest.split('(').next())
        .unwrap_or("f")
        .to_owned())
}

#[test]
fn avx512_operands_type_like_rustc_and_stop_at_the_assembly_profile() -> Result {
    let mut failures = Vec::new();
    for (name, reason) in REJECTED {
        match super::ymm::check(&case_source(name)?, &entry(name)?)? {
            (Err(error), _) if error.to_string().contains(reason) => {}
            (typed, _) => failures.push(format!("{name}: typing {typed:?}")),
        }
    }
    for (name, profile) in ACCEPTED {
        match super::ymm::check(&case_source(name)?, &entry(name)?)? {
            (Ok(()), Err(error)) if error.to_string().contains(profile) => {}
            outcome => failures.push(format!("{name}: {outcome:?}")),
        }
    }
    // Module-body export applies the same profile, so no published body
    // carries a zmm operand for a consumer to re-prove. `kreg` shares the
    // profile's class table; core's kreg forms stop earlier at export, at
    // their mask type alias.
    let mut prefix = super::lowering::native_prefix("#[repr(simd)]")?;
    prefix.push_str("impl<T:Copy,const N:usize>Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize>Copy for [T;N]{}");
    match crate::module_packets::exported_profile(
        &case_text("stream-ps512", prefix, "crate::arch::asm!(")?,
        crate::lower::instances::SourcePublication::ModuleBodies,
        "avx512_export",
    ) {
        Err(error) if error.to_string().contains(ZMM) => {}
        outcome => failures.push(format!("stream-ps512 export: {:?}", outcome.err())),
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
