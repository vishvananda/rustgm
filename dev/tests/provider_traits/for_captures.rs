//! Source iterator calls classify each surrounding closure use before lowering.
use super::TestResult;
use std::path::Path;

fn foundation() -> TestResult<String> {
    let source = crate::resolve::fixture("pa31/tests/spec/source-iteration/native-protocol.rs")?;
    let (iteration, _) = source
        .split_once("#[unsafe(no_mangle)]")
        .ok_or("iterator foundation")?;
    let source = crate::resolve::fixture("pa31/tests/spec/forwarded-calls/shared.rs")?;
    let (callable, _) = source
        .split_once("#[lang=\"destruct\"]")
        .and_then(|(_, rest)| rest.split_once("const impl<A:Tuple"))
        .ok_or("callable foundation")?;
    Ok(iteration.replace(
        "#![feature(auto_traits)]",
        "#![feature(auto_traits,unboxed_closures,const_destruct)]",
    ) + "#[lang=\"destruct\"]"
        + callable)
}

const MOVE: &str = "let values=Values{pending:true,value:7};
                    let task=||{let mut last=0;for n in values {last=n;}last};";
const BORROW: &str = "let mut last=0;
                      let mut task=||{for n in (Values{pending:true,value:7}){last=n;}};
                      task(); task(); last";

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn iterator_captures_preserve_moves_and_mutable_borrows() -> TestResult {
    let prefix = foundation()?;
    let good = format!(
        "{prefix}
         #[unsafe(no_mangle)]pub extern \"C\" fn consumed()->u8{{{MOVE}task()}}
         #[unsafe(no_mangle)]pub extern \"C\" fn mutably_borrowed()->u8{{{BORROW}}}"
    );
    let program = crate::ffi_program::exported(&good, "for_capture")?;
    let bad = format!(
        "{prefix}#[unsafe(no_mangle)]pub extern \"C\" fn twice()->u8{{{MOVE}task();task()}}"
    );
    let Err(error) = crate::ffi_program::exported(&bad, "for_capture_twice") else {
        return Err("accepted a second move of the iterator capture".into());
    };
    assert!(
        error.to_string().contains("local may be uninitialized"),
        "{error}"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let output = root.join("obj/parallel/source-iteration-v1/closure-native");
    std::fs::create_dir_all(&output)?;
    crate::ffi_program::check(
        &root,
        &output,
        "captures",
        &program,
        r#"
#include <stdint.h>
#include <stdio.h>
extern uint8_t consumed(void), mutably_borrowed(void);
int main(void) {
    printf("%u\n", (unsigned)consumed());
    printf("%u\n", (unsigned)mutably_borrowed());
    return 0;
}
"#,
        "7\n7\n",
    )
}
