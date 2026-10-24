//! Trait attribute ancestry is immutable diagnostic metadata, never selection evidence.
use super::{Declarations, Error, Report, discarded_types::Requirement};
use crate::{
    resolve::{model::DefId, unit::FileId},
    support::hash::HashMap,
    types::{ItemId, parameters::clauses::Clause},
};
struct Entry {
    item: ItemId,
    direct: Option<Requirement>,
    children: Vec<ItemId>,
}
impl Declarations {
    pub(super) fn must_use_trait(
        &self,
        report: &Report,
        root: ItemId,
    ) -> Result<Option<Requirement>, Error> {
        let Some(table) = self.must_use_contracts.as_deref() else {
            return Ok(None);
        };
        if table.sources.is_empty() && table.unknown.is_empty() {
            return Ok(None);
        }
        if let Some(result) = table.trait_requirements.borrow().get(&root).copied() {
            return Ok(result);
        }
        let (entries, indices) = self.must_use_trait_component(report, root)?;
        let mut results: Vec<_> = entries.iter().map(|entry| entry.direct).collect();
        let mut parents = vec![Vec::new(); entries.len()];
        let mut ready = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            for child in &entry.children {
                trait_step(table)?;
                if let Some(&child_index) = indices.get(child) {
                    parents[child_index].push(index);
                } else if let Some(requirement) = table
                    .trait_requirements
                    .borrow()
                    .get(child)
                    .copied()
                    .flatten()
                {
                    results[index] = results[index].or(Some(requirement));
                }
            }
            if results[index].is_some() {
                ready.push(index);
            }
        }
        while let Some(child) = ready.pop() {
            for &parent in &parents[child] {
                trait_step(table)?;
                if results[parent].is_none() {
                    results[parent] = results[child];
                    ready.push(parent);
                }
            }
        }
        let mut completed = table.trait_requirements.borrow_mut();
        if completed.len().saturating_add(entries.len()) > 1_000_000 {
            return Err(error("must_use trait storage budget exceeded"));
        }
        for (entry, result) in entries.into_iter().zip(results) {
            completed.insert(entry.item, result);
        }
        completed
            .get(&root)
            .copied()
            .ok_or_else(|| error("must_use trait component lost its root"))
    }
    fn must_use_trait_component(
        &self,
        report: &Report,
        root: ItemId,
    ) -> Result<(Vec<Entry>, HashMap<ItemId, usize>), Error> {
        let table = self
            .must_use_contracts
            .as_deref()
            .ok_or_else(|| error("missing trait diagnostic owner"))?;
        let mut entries = Vec::new();
        let mut indices = HashMap::default();
        let mut pending = vec![root];
        while let Some(item) = pending.pop() {
            trait_step(table)?;
            if indices.contains_key(&item) || table.trait_requirements.borrow().contains_key(&item)
            {
                continue;
            }
            if entries.len() >= 1_000_000 {
                return Err(error("must_use trait component budget exceeded"));
            }
            let direct = self
                .must_use_contract(report, DefId(item.0))?
                .map(|_| Requirement::Declaration(DefId(item.0)));
            let mut children = Vec::new();
            if let Some(header) = self
                .generics
                .traits
                .as_deref()
                .and_then(|traits| traits.header(item))
            {
                for located in &header.predicates {
                    trait_step(table)?;
                    if !header.self_bounds.contains(&located.source) {
                        continue;
                    }
                    let predicate = match located.value {
                        Clause::Predicate(predicate) => predicate,
                        Clause::ForAll(id) => {
                            self.types
                                .quantified(id)
                                .ok_or_else(|| error("must_use trait super-bound is missing"))?
                                .predicate
                        }
                    };
                    if let Some((application, _)) = predicate.bound.trait_parts() {
                        children.push(
                            self.types
                                .trait_application(application)
                                .ok_or_else(|| error("must_use supertrait application is missing"))?
                                .item,
                        );
                    }
                }
            } else if report
                .resolution
                .definitions
                .get(item.0 as usize)
                .is_some_and(|definition| definition.node.is_some())
            {
                return Err(error(
                    "must_use trait diagnostic requires its completed header",
                ));
            }
            indices.insert(item, entries.len());
            pending.extend(children.iter().copied());
            entries.push(Entry {
                item,
                direct,
                children,
            });
        }
        Ok((entries, indices))
    }
}
fn trait_step(table: &super::Contracts) -> Result<(), Error> {
    let steps = table
        .trait_steps
        .get()
        .checked_add(1)
        .ok_or_else(|| error("must_use trait work overflow"))?;
    table.trait_steps.set(steps);
    if steps > 10_000_000 {
        return Err(error("must_use trait diagnostic work budget exceeded"));
    }
    Ok(())
}
fn error(message: &'static str) -> Error {
    Error::at(FileId(0), 0, message)
}
