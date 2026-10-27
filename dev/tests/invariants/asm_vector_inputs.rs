//! The real vector operand reaches ordinary proof without an invented native operation.
use super::{Result, source};
use crate::{body, pipeline, types::aggregate::SimdShape};
use std::fmt::Write as _;
fn foundation() -> Result<String> {
    let mut text = source("")?.replacen("#![feature(", "#![feature(repr_simd,", 1);
    text.push_str("#[rustc_builtin_macro(asm)]macro_rules!asm{($($t:tt)*)=>{};}");
    for primitive in [
        "i8", "u16", "i16", "u32", "i32", "u64", "i64", "f32", "f64", "bool", "u128",
    ] {
        write!(
            text,
            "impl Clone for {primitive}{{fn clone(&self)->Self{{*self}}}}impl Copy for {primitive}{{}}"
        )?;
    }
    text.push_str("impl<T>Clone for *const T{fn clone(&self)->Self{*self}}impl<T>Copy for *const T{}impl<T>Clone for *mut T{fn clone(&self)->Self{*self}}impl<T>Copy for *mut T{}");
    text.push_str("impl<T:Copy,const N:usize>Clone for [T;N]{fn clone(&self)->Self{*self}}impl<T:Copy,const N:usize>Copy for [T;N]{}");
    Ok(text)
}
fn vector_source(name: &str) -> Result<String> {
    let input = crate::test_source::fixture(&format!("pa41/tests/spec/asm-vectors/{name}.rs"))?;
    let copy = input.contains("#[derive(Clone,Copy)]");
    let mut text = foundation()?;
    for line in input
        .lines()
        .filter(|line| !line.starts_with("#!") && !line.starts_with("use core::arch::asm;"))
    {
        text.push_str(&line.replace("#[derive(Clone,Copy)]", ""));
        text.push('\n');
    }
    if copy {
        text.push_str("impl Clone for V{fn clone(&self)->Self{*self}}impl Copy for V{}");
    }
    Ok(text)
}
fn prove(text: &str) -> Result {
    let report = crate::test_source::source_provider(text, "vector_asm")?;
    let mut analysis = pipeline::analyze_source(&report)?;
    crate::lower::instances::inspect_prepared(
        &report,
        &mut analysis.facts,
        analysis.prepared,
        |_, _| (),
    )?;
    Ok(())
}
#[test]
fn numeric_widths_and_nonvectors_keep_register_type_rules() -> Result {
    for lane in [
        "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "f32", "f64",
    ] {
        prove(&vector_source(&format!("numeric-{lane}"))?).map_err(|e| format!("{lane}: {e}"))?;
    }
    for (name, message) in [
        ("short", "exactly 128 bits"),
        ("padded", "exactly 128 bits"),
        ("large", "exactly 128 bits"),
        ("pointer", "numeric lanes"),
        ("wide-lane", "numeric lanes"),
        ("wrapper", "genuine representation"),
        ("general-class", "incompatible with register class"),
        ("ymm-no-feature", "register class is not implemented"),
        ("ymm-body-feature", "register class is not implemented"),
    ] {
        let error = prove(&vector_source(name)?)
            .err()
            .ok_or("invalid vector accepted")?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    prove(&vector_source("explicit")?)?;
    Ok(())
}
#[test]
fn vector_outputs_and_inputs_retain_place_copy_and_initialization_obligations() -> Result {
    prove(&vector_source("inout")?)?;
    prove(&vector_source("output-copy")?)?;
    for (name, message) in [
        ("uninitialized", "initialized"),
        ("unsafe", "unsafe context"),
        ("immutable-output", "immutable"),
        ("tied-mismatch", "same type"),
        ("noncopy", "Copy"),
        ("inout-noncopy", "Copy"),
        ("output-noncopy", "Copy"),
    ] {
        let error = prove(&vector_source(name)?)
            .err()
            .ok_or_else(|| format!("{name}: ordinary vector obligation skipped"))?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn real_stream_body_keeps_simd_types_proofs_and_capability_fences() -> Result {
    let reducer = crate::test_source::fixture("pa41/tests/spec/asm-frontend/stream-ps.rs")?;
    let original = reducer
        .split("#[inline]")
        .nth(1)
        .ok_or("original source function")?;
    let mut text = foundation()?.replacen("#![feature(", "#![feature(decl_macro,staged_api,", 1);
    text.insert_str(0, "#![unstable(feature=\"asm_fixture\",issue=\"none\")]\n");
    text = text.replace(
        "pub const trait",
        "#[rustc_const_unstable(feature=\"asm_fixture\",issue=\"none\")]pub const trait",
    );
    text.push_str("#[rustc_builtin_macro(concat)]macro_rules!concat{($($t:tt)*)=>{};}");
    text.push_str(&reducer.lines().skip(2).collect::<Vec<_>>().join("\n"));
    text.push('\n');
    text.push_str("impl Clone for __m128{fn clone(&self)->Self{*self}}impl Copy for __m128{}");
    assert!(text.contains(original));
    let report = crate::test_source::source_provider(&text, "actual_stream_asm")?;
    let mut analysis = pipeline::analyze_source(&report)?;
    let program = body::build_concrete(&report, &analysis.facts)?;
    let checked = program
        .bodies
        .iter()
        .find(|b| b.assembly.is_some())
        .ok_or("genuine stream body")?;
    super::super::validate(checked)?;
    let data = checked.assembly.as_ref().ok_or("real asm data")?;
    assert_eq!(data.operands.len(), 2);
    assert_eq!(data.templates[0].text, "movntps [{p}],{a}");
    let ty = data.operands[1].ty.ok_or("real SIMD operand type")?;
    assert!(matches!(
        analysis
            .facts
            .aggregates
            .simd_shape(&analysis.facts.types, ty)?,
        SimdShape::Ready { count: 4, .. }
    ));
    assert_eq!(
        analysis.facts.layouts.get(ty).ok_or("real layout")?.size,
        16
    );
    assert_eq!(
        data.instructions[0].options,
        crate::syntax::assembly::NOSTACK | crate::syntax::assembly::PRESERVES_FLAGS
    );
    crate::lower::instances::inspect_prepared(
        &report,
        &mut analysis.facts,
        analysis.prepared,
        |_, _| (),
    )?;
    let mut fresh = pipeline::analyze_source(&report)?;
    let root = fresh
        .facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "_mm_stream_ps"
        })
        .ok_or("actual original function")?
        .definition;
    let error = crate::lower::instances::build_roots_prepared(
        &report,
        &mut fresh.facts,
        crate::lower::Roots {
            entry: None,
            exports: &[root],
        },
        fresh.prepared,
    )
    .err()
    .ok_or("asm backend capability silently granted")?;
    assert!(
        error
            .to_string()
            .contains("unavailable locked backend capability"),
        "{error}"
    );
    Ok(())
}
