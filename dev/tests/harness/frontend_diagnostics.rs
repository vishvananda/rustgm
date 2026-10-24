//! Successful compiler diagnostics require literal, reviewed bytes and typed warning codes.
use std::process::Output;
pub(crate) fn accepted(output: &Output, expected: &[u8]) -> Result<(), &'static str> {
    if !output.status.success() || output.stdout.is_empty() {
        return Err("successful LowIR emission requires exit zero and nonempty stdout");
    }
    if output.stderr != expected {
        return Err("successful compiler stderr differs from its declared warning contract");
    }
    if expected.is_empty() {
        return Ok(());
    }
    let text = std::str::from_utf8(expected).map_err(|_| "compiler warnings must be UTF-8")?;
    let mut lines = text.lines();
    if !lines.next().is_some_and(header) {
        return Err("successful compiler stderr requires a reviewed warning code");
    }
    for line in text.lines() {
        if line.starts_with("ERROR:") || line.starts_with("error:") || line.starts_with("error[") {
            return Err("an error cannot masquerade as a successful compiler warning");
        }
        if line.starts_with("warning[") && !header(line) {
            return Err("compiler warning code was not reviewed");
        }
    }
    Ok(())
}
fn header(line: &str) -> bool {
    [
        "warning[unused_must_use]: file ",
        "warning[unused_attributes]: file ",
        "warning[duplicate_macro_attributes]: file ",
    ]
    .into_iter()
    .any(|prefix| {
        line.strip_prefix(prefix)
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(file, rest)| {
                !file.is_empty()
                    && file.bytes().all(|byte| byte.is_ascii_digit())
                    && rest.split_once(": ").is_some_and(|(offset, message)| {
                        !offset.is_empty()
                            && offset.bytes().all(|byte| byte.is_ascii_digit())
                            && !message.is_empty()
                    })
            })
    })
}
