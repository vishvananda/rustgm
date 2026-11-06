//! Actual automatic trait declarations retain normal structural and scoped proof.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
const ROOT: &str = "pa20/tests/spec/custom-auto-source";
const CASES: &[(&str, bool, bool)] = &[
    ("target-dyn.rs", false, false),
    ("target-unsized.rs", false, false),
    ("target-sized.rs", true, false),
    ("target-sized-premise.rs", true, false),
    ("negative-pointer-header.rs", true, false),
    ("negative-pointer-demand.rs", false, false),
    ("unused-regions-1.rs", false, false),
    ("unused-regions-400.rs", false, false),
    ("native.rs", true, true),
    ("original-headers.rs", true, true),
    ("original-demands.rs", true, true),
    ("original-mut-reference.rs", false, true),
    ("original-cell-negative.rs", false, true),
    ("original-shared-cell.rs", false, true),
    ("structural-positive.rs", true, true),
    ("field-negative.rs", false, true),
    ("raw-negative.rs", false, true),
    ("reference-negative.rs", false, true),
    ("zero-array-negative.rs", false, true),
    ("phantom-negative.rs", false, true),
    ("generic-premise.rs", true, true),
    ("generic-missing.rs", false, true),
    ("projection-premise.rs", true, true),
    ("projection-missing.rs", false, true),
    ("lifetime-premise.rs", true, true),
    ("lifetime-escape.rs", false, true),
    ("explicit-positive.rs", true, true),
    ("conditional-positive.rs", true, true),
    ("conditional-negative.rs", false, true),
    ("recursive-positive.rs", true, true),
    ("recursive-negative.rs", false, true),
    ("recursive-condition-negative.rs", false, true),
    ("declaration-restrictions.rs", false, true),
    ("implementation-safety.rs", false, true),
    ("where-premise.rs", true, true),
    ("where-no-inversion.rs", false, true),
    ("decl-feature-off.rs", false, false),
    ("decl-generic.rs", false, false),
    ("decl-supertrait.rs", false, false),
    ("decl-member.rs", false, false),
    ("decl-const.rs", false, false),
    ("decl-where.rs", false, false),
    ("decl-empty.rs", true, false),
];
fn analyze(input: &str, provider: bool) -> std::result::Result<(), String> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = if provider {
        crate::resolve::source_provider_config(input, "custom_auto", &config)?
    } else {
        crate::resolve::resolve_config(input, true, &config)?
    };
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[test]
fn automatic_source_headers_and_structural_consumers() -> Result {
    let mut errors = Vec::new();
    for &(name, accepted, provider) in CASES {
        if !accepted {
            continue;
        }
        let input = crate::resolve::fixture(&format!("{ROOT}/{name}"))?;
        if let Err(error) = analyze(&input, provider) {
            errors.push(format!("{name}: {error}"));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}
#[test]
fn automatic_source_opponents_preserve_actual_causes() -> Result {
    let mut errors = Vec::new();
    for &(name, accepted, provider) in CASES {
        if accepted {
            continue;
        }
        let input = crate::resolve::fixture(&format!("{ROOT}/{name}"))?;
        match analyze(&input, provider) {
            Ok(()) => errors.push(format!("{name}: unexpected acceptance")),
            Err(error) => {
                let expected = match name {
                    "target-dyn.rs" => "automatic traits cannot be implemented for dynamic objects",
                    "target-unsized.rs" => "automatic implementation generic subject must be Sized",
                    "original-mut-reference.rs" => {
                        "may not be safely transferred across an unwind boundary"
                    }
                    "lifetime-escape.rs" => "required lifetime does not outlive its target",
                    "declaration-restrictions.rs" | "decl-generic.rs" => {
                        "automatic traits cannot have generic parameters"
                    }
                    "implementation-safety.rs" => {
                        "trait and implementation safety declarations disagree"
                    }
                    "decl-feature-off.rs" => {
                        "automatic traits require their actual auto_traits feature request"
                    }
                    "decl-supertrait.rs" | "decl-where.rs" => {
                        "automatic traits cannot have supertrait or where bounds"
                    }
                    "decl-member.rs" => "automatic traits cannot declare associated members",
                    // Separate named syntax-profile gap, never semantic parity.
                    "decl-const.rs" => "expected punctuation",
                    _ => {
                        "trait obligation requires an unambiguous matching assumption or implementation"
                    }
                };
                if !error.contains(expected) {
                    errors.push(format!("{name}: expected {expected:?}, got {error}"));
                }
                eprintln!("{name}: {error}");
            }
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn genuine_automatic_bound_and_generic_struct_body_execute() -> Result {
    let input = crate::resolve::fixture(&format!("{ROOT}/native.rs"))?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(&input, "custom_auto", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/source-transport/custom-auto/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "custom-auto",
        &program,
        &crate::resolve::fixture(&format!("{ROOT}/observer.c"))?,
        "auto 23\n",
    )?;
    Ok(())
}
