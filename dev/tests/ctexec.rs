//! Real concrete cleanup selection stays fenced until the interpreter consumes it.
use crate::resolve as source;
use rustgm::pipeline;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn analyze(name: &str) -> Result<(rustgm::resolve::expansion::Report, pipeline::Analysis)> {
    let text = source::fixture(&format!("pa31/tests/spec/source-ctfe-cleanup/{name}.rs"))?;
    let report = source::source_provider(&text, "ctfe_cleanup_execution")?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn complete(name: &str) -> Result {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(name)?;
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

#[test]
fn proved_concrete_cleanup_requires_an_actual_interpreter_continuation() -> Result {
    // These are independently Rust-positive; this fence is unfinished execution,
    // not a matching Rust rejection or a completed CTFE contract.
    for name in ["ct41", "ct59", "ctonce", "ctcond"] {
        let error = analyze(name)
            .err()
            .ok_or_else(|| format!("{name}: executable cleanup was omitted"))?;
        assert!(
            error
                .to_string()
                .contains("constant interpreter cleanup scheduling is not implemented"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn real_nonconst_unsafe_and_opposing_loan_causes_remain_independent() -> Result {
    let mut failures = Vec::new();
    for (name, cause) in [
        ("ctbad", "const"),
        ("ctcall", "const"),
        ("ctunsf", "unsafe"),
        ("dborr", "loan"),
    ] {
        match complete(name) {
            Ok(()) => failures.push(format!("{name}: invalid source accepted")),
            Err(error) => {
                let text = error.to_string();
                eprintln!("{name}: {text}");
                if !text.contains(cause) || text.contains("not implemented") {
                    failures.push(format!("{name}: {text}"));
                }
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}
