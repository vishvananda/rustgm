//! Authenticated const creation and checked child effects keep ordinary closure boundaries.
#[path = "closureconst/ambient_calls.rs"]
mod ambient_calls;
#[path = "closureconst/callback_uses.rs"]
mod callback_uses;
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn real_const_closure_calls_execute_shared_mutable_and_owned_captures() -> Result {
    let input = crate::resolve::fixture("pa25/tests/spec/const-closures/execution-pure.rs")?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report =
        crate::resolve::source_provider_config(&input, "const_capture_execution", &config)?;
    let analysis = pipeline::analyze_source(&report)?;
    for (name, expected) in [("SHARED", 23), ("MUTABLE", 47), ("ONCE", 23)] {
        let initializer = analysis
            .facts
            .initializers
            .iter()
            .find(|initializer| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                    == name
            })
            .ok_or("actual const closure initializer is absent")?;
        let reader = rustgm::sema::values::Reader::new(&analysis.facts);
        let actual = reader
            .definition(initializer.definition)
            .ok_or("const closure value is incomplete")?;
        assert_eq!(
            actual.as_scalar()?,
            rustgm::sema::model::Scalar::Integer(expected)
        );
    }
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}

fn analyze(input: &str) -> std::result::Result<(), String> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(input, "constant_closures", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

#[test]
fn actual_wrapper_and_child_preserve_creation_effect_and_nested_contexts() -> Result {
    let input = crate::resolve::fixture("pa31/tests/spec/inherited-overflow/const-closure-v3.rs")?;
    analyze(&input)?;
    let runtime = input.replace("pub const fn original_word", "pub fn original_word");
    let error = analyze(&runtime)
        .err()
        .ok_or("runtime const creation accepted")?;
    assert!(error.contains("outside of const contexts"), "{error}");
    let nested = input.clone() + "pub const fn nested(){let _=||{let _=const || 23usize;};}";
    let error = analyze(&nested)
        .err()
        .ok_or("ordinary child inherited const creation")?;
    assert!(error.contains("outside of const contexts"), "{error}");
    Ok(())
}

#[test]
fn unused_const_body_checks_calls_without_changing_ordinary_child_effects() -> Result {
    let input = crate::resolve::fixture("pa31/tests/spec/inherited-overflow/const-closure-v3.rs")?;
    let source = input + "fn runtime()->usize{23}\npub const fn nested(){let _=||runtime();}";
    analyze(&source)?;
    let constant = source.replace("let _=||runtime()", "let _=const ||runtime()");
    let error = analyze(&constant)
        .err()
        .ok_or("unused const body skipped call legality")?;
    assert!(error.contains("non-const"), "{error}");
    Ok(())
}

#[test]
fn real_source_calls_keep_capture_loans_moves_and_lifetime_opponents() -> Result {
    let mention = crate::resolve::fixture("pa25/tests/spec/const-closures/source-moved-owned.rs")?;
    analyze(&mention)?;
    for (name, causes) in [
        ("source-ordinary-call", &["non-const"][..]),
        ("source-overlap", &["live reference loan"][..]),
        ("source-mutable-binding", &["immutable"][..]),
        ("source-once-twice", &["moved", "consumed"][..]),
        (
            "source-moved-owned-v2",
            &["moved", "consumed", "uninitialized"][..],
        ),
        (
            "source-escaping-capture",
            &["outlive", "lifetime", "universal"][..],
        ),
    ] {
        let input = crate::resolve::fixture(&format!("pa25/tests/spec/const-closures/{name}.rs"))?;
        let error = analyze(&input)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(
            causes.iter().any(|cause| error.contains(cause)),
            "{name}: {error}"
        );
        eprintln!("{name}: {error}");
    }
    Ok(())
}

#[test]
fn const_qualification_keeps_existing_hot_record_widths() {
    use std::mem::size_of;
    assert_eq!(size_of::<rustgm::types::closure::Declaration>(), 1);
    assert_eq!(size_of::<rustgm::types::Kind>(), 24);
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    assert_eq!(size_of::<rustgm::body::ir::Owner>(), 20);
    assert_eq!(size_of::<rustgm::body::ir::Value>(), 24);
    assert_eq!(size_of::<rustgm::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<rustgm::sema::closures::Body>(), 76);
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn captured_ctfe_values_and_ordinary_const_closure_calls_reach_native() -> Result {
    use std::{path::Path, process::Command};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let input = crate::resolve::fixture("pa25/tests/spec/const-closures/execution-pure.rs")?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(&input, "const_capture_native", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let exports = facts
        .functions
        .iter()
        .filter(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                .starts_with("pure_")
        })
        .map(|function| function.definition)
        .collect::<Vec<_>>();
    assert_eq!(exports.len(), 5);
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?;
    let folder = root
        .join("obj/parallel/const-closure-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let ir = folder.join("source.lowir");
    let object = folder.join("source.o");
    let binary = folder.join("observer");
    rustgm::lowir::writer::write(&program, &mut std::fs::File::create(&ir)?)?;
    for (stage, command) in [
        (
            "backend",
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&ir)
                .arg("-o")
                .arg(&object),
        ),
        (
            "link",
            Command::new("cc")
                .arg(root.join("pa25/tests/spec/const-closures/execution-pure.c"))
                .arg(&object)
                .arg("-o")
                .arg(&binary),
        ),
        ("native", Command::new(&binary).arg("23")),
    ] {
        std::fs::write(
            folder.join(format!("{stage}.command")),
            format!("{command:?}\n"),
        )?;
        crate::provider_traits::native::run(&folder, stage, command)?;
    }
    assert_eq!(
        std::fs::read(folder.join("native.stdout"))?,
        b"closure-source 23 47 23 23 47\n"
    );
    Ok(())
}
