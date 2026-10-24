//! Source mode never recovers physical Copy permission from a missing carrier.
use super::*;
use crate::provider_cases as cases;
use crate::resolve::{
    bindings::Context,
    load::{InputFile, Sources},
};
use crate::support::{crates::CrateIdentity, source::SourceFile};
use crate::{body, borrow, expand::config::Config, resolve, support::bindings::Mode};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[path = "read_frontier.rs"]
mod frontier;
#[path = "copy_signatures.rs"]
mod signatures;
pub(super) fn record(facts: &Declarations, body: &Body, prepared: &Prepared) {
    frontier::record(facts, body, prepared);
}
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn fixture() -> Result<(Declarations, body::templates::Templates, Vec<Contract>)> {
    fixture_source(cases::GENERIC_COPY)
}
fn fixture_source(
    source: &str,
) -> Result<(Declarations, body::templates::Templates, Vec<Contract>)> {
    let identity = CrateIdentity::new("minicore", "source_copy_invariant")?;
    let report = resolve::analyze_expanded(resolve::expansion::analyze_with_context(
        InputFile {
            identity: "provider.rs".into(),
            path: "provider.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        &Context {
            crate_name: Some("minicore".into()),
            current: Some(identity.clone()),
            mode: Mode::Source { provider: identity },
            ..Default::default()
        },
    )?)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let templates = body::templates::build(&report, &facts)?;
    let contracts = body::regions::collect(&facts)?.bodies;
    for c in &contracts {
        c.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    Ok((facts, templates, contracts))
}
#[test]
fn source_copy_missing_carrier_and_wrong_owner_never_fall_back() -> Result {
    let (mut facts, templates, mut contracts) = fixture()?;
    let body = &templates.bodies.first().ok_or("template")?.body;
    let input = contracts
        .iter_mut()
        .find(|c| c.key == body.key())
        .ok_or("contract")?;
    prepare(&mut facts, body, input)?;
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let mut proof = borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        input,
        body,
    )?;
    let mut copies = crate::types::copy::Copies::default();
    borrow::copy_view::CopyView::checked(&mut copies, body, &proof, true)?;
    let saved = proof.builtins.take();
    assert!(borrow::copy_view::CopyView::checked(&mut copies, body, &proof, true).is_err());
    borrow::copy_view::CopyView::checked(&mut copies, body, &proof, false)?;
    proof.builtins = saved;
    let requirements = proof.builtins.as_ref().ok_or("builtins")?;
    let source = requirements.source_copy.as_ref().ok_or("policy")?;
    let prepared = input
        .builtins
        .as_ref()
        .ok_or("input builtins")?
        .prepared_for_test()?;
    let ty = prepared
        .decisions
        .keys()
        .next()
        .copied()
        .ok_or("read decision")?;
    prepared.decision(body, source, requirements.environment.as_ref(), ty)?;
    let equivalent = Arc::new(source.as_ref().clone());
    assert!(
        prepared
            .decision(body, &equivalent, requirements.environment.as_ref(), ty)
            .is_err()
    );
    let environment = Arc::new(
        requirements
            .environment
            .as_ref()
            .ok_or("environment")?
            .as_ref()
            .clone(),
    );
    assert!(
        prepared
            .decision(body, source, Some(&environment), ty)
            .is_err()
    );
    let mut wrong = prepared.clone();
    wrong.key = BodyKey::Anonymous(body.source.source().ok_or("source fixture")?);
    assert!(
        wrong
            .decision(body, source, requirements.environment.as_ref(), ty)
            .is_err()
    );
    let mut missing = source.as_ref().clone();
    missing.declaration = None;
    let missing = Arc::new(missing);
    let mut invalid = prepared.clone();
    invalid.requirements = missing.clone();
    assert_eq!(
        invalid.decision(body, &missing, requirements.environment.as_ref(), ty),
        Err("source Copy language item is missing")
    );
    Arc::make_mut(proof.builtins.as_mut().ok_or("builtins")?).source_copy = None;
    assert!(borrow::copy_view::CopyView::checked(&mut copies, body, &proof, true).is_err());
    Ok(())
}
#[test]
fn source_copy_unmatched_original_contract_is_an_error() -> Result {
    let (mut facts, templates, mut contracts) = fixture()?;
    let body = &templates.bodies.first().ok_or("template")?.body;
    let input = contracts
        .iter_mut()
        .find(|c| c.key == body.key())
        .ok_or("contract")?;
    assert_eq!(
        originals(
            &mut facts,
            std::iter::empty::<&Body>(),
            std::iter::once(input)
        ),
        Err("source Copy original contract has no exact body".into())
    );
    assert_eq!(
        originals(
            &mut facts,
            std::iter::once(body),
            std::iter::empty::<&mut Contract>()
        ),
        Err("source Copy original body has no exact contract".into())
    );
    Ok(())
}
#[test]
fn source_copy_record_sizes_are_explicit() {
    println!(
        "BorrowEnvironment={} Body={} Contract={} BodyBuiltins={} SourceRequirements={} Prepared={}",
        size_of::<borrow::Environment<'_, crate::support::hash::BuildFxHasher>>(),
        size_of::<Body>(),
        size_of::<Contract>(),
        size_of::<BuiltinRequirements>(),
        size_of::<SourceCopyRequirements>(),
        size_of::<Prepared>()
    );
}
