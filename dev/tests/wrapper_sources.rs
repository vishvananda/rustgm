//! Real source wrapper outcomes use complete CFG proofs and the external native backend.
use crate::provider_traits::{TestResult, lower};
use std::path::{Path, PathBuf};
const CASES: &[(&str, bool)] = &[
    ("manual-header", true),
    ("maybe-header", true),
    ("manual-no-repr", true),
    ("manual-lifetime", true),
    ("manual-const", true),
    ("manual-two-fields", true),
    ("maybe-two-fields", true),
    ("manual-zero", false),
    ("manual-two-generics", false),
    ("maybe-zero", false),
    ("maybe-two-generics", false),
    ("manual-wrong-target", false),
    ("maybe-wrong-target", false),
    ("manual-duplicate", false),
    ("maybe-duplicate", false),
    ("maybe-valid-dangling", true),
    ("manual-nested-valid-dangling", true),
    ("maybe-null", false),
    ("maybe-unaligned", false),
    ("ordinary-dangling", false),
    ("manual-alone-dangling", false),
    ("lookalike-dangling", false),
    ("maybe-invalid-bool", false),
    ("manual-invalid-bool", false),
    ("maybe-extracted-dangling", false),
    ("maybe-sibling-memo", false),
    ("maybe-deref", false),
    ("maybe-borrow-escape", false),
    ("drop-pair", true),
    ("manual-own-destructor", true),
    ("const-layout", true),
    ("manual-move-out", true),
    ("combined-roles", false),
];
const NATIVE: &[(&str, &str)] = &[
    ("drop-pair", "23 7 0 1 1\n"),
    ("manual-own-destructor", "0 7\n"),
    ("manual-move-out", "4 7\n"),
    ("const-layout", "7 8 8\n"),
];
fn root() -> TestResult<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?
        .to_owned())
}
fn fixture(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(root()?.join(format!(
        "pa31/tests/spec/source-wrappers/{name}.{extension}"
    )))?)
}
#[test]
fn source_wrapper_outcomes_preserve_real_fields_const_validity_and_full_borrow_proofs() -> TestResult
{
    assert_eq!(CASES.len(), 33);
    for &(name, accepted) in CASES {
        let result = lower(&fixture(name, "rs")?);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.err().map(|error| error.to_string())
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_wrappers_native_preserve_actual_destructors_layout_and_owned_field_moves() -> TestResult {
    assert_eq!(NATIVE.len(), 4);
    let root = root()?;
    let folder = root
        .join("obj/parallel/source-wrappers-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &(name, expected) in NATIVE {
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &lower(&fixture(name, "rs")?)?,
            &fixture(name, "c")?,
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn source_no_drop_requires_actual_component_proofs_and_keeps_ownership_checks() -> TestResult {
    let accepted = "header manual-union mutable-reference-union nested-components copy-premise manual-premise reference-premise nominal-copy fallback-shared-reference fallback-scalar-without-copy header-unsafe header-constant header-members header-auto marker-long-tuple fallback-scalar-array native-relay fallback-copy-tuple";
    let rejected = "wrong-target generic-type generic-lifetime generic-const duplicate-role zero-array-noncopy ordinary-nondrop manual-lookalike role-lookalike explicit-impl fallback-manual fallback-mutable-reference bound-scalar-without-copy borrow-escape no-copy-grant fallback-long-tuple marker-empty-array";

    let cases = accepted
        .split_whitespace()
        .map(|name| (name, true))
        .chain(rejected.split_whitespace().map(|name| (name, false)));
    let mut count = 0;
    for (name, accepted) in cases {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa31/tests/spec/source-no-drop/{name}.rs")),
        )?;
        let result = lower(&source);
        assert_eq!(
            result.is_ok(),
            accepted,
            "{name}: {:?}",
            result.err().map(|error| error.to_string())
        );
        count += 1;
    }
    assert_eq!(count, 35);
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_no_drop_native_relay_executes_actual_mutation_and_owned_source_fields() -> TestResult {
    let root = root()?;
    let folder = root
        .join("obj/parallel/source-no-drop-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let fixture = root.join("pa31/tests/spec/source-no-drop/native-relay");
    crate::ffi_program::check(
        &root,
        &folder,
        "native-relay",
        &lower(&std::fs::read_to_string(fixture.with_extension("rs"))?)?,
        &std::fs::read_to_string(fixture.with_extension("c"))?,
        "8\n",
    )?;
    Ok(())
}

#[test]
fn source_maybe_uninit_const_outcomes_keep_bytes_validity_and_ownership_separate() -> TestResult {
    let accepted = "new-u8 new-bool new-record new-reference new-array unit empty-array unused-uninhabited opaque-union-uninhabited write-and-borrow write-through-pointer array-complete copy-preserves-uninitialized header-zero header-many header-life-const ordinary-panic-spelling opaque-invalid-bool opaque-null-reference inferred-valid-reference";
    let rejected = "uninitialized-u8 uninitialized-reference invalid-bool null-reference uninhabited-read unsafe-required uninitialized-ref-access array-partial wrong-size-transmute borrow-escape no-copy-grant header-wrong-target header-duplicate transparent-missing-gate transparent-two-fields transparent-macro-allowance renamed-source-panic";
    let mut failures = Vec::new();
    for (name, accepted) in accepted
        .split_whitespace()
        .map(|name| (name, true))
        .chain(rejected.split_whitespace().map(|name| (name, false)))
    {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa31/tests/spec/source-maybe-uninit/{name}.rs")),
        )?;
        let result = lower(&source);
        let error = result.as_ref().err().map(ToString::to_string);
        if let Some(error) = &error {
            eprintln!("{name}: {error}");
        }
        let infrastructure = error.as_ref().is_some_and(|message| {
            [
                "budget",
                "limit exceeded",
                "non-const body requested",
                "prepared scalar union",
                "source intrinsic call",
                "not implemented",
                "unsupported active source-core feature",
            ]
            .iter()
            .any(|text| message.contains(text))
        });
        let reason = match name {
            "invalid-bool" => Some("bool"),
            "null-reference" => Some("null"),
            "wrong-size-transmute" | "renamed-source-panic" => {
                Some("constant execution reached source panic_display")
            }
            _ => None,
        };
        let wrong_reason =
            reason.is_some_and(|reason| error.as_ref().is_none_or(|error| !error.contains(reason)));
        if result.is_ok() != accepted || infrastructure || wrong_reason {
            failures.push(format!("{name}: {error:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
