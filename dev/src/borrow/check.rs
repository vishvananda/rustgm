//! The executable boundary combines owned access, storage, contracts and loans.
pub(crate) mod incremental;
mod static_targets;
pub(crate) mod structure;
use super::{loans, ownership, regions};
use crate::support::hash::HashMap;
use crate::{
    body::ir::{Body, Program},
    types::{
        ItemId, Store,
        aggregate::Definitions,
        copy::{Copies, Declaration},
        outlives::LiveRegions,
    },
};
use std::hash::BuildHasher;

pub struct Checked<'a> {
    program: &'a Program,
    destruction: Option<Box<super::destruction::Proof>>,
}
impl Checked<'_> {
    #[must_use]
    pub const fn program(&self) -> &Program {
        self.program
    }
    #[must_use]
    pub fn destruction(&self) -> Option<&super::destruction::Proof> {
        self.destruction.as_deref()
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
    pub node: Option<crate::body::ir::ProofSite>,
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
    pub derives: &'a std::collections::HashMap<ItemId, Declaration, S>,
    pub copies: &'a mut Copies,
    /// Actual source-provider policy; absent body carriers never permit fallback.
    pub source_copy: bool,
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
    let checked = run::<RECORD, _>(environment, contracts, program, None, |owned, proof| {
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
    run::<false, _>(environment, contracts, program, None, |_, _| {})
}
/// Verify concrete instances against the exact templates proved before substitution.
/// Contracts must be proved in program order, each in its own substituted lifetime
/// context. Every instance also requires its original template's complete proof.
/// # Errors
/// Rejects incomplete or foreign specializations and all ordinary executable proof errors.
pub fn verify_instances<'a, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    contracts: &regions::Analysis,
    program: &'a Program,
    templates: &super::templates::Checked<'_>,
) -> Result<Checked<'a>, Error> {
    if contracts.bodies.len() != program.bodies.len() {
        return Err(Error {
            node: None,
            message: "instance proofs have different body coverage".into(),
        });
    }
    run::<false, _>(environment, contracts, program, Some(templates), |_, _| {})
}
fn run<'a, const RECORD: bool, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    contracts: &regions::Analysis,
    program: &'a Program,
    templates: Option<&super::templates::Checked<'_>>,
    mut record: impl FnMut(ownership::BodyFacts, loans::Report),
) -> Result<Checked<'a>, Error> {
    require_resolved_promotions(program)?;
    if let Some(pending) = &program.drops {
        return Err(Error {
            node: Some(pending.source.into()),
            message: crate::body::drops::PENDING_MESSAGE.into(),
        });
    }
    let by_key: HashMap<_, _> = if templates.is_none() {
        contracts.bodies.iter().map(|b| (b.key, b)).collect()
    } else {
        HashMap::default()
    };
    let mut reused = Reused::default();
    for (ordinal, body) in program.bodies.iter().enumerate() {
        let contract = if let Some(templates) = templates {
            if body.instance.is_some() && templates.contract(body).is_none() {
                return Err(Error {
                    node: Some(body.source.into()),
                    message: "instance has no checked original template".into(),
                });
            }
            contracts.bodies.get(ordinal).filter(|c| {
                c.key == body.key() && c.source == body.source && c.instance == body.instance
            })
        } else if body.instance.is_some() {
            None
        } else {
            by_key.get(&body.key()).copied()
        }
        .ok_or_else(|| Error {
            node: Some(body.source.into()),
            message: "body has no lifetime proof".into(),
        })?;
        let (owned, proof) = prove_body::<RECORD, _>(
            environment,
            &mut reused,
            body,
            contract,
            structure::Use::Own,
        )?;
        record(owned, proof);
    }
    Ok(Checked {
        program,
        destruction: None,
    })
}

/// Verify pending destruction and retain its selected recipes and body plans.
/// Ordinary assignment views keep their existing pending-elaboration boundary.
/// # Errors
/// Rejects invalid moves, lifetime contracts, destructor loans and resource limits.
pub fn verify_destruction<'a, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    contracts: &regions::Analysis,
    program: &'a Program,
    mut catalog: super::destruction::glue::Catalog<'_>,
) -> Result<Checked<'a>, Error> {
    use super::destruction;
    require_resolved_promotions(program)?;
    let error = |node, message: &str| Error {
        node,
        message: message.into(),
    };
    let pending = program
        .drops
        .as_ref()
        .ok_or_else(|| error(None, "missing destruction work"))?;
    let mut drop_regions =
        crate::types::destruction::regions::Regions::new(pending.methods.iter().map(|m| m.item))
            .map_err(|e| error(Some(pending.source.into()), e))?;
    let mut reused = Reused::default();
    let by_key: HashMap<_, _> = contracts.bodies.iter().map(|b| (b.key, b)).collect();
    let mut plans = Vec::with_capacity(program.bodies.len());
    for body in &program.bodies {
        if body.instance.is_some() {
            return Err(error(
                Some(body.source.into()),
                "specialized destruction requires checked instance contracts",
            ));
        }
        let contract = by_key
            .get(&body.key())
            .ok_or_else(|| error(Some(body.source.into()), "body has no lifetime proof"))?;
        plans.push(
            prove_cleanup(
                environment,
                &mut reused,
                &mut drop_regions,
                body,
                contract,
                &mut catalog,
                structure::Use::Own,
            )?
            .map(Box::new),
        );
    }
    Ok(Checked {
        program,
        destruction: Some(Box::new(destruction::Proof {
            plans,
            recipes: catalog.into_recipes(),
        })),
    })
}

fn require_resolved_promotions(program: &Program) -> Result<(), Error> {
    for body in &program.bodies {
        require_body_promotions(body)?;
    }
    Ok(())
}
fn require_body_promotions(body: &Body) -> Result<(), Error> {
    if body.resolved_callables.is_some() {
        return Err(Error {
            node: Some(body.source.into()),
            message: "resolved callable targets require a completed body proof".into(),
        });
    }
    if body
        .promotions
        .as_ref()
        .is_some_and(|p| p.deferred().next().is_some())
    {
        return Err(Error {
            node: Some(body.source.into()),
            message: "deferred promotion values require concrete instantiation".into(),
        });
    }
    Ok(())
}

/// Prove one body. An instance reuses the facts an earlier instance of its
/// checked template proved (`structure::Use::Shared`) or proves them into a
/// slot kept for later instances; ordinary bodies prove and drop them here.
fn prove_body<const RECORD: bool, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    reused: &mut Reused,
    body: &Body,
    contract: &regions::Body,
    structure: structure::Use<'_, '_>,
) -> Result<(ownership::BodyFacts, loans::Report), Error> {
    let mut copies = super::permissions::check(environment, contract)?.map(Copies::in_environment);
    let local;
    let (order, storage) = match structure {
        structure::Use::Shared(shared) => (shared.order, shared.storage),
        structure::Use::Keep(slot) => {
            let proved = slot.insert(structure::Proved::new(body)?);
            (proved.order.as_ref(), &proved.storage)
        }
        structure::Use::Own => {
            local = structure::Proved::new(body)?;
            (local.order.as_ref(), &local.storage)
        }
    };
    let mut copy_view = super::copy_view::CopyView::checked(
        copies.as_mut().unwrap_or(environment.copies),
        body,
        contract,
        environment.source_copy,
    )
    .map_err(|message| Error {
        node: Some(body.source),
        message: message.into(),
    })?;
    let owned = ownership::analyze_body_ordered(
        environment.types,
        environment.definitions,
        environment.derives,
        &mut copy_view,
        body,
        order,
    )
    .map_err(|e| Error {
        node: Some(e.node),
        message: e.message.into(),
    })?;
    let proof = loans::analyze::<RECORD>(
        environment.types,
        &mut reused.components,
        body,
        contract,
        &owned,
        storage,
    )
    .map_err(|e| Error {
        node: Some(e.node),
        message: e.message.into(),
    })?;
    Ok((owned, proof))
}

/// What one prover reuses across the bodies it proves: the live-region
/// component memo, and buffers each analysis takes and returns emptied (they
/// carry no facts between bodies).
#[derive(Default)]
pub(crate) struct Reused {
    components: LiveRegions,
    transitions: Vec<super::ownership::initialization::Transition>,
}

fn prove_cleanup<S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    reused: &mut Reused,
    drop_regions: &mut crate::types::destruction::regions::Regions,
    body: &Body,
    contract: &regions::Body,
    catalog: &mut super::destruction::glue::Catalog<'_>,
    structure: structure::Use<'_, '_>,
) -> Result<Option<super::destruction::Plan>, Error> {
    use super::{destruction, ownership::initialization};
    let parameters = super::permissions::check(environment, contract)?;
    let mut copies = parameters
        .as_ref()
        .map(|e| Copies::in_environment(std::sync::Arc::clone(e)));
    let mut scoped_regions = parameters.map(|e| drop_regions.scoped(e));
    let error = |node, message: &str| Error {
        node,
        message: message.into(),
    };
    let (shared, slot) = match structure {
        structure::Use::Shared(shared) => (Some(shared), None),
        structure::Use::Keep(slot) => (None, Some(slot)),
        structure::Use::Own => (None, None),
    };
    let order = if shared.is_some() {
        None
    } else {
        super::order::Order::new(body)
    };
    let mut copy_view = super::copy_view::CopyView::checked(
        copies.as_mut().unwrap_or(environment.copies),
        body,
        contract,
        environment.source_copy,
    )
    .map_err(|message| error(Some(body.source), message))?;
    let init = if let Some(shared) = shared {
        initialization::analyze_shared(
            environment.types,
            environment.definitions,
            environment.derives,
            &mut copy_view,
            body,
            shared.order,
            shared.storage,
        )
    } else {
        initialization::analyze_ordered(
            environment.types,
            environment.definitions,
            environment.derives,
            &mut copy_view,
            body,
            order.as_ref(),
        )
    }
    .map_err(|e| error(Some(e.node), e.message))?;
    let plan = destruction::select_reusing(catalog, &init, &mut reused.transitions)
        .map_err(|e| error(Some(e.node), e.message))?;
    loans::check_destruction(
        environment.types,
        environment.definitions,
        &mut reused.components,
        scoped_regions.as_mut().unwrap_or(drop_regions),
        &init,
        contract,
        &plan,
    )
    .map_err(|e| error(Some(e.node), e.message))?;
    if let Some(slot) = slot
        && let std::borrow::Cow::Owned(storage) = init.storage
    {
        *slot = Some(structure::Proved { order, storage });
    }
    // Keep no event tape when no runtime cleanup remains, after proving liveness.
    Ok(plan.cleanups.iter().any(|c| c.node.is_some()).then(|| {
        let mut plan = plan;
        plan.shrink_to_fit();
        plan
    }))
}
