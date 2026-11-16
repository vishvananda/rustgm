//! A borrow of a `#[rustc_promotable]` const function's call whose arguments
//! are constants is promoted to permanent storage, as rustc 1.98.0 promotes it
//! in runtime code: alloc's `task.rs` returns
//! `&RawWakerVTable::new(clone_waker::<W>, ...)` from a generic function. A
//! generic caller's promoted value is executed for each of its instances.
//!
//! A const function without the marker, or a marked call with a runtime
//! argument, keeps its temporary, so returning its borrow as `'static` is
//! rejected, as rustc rejects both with E0515. The `promoted-calls` packet's
//! outcomes were declared before either compiler ran; rustc 1.98.0's are
//! recorded in its `outcomes.json`.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const PACKET: &str = "pa31/tests/spec/promoted-calls";
/// What rustc's build of `marked` prints.
const NATIVE: &str = "5 7 7\n";
/// A borrowed temporary does not live for `'static`.
const TEMPORARY: &str = "required lifetime does not outlive its target";

fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(
        &crate::resolve::fixture(&format!("{PACKET}/{name}.rs"))?,
        "promoted_calls",
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

/// The marked calls are promoted; the unmarked and runtime-argument borrows
/// stay temporaries.
#[test]
fn promoted_calls_keep_marked_constant_calls_and_reject_temporaries() -> Result {
    for name in ["marked", "generic-callee"] {
        program(name)?;
    }
    for name in [
        "unmarked",
        "runtime-argument",
        "generic-unmarked",
        "generic-runtime-argument",
    ] {
        match program(name) {
            Ok(_) => return Err(format!("{name}: a temporary was returned as 'static").into()),
            Err(error) if error.to_string().contains(TEMPORARY) => {}
            Err(error) => return Err(format!("{name}: {error}").into()),
        }
    }
    Ok(())
}

/// The accepted program prints exactly what rustc's build of it prints.
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn promoted_calls_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for name in ["marked", "generic-callee"] {
        crate::observed_program::execute(
            &root.join("obj/parallel/promoted-calls/native").join(name),
            &program(name)?,
            &root.join(PACKET).join("observe.c"),
            NATIVE.as_bytes(),
        )?;
    }
    Ok(())
}
