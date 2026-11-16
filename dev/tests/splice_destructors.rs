//! A `Drop` impl's target is its nominal at the impl's own arguments, and the
//! nominal's requirements are instantiated there before the impl is checked
//! against them: the impl may assume what the struct implies and may add
//! nothing. When a field names a projection, as alloc's
//! `struct Splice<'a, I: Iterator + 'a, A> { drain: Drain<'a, I::Item, A>, .. }`
//! does, the struct also implies `I::Item: 'a`. That requirement says nothing
//! about a region or parameter, so nothing is assumed from it. An impl clause
//! bounding a projection's own lifetime, which rustc accepts here through
//! the struct's inferred `I::Item: 'a`, still fails closed and is no case.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --cap-lints=allow`). It accepts `projected-field-drop` and
//! `projected-field-named-drop` and rejects `projected-field-extra-bound` and
//! `projected-field-static-drop` with E0367. `RustGM`'s message for
//! `projected-field-extra-bound` was pinned after its first run; the one for
//! `projected-field-static-drop` was taken from the checker before it ran.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const SPLICE: &str = "pub struct Drain<'a, T> {\n    slice: &'a [T],\n}\npub struct Splice<'a, I: Iterator + 'a> {\n    drain: Drain<'a, I::Item>,\n    replace_with: I,\n}\n";

/// One `Drop` impl after `SPLICE` and its rejection, or `None` when it is
/// accepted.
struct Case {
    name: &'static str,
    destructor: &'static str,
    rejected: Option<&'static str>,
}
const CASES: [Case; 4] = [
    Case {
        name: "projected-field-drop",
        destructor: "impl<I: Iterator> Drop for Splice<'_, I> {\n    fn drop(&mut self) {}\n}\n",
        rejected: None,
    },
    Case {
        name: "projected-field-named-drop",
        destructor: "impl<'a, I: Iterator> Drop for Splice<'a, I> {\n    fn drop(&mut self) {}\n}\n",
        rejected: None,
    },
    Case {
        name: "projected-field-extra-bound",
        destructor: "impl<I: Iterator> Drop for Splice<'_, I>\nwhere\n    I::Item: Copy,\n{\n    fn drop(&mut self) {}\n}\n",
        rejected: Some(EXTRA_BOUND),
    },
    Case {
        name: "projected-field-static-drop",
        destructor: "impl<'a, I: Iterator + 'static> Drop for Splice<'a, I> {\n    fn drop(&mut self) {}\n}\n",
        rejected: Some(STATIC_BOUND),
    },
];
const EXTRA_BOUND: &str = "Drop impl adds type requirements to its nominal type";
const STATIC_BOUND: &str = "Drop impl adds lifetime requirements to its nominal type";

fn rejects(stderr: &str, message: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == message)
    })
}

/// The projected requirements of a `Drop` target are checked at the impl's
/// own arguments, which may add no requirement of their own.
#[test]
fn drop_targets_check_projected_requirements_at_their_arguments() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/splice-destructors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for case in CASES {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(&path, format!("{SPLICE}{}", case.destructor))?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
            .arg(&path)
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let matches = match case.rejected {
            None => output.status.success(),
            Some(message) => !output.status.success() && rejects(&stderr, message),
        };
        if !matches {
            failures.push(format!(
                "{}: success={} {stderr}",
                case.name,
                output.status.success()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
