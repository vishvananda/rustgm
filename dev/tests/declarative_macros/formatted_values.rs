//! Frozen independent formatting contracts exercise real source constructor calls.
use crate::provider_traits::{TestResult, lower, report};
use std::path::Path;
const LITERALS: &[(&str, &str)] = &[
    ("basic", "68656c6c6f"),
    ("const", "68656c6c6f"),
    ("newline", "68656c6c6f0a"),
    ("empty", ""),
    ("escaped-braces", "7b787d"),
    // The original physical raw literal contains LF. Its old backslash-n C
    // hypothesis is retained separately in the immutable independent review.
    ("raw", "610a62"),
    ("renamed-role", "68656c6c6f"),
    ("captured-expression", "68656c6c6f"),
];
fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-format-macros")
            .join(format!("{name}.rs")),
    )?)
}
#[test]
fn literal_formatting_calls_actual_source_constructors_and_preserves_string_bytes() -> TestResult {
    for &(name, _) in LITERALS {
        lower(&fixture(&format!("literal-complete-{name}"))?)
            .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn literal_formatting_keeps_actual_owner_input_constness_and_visibility_checks() -> TestResult {
    for (name, message) in [
        ("missing-language-owner", "format_arguments"),
        ("wrong-constructor-input", "type"),
        ("nonconst-constructor", "const"),
        (
            "private-constructor",
            "no applicable formatting constructor",
        ),
    ] {
        let error = lower(&fixture(&format!("literal-complete-{name}"))?)
            .err()
            .ok_or_else(|| format!("{name}: invalid constructor accepted"))?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn scoped_wrappers_preserve_the_actual_format_builtin_constructor_privacy() -> TestResult {
    use rustgm::support::edition::Edition;
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        crate::provider_traits::lower_in_edition(
            &fixture("holder-format-holder-wrapper")?,
            edition,
        )?;
        let error = crate::provider_traits::lower_in_edition(
            &fixture("root-format-holder-wrapper")?,
            edition,
        )
        .err()
        .ok_or("wrapper lent its private constructor access to another builtin")?;
        assert!(
            error
                .to_string()
                .contains("no applicable formatting constructor"),
            "{error}"
        );
    }
    Ok(())
}
#[test]
fn real_format_headers_keep_stability_and_claim_opponents_explicit() -> TestResult {
    for name in [
        "format-args",
        "const-format-args",
        "format-args-nl",
        "three",
    ] {
        let report = report(&fixture(&format!("declaration-original-{name}"))?)?;
        crate::provider_traits::require_macro_policies(&report)?;
        rustgm::pipeline::analyze_source(&report)?;
    }
    for name in [
        "declaration-renamed-role",
        "declaration-authored-same-spelling",
        "declaration-scoped-format-role",
    ] {
        lower(&fixture(name)?)?;
    }
    for (name, message) in [
        ("diagnostic-only", "format_args diagnostic"),
        ("wrong-diagnostic-role", "format_args diagnostic"),
        ("unknown-role", "builtin"),
        ("wrong-target", "macro"),
        ("unsafe-missing-feature", "requires allow_internal_unsafe"),
        ("unsafe-malformed", "arguments"),
        ("duplicate-diagnostic", "duplicate"),
    ] {
        let error = report(&fixture(&format!("declaration-{name}"))?)
            .err()
            .ok_or_else(|| format!("{name}: unsupported claim admitted"))?;
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
    Ok(())
}
fn consumer(bytes: &str) -> TestResult<String> {
    let expected = bytes
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| Ok(format!("0x{}", std::str::from_utf8(pair)?)))
        .collect::<TestResult<Vec<_>>>()?
        .join(",");
    let comparison = if bytes.is_empty() {
        String::new()
    } else {
        format!(
            "const unsigned char expected[]={{{expected}}};for(unsigned i=0;i<sizeof expected;i++)if(actual[i]!=expected[i])return 2;"
        )
    };
    Ok(format!(
        "#include <stdio.h>\nextern const unsigned char*answer(void);int main(void){{const unsigned char*actual=answer();if(!actual)return 1;{comparison}puts(\"7\");return 0;}}\n"
    ))
}
#[test]
#[ignore = "requires pinned CPPGM backend and host C linker"]
fn actual_format_literal_constructors_preserve_independently_observed_native_bytes() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/format-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &(name, bytes) in LITERALS {
        let program = lower(&fixture(&format!("literal-complete-{name}"))?)?;
        crate::ffi_program::check(root, &folder, name, &program, &consumer(bytes)?, "7\n")?;
    }
    for edition in [
        rustgm::support::edition::Edition::Rust2021,
        rustgm::support::edition::Edition::Rust2024,
    ] {
        let program = crate::provider_traits::lower_in_edition(
            &fixture("holder-format-holder-wrapper")?,
            edition,
        )?;
        crate::ffi_program::check(
            root,
            &folder,
            &format!("scoped-holder-{edition:?}"),
            &program,
            &consumer("68656c6c6f")?,
            "7\n",
        )?;
    }
    let c =
        std::fs::read_to_string(root.join("pa31/tests/spec/source-format-macros/consumer-u32.c"))?;
    for (name, expected) in [
        ("declaration-renamed-role", "7\n"),
        ("declaration-authored-same-spelling", "9\n"),
    ] {
        let program = lower(&fixture(name)?)?;
        crate::ffi_program::check(root, &folder, name, &program, &c, expected)?;
    }
    Ok(())
}
