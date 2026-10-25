//! Diagnostic ancestry follows retained declaration order with shared completed summaries.
use super::{Declarations, Error, Report, discarded_types::Requirement};
use crate::{
    resolve::{model::DefId, unit::FileId},
    support::hash::HashSet,
    types::{ItemId, parameters::clauses::Clause},
};
struct Entry {
    item: ItemId,
    direct: Option<Requirement>,
    children: Vec<ItemId>,
    waiting: Option<ItemId>,
}
impl Declarations {
    pub(super) fn must_use_trait(&self, report: &Report, root: ItemId) -> Result<Option<Requirement>, Error> {
        let Some(table) = self.must_use_contracts.as_deref() else { return Ok(None) };
        if table.sources.is_empty() && table.unknown.is_empty() { return Ok(None) }
        trait_charge(table, 1, 0)?;
        if let Some(result) = table.trait_requirements.borrow().get(&root).copied() { return Ok(result) }
        trait_charge(table, 1, 4 * size_of::<Entry>() + 4 * size_of::<ItemId>())?;
        let mut pending = vec![self.trait_entry(report, root)?];
        let mut active = HashSet::default();
        active.insert(root);
        while let Some(entry) = pending.last_mut() {
            trait_charge(table, 1, 0)?;
            let result = if entry.direct.is_some() { entry.direct }
            else if let Some(child) = entry.waiting.take() {
                let result = table.trait_requirements.borrow().get(&child).copied()
                    .ok_or_else(|| error("must_use trait child lost its completed summary"))?;
                if result.is_none() { continue }
                result
            } else if let Some(child) = entry.children.pop() {
                entry.waiting = Some(child);
                if table.trait_requirements.borrow().contains_key(&child) { continue }
                if active.contains(&child) { return Err(error("must_use trait ancestry contains a cycle")) }
                trait_charge(table, 1, 4 * size_of::<Entry>() + 4 * size_of::<ItemId>())?;
                active.insert(child);
                pending.push(self.trait_entry(report, child)?);
                continue;
            } else { None };
            let item = entry.item;
            trait_charge(table, 1, 4 * size_of::<(ItemId, Option<Requirement>)>())?;
            let mut completed = table.trait_requirements.borrow_mut();
            if completed.len() >= 1_000_000 { return Err(error("must_use trait storage budget exceeded")) }
            completed.insert(item, result);
            active.remove(&item);
            pending.pop();
        }
        table.trait_requirements.borrow().get(&root).copied()
            .ok_or_else(|| error("must_use trait traversal lost its root"))
    }
    fn trait_entry(&self, report: &Report, item: ItemId) -> Result<Entry, Error> {
        let table = self.must_use_contracts.as_deref().ok_or_else(|| error("missing diagnostic ancestry owner"))?;
        trait_charge(table, 1, 0)?;
        let direct = self.must_use_contract(report, DefId(item.0))?.map(|_| Requirement::Declaration(DefId(item.0)));
        let mut entry = Entry { item, direct, children: Vec::new(), waiting: None };
        if direct.is_some() { return Ok(entry) }
        if let Some(header) = self.generics.traits.as_deref().and_then(|traits| traits.header(item)) {
            for located in &header.predicates {
                trait_charge(table, 1, 0)?;
                if !header.self_bounds.contains(&located.source) { continue }
                let predicate = match located.value {
                    Clause::Predicate(predicate) => predicate,
                    Clause::ForAll(id) => self.types.quantified(id).ok_or_else(|| error("must_use super-bound is missing"))?.predicate,
                };
                if let Some((application, _)) = predicate.bound.trait_parts() {
                    let child = self.types.trait_application(application).ok_or_else(|| error("must_use supertrait application is missing"))?.item;
                    trait_charge(table, 1, 4 * size_of::<ItemId>())?;
                    entry.children.push(child);
                }
            }
        } else if report.resolution.definitions.get(item.0 as usize).is_some_and(|definition| definition.node.is_some()) {
            return Err(error("must_use trait diagnostic requires its completed header"));
        }
        Ok(entry)
    }
}
fn trait_charge(table: &super::Contracts, steps: usize, bytes: usize) -> Result<(), Error> {
    table.trait_steps.set(table.trait_steps.get().saturating_add(steps));
    table.trait_bytes.set(table.trait_bytes.get().saturating_add(bytes));
    if table.trait_steps.get() > 10_000_000 || table.trait_bytes.get() > 64 * 1024 * 1024 {
        return Err(error("must_use trait diagnostic work or storage budget exceeded"));
    }
    Ok(())
}
fn error(message: &'static str) -> Error { Error::at(FileId(0), 0, message) }
