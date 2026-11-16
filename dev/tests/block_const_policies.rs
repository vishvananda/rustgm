//! A member of a `const impl` block without its own const stability inherits
//! the block's const-unstable policy, as a trait impl's members do, even when
//! its regular policy is unstable. Only a syntactically const fn takes an
//! unstable regular policy as its implicit const policy, which rustc subjects
//! to recursive const stability. alloc relies on this in the unstable
//! `raw_vec` module: `RawVecInner::with_capacity_in`, in a
//! `#[rustc_const_unstable(feature = "const_heap")] const impl` block, calls
//! the const-unstable `handle_error`.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --cap-lints=allow`). It accepts the three `*-unstable-*` cases and rejects
//! `unstable-module-unmarked-block`, `unstable-module-const-fn` and
//! `stable-module-unmarked-block` with "const function that might be
//! (indirectly) exposed to stable cannot use `#[feature(callee_api)]`".
//! `unstable-module-unstable-trait-impl` first used the obsolete
//! `impl const Read` syntax, which rustc rejects; it now writes
//! `const impl Read for Value` and keeps its declared acceptance.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const ROOT: &str = "#![feature(staged_api, const_trait_impl)]\n#![stable(feature = \"root_api\", since = \"1.0.0\")]\n#[rustc_const_unstable(feature = \"callee_api\", issue = \"none\")]\nconst fn callee() -> u8 {\n    7\n}\n";
const UNSTABLE_MODULE: &str =
    "mod internal {\n    #![unstable(feature = \"internal_api\", issue = \"none\")]\n";
const MODULE: &str = "mod internal {\n";
const VALUE: &str = "    pub(crate) struct Value;\n";
const BLOCK_POLICY: &str =
    "    #[rustc_const_unstable(feature = \"block_api\", issue = \"none\")]\n";
const INHERENT: &str = "    const impl Value {\n        fn read() -> u8 {\n            super::callee()\n        }\n    }\n}\n";
const FREE: &str = "    const fn read() -> u8 {\n        super::callee()\n    }\n}\n";
const TRAIT: &str = "    #[rustc_const_unstable(feature = \"block_api\", issue = \"none\")]\n    pub(crate) const trait Read {\n        fn read() -> u8;\n    }\n";
const TRAIT_IMPL: &str = "    const impl Read for Value {\n        fn read() -> u8 {\n            super::callee()\n        }\n    }\n}\n";
const EXPOSED: &str = "const-stable body calls a function without const-stable API policy";

/// One ordinary crate and whether it is accepted.
struct Case {
    name: &'static str,
    items: &'static [&'static str],
    accepted: bool,
}
const CASES: [Case; 6] = [
    Case {
        name: "unstable-module-unstable-block",
        items: &[ROOT, UNSTABLE_MODULE, VALUE, BLOCK_POLICY, INHERENT],
        accepted: true,
    },
    Case {
        name: "stable-module-unstable-block",
        items: &[ROOT, MODULE, VALUE, BLOCK_POLICY, INHERENT],
        accepted: true,
    },
    Case {
        name: "unstable-module-unstable-trait-impl",
        items: &[
            ROOT,
            UNSTABLE_MODULE,
            VALUE,
            TRAIT,
            BLOCK_POLICY,
            TRAIT_IMPL,
        ],
        accepted: true,
    },
    Case {
        name: "unstable-module-unmarked-block",
        items: &[ROOT, UNSTABLE_MODULE, VALUE, INHERENT],
        accepted: false,
    },
    Case {
        name: "unstable-module-const-fn",
        items: &[ROOT, UNSTABLE_MODULE, FREE],
        accepted: false,
    },
    Case {
        name: "stable-module-unmarked-block",
        items: &[ROOT, MODULE, VALUE, INHERENT],
        accepted: false,
    },
];

fn rejects(stderr: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == EXPOSED)
    })
}

/// A const block's const-unstable policy reaches its members whatever their
/// regular policy; an unmarked member or const fn stays recursively checked.
#[test]
fn const_block_members_inherit_its_const_unstable_policy() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/block-const-policies")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for case in CASES {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(&path, case.items.concat())?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
            .arg(&path)
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let matches = if case.accepted {
            output.status.success()
        } else {
            !output.status.success() && rejects(&stderr)
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
