//! Storage recipes are structural; every closed layout caller pays their proof.
use crate::{
    sema::{
        discharge,
        model::Declarations,
        projections,
        stored_fields::{Preparation, Progress, Query},
    },
    types::{
        ItemId, TypeId,
        lifetime_proof::{Obligations, ScopeId},
    },
};

pub(super) fn prepare(
    facts: &mut Declarations,
    element: TypeId,
    budget: &mut super::Work,
) -> Result<Option<ItemId>, String> {
    let mut charged = facts.stored_field_work().steps;
    let mut preparation = Preparation::default();
    let mut query = Query::new(element);
    let mut proof = Obligations::new(facts.body_regions.probe_frontier()?);
    let mut session = projections::concrete(&mut proof, ScopeId::ROOT)?;
    let result = advance(
        facts,
        &mut preparation,
        &mut query,
        &mut session,
        &mut charged,
        budget,
    );
    drop(session);
    let pending = result?;
    if pending.is_none() {
        discharge::closed(
            facts,
            &proof,
            "source layout lifetime requirement is not satisfied",
        )?;
    }
    Ok(pending)
}

fn advance(
    facts: &mut Declarations,
    preparation: &mut Preparation,
    query: &mut Query,
    session: &mut projections::Concrete<'_>,
    charged: &mut usize,
    budget: &mut super::Work,
) -> Result<Option<ItemId>, String> {
    loop {
        let progress = preparation.advance(facts, query);
        charge(facts, preparation, session, charged, budget)?;
        match progress? {
            Progress::Complete => return Ok(None),
            Progress::Nominal(item) => return Ok(Some(item)),
            Progress::Normalize(raw) => {
                let result = session.normalize(facts, raw).and_then(|(normal, steps)| {
                    preparation.normalized(facts, query, normal, steps)
                });
                charge(facts, preparation, session, charged, budget)?;
                result?;
            }
        }
    }
}

/// Charge completed child work before starting another child, even on failure.
fn charge(
    facts: &mut Declarations,
    preparation: &Preparation,
    session: &projections::Concrete<'_>,
    charged: &mut usize,
    budget: &mut super::Work,
) -> Result<(), String> {
    let work = facts
        .stored_field_work()
        .steps
        .saturating_add(preparation.steps())
        .saturating_add(session.nested_work());
    let delta = work.saturating_sub(*charged);
    *charged = work;
    budget.charge(
        facts,
        delta,
        "source layout storage normalization work budget exceeded",
    )
}
