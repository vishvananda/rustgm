//! Prove body contracts against immutable declared and implied assumptions.
use super::{constraints::Constraints, graph};
use crate::{
    body::{
        ir::NodeKey,
        regions::{Contract, Contracts, Kind as Obligation},
    },
    types::{
        Kind, Store, TypeId,
        aggregate::Definitions,
        outlives::{Bounds, Queries},
        variance::Variances,
    },
};

pub struct Body {
    pub source: NodeKey,
    pub solution: graph::Solution,
    pub identities: super::constraints::Identities,
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
    definitions: &Definitions,
    variances: &Variances,
    contracts: &Contracts<'_>,
) -> Result<Analysis, Error> {
    run(types, definitions, variances, contracts, None)
}
/// Prove contracts with explicit borrow-place provenance from the typed CFG.
/// This still does not authorize execution: loans and storage liveness are separate.
///
/// # Errors
/// Rejects unentailed bounds, unsupported provenance, malformed CFG facts and budgets.
pub fn check_bodies(
    types: &Store,
    definitions: &Definitions,
    variances: &Variances,
    contracts: &Contracts<'_>,
    program: &crate::body::ir::Program,
) -> Result<Analysis, Error> {
    run(types, definitions, variances, contracts, Some(program))
}
fn run(
    types: &Store,
    definitions: &Definitions,
    variances: &Variances,
    contracts: &Contracts<'_>,
    program: Option<&crate::body::ir::Program>,
) -> Result<Analysis, Error> {
    let cfgs: std::collections::HashMap<_, _> = program
        .into_iter()
        .flat_map(|p| &p.bodies)
        .map(|b| (b.source, b))
        .collect();
    let bounds =
        Bounds::infer(types, definitions, contracts.nominal_bounds).map_err(|e| Error {
            source: None,
            message: e.into(),
        })?;
    let mut queries = Queries::new(types, &bounds, contracts.callables);
    let mut bodies = Vec::with_capacity(contracts.bodies.len());
    for input in &contracts.bodies {
        let cfg = cfgs.get(&input.source).copied();
        if program.is_some() && cfg.is_none() {
            return Err(Error {
                source: Some(input.source),
                message: "lifetime contract has no typed body".into(),
            });
        }
        bodies.push(body(types, variances, &mut queries, input, cfg)?);
    }
    Ok(Analysis { bodies })
}
fn signature_requirements(
    constraints: &mut Constraints<'_>,
    queries: &mut Queries<'_>,
    types: &Store,
    signature: TypeId,
) -> Result<(), String> {
    let Some(Kind::Function {
        parameters, result, ..
    }) = types.kind(signature)
    else {
        return Err("missing lifetime function signature".into());
    };
    let parameters = types
        .list(parameters)
        .ok_or("missing lifetime function parameters")?;
    for ty in parameters.iter().copied().chain(std::iter::once(result)) {
        for &(a, b) in queries.requirements(ty)? {
            constraints.assume(a, b).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
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
    for &(a, b) in &input.assumptions {
        constraints.assume(a, b).map_err(|e| error(e.to_string()))?;
    }
    if let Some(signature) = input.signature {
        signature_requirements(&mut constraints, queries, types, signature).map_err(error)?;
    }
    let provenance = cfg
        .map(|cfg| super::places::bounds(types, cfg))
        .transpose()
        .map_err(|message| error(message.into()))?
        .unwrap_or_default();
    for (index, obligation) in input.obligations.iter().chain(&provenance).enumerate() {
        let source = u32::try_from(index)
            .map_err(|_| error("too many lifetime constraint sources".into()))?;
        apply(
            queries,
            &mut constraints,
            obligation.kind,
            source,
            cfg.is_some(),
        )
        .map_err(|message| Error {
            source: Some(obligation.source),
            message,
        })?;
    }
    let proof = constraints.prove().map_err(|e| {
        let source = match e {
            graph::Error::Unsatisfied { source, .. } => input
                .obligations
                .get(source as usize)
                .or_else(|| {
                    (source as usize)
                        .checked_sub(input.obligations.len())
                        .and_then(|i| provenance.get(i))
                })
                .map_or(input.source, |o| o.source),
            _ => input.source,
        };
        Error {
            source: Some(source),
            message: e.to_string(),
        }
    })?;
    Ok(Body {
        source: input.source,
        solution: proof.solution,
        identities: proof.identities,
    })
}
fn apply(
    queries: &mut Queries<'_>,
    constraints: &mut Constraints<'_>,
    obligation: Obligation,
    source: u32,
    has_provenance: bool,
) -> Result<(), String> {
    match obligation {
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
        Obligation::WellFormed(ty) => {
            for &(a, b) in queries.requirements(ty)? {
                constraints
                    .outlives(a, b, source)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
