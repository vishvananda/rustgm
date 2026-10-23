//! Test adapter for completed destruction proofs and typed body probes.
use crate::semantic;
use rustgm::{body, borrow, lower, lowir, sema};
pub fn build(source: &str) -> Result<lowir::program::Program, Box<dyn std::error::Error>> {
    build_with(source, |_, _| Ok(()))
}
pub fn build_with(
    source: &str,
    amend: impl FnOnce(
        &rustgm::resolve::expansion::Report,
        &mut body::ir::Program,
    ) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<lowir::program::Program, Box<dyn std::error::Error>> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build(&report, &facts)?;
    body::promote::analyze(&report, &mut facts, &mut program)?;
    amend(&report, &mut program)?;
    body::constants::analyze(&report, &mut facts, &program)?;
    let contracts = body::regions::collect(&facts)?;
    let proof = borrow::regions::prepare_destruction(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )?;
    let catalog = borrow::destruction::glue::Catalog::new(
        &facts.types,
        &facts.aggregates,
        &facts.members,
        &program.drops.as_ref().ok_or("drops")?.methods,
    )?;
    let checked = borrow::verify_destruction(
        &mut borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: facts.uses_source_copy(),
            copies: &mut facts.copies,
        },
        &proof,
        &program,
        catalog,
    )?;
    let entry =
        sema::entry::select(&mut report, &facts).map_err(|e| report.diagnostic(e).to_string())?;
    lower::build(&mut facts, &checked, entry).map_err(|e| e.message.into())
}
