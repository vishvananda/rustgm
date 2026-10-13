use std::{
    io::Write,
    process::{Command, Output, Stdio},
};
fn inspect(source: &str, count: bool) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(if count {
            "--count-layouts"
        } else {
            "--emit-layouts"
        })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing inspection stdin")?
        .write_all(source.as_bytes())?;
    Ok(child.wait_with_output()?)
}
fn type_for(output: &str, name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let needle = format!(" name=\"{name}\"");
    let definition = output
        .lines()
        .find(|line| line.starts_with("def ") && line.contains(&needle))
        .ok_or("missing inspected definition")?;
    Ok(definition
        .split_whitespace()
        .find_map(|part| part.strip_prefix("type="))
        .ok_or("missing inspected type")?
        .to_owned())
}
#[test]
fn layouts_are_deterministic_and_count_the_same_completed_queries()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "#[repr(C)]struct C{a:u8,b:u64,c:u16}type A=[C;3];enum Void{}#[repr(u8)]enum E{A=7,B=Self::A as u8+2}struct Node{next:*const Self}";
    let full = inspect(source, false)?;
    assert!(
        full.status.success(),
        "{}",
        String::from_utf8_lossy(&full.stderr)
    );
    assert!(full.stderr.is_empty());
    assert_eq!(full.stdout, inspect(source, false)?.stdout);
    let text = std::str::from_utf8(&full.stdout)?;
    assert!(text.starts_with("layouts 1 target=x86_64-unknown-linux-gnu\n"));
    let record = type_for(text, "C")?;
    assert!(text.contains(&format!(
        "layout {record} size=24 align=8 inhabited=true references=false aggregate tag=-\n"
    )));
    for (field, offset) in [0, 8, 16].into_iter().enumerate() {
        let prefix = format!("field {record}:0:{field} ");
        assert!(
            text.lines()
                .any(|line| line.starts_with(&prefix)
                    && line.ends_with(&format!(" offset={offset}")))
        );
    }
    let array = type_for(text, "A")?;
    assert!(text.contains(&format!("layout {array} size=72 align=8 inhabited=true references=false array element={record} stride=24 length=3\n")));
    let empty = type_for(text, "Void")?;
    assert!(text.contains(&format!(
        "layout {empty} size=0 align=1 inhabited=false references=false aggregate tag=-\n"
    )));
    let enum_ = type_for(text, "E")?;
    assert!(text.contains(&format!("variant {enum_}:1 discriminant=9\n")));
    let count = inspect(source, true)?;
    assert!(count.status.success());
    assert!(count.stderr.is_empty());
    assert_eq!(
        text.lines().last(),
        Some(format!("eof {}", std::str::from_utf8(&count.stdout)?.trim_end()).as_str())
    );
    Ok(())
}
#[test]
fn zero_sized_layouts_preserve_reference_footprints_without_expanding_arrays()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "type R=&'static u8;struct S([R;0]);type Huge=[();18446744073709551615];";
    let output = inspect(source, false)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::str::from_utf8(&output.stdout)?;
    let record = type_for(text, "S")?;
    assert!(text.contains(&format!(
        "layout {record} size=0 align=8 inhabited=true references=true aggregate"
    )));
    assert!(text.contains("stride=0 length=18446744073709551615"));
    assert!(text.len() < 10_000);
    Ok(())
}
#[test]
fn incomplete_layouts_never_emit_a_partial_table() -> Result<(), Box<dyn std::error::Error>> {
    for source in [
        "struct Bad(Bad);",
        "#[repr(transparent)]struct Bad(u8,u64);",
        "enum E{A=1/0}",
    ] {
        for count in [false, true] {
            let output = inspect(source, count)?;
            assert_eq!(output.status.code(), Some(1), "{source}");
            assert!(output.stdout.is_empty());
            assert!(output.stderr.starts_with(b"ERROR: "));
        }
    }
    Ok(())
}
