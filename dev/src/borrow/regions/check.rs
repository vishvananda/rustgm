//! Prove body contracts against immutable declared and implied assumptions.
use super::{constraints::Constraints, graph};
use crate::{
    body::{
        ir::NodeKey,
        regions::{Contract, Contracts, Kind as Obligation},
    },
    types::{Store, outlives::Queries, variance::Variances},
};

pub struct Body {
    pub key: crate::body::ir::BodyKey,
    pub source: NodeKey,
    pub instance: Option<crate::types::instance::Instance>,
    pub solution: graph::Solution,
    pub hidden_regions: Vec<crate::types::opaque::RegionChoice>,
    pub identities: super::constraints::Identities,
    pub builtins: Option<std::sync::Arc<crate::body::regions::BuiltinRequirements>>,
}
pub struct Analysis {
    pub bodies: Vec<Body>,
}
#[derive(Debug)]
pub struct Error {
    pub source: Option<NodeKey>,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
/// Prove lifetime contracts independently of move/loan permissions. Borrow
/// expressions require provenance before this proof can pass.
///
/// # Errors
/// Rejects unentailed requirements, missing borrow provenance and budgets.
pub fn check(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
) -> Result<Analysis, Error> {
    run(types, variances, contracts, None)
}
/// Prove contracts with explicit borrow-place provenance from the typed CFG.
/// This still does not authorize execution: loans and storage liveness are separate.
///
/// # Errors
/// Rejects unentailed bounds, unsupported provenance, malformed CFG facts and budgets.
pub fn check_bodies(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    program: &crate::body::ir::Program,
) -> Result<Analysis, Error> {
    if let Some(pending) = &program.drops {
        return Err(Error {
            source: Some(pending.source),
            message: crate::body::drops::PENDING_MESSAGE.into(),
        });
    }
    run(types, variances, contracts, Some(program))
}
/// Prove contracts for pending destruction before its separate loan proof.
/// The ordinary assignment views keep their explicit pending-drop boundary.
/// # Errors
/// Rejects absent destruction work, invalid contracts and resource limits.
pub fn prepare_destruction(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    program: &crate::body::ir::Program,
) -> Result<Analysis, Error> {
    if program.drops.is_none() {
        return Err(Error {
            source: None,
            message: "destruction contracts require a pending plan".into(),
        });
    }
    run(types, variances, contracts, Some(program))
}
/// Prove only the supplied generic templates, with their actual borrow places.
/// Other contracts (including concrete functions and initializers) remain unproved.
/// Possible destruction still requires its separate lifetime and loan analysis.
/// # Errors
/// Rejects missing/duplicate body contracts, invalid provenance and query limits.
pub fn check_templates(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    templates: &crate::body::templates::Templates,
) -> Result<Analysis, Error> {
    let mut index = std::collections::HashMap::with_capacity(contracts.bodies.len());
    for input in &contracts.bodies {
        if index.insert(input.key, input).is_some() {
            return Err(Error {
                source: Some(input.source),
                message: "duplicate lifetime body contract".into(),
            });
        }
    }
    let inputs = templates.bodies.iter().map(|template| {
        let cfg = &template.body;
        index
            .remove(&cfg.key())
            .map(|input| (input, Some(cfg)))
            .ok_or_else(|| Error {
                source: Some(cfg.source),
                message: "template requires a unique lifetime body contract".into(),
            })
    });
    prove(types, variances, contracts, inputs)
}
/// Prove an explicitly paired contract per body. Instances may share source keys,
/// but every pair has its own universal/variable context and solved constraints.
/// This remains a region proof; ownership, storage and loans are checked separately.
/// # Errors
/// Rejects mismatched coverage, invalid contracts/provenance and resource limits.
pub fn check_ordered(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    program: &crate::body::ir::Program,
) -> Result<Analysis, Error> {
    if contracts.bodies.len() != program.bodies.len() {
        return Err(Error {
            source: None,
            message: "instance contracts have different body coverage".into(),
        });
    }
    let inputs = contracts
        .bodies
        .iter()
        .zip(&program.bodies)
        .map(|(input, cfg)| {
            if input.key != cfg.key() || input.source != cfg.source {
                return Err(Error {
                    source: Some(cfg.source),
                    message: "instance contract does not match its body".into(),
                });
            }
            Ok((input, Some(cfg)))
        });
    prove(types, variances, contracts, inputs)
}
/// Prove one explicitly paired body using retained structural requirement queries.
/// Its source and selected instance remain separate from every other body context.
/// # Errors
/// Rejects mismatched contracts, invalid provenance and unentailed requirements.
pub(crate) fn check_one(
    types: &Store,
    variances: &Variances,
    queries: &mut Queries<'_>,
    input: &Contract,
    cfg: &crate::body::ir::Body,
) -> Result<Body, Error> {
    if input.key != cfg.key() || input.source != cfg.source {
        return Err(Error {
            source: Some(cfg.source),
            message: "instance contract does not match its body".into(),
        });
    }
    body(types, variances, queries, input, Some(cfg))
}
fn run(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    program: Option<&crate::body::ir::Program>,
) -> Result<Analysis, Error> {
    if let Some(body) = program.and_then(|p| p.bodies.iter().find(|b| b.instance.is_some())) {
        return Err(Error {
            source: Some(body.source),
            message: "specialized bodies require ordered instance contracts".into(),
        });
    }
    let cfgs: std::collections::HashMap<_, _> = program
        .into_iter()
        .flat_map(|p| &p.bodies)
        .map(|b| (b.key(), b))
        .collect();
    let inputs = contracts.bodies.iter().map(|input| {
        let cfg = cfgs.get(&input.key).copied();
        if program.is_some() && cfg.is_none() {
            return Err(Error {
                source: Some(input.source),
                message: "lifetime contract has no typed body".into(),
            });
        }
        Ok((input, cfg))
    });
    prove(types, variances, contracts, inputs)
}
fn prove<'a>(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    inputs: impl Iterator<Item = Result<(&'a Contract, Option<&'a crate::body::ir::Body>), Error>>,
) -> Result<Analysis, Error> {
    let mut queries = Queries::new(types, contracts.nominal_requirements, contracts.callables);
    let mut bodies = Vec::with_capacity(inputs.size_hint().0);
    for input in inputs {
        let (input, cfg) = input?;
        bodies.push(body(types, variances, &mut queries, input, cfg)?);
    }
    Ok(Analysis { bodies })
}

fn body(
    types: &Store,
    variances: &Variances,
    queries: &mut Queries<'_>,
    input: &Contract,
    cfg: Option<&crate::body::ir::Body>,
) -> Result<Body, Error> {
    let error = |message| Error {
        source: Some(input.source),
        message,
    };
    let mut constraints = Constraints::new(types, variances, input.universals, 1_000_000)
        .map_err(|e| error(e.to_string()))?;
    for id in &input.parameters {
        constraints
            .parameter(*id)
            .map_err(|e| error(e.to_string()))?;
    }
    for &(a, b) in &input.assumptions {
        constraints.assume(a, b).map_err(|e| error(e.to_string()))?;
    }
    if let Some(signature) = input.signature {
        constraints
            .assume_signature(queries, signature)
            .map_err(error)?;
    }
    constraints
        .assume_outlives_clauses(queries, input.type_assumptions.iter().copied())
        .map_err(error)?;
    let provenance = cfg
        .map(|cfg| super::places::bounds(types, cfg))
        .transpose()
        .map_err(|message| error(message.into()))?
        .unwrap_or_default();
    let first_proof = input.obligations.len().saturating_add(provenance.len());
    let obligations = || input.obligations.iter().chain(&provenance).enumerate();
    let mut members = super::membership::Requests::new(types);
    if !input.proofs.is_empty() {
        // An inferred input can contain a fresh existential owned by a retained
        // selection. Register ordinary variables before proof debts, then proof
        // variables before ordinary relations. Keep original diagnostic indexes.
        require_obligations(
            obligations().filter(|(_, o)| matches!(o.kind, Obligation::Variable { .. })),
            queries,
            &mut constraints,
            cfg.is_some(),
            &mut members,
        )?;
        quantified(input, queries, &mut constraints, first_proof)?;
    }
    require_obligations(
        obligations().filter(|(_, o)| {
            input.proofs.is_empty() || !matches!(o.kind, Obligation::Variable { .. })
        }),
        queries,
        &mut constraints,
        cfg.is_some(),
        &mut members,
    )?;
    let proof = constraints.prove().map_err(|e| {
        let source = match e {
            graph::Error::Unsatisfied { source, .. }
            | graph::Error::Verification { source }
            | graph::Error::Membership { source } => input
                .obligations
                .get(source as usize)
                .or_else(|| {
                    (source as usize)
                        .checked_sub(input.obligations.len())
                        .and_then(|i| provenance.get(i))
                })
                .map(|o| o.source)
                .or_else(|| {
                    (source as usize)
                        .checked_sub(first_proof)
                        .and_then(|i| proof_source(input, i))
                })
                .unwrap_or(input.source),
            _ => input.source,
        };
        Error {
            source: Some(source),
            message: e.to_string(),
        }
    })?;
    let hidden_regions = members.finish(&proof).map_err(|e| error(e.to_string()))?;
    Ok(Body {
        hidden_regions,
        key: input.key,
        source: input.source,
        instance: cfg.and_then(|body| body.instance),
        solution: proof.solution,
        identities: proof.identities,
        builtins: input.builtins.clone(),
    })
}
fn require_obligations<'a>(
    obligations: impl Iterator<Item = (usize, &'a crate::body::regions::Obligation)>,
    queries: &mut Queries<'_>,
    constraints: &mut Constraints<'_>,
    has_provenance: bool,
    members: &mut super::membership::Requests<'_>,
) -> Result<(), Error> {
    let mut formation = None;
    for (index, obligation) in obligations {
        let error = |message| Error {
            source: Some(obligation.source),
            message,
        };
        let source = u32::try_from(index)
            .map_err(|_| error("too many lifetime constraint sources".into()))?;
        apply(
            queries,
            constraints,
            obligation.kind,
            source,
            has_provenance,
            &mut formation,
            members,
        )
        .map_err(error)?;
    }
    Ok(())
}
fn quantified(
    input: &Contract,
    queries: &mut Queries<'_>,
    constraints: &mut Constraints<'_>,
    first: usize,
) -> Result<(), Error> {
    let mut first = first;
    for evidence in &input.proofs {
        let error = |message| Error {
            source: Some(evidence.source),
            message,
        };
        let count = evidence.sources.len().saturating_add(1);
        let index =
            u32::try_from(first).map_err(|_| error("too many lifetime proof sources".into()))?;
        let count =
            u32::try_from(count).map_err(|_| error("too many lifetime proof sources".into()))?;
        evidence
            .proof
            .require_sources(constraints, queries, index, count)
            .map_err(error)?;
        first = first.saturating_add(count as usize);
    }
    Ok(())
}
// Resolve only a failed constraint; successful proofs allocate no source index.
fn proof_source(input: &Contract, mut index: usize) -> Option<crate::body::ir::NodeKey> {
    for evidence in &input.proofs {
        if index == 0 {
            return Some(evidence.source);
        }
        if let Some(source) = evidence.sources.get(index - 1) {
            return Some(*source);
        }
        index = index.checked_sub(evidence.sources.len().checked_add(1)?)?;
    }
    None
}
fn apply<'a>(
    queries: &mut Queries<'a>,
    constraints: &mut Constraints<'_>,
    obligation: Obligation,
    source: u32,
    has_provenance: bool,
    formation: &mut Option<Box<crate::types::outlives::Formation<'a>>>,
    members: &mut super::membership::Requests<'_>,
) -> Result<(), String> {
    match obligation {
        Obligation::Membership { region, captures } => {
            members.require(constraints, region, captures, source)?;
        }
        Obligation::Variable {
            id,
            needs_provenance,
        } => {
            if needs_provenance && !has_provenance {
                return Err("reference contract requires typed borrow-place provenance".into());
            }
            constraints.variable(id).map_err(|e| e.to_string())?;
        }
        Obligation::LocalBorrow(region) => constraints
            .local(region, source)
            .map_err(|e| e.to_string())?,
        Obligation::Equality((a, b)) => {
            constraints
                .outlives(a, b, source)
                .map_err(|e| e.to_string())?;
            constraints
                .outlives(b, a, source)
                .map_err(|e| e.to_string())?;
        }
        Obligation::Outlives((a, b)) => {
            constraints
                .outlives(a, b, source)
                .map_err(|e| e.to_string())?;
        }
        Obligation::Coercion {
            actual,
            expected,
            unsize,
            reference,
        } => {
            if let Some(reference) = reference {
                constraints.reborrow(
                    actual,
                    expected,
                    reference.dereferences,
                    reference.unsize,
                    source,
                )
            } else {
                constraints.coerce(actual, expected, unsize, source)
            }
            .map_err(|e| e.to_string())?;
        }
        Obligation::ScopedWellFormed { ty, bound_regions } => {
            constraints.require_requirements(
                formation
                    .get_or_insert_with(|| Box::new(queries.formation()))
                    .requirements(ty, bound_regions)?,
                crate::types::relations::ProofScope::ROOT,
                source,
            )?;
        }
        Obligation::WellFormed(ty) => {
            constraints.require_requirements(
                queries.symbolic_requirements(ty)?,
                crate::types::relations::ProofScope::ROOT,
                source,
            )?;
        }
        Obligation::TypeOutlives { ty, shorter } => {
            constraints.require_type(
                queries,
                ty,
                shorter,
                crate::types::relations::ProofScope::ROOT,
                source,
            )?;
        }
    }
    Ok(())
}
