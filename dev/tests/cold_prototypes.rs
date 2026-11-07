//! No-body advisories preserve actual trait obligations and foreign call contracts.
use rustgm::{pipeline, resolve::expansion::Report};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = "pa31/tests/spec/source-cold-prototypes";
fn input(name: &str) -> std::io::Result<String> {
    crate::resolve::fixture(&format!("{ROOT}/{name}.rs"))
}
fn full(report: &Report) -> std::result::Result<rustgm::lowir::program::Program, String> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(report).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .map_err(|e| e.to_string())
}
#[test]
fn no_body_cold_words_use_actual_trait_and_foreign_headers() -> Result {
    for name in [
        "trait-prototype-allow",
        "trait-required-call",
        "foreign-deny",
        "foreign-duplicate-allow",
        "foreign-safe-call",
        "macro-prototype",
        "duplicate-prototype-allow",
    ] {
        let report = crate::resolve::resolve(&input(name)?, true)?;
        full(&report).map_err(|error| format!("{name}: {error}"))?;
    }
    let report = crate::resolve::source_provider(&input("source-native")?, "cold_prototypes")?;
    full(&report)?;
    Ok(())
}
#[test]
fn cold_prototypes_never_pay_body_bound_abi_safety_const_or_lint_obligations() -> Result {
    let mut failures = Vec::new();
    for (name, causes) in [
        (
            "trait-prototype-deny",
            &["cold attribute has no effect on a required trait method"][..],
        ),
        ("trait-missing-method", &["missing"][..]),
        (
            "trait-missing-bound",
            &["trait", "method", "associated"][..],
        ),
        ("foreign-duplicate-deny", &["duplicate cold attribute"][..]),
        ("foreign-unsafe-call", &["unsafe"][..]),
        ("foreign-const-call", &["const"][..]),
        ("foreign-wrong-result", &["type"][..]),
        ("foreign-wrong-abi", &["ABI", "convention"][..]),
        ("foreign-generic", &["foreign"][..]),
        ("free-prototype", &["requires a definition"][..]),
        (
            "macro-prototype-deny",
            &["cold attribute has no effect on a required trait method"][..],
        ),
        ("source-free-prototype", &["requires a definition"][..]),
    ] {
        let source = input(name)?;
        let report = if name == "source-free-prototype" {
            crate::resolve::source_provider(&source, "cold_prototypes")
        } else {
            crate::resolve::resolve(&source, true)
        };
        let actual = report.and_then(|report| full(&report));
        match actual {
            Ok(_) => failures.push(format!("accepted {name}")),
            Err(error)
                if !causes.iter().any(|cause| error.contains(cause))
                    || error.contains("not implemented") =>
            {
                failures.push(format!("{name}: {error}"));
            }
            Err(error) => eprintln!("{name}: {error}"),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM backend and a C compiler"]
fn cold_foreign_declarations_execute_real_unsafe_and_safe_c_calls() -> Result {
    let report = crate::resolve::source_provider(&input("source-native")?, "cold_prototypes")?;
    let program = full(&report)?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/source-transport/cold-prototypes/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "cold-foreign",
        &program,
        &crate::resolve::fixture(&format!("{ROOT}/observer.c"))?,
        "cold-foreign 28 36 2\n",
    )?;
    Ok(())
}
