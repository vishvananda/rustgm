//! Scoped lint levels neither grant unsafe permission nor disappear at item boundaries.
use crate::semantic;
use crate::source_cases as source;

fn deprecated_safe_source(
    name: &str,
    edition: rustgm::support::edition::Edition,
) -> Result<rustgm::resolve::expansion::Report, Box<dyn std::error::Error>> {
    let fixture = crate::resolve::fixture(&format!(
        "pa31/tests/spec/deprecated-safe-2024/cases/{name}.rs"
    ))?;
    let source = if name == "missing-request" {
        format!("#![feature(no_core)]\n#![no_core]\n{fixture}")
    } else {
        let provider = crate::resolve::fixture(
            "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
        )?;
        let attributes = fixture
            .split_inclusive('\n')
            .take_while(|line| line.starts_with("#!["))
            .map(str::len)
            .sum();
        let (attributes, declarations) = fixture.split_at(attributes);
        format!("{attributes}{provider}{declarations}")
    };
    let config = rustgm::expand::config::Config::in_edition(edition);
    Ok(crate::resolve::source_provider_config(
        &source,
        "deprecated_safe_2024",
        &config,
    )?)
}

#[test]
fn deprecated_safe_2024_requires_the_actual_direct_callee_and_call_edition()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::support::edition::Edition::{Rust2021, Rust2024};

    const UNSAFE_CALL: &str = "call to unsafe function requires an unsafe context";
    for (name, edition, expected) in [
        ("direct-2021", Rust2021, None),
        ("direct-2024", Rust2024, Some(UNSAFE_CALL)),
        (
            "deny-migration-2021",
            Rust2021,
            Some("deprecated_safe_2024 call"),
        ),
        ("explicit-2024", Rust2024, None),
        ("pointer-2021", Rust2021, Some(UNSAFE_CALL)),
        ("safe-pointer-conversion-2021", Rust2021, Some("callable")),
        ("unmarked-2021", Rust2021, Some(UNSAFE_CALL)),
        ("associated-2021", Rust2021, None),
        ("associated-2024", Rust2024, Some(UNSAFE_CALL)),
        (
            "missing-request",
            Rust2024,
            Some("requires the rustc_attrs source profile"),
        ),
        ("missing-list", Rust2024, Some("requires one audit_that")),
        ("wrong-key", Rust2024, Some("requires one audit_that")),
        ("non-string", Rust2024, Some("requires a string literal")),
        (
            "wrong-owner",
            Rust2024,
            Some("requires a function or method"),
        ),
        (
            "foreign-owner",
            Rust2024,
            Some("requires a nonforeign function or method"),
        ),
        ("disabled", Rust2024, None),
    ] {
        let report = deprecated_safe_source(name, edition)?;
        let outcome = rustgm::pipeline::analyze_source(&report);
        if let Some(expected) = expected {
            let message = outcome
                .err()
                .ok_or("expected deprecated-safe rejection")?
                .to_string();
            assert!(message.contains(expected), "{name}: {message}");
        } else {
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = outcome?;
            if let Some((index, _)) =
                report
                    .resolution
                    .definitions
                    .iter()
                    .enumerate()
                    .find(|(_, definition)| {
                        definition.kind == rustgm::resolve::model::DefKind::Function
                            && report.package.names.get(definition.name) == "call"
                    })
            {
                rustgm::lower::instances::build_prepared(
                    &report,
                    &mut facts,
                    rustgm::resolve::model::DefId(u32::try_from(index)?),
                    prepared,
                )
                .map_err(|error| format!("{name}: {error}"))?;
            } else {
                assert_eq!(name, "disabled");
            }
        }
    }
    Ok(())
}

#[test]
fn deprecated_safe_2024_metadata_cannot_erase_the_migration_contract()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::lower::instances::{SourcePublication, build_source_library_profile};
    let report =
        deprecated_safe_source("direct-2021", rustgm::support::edition::Edition::Rust2021)?;
    let identity = report
        .package
        .source_core()
        .ok_or("source provider")?
        .provider
        .clone();
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let scalar = rustgm::sema::crates::export(&report, &mut facts, identity.clone())
        .err()
        .ok_or("scalar metadata published a migration contract")?;
    let source = build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleBodyOperations,
    )
    .err()
    .ok_or("legacy metadata published a migration contract")?;
    for error in [scalar, source] {
        assert!(
            error
                .to_string()
                .contains("requires declaration and call-origin transport"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn successful_source_diagnostics_require_exact_named_nonempty_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::support::diagnostics::WarningCode;
    let cases = [
        ("warned", true, "fn f(){}"),
        ("silent", true, "fn g(){}"),
        ("rejected", false, "bad"),
    ];
    let text = "warning[unsafe_op_in_unsafe_fn]: file 0: 7: declared warning\n";
    let valid = source::WarningCase {
        name: "warned",
        stderr: text,
        codes: &[(WarningCode::UnsafeOpInUnsafeFn, 1)],
    };
    source::validate_warnings(&cases, std::slice::from_ref(&valid))?;
    source::verify_stderr(text.as_bytes(), Some(&valid))?;
    source::verify_stderr(b"", None)?;
    for actual in [b"".as_slice(), b"warning[static_mut_refs]: file 0: 7: declared warning\n", b"warning[unsafe_op_in_unsafe_fn]: file 0: 7: declared warning\nERROR: file 0: 9: primary\n", b"warning[unsafe_op_in_unsafe_fn]: file 0: 7: declared warning\ntrailing\n"] {
        assert!(source::verify_stderr(actual, Some(&valid)).is_err());
    }
    assert!(source::verify_stderr(text.as_bytes(), None).is_err());
    for (name, stderr, codes) in [
        ("missing", text, valid.codes),
        ("rejected", text, valid.codes),
        ("warned", "", valid.codes),
        ("warned", text, &[(WarningCode::UnsafeOpInUnsafeFn, 2)][..]),
        ("warned", text, &[(WarningCode::StaticMutRefs, 1)][..]),
        ("warned", text, &[(WarningCode::UnsafeOpInUnsafeFn, 0)][..]),
        (
            "warned",
            text,
            &[
                (WarningCode::UnsafeOpInUnsafeFn, 1),
                (WarningCode::UnsafeOpInUnsafeFn, 1),
            ][..],
        ),
        (
            "warned",
            "warning[unsafe_op_in_unsafe_fn]: file 0: 7: warning\nERROR: file 0: 8: bad\n",
            valid.codes,
        ),
        (
            "warned",
            "warning[unsafe_op_in_unsafe_fn]: file 0: 7: warning\n  in macro arbitrary unvalidated text\n",
            valid.codes,
        ),
    ] {
        let wrong = source::WarningCase {
            name,
            stderr,
            codes,
        };
        assert!(source::validate_warnings(&cases, &[wrong]).is_err());
    }
    let duplicate = source::WarningCase {
        name: valid.name,
        stderr: valid.stderr,
        codes: valid.codes,
    };
    assert!(source::validate_warnings(&cases, &[valid, duplicate]).is_err());
    Ok(())
}

const WARNINGS: &[source::WarningCase<'_>] = &[
    source::WarningCase {
        name: "operation-warn",
        stderr: "warning[unsafe_op_in_unsafe_fn]: file 0: 60: unsafe operation in unsafe function requires an explicit unsafe block\n",
        codes: &[(
            rustgm::support::diagnostics::WarningCode::UnsafeOpInUnsafeFn,
            1,
        )],
    },
    source::WarningCase {
        name: "reference-warn",
        stderr: "warning[static_mut_refs]: file 0: 69: creating a reference to a mutable static is discouraged\n",
        codes: &[(rustgm::support::diagnostics::WarningCode::StaticMutRefs, 1)],
    },
];

const CASES: &[(&str, bool, &str)] = &[
    (
        "statement-operation-allow",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8){#[allow(unsafe_op_in_unsafe_fn)]let _=*p;}",
    ),
    (
        "statement-operation-deny",
        false,
        "unsafe fn f(p:*const u8){#[deny(unsafe_op_in_unsafe_fn)]let _=*p;}",
    ),
    (
        "statement-reference-allow",
        true,
        "static mut X:u8=0;fn f(){unsafe{#[allow(static_mut_refs)]let _=&X;}}",
    ),
    (
        "statement-reference-deny",
        false,
        "#![allow(static_mut_refs)]static mut X:u8=0;fn f(){unsafe{#[deny(static_mut_refs)]let _=&X;}}",
    ),
    (
        "operation-deny",
        false,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "operation-forbid",
        false,
        "#[forbid(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "operation-allow",
        true,
        "#[allow(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "operation-warn",
        true,
        "#[warn(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "operation-explicit",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{unsafe{*p}}",
    ),
    (
        "allow-safe-function",
        false,
        "#[allow(unsafe_op_in_unsafe_fn)]fn f(p:*const u8)->u8{*p}",
    ),
    (
        "deny-call",
        false,
        "unsafe fn g(){}#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(){g();}",
    ),
    (
        "deny-static",
        false,
        "static mut X:u8=0;#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f()->u8{X}",
    ),
    (
        "crate-deny",
        false,
        "#![deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "crate-override",
        true,
        "#![deny(unsafe_op_in_unsafe_fn)]#[allow(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "block-override",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{#![allow(unsafe_op_in_unsafe_fn)]*p}",
    ),
    (
        "nested-item-inherit",
        false,
        "#[deny(unsafe_op_in_unsafe_fn)]fn f(){unsafe fn g(p:*const u8)->u8{*p}}",
    ),
    (
        "nested-item-override",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]fn f(){#[allow(unsafe_op_in_unsafe_fn)]unsafe fn g(p:*const u8)->u8{*p}}",
    ),
    (
        "inline-module-inherit",
        false,
        "#![deny(unsafe_op_in_unsafe_fn)]mod m{pub unsafe fn f(p:*const u8)->u8{*p}}",
    ),
    (
        "inline-module-override",
        true,
        "#![deny(unsafe_op_in_unsafe_fn)]#[allow(unsafe_op_in_unsafe_fn)]mod m{pub unsafe fn f(p:*const u8)->u8{*p}}",
    ),
    (
        "inline-inner-deny",
        false,
        "mod m{#![deny(unsafe_op_in_unsafe_fn)]pub unsafe fn f(p:*const u8)->u8{*p}}",
    ),
    (
        "forbid-allow",
        false,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[allow(unsafe_op_in_unsafe_fn)]fn f(){}",
    ),
    (
        "forbid-warn",
        false,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[warn(unsafe_op_in_unsafe_fn)]fn f(){}",
    ),
    (
        "forbid-deny",
        true,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[deny(unsafe_op_in_unsafe_fn)]fn f(){}",
    ),
    (
        "forbid-same",
        true,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[forbid(unsafe_op_in_unsafe_fn)]fn f(){}",
    ),
    (
        "forbid-deny-allow",
        false,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[deny(unsafe_op_in_unsafe_fn)]mod m{#[allow(unsafe_op_in_unsafe_fn)]fn f(){}}",
    ),
    (
        "reference-allow",
        true,
        "static mut X:u8=0;#[allow(static_mut_refs)]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "reference-warn",
        true,
        "static mut X:u8=0;#[warn(static_mut_refs)]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "reference-safe",
        false,
        "static mut X:u8=0;#[allow(static_mut_refs)]fn f()->&'static u8{&X}",
    ),
    (
        "reference-deny-override",
        false,
        "#![allow(static_mut_refs)]static mut X:u8=0;#[deny(static_mut_refs)]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "reference-forbid",
        false,
        "#![forbid(static_mut_refs)]#[allow(static_mut_refs)]fn f(){}",
    ),
    (
        "reference-binding-allow",
        true,
        "static mut X:u8=0;#[allow(static_mut_refs)]fn f(){unsafe{let ref r=X;let _=r;}}",
    ),
    (
        "reference-method-allow",
        true,
        "struct S;impl S{fn g(&self){}}static mut X:S=S;#[allow(static_mut_refs)]fn f(){unsafe{X.g();}}",
    ),
    (
        "configured-allow",
        true,
        "static mut X:u8=0;#[cfg_attr(all(),allow(static_mut_refs))]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "configured-disabled",
        false,
        "static mut X:u8=0;#[cfg_attr(any(),allow(static_mut_refs))]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "unknown-path",
        false,
        "static mut X:u8=0;#[allow(clippy::static_mut_refs)]fn f()->&'static u8{unsafe{&X}}",
    ),
];

#[test]
fn scoped_unsafe_lints() -> Result<(), Box<dyn std::error::Error>> {
    source::check_with_warnings("unsafe-policies", CASES, WARNINGS)
}

#[test]
fn policies_allocate_only_when_relevant_and_walk_additively()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for attrs in [false, true] {
        for functions in [8, 32, 128] {
            for depth in [1, 8, 32] {
                let mut text = String::new();
                for index in 0..functions {
                    if attrs {
                        text.push_str("#[deny(unsafe_op_in_unsafe_fn)]");
                    }
                    write!(text, "unsafe fn f{index}(p:*const u8)->u8{{")?;
                    for _ in 0..depth {
                        text.push_str("unsafe{");
                    }
                    text.push_str("*p");
                    for _ in 0..=depth {
                        text.push('}');
                    }
                }
                let (report, facts) = semantic::analyze(&text)?;
                let work = facts.safety_work();
                if attrs {
                    let nodes: usize = report
                        .package
                        .units
                        .iter()
                        .map(|u| u.configured.ast.nodes.len())
                        .sum();
                    assert_eq!(work.attributes, functions);
                    assert_eq!(work.entries, functions);
                    assert_eq!(work.retained_nodes, nodes);
                    assert!(work.nodes <= 2 * nodes + report.package.modules.len());
                    assert!(work.edges <= 2 * nodes);
                } else {
                    assert_eq!(work.attributes, 0);
                    assert_eq!(work.retained_nodes, 0);
                    assert_eq!(work.nodes, 0);
                    assert_eq!(work.edges, 0);
                }
            }
        }
    }
    Ok(())
}
