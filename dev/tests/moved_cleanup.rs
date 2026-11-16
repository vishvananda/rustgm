//! Proven initialization controls whether symbolic cleanup needs a type query.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    body,
    borrow::{destruction::symbolic, ownership::initialization},
    types::{copy::Copies, destruction::NeedsDrop},
};
use std::{fmt::Write as _, sync::Arc};

#[test]
fn moved_values_demand_no_recursive_cleanup_properties() -> Result<(), Box<dyn std::error::Error>> {
    for depth in [4, 16, 64] {
        for live in [false, true] {
            let mut source = String::from("struct S0<T>(T);");
            for level in 1..=depth {
                write!(source, "struct S{level}<T>(S{}<T>);", level - 1)?;
            }
            if live {
                write!(
                    source,
                    "fn probe<T>(x:S{depth}<T>,take:bool){{if take{{let y=x;}}}}"
                )?;
            } else {
                write!(source, "fn probe<T>(x:S{depth}<T>)->S{depth}<T>{{x}}")?;
            }
            rustc::qualify(&format!("cleanup-demand-{depth}-{live}"), &source, true)?;
            let (report, facts) = semantic::analyze(&source)?;
            let templates = body::templates::build(&report, &facts)?;
            let template = templates.bodies.first().ok_or("missing probe")?;
            let mut copies = Copies::in_environment(Arc::clone(&template.environment));
            let mut initialized = initialization::analyze(
                &facts.types,
                &facts.aggregates,
                &facts.derives,
                &mut copies,
                &template.body,
            )
            .map_err(|error| error.message)?;
            let mut needed = NeedsDrop::in_environment([], Arc::clone(&template.environment));
            symbolic::select(
                &facts.types,
                &facts.aggregates,
                &mut needed,
                &mut initialized,
                |_| false,
                |_, _| Err("unexpected field move"),
            )
            .map_err(|error| error.message)?;
            if live {
                assert!(needed.steps > depth, "live cleanup was not checked");
            } else {
                assert_eq!(needed.steps, 0, "moved value opened a type graph");
            }
        }
    }
    Ok(())
}

const ABSTRACT: &str = "pa31/tests/spec/abstract-cleanup";
const ABSTRACT_CASES: [&str; 5] = [
    "projected",
    "take-while",
    "opaque-kept",
    "dropped",
    "parameter",
];
fn abstract_program(
    name: &str,
) -> Result<rustgm::lowir::program::Program, Box<dyn std::error::Error>> {
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(
        &crate::resolve::fixture(&format!("{ABSTRACT}/{name}.rs"))?,
        "abstract_cleanup",
        &config,
    )?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
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
/// A conditionally moved `I::Item` or opaque value, as in core's
/// `TakeWhile::next`, has the possible cleanup of the whole value.
#[test]
fn conditionally_moved_abstract_values_clean_up_whole() -> Result<(), Box<dyn std::error::Error>> {
    for name in ABSTRACT_CASES {
        abstract_program(name).map_err(|error| format!("{name}: {error}"))?;
    }
    let error = abstract_program("moved-read")
        .err()
        .ok_or("accepted moved-read")?;
    assert!(
        error
            .to_string()
            .contains("borrowed place may be uninitialized or moved"),
        "moved-read: {error}"
    );
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn conditionally_moved_abstract_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in ABSTRACT_CASES {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/abstract-cleanup/native/{name}")),
            &abstract_program(name)?,
            &root.join(ABSTRACT).join("observe.c"),
            b"41 59\n",
        )?;
    }
    Ok(())
}
