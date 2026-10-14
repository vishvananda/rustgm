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
    if let Some(pending) = &program.drops {
        return Err(Error {
            source: Some(pending.source),
            message: crate::body::drops::PENDING_MESSAGE.into(),
        });
    }
    run(types, definitions, variances, contracts, Some(program))
}
/// Prove contracts for pending destruction before its separate loan proof.
/// The ordinary assignment views keep their explicit pending-drop boundary.
/// # Errors
/// Rejects absent destruction work, invalid contracts and resource limits.
pub fn prepare_destruction(
    types: &Store,
    definitions: &Definitions,
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
    run(types, definitions, variances, contracts, Some(program))
}
/// Prove only the supplied generic templates, with their actual borrow places.
/// Other contracts (including concrete functions and initializers) remain unproved.
/// Possible destruction still requires its separate lifetime and loan analysis.
/// # Errors
/// Rejects missing/duplicate body contracts, invalid provenance and query limits.
pub fn check_templates(
    types: &Store,
    definitions: &Definitions,
    variances: &Variances,
    contracts: &Contracts<'_>,
    templates: &crate::body::templates::Templates,
) -> Result<Analysis, Error> {
    let mut index = std::collections::HashMap::with_capacity(contracts.bodies.len());
    for input in &contracts.bodies {
        if index.insert(input.source, input).is_some() {
            return Err(Error {
                source: Some(input.source),
                message: "duplicate lifetime body contract".into(),
            });
        }
    }
    let inputs = templates.bodies.iter().map(|template| {
        let cfg = &template.body;
        index
            .remove(&cfg.source)
            .map(|input| (input, Some(cfg)))
            .ok_or_else(|| Error {
                source: Some(cfg.source),
                message: "template requires a unique lifetime body contract".into(),
            })
    });
    prove(types, definitions, variances, contracts, inputs)
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
    let inputs = contracts.bodies.iter().map(|input| {
        let cfg = cfgs.get(&input.source).copied();
        if program.is_some() && cfg.is_none() {
            return Err(Error {
                source: Some(input.source),
                message: "lifetime contract has no typed body".into(),
            });
        }
        Ok((input, cfg))
    });
    prove(types, definitions, variances, contracts, inputs)
}
fn prove<'a>(
    types: &Store,
    definitions: &Definitions,
    variances: &Variances,
    contracts: &Contracts<'_>,
    inputs: impl Iterator<Item = Result<(&'a Contract, Option<&'a crate::body::ir::Body>), Error>>,
) -> Result<Analysis, Error> {
    let bounds =
        Bounds::infer(types, definitions, contracts.nominal_bounds).map_err(|e| Error {
            source: None,
            message: e.into(),
        })?;
    let mut queries = Queries::new(types, &bounds, contracts.callables);
    let mut bodies = Vec::with_capacity(inputs.size_hint().0);
    for input in inputs {
        let (input, cfg) = input?;
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
        let requirements = queries.symbolic_requirements(ty)?;
        for &(a, b) in requirements.regions {
            constraints.assume(a, b).map_err(|e| e.to_string())?;
        }
        for &(id, shorter) in requirements.parameters {
            constraints
                .assume_parameter(id, shorter)
                .map_err(|e| e.to_string())?;
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
    for id in &input.parameters {
        constraints
            .parameter(*id)
            .map_err(|e| error(e.to_string()))?;
    }
    for &(a, b) in &input.assumptions {
        constraints.assume(a, b).map_err(|e| error(e.to_string()))?;
    }
    if let Some(signature) = input.signature {
        signature_requirements(&mut constraints, queries, types, signature).map_err(error)?;
    }
    for &(ty, shorter) in &input.type_assumptions {
        let parts = queries.components(ty).map_err(|e| error(e.into()))?;
        for longer in parts.regions {
            constraints
                .assume(*longer, shorter)
                .map_err(|e| error(e.to_string()))?;
        }
        for parameter in parts.parameters {
            constraints
                .assume_parameter(*parameter, shorter)
                .map_err(|e| error(e.to_string()))?;
        }
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
            let requirements = queries.symbolic_requirements(ty)?;
            for &(a, b) in requirements.regions {
                constraints
                    .outlives(a, b, source)
                    .map_err(|e| e.to_string())?;
            }
            for &(id, shorter) in requirements.parameters {
                constraints
                    .parameter_outlives(id, shorter, source)
                    .map_err(|e| e.to_string())?;
            }
        }
        Obligation::TypeOutlives { ty, shorter } => {
            let parts = queries.components(ty)?;
            for longer in parts.regions {
                constraints
                    .outlives(*longer, shorter, source)
                    .map_err(|e| e.to_string())?;
            }
            for parameter in parts.parameters {
                constraints
                    .parameter_outlives(*parameter, shorter, source)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
