use std::{
    io::Write as _,
    process::{Command, Output, Stdio},
};
fn run(mode: &str, source: &str) -> std::io::Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .env_clear()
        .args([mode, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("missing stdin"))?
        .write_all(source.as_bytes())?;
    child.wait_with_output()
}
#[test]
fn type_view_is_deterministic_and_count_uses_the_same_semantics() -> std::io::Result<()> {
    let source = "macro_rules! m{()=>{2}} fn f()->u8{m!()}";
    let first = run("--emit-types", source)?;
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(first.stderr.is_empty());
    let second = run("--emit-types", source)?;
    assert_eq!(first.stdout, second.stdout);
    assert!(second.status.success());
    let view = String::from_utf8_lossy(&first.stdout);
    assert!(
        view.starts_with("typed-crate 1\ntype 0 never\ntype 1 tuple []\ntype 2 primitive bool\n")
    );
    assert!(view.contains("type 11 primitive u8\n"));
    assert!(view.contains("integer bits=0x00000000000000000000000000000002\n"));
    let count = run("--count-types", source)?;
    assert!(count.status.success());
    assert!(count.stderr.is_empty());
    let footer = view
        .lines()
        .last()
        .and_then(|line| line.strip_prefix("eof "))
        .ok_or_else(|| std::io::Error::other("missing footer"))?;
    assert_eq!(
        String::from_utf8_lossy(&count.stdout),
        format!("{footer}\n")
    );
    Ok(())
}
#[test]
fn type_errors_and_occurs_checks_emit_no_partial_view() -> std::io::Result<()> {
    for source in [
        "fn f()->u8{false}",
        "fn f()->u8{256}",
        "type A=A;",
        "fn f(){let x;}",
        "fn f(){let g:fn(_);g(g);}",
        "macro_rules! m{()=>{false}} fn f()->u8{m!()}",
    ] {
        for mode in ["--emit-types", "--count-types"] {
            let result = run(mode, source)?;
            assert_eq!(result.status.code(), Some(1), "{source}");
            assert!(result.stdout.is_empty());
            assert!(result.stderr.starts_with(b"ERROR: file "));
        }
    }
    let result = run("--emit-types", "fn f(){let g:fn(_);g(g);}")?;
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("type variable occurs in its own type")
    );
    Ok(())
}

#[test]
fn inferred_function_items_render_all_argument_kinds() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{Primitive, TypeId};
    let source = "fn f<T,const N:usize>(x:[T;N])->[T;N]{x}fn run()->[u8;3]{f([1,2,3])}";
    let output = run("--emit-types", source)?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let view = String::from_utf8(output.stdout)?;
    let expected = format!(
        "arguments=[type:{},const:{}:0x00000000000000000000000000000003]",
        TypeId::primitive(Primitive::U8).0,
        TypeId::primitive(Primitive::Usize).0
    );
    assert!(view.contains(&expected), "{view}");
    let repeat = run("--emit-types", source)?;
    assert_eq!(repeat.stdout, view.as_bytes());
    assert_eq!(repeat.status.code(), Some(0));
    let counts = run("--count-types", source)?;
    assert_eq!(counts.status.code(), Some(0));
    let footer = view
        .lines()
        .last()
        .and_then(|s| s.strip_prefix("eof "))
        .ok_or("missing type footer")?;
    assert_eq!(String::from_utf8(counts.stdout)?, format!("{footer}\n"));
    Ok(())
}
