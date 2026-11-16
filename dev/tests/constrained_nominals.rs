//! A nominal's parameter that no field uses is still used when a
//! projection-equality clause of the nominal constrains it from parameters
//! that are: rustc's `identify_constrained_generic_params`, which accepts
//! alloc's `struct DedupSortedIter<K, V, I> where I: Iterator<Item = (K, V)>`.
//! A parameter that appears only in a bound, in a clause whose projection
//! mentions an unconstrained parameter, or inside another projection is
//! still never used.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --cap-lints=allow`). It accepts `projection-constrained-types`,
//! `projection-constrained-lifetime` and `projection-chain`, and rejects
//! `unconstrained-input`, `bound-only-parameter` and `projection-input-only`
//! with E0392.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const UNUSED: &str = "nominal lifetime parameter is unused or only used recursively";

/// One ordinary crate and whether it is accepted.
struct Case {
    name: &'static str,
    source: &'static str,
    accepted: bool,
}
const CASES: [Case; 6] = [
    Case {
        name: "projection-constrained-types",
        source: "pub struct DedupSortedIter<K, V, I>\nwhere\n    I: Iterator<Item = (K, V)>,\n{\n    iter: I,\n}\n",
        accepted: true,
    },
    Case {
        name: "projection-constrained-lifetime",
        source: "pub struct Borrowed<'a, I>\nwhere\n    I: Iterator<Item = &'a u8>,\n{\n    iter: I,\n}\n",
        accepted: true,
    },
    Case {
        name: "projection-chain",
        source: "pub struct Chain<A, B, I>\nwhere\n    I: Iterator<Item = A>,\n    A: Iterator<Item = B>,\n{\n    iter: I,\n}\n",
        accepted: true,
    },
    Case {
        name: "unconstrained-input",
        source: "pub struct Loose<K, I>\nwhere\n    K: Iterator<Item = I>,\n{\n    value: u8,\n}\n",
        accepted: false,
    },
    Case {
        name: "bound-only-parameter",
        source: "pub struct Bounded<T>\nwhere\n    T: Clone,\n{\n    value: u8,\n}\n",
        accepted: false,
    },
    Case {
        name: "projection-input-only",
        source: "pub struct Inside<K, I>\nwhere\n    K: Iterator,\n    I: Iterator<Item = <K as Iterator>::Item>,\n{\n    iter: I,\n}\n",
        accepted: false,
    },
];

/// A projection clause constrains its result's parameters from its own
/// constrained inputs, to a fixpoint; nothing else rescues an unused one.
#[test]
fn projection_clauses_constrain_otherwise_unused_parameters() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/constrained-nominals")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for case in CASES {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(&path, case.source)?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
            .arg(&path)
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let matches = if case.accepted {
            output.status.success()
        } else {
            !output.status.success()
                && stderr.lines().any(|line| {
                    line.strip_prefix("ERROR: ")
                        .and_then(|rest| rest.rsplit_once(": "))
                        .is_some_and(|(_, message)| message == UNUSED)
                })
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
