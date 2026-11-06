//! Actual `Reborrow` and `CoerceShared` preserve ordinary source and loan proof.
use crate::resolve as source;
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/reborrow-source";
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let text = source::fixture(&format!("{PACKET}/{name}.rs"))?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = source::source_provider_config(&text, "value_reborrow", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_source_operations_and_unused_alternate_headers_remain_real() -> Result {
    for name in ["header-extra", "header-unsafe", "source", "native"] {
        program(name).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn source_impl_fields_and_operation_loans_retain_their_ordinary_rejections() -> Result {
    for (name, causes) in [
        ("enum", &["struct"][..]),
        ("noncopy", &["field", "Copy", "trait"][..]),
        ("multiple-fields", &["field"][..]),
        ("no-life", &["lifetime"][..]),
        ("two-life", &["lifetime"][..]),
        ("target-type", &["field", "trait", "coercion"][..]),
        ("target-copy", &["bound", "Copy", "trait"][..]),
        ("missing", &["bound", "trait"][..]),
        ("fake", &["moved"][..]),
        ("overlap-v3", &["borrow", "loan"][..]),
        ("shared-write-v2", &["borrow", "loan"][..]),
        ("shared-move", &["borrow", "loan"][..]),
        ("escape", &["lifetime", "borrow"][..]),
        ("unsafe-source-v2", &["unsafe"][..]),
    ] {
        let name = format!("local-{name}");
        let error = program(&name)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        let text = error.to_string();
        assert!(
            causes.iter().any(|cause| text.contains(cause)),
            "{name}: {text}"
        );
        assert!(
            !["not implemented", "budget", "exhausted", "missing layout"]
                .iter()
                .any(|cause| text.contains(cause)),
            "{name}: {text}"
        );
    }
    Ok(())
}
#[test]
fn generic_expectations_and_operation_field_lifetimes_remain_ordinary() {
    let mut failures = Vec::new();
    for name in ["successor-native-v3", "successor-static-unused-v1"] {
        if let Err(error) = program(name) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for (name, causes) in [
        ("successor-inferred-v1", &["moved"][..]),
        ("successor-relay-v1", &["lifetime", "borrow"][..]),
        ("successor-static-use-v1", &["lifetime", "borrow"][..]),
        ("successor-unsafe-v2", &["unsafe"][..]),
    ] {
        match program(name) {
            Ok(_) => failures.push(format!("accepted {name}")),
            Err(error) => {
                let text = error.to_string();
                if !causes.iter().any(|cause| text.contains(cause))
                    || ["not implemented", "budget", "exhausted", "missing layout"]
                        .iter()
                        .any(|cause| text.contains(cause))
                {
                    failures.push(format!("{name}: {text}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn wrapper_parameters_keep_loans_without_explicit_reference_values() -> Result {
    program("wrapper-parameter-valid-v2")?;
    for name in ["wrapper-parameter-valid", "wrapper-parameter-overlap"] {
        let error = program(name)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        let text = error.to_string();
        assert!(
            ["borrow", "lifetime", "loan"]
                .iter()
                .any(|cause| text.contains(cause)),
            "{name}: {text}"
        );
        assert!(
            !["not implemented", "budget", "exhausted"]
                .iter()
                .any(|cause| text.contains(cause)),
            "{name}: {text}"
        );
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn runtime_operands_implicit_copy_explicit_clone_and_ctfe_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/value-reborrow-native/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        ("native", "observer", "reborrow 23 23 47 47 0 99 31\n"),
        (
            "successor-native-v3",
            "successor-native-v3",
            "successor 17 23 19 31 23 59 1\n",
        ),
    ] {
        let directory = folder.join(name);
        std::fs::create_dir_all(&directory)?;
        crate::ffi_program::check(
            &root,
            &directory,
            name,
            &program(name)?,
            &source::fixture(&format!("{PACKET}/{observer}.c"))?,
            expected,
        )?;
    }
    Ok(())
}

#[test]
fn checked_source_operations_retain_their_original_places() -> Result {
    let text = source::fixture(&format!("{PACKET}/source.rs"))?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = source::source_provider_config(&text, "value_reborrow", &config)?;
    let facts = rustgm::sema::declarations::analyze(&report)?;
    let facts = rustgm::sema::bodies::analyze(&report, facts)?;
    let program = rustgm::body::build_concrete(&report, &facts)?;
    let mut rendered = Vec::new();
    rustgm::body::body_view::write(&program, &facts.types, &mut rendered)?;
    let rendered = String::from_utf8(rendered)?;
    let mut count = 0;
    for body in &program.bodies {
        for value in &body.values {
            if let rustgm::body::ir::Rvalue::Reborrow { place, .. } = value.kind {
                count += 1;
                assert!(
                    matches!(place, rustgm::body::ir::Place::Local(_)),
                    "{rendered}"
                );
            }
        }
    }
    // The independently retained Rust MIR has exactly two exclusive loans and
    // one shared loan of the local wrapper, with no constructor-initializer loan.
    assert_eq!(count, 3, "{rendered}");
    Ok(())
}
