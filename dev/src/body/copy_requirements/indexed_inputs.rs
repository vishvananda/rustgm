//! Exact primary and displaced source inputs share one bounded actual-body index.
use super::{Body, BodyKey, Contract, Declarations, HashMap, HashSet, prepare};

/// Original proof components and displaced source rows keep their exact bodies.
/// Index bodies once, lazily for defining requirements outside source mode.
#[cfg(test)]
pub(crate) fn originals<'a, 'b>(
    facts: &mut Declarations,
    bodies: impl IntoIterator<Item = &'a Body>,
    contracts: impl IntoIterator<Item = &'b mut Contract>,
) -> Result<(), String> {
    originals_using(
        facts,
        bodies,
        contracts,
        std::iter::empty(),
        &OriginalIndexBudget::new(1_000_000, 64 * 1024 * 1024),
    )
}

/// Displaced source inputs are independent proofs of the same actual bodies.
/// They do not consume or relax the exhaustive primary input inventory.
pub(crate) fn originals_and_source<'a, 'b, 'c>(
    facts: &mut Declarations,
    bodies: impl IntoIterator<Item = &'a Body>,
    contracts: impl IntoIterator<Item = &'b mut Contract>,
    source: impl IntoIterator<Item = &'c mut Contract>,
) -> Result<(), String> {
    originals_using(
        facts,
        bodies,
        contracts,
        source,
        &OriginalIndexBudget::new(1_000_000, 64 * 1024 * 1024),
    )
}

/// Temporary index costs have no declaration/header authority. Failed attempted
/// work and capacity payments remain sticky for this one construction owner.
pub(crate) struct OriginalIndexBudget {
    steps: std::cell::Cell<usize>,
    bytes: std::cell::Cell<usize>,
    work_limit: usize,
    byte_limit: usize,
}
impl OriginalIndexBudget {
    pub(crate) const fn new(work_limit: usize, byte_limit: usize) -> Self {
        Self {
            steps: std::cell::Cell::new(0),
            bytes: std::cell::Cell::new(0),
            work_limit,
            byte_limit,
        }
    }
    pub(crate) fn costs(&self) -> (usize, usize) {
        (self.steps.get(), self.bytes.get())
    }
    pub(crate) fn charge(&self, work: usize, bytes: usize) -> Result<(), &'static str> {
        self.steps.set(self.steps.get().saturating_add(work));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        if self.steps.get() > self.work_limit || self.bytes.get() > self.byte_limit {
            return Err("original body index exceeds its scratch work/storage budget");
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn originals_with_budget<'a, 'b>(
    facts: &mut Declarations,
    bodies: impl IntoIterator<Item = &'a Body>,
    contracts: impl IntoIterator<Item = &'b mut Contract>,
    budget: &OriginalIndexBudget,
) -> Result<(), String> {
    originals_using(facts, bodies, contracts, std::iter::empty(), budget)
}

fn originals_using<'a, 'b, 'c>(
    facts: &mut Declarations,
    bodies: impl IntoIterator<Item = &'a Body>,
    contracts: impl IntoIterator<Item = &'b mut Contract>,
    overlays: impl IntoIterator<Item = &'c mut Contract>,
    budget: &OriginalIndexBudget,
) -> Result<(), String> {
    let source = facts.source_bindings.is_some();
    let mut bodies = Some(bodies.into_iter());
    let mut index = if source {
        Some(original_index(
            bodies.take().ok_or("missing original bodies")?,
            &mut facts.body_work.canonical_steps,
            None,
        )?)
    } else {
        None
    };
    prepare_overlays(facts, &mut bodies, &mut index, overlays, budget)?;
    for contract in contracts {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
        if !source && !contract.requires_defining_cleanup_preparation() {
            continue;
        }
        ensure_index(facts, &mut bodies, &mut index, budget)?;
        let body = index
            .as_mut()
            .ok_or("missing original body index")?
            .remove(&contract.key)
            .ok_or("source Copy original contract has no exact body")?;
        prepare(facts, body, contract)?;
    }
    if source && index.is_some_and(|index| !index.is_empty()) {
        return Err("source Copy original body has no exact contract".into());
    }
    Ok(())
}

fn ensure_index<'a>(
    facts: &mut Declarations,
    bodies: &mut Option<impl Iterator<Item = &'a Body>>,
    index: &mut Option<HashMap<BodyKey, &'a Body>>,
    budget: &OriginalIndexBudget,
) -> Result<(), String> {
    if index.is_none() {
        let mut attempted = 0;
        let prepared = original_index(
            bodies
                .take()
                .ok_or("original body index was consumed twice")?,
            &mut attempted,
            Some(&|work, bytes| budget.charge(work, bytes)),
        );
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(attempted);
        *index = Some(prepared?);
    }
    Ok(())
}

fn prepare_overlays<'a, 'b>(
    facts: &mut Declarations,
    bodies: &mut Option<impl Iterator<Item = &'a Body>>,
    index: &mut Option<HashMap<BodyKey, &'a Body>>,
    overlays: impl IntoIterator<Item = &'b mut Contract>,
    budget: &OriginalIndexBudget,
) -> Result<(), String> {
    let mut seen = HashSet::default();
    for contract in overlays {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
        budget.charge(1, 0)?;
        if facts.source_bindings.is_none() && !contract.requires_defining_cleanup_preparation() {
            continue;
        }
        if seen.contains(&contract.key) {
            return Err("duplicate displaced source Copy input".into());
        }
        let growth =
            crate::support::retained::Growth::map::<BodyKey, ()>(seen.len(), seen.capacity(), 1);
        facts.body_work.canonical_steps =
            facts.body_work.canonical_steps.saturating_add(growth.work);
        budget.charge(growth.work, growth.bytes)?;
        seen.try_reserve(1)
            .map_err(|_| "displaced source Copy input allocation failed")?;
        seen.insert(contract.key);
        ensure_index(facts, bodies, index, budget)?;
        let body = index
            .as_ref()
            .and_then(|index| index.get(&contract.key))
            .ok_or("displaced source Copy input has no exact body")?;
        prepare(facts, body, contract)?;
    }
    Ok(())
}

fn original_index<'a>(
    bodies: impl Iterator<Item = &'a Body>,
    steps: &mut usize,
    payment: Option<crate::support::retained::Payment<'_>>,
) -> Result<HashMap<BodyKey, &'a Body>, String> {
    let mut index = HashMap::default();
    for body in bodies {
        *steps = steps.saturating_add(1);
        if let Some(payment) = payment {
            payment(1, 0)?;
        }
        if index.len() >= 1_000_000 || index.contains_key(&body.key()) {
            return Err("source Copy original body index is duplicate or exhausted".into());
        }
        let growth = crate::support::retained::Growth::map::<BodyKey, &Body>(
            index.len(),
            index.capacity(),
            1,
        );
        *steps = steps.saturating_add(growth.work);
        if let Some(payment) = payment {
            payment(growth.work, growth.bytes)?;
        }
        if growth.bytes > 64 * 1024 * 1024 {
            return Err("source Copy original body index exceeds storage budget".into());
        }
        index
            .try_reserve(1)
            .map_err(|_| "source Copy original body index allocation failed")?;
        index.insert(body.key(), body);
    }
    Ok(index)
}
