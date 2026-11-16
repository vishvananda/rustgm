//! Source defining policy and reached intrinsic provenance are independent authorities.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = "pa31/tests/spec/source-guaranteed-compare";
fn report_file(name: &str) -> Result<rustgm::resolve::expansion::Report> {
    packet_report(ROOT, name)
}
fn packet_report(packet: &str, name: &str) -> Result<rustgm::resolve::expansion::Report> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(crate::resolve::source_provider_config(
        &crate::resolve::fixture(&format!("{packet}/{name}.rs"))?,
        "guaranteed_pointers",
        &config,
    )?)
}
fn report(name: &str) -> Result<rustgm::resolve::expansion::Report> {
    report_file(&format!("{name}-source-profile"))
}
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    program_file(&format!("{name}-source-profile"))
}
fn program_file(name: &str) -> Result<rustgm::lowir::program::Program> {
    lowered(&report_file(name)?)
}
fn lowered(report: &rustgm::resolve::expansion::Report) -> Result<rustgm::lowir::program::Program> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn actual_pointer_query_keeps_static_memory_integer_and_expired_provenance() -> Result {
    for name in [
        "int-equal",
        "int-different",
        "static-same",
        "static-different",
        "static-null",
        "memory-same",
        "memory-null",
        "expired-pointer-null",
    ] {
        program(&format!("source-{name}-v2-builtin-tool"))?;
    }
    for name in [
        "static-offset",
        "static-one-past",
        "static-far-aligned",
        "static-far-residue",
        "memory-residue",
        "zst-same",
        "zst-different",
        "function-same",
        "function-null",
    ] {
        program(&format!("source-{name}-v3-builtin-tool"))?;
    }
    Ok(())
}
fn rejected(name: &str) -> String {
    match report(name).and_then(|report| {
        pipeline::analyze_source(&report)
            .map(|_| ())
            .map_err(Into::into)
    }) {
        Ok(()) => format!("accepted {name}"),
        Err(error) => error.to_string(),
    }
}
#[test]
fn defining_marker_does_not_supply_call_unsafe_or_const_permission() -> Result {
    pipeline::analyze_source(&report("source-marked-unused-builtin-tool")?)?;
    let mut failures = Vec::new();
    for (name, causes) in [
        (
            "source-marker-missing-v2-builtin-tool",
            &["trait obligation", "raw pointer comparisons"][..],
        ),
        ("source-unsafe-body-v2-builtin-tool", &["unsafe"][..]),
        (
            "source-ordinary-const-lookalike-v2-builtin-tool",
            &["const", "trait obligation", "raw pointer comparisons"][..],
        ),
        (
            "source-ordinary-runtime-lookalike-v2-builtin-tool",
            &["const"][..],
        ),
        (
            "source-nonconst-intrinsic-call-v2-builtin-tool",
            &["const"][..],
        ),
        ("source-marked-pointer-called-builtin-tool", &["const"][..]),
        ("source-marked-ordinary-called-builtin-tool", &["const"][..]),
    ] {
        let error = rejected(name);
        eprintln!("{name}: {error}");
        if !causes.iter().any(|cause| error.contains(cause)) || error.contains("not implemented") {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn actual_feature_signature_reification_and_sized_checks_remain() {
    let mut failures = Vec::new();
    for (name, causes) in [
        (
            "intrinsics-feature-missing",
            &["explicit source features"][..],
        ),
        (
            "internal-feature-missing",
            &["rustc_attrs", "explicit source features"][..],
        ),
        ("signature-unsafe", &["safety"][..]),
        ("signature-result", &["intrinsic", "type"][..]),
        ("signature-generics", &["generic"][..]),
        ("signature-mutability", &["intrinsic", "type"][..]),
        ("reification", &["intrinsic"][..]),
        ("wide-call", &["Sized", "size", "sized"][..]),
    ] {
        let error = rejected(&format!("source-{name}-builtin-tool"));
        eprintln!("{name}: {error}");
        if !causes.iter().any(|cause| error.contains(cause))
            || error.contains("source tool registration")
        {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn comparison_preserves_one_past_nullness_and_actual_pointee_liveness() -> Result {
    let source = |name: &str| -> Result<rustgm::resolve::expansion::Report> {
        Ok(crate::resolve::source_provider(
            &crate::resolve::fixture(&format!("{ROOT}/{name}.rs"))?,
            "guaranteed_storage",
        )?)
    };
    for name in ["source-one-past-null-profile", "source-live-read-profile"] {
        pipeline::analyze_source(&source(name)?)?;
    }
    let error = pipeline::analyze_source(&source("source-expired-read-profile")?)
        .err()
        .ok_or("expired pointee read was accepted after comparison")?
        .to_string();
    assert!(
        error.contains("storage") || error.contains("expired"),
        "{error}"
    );
    Ok(())
}
#[test]
fn native_demand_keeps_the_original_and_changed_actual_fallback_bodies() -> Result {
    for name in [
        "source-native-standard-v2-builtin-tool",
        "source-native-changed-v2-builtin-tool",
    ] {
        program(name)?;
    }
    Ok(())
}
#[test]
fn original_intrinsic_interval_is_retained() -> Result {
    let original = crate::resolve::fixture(&format!("{ROOT}/original-function.txt"))?;
    for name in [
        "source-native-standard-v2-builtin-tool",
        "source-int-equal-v2-builtin-tool",
    ] {
        assert!(crate::resolve::fixture(&format!("{ROOT}/{name}.rs"))?.contains(&original));
    }
    Ok(())
}
fn native(name: &str, observer: &str, expected: &str) -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/guaranteed-pointers/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        name,
        &program_file(name)?,
        &crate::resolve::fixture(&format!("{ROOT}/{observer}.c"))?,
        expected,
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn native_original_fallback_matches_actual_pointer_equality() -> Result {
    native(
        "source-native-standard-v2-builtin-tool-source-profile",
        "observer",
        "guaranteed 1 0 1\n",
    )
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn native_changed_fallback_executes_real_ordinary_helper() -> Result {
    native(
        "source-native-changed-v2-builtin-tool-source-profile",
        "observer",
        "guaranteed 99 99 1\n",
    )
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn native_operator_syntax_and_explicit_method_keep_distinct_semantics() -> Result {
    native(
        "source-altered-equality-profile",
        "operator-observer",
        "primitive-pointer 1 0 0 0\n",
    )
}

const ORDERING: &str = "pa31/tests/spec/pointer-ordering";
const ORDERING_CASES: [(&str, &str); 4] = [
    ("operators", "3 10 12\n"),
    ("explicit", "3 10 12\n"),
    ("generic", "3 10 12\n"),
    ("nominal", "12 10 3\n"),
];
/// Functions whose body calls their own symbol.
fn self_calls(text: &str) -> Vec<&str> {
    let mut current = None;
    let mut calls = Vec::new();
    for line in text.lines() {
        if let Some(header) = line.strip_prefix("function @") {
            current = header.split('(').next();
        } else if let (Some(name), Some((_, callee))) = (current, line.split_once(" call ")) {
            let target = callee
                .split_once('@')
                .and_then(|(_, symbol)| symbol.split('(').next());
            if target == Some(name) && calls.last() != Some(&name) {
                calls.push(name);
            }
        }
    }
    calls
}
/// Every comparison of two raw pointers is builtin, as rustc's
/// `is_builtin_binop` makes it, so core's raw pointer `PartialOrd` bodies
/// `*self < *other` compare addresses instead of calling themselves.
#[test]
fn ordered_raw_pointer_operators_are_builtin_comparisons() -> Result {
    for (name, _) in ORDERING_CASES {
        let program =
            lowered(&packet_report(ORDERING, name)?).map_err(|error| format!("{name}: {error}"))?;
        let mut text = Vec::new();
        rustgm::lowir::writer::write(&program, &mut text)?;
        let text = String::from_utf8(text)?;
        assert_eq!(self_calls(&text), Vec::<&str>::new(), "{name}");
        assert!(
            text.contains(" cmp ult ptr "),
            "{name}: no ordered comparison"
        );
    }
    for (name, cause) in [
        ("const-operator", "can't compare `*const T` with `*const T`"),
        ("mismatch", "incompatible types"),
    ] {
        let error = packet_report(ORDERING, name)
            .and_then(|report| lowered(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn native_ordered_raw_pointer_operators_compare_addresses() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/pointer-ordering/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let observer = crate::resolve::fixture(&format!("{ORDERING}/observe.c"))?;
    for (name, expected) in ORDERING_CASES {
        let program = lowered(&packet_report(ORDERING, name)?)?;
        crate::ffi_program::check(&root, &folder, name, &program, &observer, expected)?;
    }
    Ok(())
}
