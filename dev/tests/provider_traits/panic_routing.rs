//! Local source panic delegation reuses independently observed source programs.
use super::{TestResult, lower_in_edition, report_in_edition};
use rustgm::{expand::builtin::Builtin, resolve::expansion::Body, support::edition::Edition};
use std::path::Path;

const POSITIVE: &[(&str, &str)] = &[
    ("direct-core_panic", "21\n"),
    ("direct-std_panic", "21\n"),
    ("direct-unreachable", "21\n"),
    ("imported-alias", "21\n"),
    ("caller-module-shadow", "21\n"),
    ("ordinary-spelling", "9\n"),
    ("missing-legacy-handler", "21\n"),
    ("permission-valid-wrapper", "21\n"),
    ("forwarded-expression", "7\n"),
];
const HEADER_POSITIVE: &[&str] = &[
    "permission-unknown-feature",
    "permission-duplicate",
    "permission-empty",
    "allowance-panic-header",
    "allowance-intrinsic-header",
    "allowance-combined-header",
    "allowance-reordered-header",
    "allowance-repeated-attributes",
    "allowance-data-duplicates-in-list",
    "allowance-data-combined-unknown-attributes",
    "allowance-data-arbitrary-identifiers",
    "allowance-data-raw-feature-identifier",
    "allowance-data-list-width-8",
    "allowance-data-list-width-32",
    "allowance-data-list-width-128",
    "allowance-unicode-nfc",
    "allowance-unicode-nfd",
    "allowance-unicode-combined",
];
fn source(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa31/tests/spec/source-panic-macros/{name}.rs")),
    )?)
}
fn editions() -> [Edition; 2] {
    [Edition::Rust2021, Edition::Rust2024]
}
fn admitted(name: &str, edition: Edition) -> bool {
    edition == Edition::Rust2024
        || !matches!(name, "permission-valid-wrapper" | "forwarded-expression")
}

#[test]
fn actual_source_panic_roles_select_real_handlers_in_admitted_editions() -> TestResult {
    for edition in editions() {
        for &(name, _) in POSITIVE {
            if !admitted(name, edition) {
                continue;
            }
            let source = source(name)?;
            let report = report_in_edition(&source, edition)
                .map_err(|error| format!("{name}/{edition:?}: {error}"))?;
            let roles = report.invocations.iter().filter(|invocation|
                matches!(report.definitions[invocation.definition].body, Body::Builtin(role) if role.source_only()))
                .collect::<Vec<_>>();
            if name == "ordinary-spelling" {
                assert!(roles.is_empty());
            } else {
                assert_eq!(roles.len(), 1, "{name}/{edition:?}");
                assert!(report.definitions[roles[0].definition].node.is_some());
                assert!(
                    report.invocations.iter().any(|invocation| {
                        let definition = &report.definitions[invocation.definition];
                        matches!(definition.body, Body::Declarative(_))
                            && matches!(
                                report.package.names.get(definition.name),
                                "selected_2021" | "chosen"
                            )
                    }),
                    "{name}/{edition:?}"
                );
            }
            super::lower_report(&report).map_err(|error| format!("{name}/{edition:?}: {error}"))?;
        }
    }
    assert!(Builtin::CorePanic.source_only());
    Ok(())
}

#[test]
fn complete_allowance_headers_retain_data_without_source_builtin_privilege() -> TestResult {
    for name in HEADER_POSITIVE {
        let report = report_in_edition(&source(name)?, Edition::Rust2024)
            .map_err(|error| format!("{name}: {error}"))?;
        assert!(
            report.invocations.iter().all(|invocation| {
                !matches!(report.definitions[invocation.definition].body,
                Body::Builtin(role) if role.source_only())
            }),
            "{name}"
        );
        super::lower_report(&report).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

#[test]
fn panic_delegation_relocates_existing_nonzero_expression_handles_without_new_payloads()
-> TestResult {
    let report = report_in_edition(&source("forwarded-expression")?, Edition::Rust2024)?;
    assert_eq!(report.captures.len(), 2);
    let mut forwarded = 0;
    let mut identity = None;
    for unit in &report.package.units {
        for capture in &unit.configured.ast.captures {
            for tree in capture.nodes() {
                let rustgm::syntax::token_tree::TreeKind::Token(
                    rustgm::lex::token::Kind::Interpolated(id),
                ) = tree.kind
                else {
                    continue;
                };
                let (file, tokens) = report.captures.tokens(id).map_err(|error| error.message)?;
                if let [token] = tokens
                    && report
                        .package
                        .unit(file)
                        .source
                        .text()
                        .get(token.start as usize..token.end as usize)
                        == Some("7u32")
                {
                    assert!(id.get() > 0);
                    assert!(identity.is_none_or(|previous| previous == id));
                    identity = Some(id);
                    forwarded += 1;
                }
            }
        }
    }
    assert!(identity.is_some());
    assert!(forwarded >= 2);
    Ok(())
}

#[test]
fn source_panic_rejects_actual_handler_and_permission_frontiers() -> TestResult {
    for edition in editions() {
        for (name, diagnostic) in [
            (
                "missing-modern-handler",
                "unresolved macro is not in textual scope or path scope",
            ),
            (
                "rejected-argument-tokens",
                "no macro rule matches this invocation",
            ),
            (
                "unknown-builtin-role",
                "source builtin macro expansion is not implemented",
            ),
        ] {
            let error = report_in_edition(&source(name)?, edition)
                .err()
                .ok_or_else(|| format!("{name}/{edition:?}: invalid source accepted"))?;
            assert!(
                error.to_string().contains(diagnostic),
                "{name}/{edition:?}: {error}"
            );
        }
    }
    for (name, diagnostic) in [
        (
            "permission-missing-feature",
            "internal macro allowance requires allow_internal_unstable",
        ),
        (
            "permission-nonmacro-owner",
            "internal macro allowance requires a macro declaration",
        ),
        (
            "permission-malformed-list",
            "internal macro allowance list is not implemented",
        ),
    ] {
        let error = report_in_edition(&source(name)?, Edition::Rust2024)
            .err()
            .ok_or_else(|| format!("{name}: rejected candidate profile accepted"))?;
        assert!(error.to_string().contains(diagnostic), "{name}: {error}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_panic_native_answers_preserve_handler_identity_and_nested_inputs() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/source-panic-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let consumer =
        std::fs::read_to_string(root.join("pa31/tests/spec/source-panic-macros/consumer.c"))?;
    for edition in editions() {
        for &(name, expected) in POSITIVE {
            if !admitted(name, edition) {
                continue;
            }
            let program = lower_in_edition(&source(name)?, edition)?;
            crate::ffi_program::check(
                root,
                &folder,
                &format!("{name}-{edition:?}"),
                &program,
                &consumer,
                expected,
            )?;
        }
    }
    for name in HEADER_POSITIVE {
        let program = lower_in_edition(&source(name)?, Edition::Rust2024)?;
        crate::ffi_program::check(root, &folder, name, &program, &consumer, "7\n")?;
    }
    Ok(())
}
