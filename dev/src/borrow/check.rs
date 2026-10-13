//! The executable boundary combines owned access, storage, contracts and loans.
use super::{loans, ownership, regions, storage};
use crate::{
    body::ir::{NodeKey, Program},
    types::{
        ItemId, Store,
        aggregate::Definitions,
        copy::{Copies, Properties},
        outlives::LiveRegions,
    },
};
use std::{collections::HashMap, hash::BuildHasher};

pub struct Checked<'a> {
    program: &'a Program,
}
impl Checked<'_> {
    #[must_use]
    pub const fn program(&self) -> &Program {
        self.program
    }
}
pub struct Analysis<'a> {
    checked: Checked<'a>,
    owned: ownership::Analysis<'a>,
    pub loans: Vec<loans::Analysis>,
    pub inspection: Vec<loans::Inspection>,
}
impl Analysis<'_> {
    #[must_use]
    pub const fn checked(&self) -> &Checked<'_> {
        &self.checked
    }
    #[must_use]
    pub const fn owned(&self) -> &ownership::Analysis<'_> {
        &self.owned
    }
}
#[derive(Debug)]
pub struct Error {
    pub node: Option<NodeKey>,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
/// Typed global inputs shared by each body proof. Layout/alignment validation
/// and lowering remain downstream responsibilities; no erased text is an input.
pub struct Environment<'a, S: BuildHasher> {
    pub types: &'a Store,
    pub definitions: &'a Definitions,
    pub derives: &'a HashMap<ItemId, Properties, S>,
    pub copies: &'a mut Copies,
}
/// # Errors
/// Rejects invalid contracts, unavailable storage, owned access or loan conflicts.
pub fn check<'a, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    contracts: &regions::Analysis,
    program: &'a Program,
) -> Result<Analysis<'a>, Error> {
    analyze::<false, _>(environment, contracts, program)
}
/// Retain canonical loans and sparse point states from the same completed proof.
/// # Errors
/// Reports `check` failures and the additional inspection storage limit.
pub fn inspect<'a, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    contracts: &regions::Analysis,
    program: &'a Program,
) -> Result<Analysis<'a>, Error> {
    analyze::<true, _>(environment, contracts, program)
}
fn analyze<'a, const RECORD: bool, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    contracts: &regions::Analysis,
    program: &'a Program,
) -> Result<Analysis<'a>, Error> {
    let mut bodies = Vec::with_capacity(program.bodies.len());
    let mut loans = Vec::with_capacity(program.bodies.len());
    let mut inspection = Vec::new();
    let checked = run::<RECORD, _>(environment, contracts, program, |owned, proof| {
        bodies.push(owned);
        loans.push(proof.analysis);
        inspection.extend(proof.inspection);
    })?;
    Ok(Analysis {
        checked,
        owned: ownership::Analysis::from_proved(program, bodies),
        loans,
        inspection,
    })
}
/// Run the same proof while releasing per-body inspection facts promptly.
/// # Errors
/// Reports the same semantic and resource errors as `check`.
pub fn verify<'a, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    contracts: &regions::Analysis,
    program: &'a Program,
) -> Result<Checked<'a>, Error> {
    run::<false, _>(environment, contracts, program, |_, _| {})
}
fn run<'a, const RECORD: bool, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    contracts: &regions::Analysis,
    program: &'a Program,
    mut record: impl FnMut(ownership::BodyFacts, loans::Report),
) -> Result<Checked<'a>, Error> {
    let by_source: HashMap<_, _> = contracts.bodies.iter().map(|b| (b.source, b)).collect();
    let mut components = LiveRegions::default();
    for body in &program.bodies {
        let contract = by_source.get(&body.source).ok_or_else(|| Error {
            node: Some(body.source),
            message: "body has no lifetime proof".into(),
        })?;
        let storage = storage::check(body).map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
        let owned = ownership::analyze_body(
            environment.types,
            environment.definitions,
            environment.derives,
            environment.copies,
            body,
        )
        .map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
        let proof = loans::analyze::<RECORD>(
            environment.types,
            &mut components,
            body,
            contract,
            &owned,
            &storage,
        )
        .map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
        record(owned, proof);
    }
    Ok(Checked { program })
}
