//! Source arithmetic selects its own `Rhs` before applying real operand coercions.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/operator-rhs";
const REJECTED: [(&str, &[&str]); 9] = [
    (
        "closed-missing-rhs.rs",
        &["associated type requires an unambiguous validated implementation"],
    ),
    ("missing-deref.rs", &["incompatible types"]),
    ("wrong-target.rs", &["incompatible types"]),
    ("bound-expects-original.rs", &["incompatible types"]),
    ("wrong-output.rs", &["incompatible types"]),
    (
        "missing-mutable-deref.rs",
        &["trait obligation requires an unambiguous matching assumption or implementation"],
    ),
    (
        "shared-cannot-be-mut.rs",
        &["cannot coerce a shared reference to a mutable reference"],
    ),
    (
        "moved-lhs.rs",
        &["already consumed", "uninitialized or moved"],
    ),
    (
        "static-escape.rs",
        &[
            "does not outlive",
            "borrowed value does not live",
            "local storage",
        ],
    ),
];
const NATIVE: [(&str, &str); 10] = [
    ("direct-rhs", "add\nvalue 7\n"),
    ("direct", "add\nvalue 7\n"),
    ("borrowed", "deref\nadd\nvalue 7\n"),
    ("temporary", "deref\nadd\nvalue 7\n"),
    ("nested", "deref\nderef\nadd\nvalue 9\n"),
    ("generic-lhs", "deref\nadd\nvalue 7\n"),
    ("chain", "deref\nadd\nderef\nadd\nvalue 9\n"),
    ("mutable", "mut\nadd\nvalue 7\n"),
    ("borrowed-output", "deref\nadd\nvalue 7\n"),
    ("independent-input-lifetimes", "add\nvalue 7\n"),
];

#[test]
fn arithmetic_rhs_coercions_keep_selected_impls_and_operand_duties() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut accepted = 0;
    let mut rejected = 0;
    let mut failures = Vec::new();
    for row in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let [path, status, view] = fields.as_slice() else {
            return Err("operator outcome row".into());
        };
        let expected = status.parse::<i32>()?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                view,
                "--cap-lints=allow",
                "--edition=2024",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "operator_rhs",
                "--source-core",
                "minicore@operator_rhs",
            ])
            .arg(root.join(path))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.code() != Some(expected) {
            failures.push(format!(
                "{path}: {:?}, expected {expected}: {stderr}",
                output.status.code()
            ));
        }
        if expected == 0 {
            if !stderr.is_empty() || output.stdout.is_empty() {
                failures.push(format!(
                    "{path}: expected {view} output and empty stderr: {stderr}"
                ));
            }
            accepted += 1;
        } else {
            assert_eq!(expected, 1);
            let (_, fragments) = REJECTED
                .iter()
                .find(|(name, _)| path.ends_with(name))
                .ok_or("undeclared operator opponent")?;
            if !fragments.iter().any(|fragment| stderr.contains(fragment))
                || !stderr.contains("ERROR: ")
            {
                failures.push(format!("{path}: wrong opposing cause: {stderr}"));
            }
            for forbidden in ["not implemented", "panicked at", "work budget", "lost its"] {
                if stderr.contains(forbidden) {
                    failures.push(format!("{path}: forbidden failure {forbidden}: {stderr}"));
                }
            }
            rejected += 1;
        }
    }
    assert_eq!((accepted, rejected), (10, 9));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the pinned LowIR backend and a C linker"]
fn rhs_dereference_and_add_traces_match_independent_observations() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root.join(format!(
        "obj/parallel/operator-rhs-native/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let observer = crate::resolve::fixture(&format!("{PACKET}/observe.c"))?;
    for (name, expected) in NATIVE {
        let source = crate::resolve::fixture(&format!("{PACKET}/{name}.rs"))?;
        let program = crate::ffi_program::exported(&source, "operator_rhs")?;
        crate::ffi_program::check(root, &folder, name, &program, &observer, expected)?;
    }
    Ok(())
}

const COMPOUND: &str = "pa31/tests/spec/source-compound-rhs";
const COMPOUND_NATIVE: [(&str, &str); 2] = [
    ("coerced-rhs", "left\nrhs\nderef\nassign\nvalue 7\n"),
    ("direct-overloaded-rhs", "assign\nvalue 7\n"),
];

#[test]
fn compound_rhs_coercions_preserve_exact_place_and_reject_opponents() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let cases = [
        ("coerced-rhs", "--emit-body-ir", None),
        ("direct-overloaded-rhs", "--emit-body-ir", None),
        ("primitive-rhs", "--emit-body-ir", None),
        (
            "immutable-place",
            "--emit-loans",
            Some("cannot mutably borrow an immutable local"),
        ),
        (
            "closed-wrong-rhs",
            "--emit-body-ir",
            Some("incompatible types"),
        ),
        (
            "overlapping-receiver",
            "--emit-loans",
            Some("access conflicts with a live reference loan"),
        ),
    ];
    let mut failures = Vec::new();
    for (name, view, cause) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                view,
                "--cap-lints=allow",
                "--edition=2024",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "compound_rhs",
                "--source-core",
                "minicore@compound_rhs",
            ])
            .arg(root.join(COMPOUND).join(format!("{name}.rs")))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let valid = match cause {
            Some(cause) => {
                output.status.code() == Some(1)
                    && stderr
                        .lines()
                        .any(|line| line.starts_with("ERROR: ") && line.contains(cause))
                    && !["not implemented", "panicked", "work budget", "lost its"]
                        .iter()
                        .any(|word| stderr.contains(word))
            }
            None => output.status.success() && !output.stdout.is_empty() && stderr.is_empty(),
        };
        if !valid {
            failures.push(format!(
                "{name}: {view} {:?}: {stderr}",
                output.status.code()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the pinned LowIR backend and a C linker"]
fn compound_rhs_native_traces_keep_place_and_coercion_order() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root.join(format!(
        "obj/parallel/source-compound-rhs-native/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let observer = crate::resolve::fixture(&format!("{COMPOUND}/observe.c"))?;
    for (name, expected) in COMPOUND_NATIVE {
        let source = crate::resolve::fixture(&format!("{COMPOUND}/{name}.rs"))?;
        let program = crate::ffi_program::exported(&source, "compound_rhs")?;
        crate::ffi_program::check(root, &folder, name, &program, &observer, expected)?;
    }
    Ok(())
}
