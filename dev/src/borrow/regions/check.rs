//! Prove body contracts against immutable declared and implied assumptions.
use super::{constraints::Constraints, graph};
use crate::{
    body::regions::{Contract, Contracts, Kind as Obligation},
    types::{Store, outlives::Queries, variance::Variances},
};

pub struct Body {
    pub key: crate::body::ir::BodyKey,
    pub source: crate::body::ir::BodyOrigin,
    pub instance: Option<crate::types::instance::Instance>,
    pub solution: graph::Solution,
    pub hidden_regions: Vec<crate::types::opaque::RegionChoice>,
    pub identities: super::constraints::Identities,
    pub builtins: Option<std::sync::Arc<crate::body::regions::BuiltinRequirements>>,
    pub output: Option<super::OutputSummary>,
}
impl Body {
    fn needs_parent(&self) -> bool {
        self.output
            .as_ref()
            .is_some_and(super::output::Summary::requires_parent)
    }
}
pub struct Analysis {
    pub bodies: Vec<Body>,
}
#[derive(Debug)]
pub struct Error {
    pub source: Option<crate::body::ir::ProofSite>,
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
            source: Some(pending.source.into()),
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
    let mut index = crate::support::hash::HashMap::with_capacity_and_hasher(
        contracts.bodies.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    for input in templates.inputs(&contracts.bodies) {
        if index.insert(input.key, input).is_some() {
            return Err(Error {
                source: Some(input.source.into()),
                message: "duplicate lifetime body contract".into(),
            });
        }
    }
    let inputs = templates.iter().map(|template| {
        let cfg = &template.body;
        index
            .remove(&cfg.key())
            .map(|input| (input, Some(cfg)))
            .ok_or_else(|| Error {
                source: Some(cfg.source.into()),
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
                    source: Some(cfg.source.into()),
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
            source: Some(cfg.source.into()),
            message: "instance contract does not match its body".into(),
        });
    }
    let result = body(types, variances, queries, input, Some(cfg), &[])?;
    if result.needs_parent() {
        return Err(Error {
            source: Some(input.source.into()),
            message: "nested output requires its checked parent contract".into(),
        });
    }
    Ok(result)
}

/// Inspect an exact original child's conditional requirements. This region
/// result grants no standalone ownership or executable authority.
pub(crate) fn check_original_child(
    types: &Store,
    variances: &Variances,
    queries: &mut Queries<'_>,
    input: &Contract,
    cfg: &crate::body::ir::Body,
) -> Result<Option<Body>, Error> {
    if !matches!(cfg.owner, crate::body::ir::Owner::AnonymousConstant)
        || cfg.instance.is_some()
        || input.key != cfg.key()
        || input.source != cfg.source
    {
        return Err(Error {
            source: Some(cfg.source.into()),
            message: "original child contract does not match its anonymous body".into(),
        });
    }
    let result = body(types, variances, queries, input, Some(cfg), &[])?;
    Ok((!result.needs_parent()).then_some(result))
}

pub(crate) fn check_nested_bodies<'a>(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    inputs: impl Iterator<Item = Result<(&'a Contract, &'a crate::body::ir::Body), Error>>,
) -> Result<Analysis, Error> {
    let inputs = inputs.map(|pair| {
        let (input, body) = pair?;
        if input.key != body.key() || input.source != body.source || body.instance.is_some() {
            return Err(Error {
                source: Some(body.source.into()),
                message: "nested original proof has mismatched CFG ownership".into(),
            });
        }
        Ok((input, Some(body)))
    });
    super::output::prove(types, variances, contracts, inputs)
}
fn run(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    program: Option<&crate::body::ir::Program>,
) -> Result<Analysis, Error> {
    if let Some(body) = program.and_then(|p| p.bodies.iter().find(|b| b.instance.is_some())) {
        return Err(Error {
            source: Some(body.source.into()),
            message: "specialized bodies require ordered instance contracts".into(),
        });
    }
    let cfgs: crate::support::hash::HashMap<_, _> = program
        .into_iter()
        .flat_map(|p| &p.bodies)
        .map(|b| (b.key(), b))
        .collect();
    let inputs = contracts.bodies.iter().map(|input| {
        let cfg = cfgs.get(&input.key).copied();
        if program.is_some() && cfg.is_none() {
            return Err(Error {
                source: Some(input.source.into()),
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
    if contracts
        .bodies
        .iter()
        .any(|contract| contract.output.is_some())
    {
        return super::output::prove(types, variances, contracts, inputs);
    }
    let mut queries = Queries::new(types, contracts.nominal_requirements, contracts.callables);
    let mut bodies = Vec::with_capacity(inputs.size_hint().0);
    for input in inputs {
        let (input, cfg) = input?;
        bodies.push(body(types, variances, &mut queries, input, cfg, &[])?);
    }
    Ok(Analysis { bodies })
}

pub(super) fn body(
    types: &Store,
    variances: &Variances,
    queries: &mut Queries<'_>,
    input: &Contract,
    cfg: Option<&crate::body::ir::Body>,
    children: &[&super::output::Summary],
) -> Result<Body, Error> {
    let mut omitted = Vec::new();
    loop {
        let terms = Terms {
            paid: None,
            omitted: &omitted,
        };
        match body_inner(types, variances, queries, input, cfg, children, &terms) {
            Ok(body) => return Ok(body),
            // Bring-up proves the body without a requirement that this proof
            // cannot discharge, and names it: rustc accepted the source.
            Err(Failure {
                error,
                obligation: Some(index),
            }) if omitted.len() < 64
                && !omitted.contains(&index)
                && UNPROVEN.contains(&error.message.as_str())
                && crate::body::bringup::unproven_site(
                    crate::support::bringup::Stage::Lifetimes,
                    error.source,
                    &error.message,
                ) =>
            {
                omitted.push(index);
            }
            Err(failure) => return Err(failure.error),
        }
    }
}
/// Results that mean "not proven", as opposed to malformed inputs.
const UNPROVEN: [&str; 2] = [
    "required lifetime does not outlive its target",
    "required type does not outlive its target",
];
/// A failed proof and, when one requirement that the proof may omit caused
/// it, that requirement's source index among the body's obligations and its
/// retained trait evidence.
struct Failure {
    error: Error,
    obligation: Option<usize>,
}
impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Self {
            error,
            obligation: None,
        }
    }
}
/// What one attempt may assume or leave out.
struct Terms<'a> {
    paid: Option<&'a crate::borrow::incremental::PaidInline<'a>>,
    /// Source indexes of the requirements left out of this attempt.
    omitted: &'a [usize],
}
/// Whether leaving this obligation out only weakens the proof. A variable, a
/// hidden choice and a storage bound define identities that later stages read.
const fn omissible(kind: Obligation) -> bool {
    !matches!(
        kind,
        Obligation::Variable { .. } | Obligation::Membership { .. } | Obligation::LocalBorrow(_)
    )
}
pub(crate) fn paid_inline(
    types: &Store,
    variances: &Variances,
    queries: &mut Queries<'_>,
    input: &Contract,
    cfg: &crate::body::ir::Body,
    paid: &crate::borrow::incremental::PaidInline<'_>,
) -> Result<Body, Error> {
    let terms = Terms {
        paid: Some(paid),
        omitted: &[],
    };
    let result = body_inner(types, variances, queries, input, Some(cfg), &[], &terms)
        .map_err(|failure| failure.error)?;
    if result.needs_parent() {
        return Err(Error {
            source: Some(input.source.into()),
            message: "paid child retains an unpaid parent requirement".into(),
        });
    }
    Ok(result)
}
fn body_inner(
    types: &Store,
    variances: &Variances,
    queries: &mut Queries<'_>,
    input: &Contract,
    cfg: Option<&crate::body::ir::Body>,
    children: &[&super::output::Summary],
    terms: &Terms<'_>,
) -> Result<Body, Failure> {
    let error = |message| Error {
        source: Some(input.source.into()),
        message,
    };
    super::interfaces::validate_body(input, cfg)?;
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
    assume_paid(terms.paid, cfg, input, &mut constraints).map_err(error)?;
    super::interfaces::assume_inputs(types, queries, input, &mut constraints)?;
    constraints
        .assume_outlives_clauses(queries, input.type_assumptions.iter().copied())
        .map_err(error)?;
    let mut provenance = cfg
        .map(|cfg| super::places::bounds(types, cfg))
        .transpose()
        .map_err(|message| error(message.into()))?
        .unwrap_or_default();
    for child in children {
        if child.parent != input.source {
            return Err(error("nested output summary belongs to another parent".into()).into());
        }
        provenance.extend(child.requirements.iter().copied());
    }
    let first_proof = input.obligations.len().saturating_add(provenance.len());
    let obligations = || {
        input
            .obligations
            .iter()
            .chain(&provenance)
            .enumerate()
            .filter(|(index, _)| !terms.omitted.contains(index))
    };
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
        quantified(input, queries, &mut constraints, first_proof, terms.omitted)?;
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
    super::interfaces::require_return(types, input, cfg, &mut constraints, first_proof)?;
    let (proof, output) = super::output::finish(constraints, input)
        .map_err(|e| unsatisfied(input, &provenance, first_proof, e))?;
    let hidden_regions = members.finish(&proof).map_err(|e| error(e.to_string()))?;
    Ok(Body {
        hidden_regions,
        key: input.key,
        source: input.source,
        instance: cfg.and_then(|body| body.instance),
        solution: proof.solution,
        identities: proof.identities,
        builtins: input.builtins.clone(),
        output,
    })
}
/// Name the requirement that the solved constraints leave unsatisfied.
fn unsatisfied(
    input: &Contract,
    provenance: &[crate::body::regions::Obligation],
    first_proof: usize,
    failure: graph::Error,
) -> Failure {
    let mut obligation = None;
    let source = match failure {
        graph::Error::Unsatisfied { source, .. }
        | graph::Error::Verification { source }
        | graph::Error::Membership { source } => {
            let index = source as usize;
            let found = input.obligations.get(index).or_else(|| {
                index
                    .checked_sub(input.obligations.len())
                    .and_then(|i| provenance.get(i))
            });
            let evidence = index
                .checked_sub(first_proof)
                .and_then(|i| proof_source(input, i));
            // A requirement of retained trait evidence names that evidence.
            obligation = match found {
                Some(found) => omissible(found.kind).then_some(index),
                None => evidence.map(|_| index),
            };
            found
                .map(|o| o.source)
                .or(evidence)
                .unwrap_or(input.source.into())
        }
        _ => input.source.into(),
    };
    Failure {
        error: Error {
            source: Some(source),
            message: failure.to_string(),
        },
        obligation,
    }
}
fn require_obligations<'a>(
    obligations: impl Iterator<Item = (usize, &'a crate::body::regions::Obligation)>,
    queries: &mut Queries<'_>,
    constraints: &mut Constraints<'_>,
    has_provenance: bool,
    members: &mut super::membership::Requests<'_>,
) -> Result<(), Failure> {
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
        .map_err(|message| Failure {
            error: error(message),
            obligation: omissible(obligation.kind).then_some(index),
        })?;
    }
    Ok(())
}
fn quantified(
    input: &Contract,
    queries: &mut Queries<'_>,
    constraints: &mut Constraints<'_>,
    first: usize,
    omitted: &[usize],
) -> Result<(), Error> {
    let mut first = first;
    for evidence in &input.proofs {
        let error = |message| Error {
            source: Some(evidence.source),
            message,
        };
        let sources = first..first.saturating_add(evidence.sources.len().saturating_add(1));
        let index =
            u32::try_from(first).map_err(|_| error("too many lifetime proof sources".into()))?;
        let count = u32::try_from(sources.len())
            .map_err(|_| error("too many lifetime proof sources".into()))?;
        first = sources.end;
        // An attempt leaves out the whole evidence that a left-out source names.
        if omitted.iter().any(|source| sources.contains(source)) {
            continue;
        }
        evidence
            .proof
            .require_sources(constraints, queries, index, count)
            .map_err(error)?;
    }
    Ok(())
}
// Resolve only a failed constraint; successful proofs allocate no source index.
fn proof_source(input: &Contract, mut index: usize) -> Option<crate::body::ir::ProofSite> {
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
                constraints.body_coercion(actual, expected, unsize, source)
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

fn assume_paid(
    paid: Option<&crate::borrow::incremental::PaidInline<'_>>,
    cfg: Option<&crate::body::ir::Body>,
    input: &Contract,
    constraints: &mut Constraints<'_>,
) -> Result<(), String> {
    if let Some(paid) = paid {
        let cfg = cfg.ok_or("paid child proof requires its actual CFG")?;
        for &(a, b) in paid.conditions(input, cfg)?.unwrap_or(&[]) {
            constraints.assume(a, b).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
