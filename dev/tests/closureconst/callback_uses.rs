//! Creating an initializer and invoking it retain different declaration effects.
use std::process::Command;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn callback_formation_pays_ordinary_bounds_and_actual_calls_pay_const_effects() -> Result {
    let mut failures = Vec::new();
    let mut count = 0;
    for pa in [19, 24, 25, 31] {
        let cases =
            crate::resolve::fixture(&format!("pa{pa}/tests/spec/callback-formation/cases.tsv"))?;
        for line in cases.lines() {
            let (path, expected) = line.split_once('\t').ok_or("callback outcome columns")?;
            let pending = path.ends_with("/alias-paid.rs") || path.ends_with("/reference-paid.rs");
            for mode in ["--emit-types", "--emit-lowir"] {
                if pending && mode == "--emit-lowir" {
                    continue;
                }
                let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
                command.args([mode, "--edition=2024", "--cap-lints=allow"]);
                if pa == 31 {
                    command.args([
                        "--source-core",
                        "minicore@callback",
                        "--crate-name",
                        "minicore",
                        "--crate-disambiguator",
                        "callback",
                    ]);
                }
                let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .ok_or("repository root")?
                    .join(path);
                let output = command.arg(source).output()?;
                let error = String::from_utf8_lossy(&output.stderr);
                let status = if mode == "--emit-types" && path.ends_with("/pointer-call-reject.rs")
                {
                    // Pointer call legality belongs to the checked body; the pointer
                    // type itself does not promise const invocation.
                    0
                } else {
                    expected.parse::<i32>()?
                };
                if output.status.code() != Some(status)
                    || [
                        "budget",
                        "not implemented",
                        "changed its",
                        "invalid",
                        "panicked",
                    ]
                    .iter()
                    .any(|cause| error.contains(cause))
                {
                    failures.push(format!("{path} {mode}: {:?}: {error}", output.status));
                }
            }
            count += 1;
        }
    }
    assert_eq!(count, 46);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn creation_and_paid_invocation_work_scales_without_unrelated_type_work() -> Result {
    for prefix in ["width", "calls-width"] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let mut baseline = None;
            for noise in [0, 128] {
                let source = crate::resolve::fixture(&format!(
                    "pa25/tests/spec/callback-formation/{prefix}-{width}-noise-{noise}.rs"
                ))?;
                let report = crate::resolve::resolve(&source, true)?;
                let facts = rustgm::pipeline::analyze_source(&report)?.facts;
                let work = [
                    facts.body_work.inference_steps,
                    facts.body_work.trait_selections,
                    facts.methods.selection_steps,
                    facts.methods.selection_probes,
                ];
                println!("callback {prefix} width={width} noise={noise} work={work:?}");
                if noise == 0 {
                    if let Some(before) = previous {
                        for (old, actual) in std::iter::zip(before, work) {
                            assert!(actual <= 4 * old + 2048, "{before:?}->{work:?}");
                        }
                    }
                    previous = Some(work);
                    baseline = Some(work);
                } else {
                    assert_eq!(baseline, Some(work));
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM backend"]
fn initializer_storage_and_actual_const_call_execute_the_original_callbacks() -> Result {
    let inputs = [
        "native-initializer",
        "native-const-pointer",
        "native-paid-call",
    ]
    .map(|name| {
        crate::resolve::fixture(&format!("pa25/tests/spec/callback-formation/{name}.rs"))
            .map(|source| (name, source))
    });
    let inputs = inputs
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let cases = inputs
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect::<Vec<_>>();
    crate::native_references::check_literal_cases_bootstrap(
        "pa25-callback-formation/native",
        &cases,
    )
}
