//! Explicit call-trait methods retain their validated tuple ABI and current effect.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/explicit-call-members";
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
                "entry" | "probe"
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
fn explicit_const_members_require_their_actual_role_tuple_and_receiver() -> Result {
    let mut failures = Vec::new();
    for name in [
        "direct-mut",
        "direct-shared",
        "direct-once",
        "relay",
        "original-drain",
        "native-runtime",
    ] {
        if let Err(error) = report(name).and_then(|report| program(&report)) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for name in [
        "plain-bound",
        "ordinary-impl",
        "immutable",
        "wrong-tuple",
        "unsafe-impl",
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
fn explicit_member_work_tracks_actual_typed_calls() -> Result {
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
        println!("explicit members={count} work={work:?}");
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
fn explicit_const_delegation_executes_the_actual_callable_and_mutable_receiver() -> Result {
    native(PACKET, "obj/parallel/explicit-call-members/native")
}
fn native(packet: &str, output: &str) -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    crate::observed_program::execute(
        &root.join(output),
        &program(&packet_report(packet, "native-runtime")?)?,
        &root.join(packet).join("observe.c"),
        b"41 59\n",
    )
}

const BORROWED: &str = "pa31/tests/spec/borrowed-callables";
#[test]
fn borrowed_const_callables_keep_source_adjustments_and_permissions() -> Result {
    let mut failures = Vec::new();
    for name in [
        "direct-mut",
        "field-mut",
        "direct-shared",
        "field-shared",
        "nested-mut",
        "nested-shared",
        "shared-through-mut",
        "native-runtime",
        "borrowed-relay",
    ] {
        if let Err(error) = packet_report(BORROWED, name).and_then(|report| program(&report)) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for name in [
        "plain-bound",
        "shared-cannot-mutate",
        "field-cannot-mutate",
        "once-cannot-move",
        "wrong-argument",
    ] {
        let error = packet_report(BORROWED, name)
            .and_then(|report| program(&report))
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        if [
            "not implemented",
            "unsupported",
            "exhausted",
            "budget",
            "differs from its defining",
            "requires its source syntax",
            "changed its source",
            "changed its exact source",
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
fn borrowed_callable_work_tracks_actual_reference_paths() -> Result {
    let mut previous: Option<[usize; 2]> = None;
    for count in [8, 32, 128] {
        let report = packet_report(BORROWED, &format!("scaling-{count}"))?;
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
        println!("borrowed callable {count}: {work:?}");
        if let Some(before) = previous {
            for (actual, old) in work.into_iter().zip(before) {
                assert!(actual <= old.saturating_mul(5));
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn borrowed_callable_fields_execute_the_actual_mutable_reborrow() -> Result {
    native(BORROWED, "obj/parallel/borrowed-callables/native")
}
