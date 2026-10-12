use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{
        bodies, declarations,
        model::{AdjustmentKind, Cast, Declarations, Operation, Scalar},
    },
    support::source::SourceFile,
    types::{Primitive, TypeId},
};
use std::{
    io::Write as _,
    path::Path,
    process::{Command, Stdio},
};
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn analyze(source: &str) -> Result<(Report, Declarations), String> {
    let report = resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .and_then(resolve::analyze_expanded)
    .map_err(|e| e.to_string())?;
    let declarations = declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = bodies::analyze(&report, declarations).map_err(|e| e.to_string())?;
    Ok((report, facts))
}
fn definition(report: &Report, facts: &Declarations, name: &str) -> Result<TypeId, String> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| report.package.names.get(d.name) == name)
        .and_then(|(i, _)| facts.definitions[i])
        .ok_or_else(|| format!("missing type for {name}"))
}
#[test]
fn body_constraints_delay_fallback_and_retain_selected_operations() -> Result<(), String> {
    let (report, facts) = analyze(
        "fn pass(x:u8)->u8{x} fn f()->u8{let a=17; let b=pass(a); let default_integer=1; let default_float=1.0; b+1}",
    )?;
    for name in ["a", "b"] {
        assert_eq!(
            definition(&report, &facts, name)?,
            TypeId::primitive(Primitive::U8)
        );
    }
    assert_eq!(
        definition(&report, &facts, "default_integer")?,
        TypeId::primitive(Primitive::I32)
    );
    assert_eq!(
        definition(&report, &facts, "default_float")?,
        TypeId::primitive(Primitive::F64)
    );
    assert!(
        facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.operation, Some(Operation::Call { .. })))
    );
    assert!(
        facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.scalar, Some(Scalar::Integer(17))))
    );
    Ok(())
}
#[test]
fn casts_function_reification_and_minimum_literals_publish_lowering_facts() -> Result<(), String> {
    let (_, facts) = analyze(
        "type F=fn(u8)->u8; fn pass(x:u8)->u8{x} fn f()->i8{let callback:F=pass; let n=callback(1) as i16; -(128i8)}",
    )?;
    assert!(facts.nodes.iter().flatten().any(|n| matches!(
        n.operation,
        Some(Operation::Cast {
            kind: Cast::IntegerToInteger,
            ..
        })
    )));
    assert!(facts.nodes.iter().flatten().any(|n| {
        n.adjustment
            .is_some_and(|a| a.kind == AdjustmentKind::FunctionPointer)
    }));
    assert!(
        facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.scalar, Some(Scalar::Integer(128))))
    );
    assert!(
        facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.operation, Some(Operation::Unary { literal: true, .. })))
    );
    Ok(())
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "independent scalar acceptance matrix and pinned oracle invocation are reviewed together"
)]
fn scalar_body_acceptance_matches_pinned_rustc() -> Result<(), String> {
    let version = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| e.to_string())?;
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("rustc 1.98.0 "));
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa9-work/body-rustc")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let cases = [
        ("fn f()->u8{1+2}", true),
        ("fn f()->f32{1e100}", false),
        ("fn f()->f64{1e100}", true),
        ("fn f()->f64{1e1000}", false),
        (
            "fn f()->i128{-170141183460469231731687303715884105728}",
            true,
        ),
        (
            "fn f()->i128{170141183460469231731687303715884105728}",
            false,
        ),
        ("fn f(){let a=return;}", true),
        ("fn f()->u8{if true{return 1}else{return 2}}", true),
        ("fn f()->u8{let a=1;a}", true),
        ("fn f()->u8{let a=1;let b=a;b}", true),
        ("fn f(){let a=1;let b=2.0;}", true),
        ("fn f(){let a;}", false),
        ("fn f(){let a:_;}", false),
        ("fn f(){let a:u8;}", true),
        ("fn f()->u8{let a:i16=1;a}", false),
        ("fn f()->u8{1i16}", false),
        ("fn f()->u8{256}", false),
        ("fn f()->i8{128}", false),
        ("fn f()->i8{-128}", true),
        ("fn f()->i8{-(128)}", true),
        ("fn f()->i8{-129}", false),
        ("fn f()->u8{-1}", false),
        ("fn f()->u8{!1}", true),
        ("fn f()->bool{!false}", true),
        ("fn f()->f32{!1.0}", false),
        ("fn f()->bool{true+false}", false),
        ("fn f()->bool{true&false}", true),
        ("fn f()->bool{'a'<'b'}", true),
        ("fn f()->bool{false<true}", true),
        ("fn f()->bool{1u8==1u16}", false),
        ("fn f()->u8{1u8<<2u16}", true),
        ("fn f()->u8{1u8<<2.0}", false),
        ("fn f()->bool{true&&false}", true),
        ("fn f()->bool{true&&1}", false),
        ("fn f()->bool{true&&(return false)}", true),
        ("fn f()->u8{if true{1}else{2}}", true),
        ("fn f()->u8{if true{return 1}else{2}}", true),
        ("fn f()->u8{if true{1}else{return 2}}", true),
        ("fn f(){if true{1}}", false),
        ("fn f(){if true{return}}", true),
        ("fn f()->u8{return 1;}", true),
        ("fn f()->u8{return;}", false),
        ("fn f()->u8{return 1;false}", false),
        ("fn f()->u8{g(1)} fn g(x:u8)->u8{x}", true),
        ("fn f(){g();} fn g(x:u8){}", false),
        ("fn f(){g(1,2);} fn g(x:u8){}", false),
        ("fn f(){g(false);} fn g(x:u8){}", false),
        ("fn f()->u8{let h=g;h(2)} fn g(x:u8)->u8{x}", true),
        ("fn f()->u8{257u16 as u8}", true),
        ("fn f()->u8{1.5 as u8}", true),
        ("fn f()->f32{1u8 as f32}", true),
        ("fn f()->f32{true as f32}", false),
        ("fn f()->u8{true as u8}", true),
        ("fn f()->u32{'a' as u32}", true),
        ("fn f()->char{65u8 as char}", true),
        ("fn f()->char{65u32 as char}", false),
        ("fn f()->bool{1 as bool}", false),
        ("fn f()->u8{let _:u16=1;2}", true),
        ("fn f()->u8{#[cfg(any())] let x=missing;2}", true),
        ("macro_rules! m{()=>{1+2}} fn f()->u8{m!()}", true),
    ];
    for (index, (source, expected)) in cases.iter().enumerate() {
        let prefix = directory.join(index.to_string());
        std::fs::write(prefix.with_extension("rs"), source).map_err(|e| e.to_string())?;
        let mut child = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=body_check",
                "--emit=metadata",
                "-o",
            ])
            .arg(prefix.with_extension("rmeta"))
            .arg("-")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        child
            .stdin
            .take()
            .ok_or("missing rustc stdin")?
            .write_all(source.as_bytes())
            .map_err(|e| e.to_string())?;
        let result = child.wait_with_output().map_err(|e| e.to_string())?;
        std::fs::write(prefix.with_extension("stderr"), &result.stderr)
            .map_err(|e| e.to_string())?;
        assert_eq!(
            result.status.success(),
            *expected,
            "rustc case {index}: {source}\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let ours = analyze(source);
        assert_eq!(
            ours.is_ok(),
            *expected,
            "frontend case {index}: {source}\n{:?}",
            ours.err()
        );
    }
    Ok(())
}
