//! Checked asm bodies lower to `LowIR` templates in the initial assembly
//! profile. Every other checked form keeps its own declared fence text, and
//! asm-free programs allocate no template storage.
use super::vectors::{fixture_source, foundation};
use crate::{lower::instances::SourcePublication, pipeline, resolve::model::DefId};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SPEC: &str = "pa41/tests/spec/asm-native";
/// The crate holding the real stream body. B1 observed it stopping at the
/// crate-wide macro provenance fence, since `vps!` and `concat!` expand inside
/// it. B2's macro-context slice admits those semi-opaque marks, and its
/// const-parameter slice lifts the const-formal gate, so it stops at
/// `__m128`'s `repr(simd)` representation, declared before the run.
const STREAM_OBSERVED: &str = "source nominal representation requires its wider complete contract";
/// The provider function whose `LowIR` the control demands.
fn root(
    report: &crate::resolve::expansion::Report,
    facts: &crate::sema::model::Declarations,
    name: &str,
) -> Result<DefId> {
    Ok(facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == name
        })
        .ok_or("demanded asm function")?
        .definition)
}
/// Lower `name` as the only export and return the written `LowIR`.
fn lowered(text: &str, tag: &str, name: &str) -> Result<(String, Option<usize>)> {
    let report = crate::test_source::source_provider(text, tag)?;
    let mut analysis = pipeline::analyze_source(&report)?;
    let export = root(&report, &analysis.facts, name)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut analysis.facts,
        crate::lower::Roots {
            entry: None,
            exports: &[export],
        },
        analysis.prepared,
    )?;
    let mut bytes = Vec::new();
    crate::lowir::writer::write(&program, &mut bytes)?;
    Ok((String::from_utf8(bytes)?, program.assembly_templates()))
}
/// Each `assembly` block of written `LowIR`, through its closing brace.
fn templates(lowir: &str) -> Vec<&str> {
    lowir
        .match_indices("assembly @asm")
        .filter_map(|(start, _)| {
            lowir[start..]
                .find("\n}\n")
                .map(|end| &lowir[start..start + end + 3])
        })
        .collect()
}
#[test]
fn every_form_outside_the_profile_keeps_its_declared_fence() -> Result {
    let cases = crate::test_source::fixture(&format!("{SPEC}/opponents.tsv"))?;
    let mut count = 0;
    for line in cases.lines().filter(|line| !line.starts_with('#')) {
        let fields: Vec<_> = line.split('\t').collect();
        let &[path, status, expected] = &fields[..] else {
            return Err(format!("opponent row: {line}").into());
        };
        assert_eq!(status, "0", "{path}: rustc must accept every opponent");
        let text = fixture_source(&format!("{SPEC}/{path}"))?;
        let error = lowered(&text, "asm_form_fence", "f")
            .err()
            .ok_or_else(|| format!("{path}: unsupported form lowered"))?;
        assert!(error.to_string().contains(expected), "{path}: {error}");
        count += 1;
    }
    assert_eq!(count, 16);
    Ok(())
}
#[test]
fn joined_templates_unescape_braces_and_keep_one_newline() -> Result {
    let text = fixture_source(&format!("{SPEC}/opponents/joined.rs"))?;
    let (lowir, count) = lowered(&text, "asm_joined", "f")?;
    assert_eq!(count, Some(1));
    let found = templates(&lowir);
    let &[template] = &found[..] else {
        return Err("one joined template".into());
    };
    assert!(template.starts_with("assembly @asm0 intel readwrite 1 24 0 1 {\n"));
    assert!(template.ends_with("  text \"/* { \"\n  use 0 0 0 0\n  text \" } */\\nnop\"\n}\n"));
    assert_eq!(lowir.matches("asm @asm0(").count(), 1);
    Ok(())
}
/// `native.rs` before `marker`: its lang-item prelude and the source-owned
/// `arch::asm` and `concat!` declarations, without the test foundation's
/// const-generic array impls.
fn native_prefix(marker: &str) -> Result<String> {
    let native = crate::test_source::fixture(&format!("{SPEC}/native.rs"))?;
    let end = native.find(marker).ok_or("native.rs section")?;
    Ok(native[..end].to_owned())
}
/// The std-form pointer-only control on the native prelude, naming `asm!`
/// through its source-owned `arch` declaration.
fn transport_source() -> Result<String> {
    let mut text = native_prefix("#[repr(simd)]")?;
    let control = crate::test_source::fixture(&format!("{SPEC}/opponents/transport.rs"))?;
    for line in control
        .lines()
        .filter(|line| !line.starts_with("use core::arch::asm;"))
    {
        text.push_str(&line.replace("asm!(", "crate::arch::asm!("));
        text.push('\n');
    }
    Ok(text)
}
/// B2 slice A (amended 2026-11-09): the pointer-only control's body now
/// joins the v13 module-body publication as one assembly value and its
/// columns, copied verbatim from the lowered payload.
#[test]
fn inline_asm_bodies_join_v13_module_body_transport() -> Result {
    use crate::support::crates::{self, generics::source as s};
    let text = transport_source()?;
    let (lowir, count) = lowered(&text, "asm_transport_lowered", "f")?;
    assert_eq!(count, Some(1));
    assert!(lowir.contains("  text \"prefetchnta [\"\n  use 0 0 0 0\n  text \"]\"\n"));
    let interface = crate::module_packets::exported_profile(
        &text,
        SourcePublication::ModuleBodies,
        "asm_transport",
    )?;
    let decoded = crates::decode(&crates::encode(&interface)?)?;
    let (rows, _, _) = crate::module_packets::raw(&decoded)?;
    let texts: Vec<&str> = rows
        .originals
        .iter()
        .filter_map(|original| original.assembly.as_deref())
        .flat_map(|columns| columns.templates.iter().map(|template| &*template.text))
        .collect();
    assert_eq!(texts, ["prefetchnta [{p}]"]);
    assert!(rows.originals.iter().any(|original| {
        original
            .values
            .iter()
            .any(|value| matches!(value.kind, s::ValueKind::Assembly { instruction: 0, .. }))
    }));
    crate::module_packets::exported_profile(
        &native_prefix("#[repr(simd)]")?,
        SourcePublication::ModuleBodies,
        "asm_transport_prelude",
    )?;
    let stream = crate::module_packets::exported_profile(
        &native_prefix("// Authored observers")?,
        SourcePublication::ModuleBodies,
        "asm_stream_transport",
    )
    .err()
    .ok_or("v13 publication exported the stream asm body")?;
    assert!(stream.to_string().contains(STREAM_OBSERVED), "{stream}");
    Ok(())
}
/// A generic body instantiated for `n` distinct types gives exactly `n`
/// templates; `n` asm-free functions allocate no template storage.
fn scaled(n: usize, asm: bool) -> Result<(usize, Option<usize>)> {
    let mut text = foundation()?;
    let mut calls = String::new();
    for index in 0..n {
        write!(text, "pub struct T{index:02};")?;
        write!(calls, "tagged::<T{index:02}>(p);")?;
    }
    let body = if asm {
        "unsafe{asm!(\"prefetchnta [{p}]\",p=in(reg)p,options(nostack,preserves_flags));}"
    } else {
        ""
    };
    write!(
        text,
        "fn tagged<T>(p:*const u8){{{body}}}pub fn f(p:*const u8){{{calls}}}"
    )?;
    let (lowir, count) = lowered(&text, "asm_scaling", "f")?;
    Ok((lowir.matches("\n    asm @asm").count(), count))
}
#[test]
fn templates_scale_with_emitted_instances_and_asm_free_bodies_allocate_none() -> Result {
    for n in [1, 8, 32] {
        assert_eq!(scaled(n, true)?, (n, Some(n)), "{n} asm instances");
        assert_eq!(scaled(n, false)?, (0, None), "{n} asm-free instances");
    }
    Ok(())
}
