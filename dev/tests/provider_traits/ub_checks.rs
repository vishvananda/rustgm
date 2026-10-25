//! Explicit source policy selects local UB booleans without a backend operation.
use super::{TestResult, lower_report, report_with_config};
use rustgm::{expand::config::Config, support::runtime_checks::RuntimeCheckConfig};
use std::{path::Path, process::Command};
fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/ub-checks")
            .join(format!("{name}.rs")),
    )?)
}
fn configured(source: &str, enabled: bool) -> TestResult<rustgm::resolve::expansion::Report> {
    let mut config = Config::default();
    config.set_runtime_checks(RuntimeCheckConfig::with_ub_checks(enabled));
    report_with_config(source, &config)
}
#[test]
fn actual_ub_declarations_const_roots_and_generic_runtime_calls_lower_in_both_profiles()
-> TestResult {
    for name in ["local-fold", "local-preserve", "ordinary-same-name"] {
        for enabled in [false, true] {
            let report = configured(&fixture(name)?, enabled)?;
            assert_eq!(report.package.runtime_checks().ub_checks(), enabled);
            lower_report(&report)?;
        }
    }
    Ok(())
}
#[test]
fn source_ub_signatures_and_feature_requests_are_not_name_permissions() -> TestResult {
    let source = fixture("local-fold")?;
    for (from, to, expected) in [
        (
            "pub const fn ub_checks()->bool",
            "pub const unsafe fn ub_checks()->bool",
            "safe Rust signature",
        ),
        (
            "pub const fn ub_checks()->bool",
            "pub const fn ub_checks()->u8",
            "bool result",
        ),
        (
            "pub const fn ub_checks()->bool",
            "pub fn ub_checks()->bool",
            "declaration must be const",
        ),
        (
            "pub const fn ub_checks()->bool",
            "pub const fn ub_checks<T>()->bool",
            "no parameters or predicates",
        ),
        (
            "pub const fn ub_checks()->bool",
            "pub const fn ub_checks<'a>()->bool",
            "no parameters or predicates",
        ),
        (
            "pub const fn ub_checks()->bool",
            "pub const fn ub_checks(_:u8)->bool",
            "no parameters or predicates",
        ),
        (",intrinsics", "", "explicit source features"),
        (
            ",rustc_attrs",
            "",
            "builtin macro binding requires rustc_attrs",
        ),
    ] {
        let changed = source.replace(from, to);
        assert_ne!(source, changed);
        let error = configured(&changed, false)
            .and_then(|report| lower_report(&report))
            .err()
            .ok_or("invalid intrinsic accepted")?;
        assert!(
            error.to_string().contains(expected),
            "{from}=>{to}: {error}"
        );
    }
    let invalid_unused = format!("{source}\nfn unused_bad()->u8{{false}}\n");
    assert!(lower_report(&configured(&invalid_unused, false)?).is_err());
    Ok(())
}
#[test]
fn source_ub_reification_and_indirect_calls_keep_the_unadapted_target_denial() -> TestResult {
    for name in ["reify-ub", "indirect-ub"] {
        let error = lower_report(&configured(&fixture(name)?, false)?)
            .err()
            .ok_or("unadapted UB target accepted")?;
        assert!(
            error
                .to_string()
                .contains("source intrinsic cannot be coerced or cast to a function pointer"),
            "{name}: {error}"
        );
    }
    lower_report(&configured(&fixture("reify-ordinary")?, false)?)?;
    Ok(())
}

#[test]
fn demanded_cfg_uses_preserve_false_owner_payload_and_token_origin_boundaries() -> TestResult {
    for (name, accepts) in [
        ("cfg-cfg-macro-truth-irrelevant", false),
        ("cfg-cfg-attribute-truth-irrelevant", false),
        ("cfg-cfg-attr-predicate-truth-irrelevant", false),
        ("cfg-false-payload", true),
        ("cfg-true-payload", false),
        ("cfg-false-owner-later-cfg", true),
        ("cfg-false-owner-body-macro", true),
        ("cfg-inactive-feature-request", false),
        ("cfg-active-feature-request", true),
        ("cfg-allowed-definition-token", true),
        ("cfg-forwarded-caller-token", false),
        ("cfg-allowed-generated-attribute", true),
        ("cfg-forwarded-attribute-token", false),
        ("cfg-ancestor-is-not-use-owner", false),
        ("choice-false-self-owner", false),
        ("choice-first-predicate", false),
        ("choice-later-predicate", false),
        ("choice-after-wildcard-predicate", false),
        ("choice-inactive-body", true),
        ("choice-allowed-definition", true),
        ("choice-forwarded-caller", false),
    ] {
        match configured(&fixture(name)?, false) {
            Ok(_) => assert!(accepts, "{name}: unavailable feature accepted"),
            Err(error) => assert!(
                !accepts && error.to_string().contains("cfg(ub_checks) requires"),
                "{name}: {error}"
            ),
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn local_ub_const_runtime_and_configuration_native_answers_match_independent_source() -> TestResult
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/ub-check-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for name in ["local-fold", "local-preserve", "ordinary-same-name"] {
        for enabled in [false, true] {
            let stem = format!("{name}-ub-{}", u8::from(enabled));
            let program = lower_report(&configured(&fixture(name)?, enabled)?)?;
            let ir = folder.join(format!("{stem}.lowir"));
            let object = folder.join(format!("{stem}.o"));
            let binary = folder.join(format!("{stem}.native"));
            let client = folder.join(format!("{stem}.c"));
            let mut text = Vec::new();
            rustgm::lowir::writer::write(&program, &mut text)?;
            std::fs::write(&ir, text)?;
            std::fs::copy(root.join("pa31/tests/spec/ub-checks/observe.c"), &client)?;
            super::native::run(
                &folder,
                &format!("{stem}-backend"),
                Command::new("python3")
                    .arg(root.join("scripts/backend.py"))
                    .arg("--emit-object")
                    .arg(&ir)
                    .arg("-o")
                    .arg(&object),
            )?;
            super::native::run(
                &folder,
                &format!("{stem}-link"),
                Command::new("cc")
                    .arg(&client)
                    .arg(&object)
                    .arg("-o")
                    .arg(&binary),
            )?;
            super::native::run(
                &folder,
                &format!("{stem}-execute"),
                &mut Command::new(&binary),
            )?;
            let ordinary = name == "ordinary-same-name";
            let expected = format!(
                "{} {} {} {}\n",
                u8::from(!ordinary),
                u8::from(enabled && !ordinary),
                u8::from(enabled),
                u8::from(enabled && !ordinary)
            );
            assert_eq!(
                std::fs::read(folder.join(format!("{stem}-execute.stdout")))?,
                expected.as_bytes()
            );
            assert!(std::fs::read(folder.join(format!("{stem}-execute.stderr")))?.is_empty());
            super::native::run(
                &folder,
                &format!("{stem}-hashes"),
                Command::new("sha256sum")
                    .arg(&ir)
                    .arg(&object)
                    .arg(&binary)
                    .arg(&client),
            )?;
        }
    }
    Ok(())
}
