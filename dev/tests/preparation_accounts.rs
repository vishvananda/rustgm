//! Whole-crate observations must not spend the source layout provider's work budget.
use crate::{provider_traits, resolve};
use rustgm::{lowir::program::Program, pipeline};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn input(noise: usize) -> Result<String> {
    Ok(resolve::fixture(&format!(
        "pa31/tests/spec/layout-work-owner/noise-{noise}.rs"
    ))?)
}
fn build(report: &rustgm::resolve::expansion::Report, unrelated: usize) -> Result<Program> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(report)?;
    let before = facts.body_work.canonical_steps;
    facts.body_work.canonical_steps = before
        .checked_add(unrelated)
        .ok_or("test observation counter overflow")?;
    let start = facts.body_work.canonical_steps;
    let program = rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    );
    println!(
        "layout unrelated={unrelated} canonical_before={before} start={start} after={}",
        facts.body_work.canonical_steps
    );
    assert!(facts.body_work.canonical_steps >= start);
    Ok(program?)
}
fn text(program: &Program) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    rustgm::lowir::writer::write(program, &mut output)?;
    Ok(output)
}
#[test]
fn unrelated_observations_do_not_spend_source_layout_work() -> Result {
    for noise in [0, 8, 32, 128] {
        let report = provider_traits::report(&input(noise)?)?;
        let ordinary = build(&report, 0)?;
        let observed = build(&report, 8_047_612)?;
        assert_eq!(text(&ordinary)?, text(&observed)?, "noise={noise}");
    }
    Ok(())
}
#[test]
fn layout_permission_and_const_guards_remain_required() -> Result {
    for name in ["unsized", "nonconst-call", "unpaid-projection"] {
        let input = resolve::fixture(&format!("pa31/tests/spec/layout-work-owner/{name}.rs"))?;
        let error = provider_traits::lower(&input)
            .err()
            .ok_or_else(|| format!("{name} accepted"))?;
        println!("{name}: {error}");
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned CPPGM and the host C linker"]
fn layouts_execute_with_large_unrelated_work_observations() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/layout-work-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let observer = resolve::fixture("pa31/tests/spec/layout-work-owner/observe.c")?;
    for noise in [0, 128] {
        let report = provider_traits::report(&input(noise)?)?;
        let program = build(&report, 8_047_612)?;
        crate::ffi_program::check(
            root,
            &folder,
            &format!("layout-noise-{noise}"),
            &program,
            &observer,
            "2 4 6 8 0 8 8\n",
        )?;
    }
    Ok(())
}
