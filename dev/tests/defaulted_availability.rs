//! A staged library may mark a type parameter that has a default with its
//! own availability, as alloc's collections mark
//! `#[unstable(feature = "allocator_api", issue = "32838")] A: Allocator = Global`.
//! rustc consults it only where another crate writes an explicit argument for
//! the parameter; its own crate may write one freely. Every other generic
//! parameter is no stability target.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --cap-lints=allow`). It accepts `defaulted-type-parameter`,
//! `defaulted-stable-parameter` and `local-explicit-argument`. It rejects
//! `undefaulted-type-parameter`, `defaulted-const-parameter` and
//! `lifetime-parameter` with "`#[unstable]` attribute cannot be used on" type,
//! const or lifetime parameters, `const-stability-on-parameter` with
//! "`#[rustc_const_unstable]` attribute cannot be used on type parameters"
//! and `unstaged-parameter` with E0658.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const STAGED: &str =
    "#![feature(staged_api)]\n#![stable(feature = \"fixture_root\", since = \"1.0.0\")]\n";
const STABLE: &str = "#[stable(feature = \"fixture_root\", since = \"1.0.0\")]\n";
const HOLDER: &str = "pub struct Holder<T, #[unstable(feature = \"fixture_store\", issue = \"none\")] A = u8>(T, A);\n";
const TARGET: &str =
    "library stability on a generic parameter requires a type parameter with a default";

/// One ordinary crate and its rejection, or `None` when it is accepted.
struct Case {
    name: &'static str,
    items: &'static [&'static str],
    rejected: Option<&'static str>,
}
const CASES: [Case; 8] = [
    Case {
        name: "defaulted-type-parameter",
        items: &[STAGED, STABLE, HOLDER],
        rejected: None,
    },
    Case {
        name: "defaulted-stable-parameter",
        items: &[
            STAGED,
            STABLE,
            "pub struct Holder<T, #[stable(feature = \"fixture_root\", since = \"1.0.0\")] A = u8>(T, A);\n",
        ],
        rejected: None,
    },
    Case {
        name: "local-explicit-argument",
        items: &[
            STAGED,
            STABLE,
            HOLDER,
            STABLE,
            "pub fn make() -> Holder<u8, u16> {\n    Holder(1, 2)\n}\n",
        ],
        rejected: None,
    },
    Case {
        name: "undefaulted-type-parameter",
        items: &[
            STAGED,
            STABLE,
            "pub struct Bare<#[unstable(feature = \"fixture_store\", issue = \"none\")] T>(T);\n",
        ],
        rejected: Some(TARGET),
    },
    Case {
        name: "defaulted-const-parameter",
        items: &[
            STAGED,
            STABLE,
            "pub struct Count<#[unstable(feature = \"fixture_store\", issue = \"none\")] const N: usize = 3>;\n",
        ],
        rejected: Some(TARGET),
    },
    Case {
        name: "lifetime-parameter",
        items: &[
            STAGED,
            STABLE,
            "pub struct Borrowed<#[unstable(feature = \"fixture_store\", issue = \"none\")] 'a>(&'a u8);\n",
        ],
        rejected: Some(TARGET),
    },
    Case {
        name: "const-stability-on-parameter",
        items: &[
            STAGED,
            STABLE,
            "pub struct Holder<T, #[rustc_const_unstable(feature = \"fixture_const\", issue = \"none\")] A = u8>(T, A);\n",
        ],
        rejected: Some("const library stability is not supported on generic parameters"),
    },
    Case {
        name: "unstaged-parameter",
        items: &[HOLDER],
        rejected: Some("library stability requires staged_api"),
    },
];

fn rejects(stderr: &str, message: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == message)
    })
}

/// Availability on a defaulted type parameter is accepted and leaves its own
/// crate's explicit arguments free; on any other parameter it is rejected.
#[test]
fn only_defaulted_type_parameters_carry_availability() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/defaulted-availability")
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
