//! A source core's own call traits select the slot of an object call.
use super::implemented_copies::{cases, run};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CALLS: &str = "pa31/tests/spec/source-object-calls";
/// A shared reference to a `dyn FnMut` cannot be called.
const REJECTED: &str = "cannot mutably borrow shared or static storage";

/// `&dyn Fn`, `&mut dyn FnMut` and `&&dyn Fn` calls lower from a root `main`.
#[test]
fn object_calls_use_the_call_traits_a_source_core_declares() -> Result {
    let mut failures = Vec::new();
    for (path, accepted) in cases(CALLS, 4)? {
        let output = run("--emit-lowir", &path)?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let expected = if accepted { Some(0) } else { Some(1) };
        if output.status.code() != expected || !(accepted || stderr.contains(REJECTED)) {
            failures.push(format!("{path}: {:?} {stderr}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
