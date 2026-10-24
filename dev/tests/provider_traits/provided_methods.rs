//! Const trait defaults check defining legality without granting const impl use.
use super::{TestResult, lower, report};
use rustgm::{pipeline, sema::model::Operation, support::constness::Constness};

#[test]
fn direct_member_partial_inference_retains_its_first_rejection() -> TestResult {
    let source =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../pa31/tests/spec/trait-default-effects/recovery/provider-partial-inference.rs",
        ))?;
    let error = pipeline::analyze_source(&report(&source)?)
        .err()
        .ok_or("unresolved member accepted")?;
    println!("partial_member_first_rejection={error}");
    Ok(())
}

fn fixture(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/trait-default-effects/provider")
            .join(format!("{name}.rs")),
    )?)
}

#[test]
fn implicit_self_member_defaults_keep_ordinary_implementation_runtime_and_const_opposites()
-> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/trait-default-effects/members");
    for name in ["implicit-unused", "implicit-runtime"] {
        let source = std::fs::read_to_string(folder.join(format!("provider-{name}.rs")))?;
        lower(&source).map_err(|e| format!("{name}: {e}"))?;
    }
    for name in [
        "ordinary-const-reject",
        "ordinary-bound-reject",
        "cross-ordinary-reject",
    ] {
        let source = std::fs::read_to_string(folder.join(format!("provider-{name}.rs")))?;
        assert!(lower(&source).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn defaults_reject_nonconst_helpers_even_unused_and_preserve_runtime_opposites() -> TestResult {
    for name in [
        "invalid-unused",
        "invalid-ordinary-runtime",
        "invalid-const-unused",
        "nested-called-reject",
    ] {
        let error = lower(&fixture(name)?)
            .err()
            .ok_or("invalid default accepted")?;
        assert!(
            error
                .to_string()
                .contains("non-const calls are not allowed"),
            "{name}: {error}"
        );
    }
    for name in [
        "pure-unused",
        "ordinary-runtime",
        "conditional-runtime",
        "nested-ordinary",
        "ordinary-trait",
    ] {
        lower(&fixture(name)?).map_err(|e| format!("{name}: {e}"))?;
    }
    for name in ["ordinary-const-reject", "conditional-const-unpaid"] {
        assert!(lower(&fixture(name)?).is_err(), "{name}");
    }
    for name in ["const-required", "conditional-const-required"] {
        let error = lower(&fixture(name)?)
            .err()
            .ok_or("unsupported default executed")?;
        assert!(
            error.to_string().contains("not implemented"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn actual_default_and_nested_call_use_facts_have_distinct_defining_owners() -> TestResult {
    let report = report(&fixture("nested-ordinary")?)?;
    let analysis = pipeline::analyze_source(&report)?;
    for (name, expected) in [
        ("provided", Constness::Conditional),
        ("nested", Constness::Ordinary),
    ] {
        let function = analysis
            .facts
            .functions
            .iter()
            .find(|function| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == name
            })
            .ok_or("function")?;
        let owner = function.body.ok_or("actual body")?;
        let ast = &report.package.unit(owner.file).configured.ast;
        let mut found = false;
        for (index, _) in ast.nodes.iter().enumerate() {
            let key = rustgm::resolve::unit::NodeKey {
                file: owner.file,
                node: rustgm::syntax::ast::NodeId::new(u32::try_from(index)?),
            };
            let Some(Operation::Call {
                source_use: Some(source),
                ..
            }) = analysis.facts.node(key).and_then(|node| node.operation)
            else {
                continue;
            };
            if let Ok(effect) = source.effect_for(&analysis.facts, owner) {
                assert_eq!(effect, expected);
                found = true;
            }
        }
        assert!(found, "{name}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn actual_default_runtime_and_cleanup_native_outcomes_match() -> TestResult {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/default-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for name in [
        "ordinary-runtime",
        "conditional-runtime",
        "nested-ordinary",
        "ordinary-trait",
    ] {
        super::native::execute(&folder, name, &fixture(name)?, 29)?;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/trait-default-effects");
    let member = std::fs::read_to_string(root.join("members/provider-implicit-runtime.rs"))?;
    super::native::execute(&folder, "implicit-runtime", &member, 29)?;
    let cleanup = std::fs::read_to_string(root.join("cleanup/runtime.rs"))?;
    super::native::execute(&folder, "runtime-cleanup", &cleanup, 7)?;
    Ok(())
}

#[test]
fn ordinary_impl_of_const_default_retains_runtime_destructor_cleanup() -> TestResult {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/trait-default-effects/cleanup/runtime.rs");
    lower(&std::fs::read_to_string(path)?)?;
    Ok(())
}

#[test]
fn defining_drop_promises_reject_missing_bounds_and_keep_scoped_copy() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/trait-default-effects/destruction");
    lower(&std::fs::read_to_string(
        root.join("provider-copy-dropless.rs"),
    )?)?;
    let error = lower(&std::fs::read_to_string(
        root.join("provider-missing-destruct.rs"),
    )?)
    .err()
    .ok_or("missing defining Destruct promise accepted")?;
    assert!(
        error
            .to_string()
            .contains("conditional Destruct obligation"),
        "{error}"
    );
    Ok(())
}

#[test]
fn defining_drop_uses_live_cleanup_and_rejects_lookalike_or_ordinary_promises() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/trait-default-effects/destruction");
    lower(&std::fs::read_to_string(
        root.join("provider-moved-self.rs"),
    )?)?;
    for name in [
        "provider-lookalike-copy.rs",
        "provider-ordinary-destruct.rs",
    ] {
        assert!(
            lower(&std::fs::read_to_string(root.join(name))?).is_err(),
            "{name}"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn copy_bound_defining_default_remains_runtime_dropless_native_seven() -> TestResult {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/trait-default-effects/destruction/provider-copy-dropless.rs"),
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/default-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    super::native::execute(&folder, "copy-dropless", &source, 7)?;
    let moved = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/trait-default-effects/destruction/provider-moved-self.rs"),
    )?;
    super::native::execute(&folder, "moved-self", &moved, 7)
}

#[test]
fn whole_source_defining_cleanup_width_and_unrelated_definitions_keep_original_proofs() -> TestResult
{
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/trait-default-effects/destruction/work");
    for (width, noise) in [(8, 0), (32, 0), (128, 0), (8, 128)] {
        let name = format!("claims-width-{width}-noise-{noise}.rs");
        lower(&std::fs::read_to_string(root.join(&name))?)
            .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn defining_cleanup_width_and_unrelated_definitions_execute_native_seven() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa31/tests/spec/trait-default-effects/destruction/work");
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/default-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (width, noise) in [(8, 0), (32, 0), (128, 0), (8, 128)] {
        let name = format!("claims-width-{width}-noise-{noise}");
        super::native::execute(
            &folder,
            &name,
            &std::fs::read_to_string(root.join(format!("{name}.rs")))?,
            7,
        )?;
    }
    Ok(())
}
