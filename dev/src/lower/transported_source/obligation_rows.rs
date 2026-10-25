//! Reproof receives complete raw allocation and relation inputs, never solutions.
mod builtin_rows;
mod normalized_inputs;
mod provenance_rows;
mod request_ports;
use super::{
    Body, Context, Contract, Declarations, DefId, ItemId, NodeKey, RegionId, Source, Stage, g, ir,
    locations, s,
};
use crate::body::regions::Kind;
use locations::Sites;
pub(super) use request_ports::emit as calls;

pub(super) fn allocations(
    stage: &mut Stage,
    input: &Contract,
    context: Context,
    sites: &mut Sites,
) -> Result<Vec<s::OwnedRegion>, &'static str> {
    let body = context
        .body()
        .ok_or("source allocation has no actual body owner")?;
    let mut rows = Vec::new();
    for obligation in &input.obligations {
        stage.charge(1)?;
        let Kind::Variable {
            id,
            needs_provenance,
        } = obligation.kind
        else {
            continue;
        };
        let s::InputSite::Body(site) = sites.input(stage, obligation.source)? else {
            return Err("source inference allocation is not a body-owned site");
        };
        stage.reserve(4 * size_of::<s::OwnedRegion>())?;
        stage.owned_region(
            body,
            id,
            u32::try_from(rows.len()).map_err(|_| "source owned allocation table overflow")?,
        )?;
        rows.push(s::OwnedRegion {
            site,
            allocation: s::RegionAllocation::Inference { needs_provenance },
        });
    }
    for (index, proof) in input.proofs.iter().enumerate() {
        stage.charge(1)?;
        if proof.proof.has_quantified() {
            return Err(
                "quantified source normalization ownership needs a complete scoped transport profile",
            );
        }
        let s::InputSite::Body(site) = sites.input(stage, proof.source)? else {
            return Err("source normalization allocation is not a body-owned site");
        };
        let ordinal =
            u32::try_from(index).map_err(|_| "source normalization input ordinal overflow")?;
        for range in proof.proof.owned_regions() {
            stage.charge(1)?;
            let count = range
                .end
                .checked_sub(range.start)
                .ok_or("source owned normalization range is reversed")?;
            stage.reserve((count as usize).saturating_mul(4 * size_of::<s::OwnedRegion>()))?;
            for id in range {
                stage.charge(1)?;
                stage.owned_region(
                    body,
                    RegionId(id),
                    u32::try_from(rows.len())
                        .map_err(|_| "source owned normalization table overflow")?,
                )?;
                rows.push(s::OwnedRegion {
                    site,
                    allocation: s::RegionAllocation::Normalization(ordinal),
                });
            }
        }
    }
    Ok(rows)
}

pub(super) fn emit(
    stage: &mut Stage,
    facts: &Declarations,
    cfg: &Body,
    definition: DefId,
    input: &Contract,
    context: Context,
    sites: &mut Sites,
) -> Result<s::BodyInputs, &'static str> {
    let body = context
        .body()
        .ok_or("source input has no actual original body")?;
    let mut rows = s::BodyInputs {
        body,
        frame: stage.binder(context)?,
        parameters: Vec::new(),
        environment: builtin_rows::body_environment(stage, facts, definition)?,
        assumptions: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: Vec::new(),
        normalizations: Vec::new(),
        builtins: None,
    };
    stage.reserve(
        input
            .parameters
            .len()
            .saturating_mul(size_of::<s::SlotRef>()),
    )?;
    for &parameter in &input.parameters {
        stage.charge(1)?;
        rows.parameters.push(stage.parameter_slot(parameter)?);
    }
    for &pair in &input.assumptions {
        stage.reserve(4 * size_of::<s::RegionBound>())?;
        rows.assumptions.push(relation(stage, pair, context)?);
    }
    let root = sites.input(stage, input.source.root_site())?;
    for &clause in &input.type_assumptions {
        stage.reserve(4 * size_of::<s::LocatedRequirement>())?;
        let predicate = clause.ordinary().map_err(
            |_| "quantified original assumptions require a complete scoped transport profile",
        )?;
        rows.type_assumptions.push(s::LocatedRequirement {
            site: root,
            value: stage.requirement(facts, predicate, context)?,
        });
    }
    for obligation in &input.obligations {
        stage.reserve(4 * size_of::<s::Obligation>())?;
        let site = sites.input(stage, obligation.source)?;
        let kind = obligation_kind(stage, facts, obligation.kind, context)?;
        rows.obligations.push(s::Obligation { site, kind });
    }
    provenance_rows::append(stage, facts, cfg, context, sites, &mut rows.obligations)?;
    for proof in &input.proofs {
        stage.reserve(4 * size_of::<s::NormalizationInput>())?;
        rows.normalizations.push(normalized_inputs::emit(
            stage, facts, proof, context, sites,
        )?);
    }
    rows.builtins = input
        .builtins
        .as_deref()
        .map(|builtins| builtin_rows::emit(stage, facts, definition, builtins, context, sites))
        .transpose()?;
    Ok(rows)
}

fn obligation_kind(
    stage: &mut Stage,
    facts: &Declarations,
    kind: Kind,
    context: Context,
) -> Result<s::InputKind, &'static str> {
    stage.charge(1)?;
    Ok(match kind {
        Kind::Variable {
            id,
            needs_provenance,
        } => s::InputKind::Variable {
            slot: stage.owned_slot(
                context.body().ok_or("source variable lacks an original")?,
                id,
            )?,
            needs_provenance,
        },
        Kind::Membership { region, captures } => {
            if facts.types.region_args(captures).is_none() {
                return Err("source membership captures must retain their lifetime-only kind");
            }
            s::InputKind::Membership {
                region: stage.region(region, context)?,
                captures: stage.arguments(facts, captures, context)?,
            }
        }
        Kind::LocalBorrow(region) => s::InputKind::LocalBorrow(stage.region(region, context)?),
        Kind::Equality(pair) => s::InputKind::Equality(relation(stage, pair, context)?),
        Kind::Outlives(pair) => s::InputKind::Outlives(relation(stage, pair, context)?),
        Kind::Coercion {
            actual,
            expected,
            unsize,
            reference,
        } => s::InputKind::Coercion {
            actual: stage.ty(facts, actual, context)?,
            expected: stage.ty(facts, expected, context)?,
            unsize,
            reference: reference.map(|reference| s::ReferenceConversion {
                dereferences: reference.dereferences,
                unsize: reference.unsize,
                two_phase: reference.two_phase,
            }),
        },
        Kind::WellFormed(ty) => s::InputKind::WellFormed(stage.ty(facts, ty, context)?),
        Kind::ScopedWellFormed { ty, bound_regions } => {
            let inner = stage.scoped_context(context, bound_regions)?;
            s::InputKind::ScopedWellFormed {
                ty: stage.ty(facts, ty, inner)?,
                frame: stage.binder(inner)?,
            }
        }
        Kind::TypeOutlives { ty, shorter } => s::InputKind::TypeOutlives {
            ty: stage.ty(facts, ty, context)?,
            shorter: stage.region(shorter, context)?,
        },
    })
}
fn relation(
    stage: &mut Stage,
    pair: (RegionId, RegionId),
    context: Context,
) -> Result<s::RegionBound, &'static str> {
    Ok(s::RegionBound {
        longer: stage.region(pair.0, context)?,
        shorter: stage.region(pair.1, context)?,
    })
}
