//! Implicit const Try calls consume the actual typed source protocol plans.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/const-try-calls";
fn report(name: &str) -> Result<rustgm::resolve::expansion::Report> {
    packet_report(PACKET, name)
}
fn packet_report(packet: &str, name: &str) -> Result<rustgm::resolve::expansion::Report> {
    let source = crate::resolve::fixture(&format!("{packet}/{name}.rs"))?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(crate::resolve::source_provider_config(
        &source,
        "explicit_calls",
        &config,
    )?)
}
fn program(report: &rustgm::resolve::expansion::Report) -> Result<rustgm::lowir::program::Program> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(report)?;
    let exports = facts
        .functions
        .iter()
        .filter(|function| {
            matches!(
                report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name),
                "entry" | "probe" | "fail"
            )
        })
        .map(|function| function.definition)
        .collect::<Vec<_>>();
    Ok(rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?)
}
#[test]
fn implicit_const_try_calls_keep_roles_effects_and_residual_obligations() -> Result {
    let mut failures = Vec::new();
    for name in [
        "block",
        "question-return",
        "block-question",
        "nested",
        "projected-block",
        "projected-question",
        "runtime",
        "native-runtime",
    ] {
        if let Err(error) = report(name).and_then(|report| program(&report)) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for name in [
        "plain-bound",
        "plain-maker",
        "wrong-output",
        "missing-gate",
        "ordinary-impl",
        "unpaid-residual-drop",
        "unpaid-residual-conversion",
    ] {
        let error = report(name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        if [
            "not implemented",
            "unsupported",
            "exhausted",
            "budget",
            "lost its typed Call",
            "changed its actual source protocol",
            "differs from its complete selected",
        ]
        .iter()
        .any(|message| error.contains(message))
        {
            failures.push(format!("unqualified {name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn implicit_try_work_tracks_actual_protocol_calls() -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let report = report(&format!("scaling-{count}"))?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
        ];
        println!("implicit Try {count}: {work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(actual <= 4 * old + 512, "{before:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn implicit_try_calls_execute_success_and_residual_branches() -> Result {
    native(PACKET, "obj/parallel/const-try-calls/native")
}
fn native(packet: &str, output: &str) -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    crate::observed_program::execute(
        &root.join(output),
        &program(&packet_report(packet, "native-runtime")?)?,
        &root.join(packet).join("observe.c"),
        b"41 59 23\n",
    )
}

const UNSAFE_PACKET: &str = "pa31/tests/spec/unsafe-const-members";
#[test]
fn unsafe_blanket_members_keep_completed_call_safety_and_effects() -> Result {
    let mut failures = Vec::new();
    for name in [
        "blanket",
        "direct-premise",
        "safe-control",
        "runtime",
        "wrapper-owner",
        "raw-pointer-shape",
        "unsafe-caller",
        "native-runtime",
        "native-ctfe",
    ] {
        if let Err(error) = packet_report(UNSAFE_PACKET, name).and_then(|report| program(&report)) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for name in [
        "plain-bound",
        "ordinary-impl",
        "outside-unsafe",
        "wrong-application",
        "wrong-argument",
    ] {
        let error = packet_report(UNSAFE_PACKET, name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        if [
            "not implemented",
            "unsupported",
            "exhausted",
            "budget",
            "current conditional trait premise",
            "normalized safe source signature",
        ]
        .iter()
        .any(|message| error.contains(message))
        {
            failures.push(format!("unqualified {name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn unsafe_member_work_tracks_completed_safety_rows() -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let report = packet_report(UNSAFE_PACKET, &format!("scaling-{count}"))?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        rustgm::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            rustgm::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
        ];
        println!("unsafe members {count}: {work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(actual <= 5 * old + 512, "{before:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn unsafe_const_member_code_executes_only_its_actual_const_implementation() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in ["native-runtime", "native-ctfe"] {
        crate::observed_program::execute(
            &root
                .join("obj/parallel/unsafe-const-members/native")
                .join(name),
            &program(&packet_report(UNSAFE_PACKET, name)?)?,
            &root.join(UNSAFE_PACKET).join("observe.c"),
            b"41 59\n",
        )?;
    }
    Ok(())
}
