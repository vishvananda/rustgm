//! Complete demanded field/argument instances before immutable property queries.
use super::Definitions;
use crate::support::hash::HashSet;
use crate::types::{ItemId, Kind, ListId, Store, TypeId};
#[cfg(test)]
#[path = "../../../tests/invariants/resumable_field_walks.rs"]
mod tests;

/// A storage walk that waits for field templates their declarations have not
/// published yet. Its pending types and visited set survive each wait, so a
/// resumed walk continues at the waited nominal rather than at its root.
#[derive(Default)]
pub(crate) struct PendingFields {
    work: Vec<TypeId>,
    seen: HashSet<TypeId>,
}

impl Definitions {
    /// Prepare this root's fields and nominal type arguments without expanding
    /// pointer/reference referents. Type arguments can have derive obligations
    /// even when stored behind a pointer. This proves neither sizedness nor cycles.
    /// Declarations must already be complete and remain immutable afterwards.
    /// # Errors
    /// Rejects missing/malformed fields and bounded substitution/graph exhaustion.
    pub fn prepare(&mut self, types: &mut Store, root: TypeId) -> Result<(), &'static str> {
        self.prepare_with(types, root, false, true, &mut None)
    }
    /// Equality additionally inspects reference referents. Raw pointer targets
    /// and function signatures remain outside that dependency graph.
    /// # Errors
    /// Rejects missing/malformed fields and bounded substitution/graph exhaustion.
    pub fn prepare_equality(
        &mut self,
        types: &mut Store,
        root: TypeId,
    ) -> Result<(), &'static str> {
        self.prepare_with(types, root, true, true, &mut None)
    }
    /// A layout demand may meet a declaration that is still being completed.
    /// Reuse real substitution, but never publish that pending root as failed.
    /// This is structural data and supplies no Sized or source-call permission.
    pub(crate) fn prepare_storage_fields(
        &mut self,
        types: &mut Store,
        root: TypeId,
    ) -> Result<(), &'static str> {
        self.prepare_with(types, root, false, false, &mut None)
    }
    /// The same storage preparation, naming a nominal whose field template its
    /// declaration has not yet published instead of failing on it.
    /// # Errors
    /// Rejects malformed fields and bounded substitution/graph exhaustion.
    pub(crate) fn prepare_storage_fields_pending(
        &mut self,
        types: &mut Store,
        root: TypeId,
    ) -> Result<Option<ItemId>, &'static str> {
        let mut missing = None;
        match (
            self.prepare_with(types, root, false, false, &mut missing),
            missing,
        ) {
            (Ok(()), _) => Ok(None),
            (Err(_), Some(item)) => Ok(Some(item)),
            (Err(error), None) => Err(error),
        }
    }
    /// A declaration-time body's storage walk, resumable: after a wait,
    /// `pending` continues at the nominal that lacked its template. Each
    /// visited type is charged once, when it is visited, and is published as
    /// the crate-wide preparation publishes it, so a body never pays again for
    /// rows other types already occupy.
    /// # Errors
    /// Rejects malformed fields and bounded substitution/graph exhaustion.
    pub(crate) fn resume_storage_fields(
        &mut self,
        types: &mut Store,
        root: TypeId,
        pending: &mut PendingFields,
    ) -> Result<Option<ItemId>, &'static str> {
        if pending.work.is_empty() {
            if self
                .instances
                .as_ref()
                .and_then(|instances| instances.prepared.get(root, false))
                .is_some_and(|result| result.is_ok())
            {
                return Ok(None);
            }
            pending.work.push(root);
        }
        let mut missing = None;
        let result = self.prepare_graph(
            types,
            &mut pending.work,
            false,
            false,
            &mut pending.seen,
            &mut missing,
        );
        match (result, missing) {
            (Ok(()), _) => {
                self.publish_prepared(std::mem::take(&mut pending.seen), false, false)?;
                Ok(None)
            }
            (Err(_), Some(item)) => Ok(Some(item)),
            (Err(error), None) => Err(error),
        }
    }
    fn prepare_with(
        &mut self,
        types: &mut Store,
        root: TypeId,
        references: bool,
        cache_failures: bool,
        missing: &mut Option<ItemId>,
    ) -> Result<(), &'static str> {
        if let Some(result) = self
            .instances
            .as_ref()
            .and_then(|i| i.prepared.get(root, references))
            && (cache_failures || result.is_ok())
        {
            return result;
        }
        let mut seen = HashSet::default();
        let result = self.prepare_graph(
            types,
            &mut vec![root],
            references,
            cache_failures,
            &mut seen,
            missing,
        );
        if result.is_ok() {
            self.publish_prepared(seen, references, !cache_failures)?;
        } else if cache_failures && (seen.contains(&root) || missing.is_some()) {
            // A failed query cannot publish unfinished descendants as prepared.
            // Its declarations are complete, so a missing template is final
            // here too; only storage preparation waits for one.
            self.instances
                .get_or_insert_with(Default::default)
                .prepared
                .insert(root, references, result);
        }
        result
    }
    /// Publish a completed walk's visited types as prepared. A walk that does
    /// not cache failures also pays for the rows it publishes.
    fn publish_prepared(
        &mut self,
        seen: HashSet<TypeId>,
        references: bool,
        charged: bool,
    ) -> Result<(), &'static str> {
        let instances = self.instances.get_or_insert_with(Default::default);
        if charged {
            instances.charge(seen.len())?;
            let rows = seen.iter().map(|ty| ty.0 as usize + 1).max().unwrap_or(0);
            instances.prepare_success_rows(rows)?;
        }
        for ty in seen {
            instances.prepared.insert(ty, references, Ok(()));
        }
        Ok(())
    }
    /// Walk `work` to completion. A nominal without a published template stays
    /// on top of `work`, unvisited and uncharged, and is named in `missing`.
    fn prepare_graph(
        &mut self,
        types: &mut Store,
        work: &mut Vec<TypeId>,
        references: bool,
        reuse_failures: bool,
        seen: &mut HashSet<TypeId>,
        missing: &mut Option<ItemId>,
    ) -> Result<(), &'static str> {
        while let Some(&ty) = work.last() {
            let cached = self
                .instances
                .as_ref()
                .and_then(|instances| instances.prepared.get(ty, references))
                .filter(|result| reuse_failures || result.is_ok());
            if cached.is_none()
                && !seen.contains(&ty)
                && let Some(Kind::Adt(item, _)) = types.kind(ty)
                && self.get(item).is_none()
            {
                *missing = Some(item);
                return Err("missing aggregate field template");
            }
            work.pop();
            let instances = self.instances.get_or_insert_with(Default::default);
            instances.charge(1)?;
            if let Some(result) = cached {
                result?;
                continue;
            }
            if seen.contains(&ty) {
                continue;
            }
            instances.retain(64)?;
            seen.insert(ty);
            match types.kind(ty).ok_or("invalid field preparation type")? {
                Kind::Adt(_, arguments) => {
                    self.instantiate(types, ty)?;
                    let shape = self.raw_shape(types, ty)?;
                    let mut count = 0_usize;
                    for variant in shape.variants() {
                        count = count
                            .checked_add(
                                types
                                    .list(variant.fields)
                                    .ok_or("invalid prepared fields")?
                                    .len(),
                            )
                            .ok_or("field preparation work overflow")?;
                    }
                    self.instances
                        .as_mut()
                        .ok_or("missing field query")?
                        .charge(count)?;
                    for variant in self.raw_shape(types, ty)?.variants() {
                        extend(types, variant.fields, work)?;
                    }
                    let arguments = types
                        .arguments(arguments)
                        .ok_or("invalid field arguments")?;
                    self.instances
                        .as_mut()
                        .ok_or("missing field query")?
                        .charge(arguments.len())?;
                    if work.len().saturating_add(arguments.len()) > 1_000_000 {
                        return Err("field preparation pending work budget exceeded");
                    }
                    for argument in arguments {
                        if let crate::types::arguments::Argument::Type(ty) = argument {
                            work.push(ty);
                        }
                    }
                }
                Kind::Pattern { base, .. } => work.push(base),
                Kind::Closure { environment, .. } => work.push(environment),
                Kind::Tuple(list) => {
                    let count = types
                        .list(list)
                        .ok_or("invalid field preparation tuple")?
                        .len();
                    instances.charge(count)?;
                    extend(types, list, work)?;
                }
                Kind::Array { element, .. }
                | Kind::ArrayTerm { element, .. }
                | Kind::Slice(element) => work.push(element),
                Kind::Reference { referent, .. } if references => work.push(referent),
                Kind::Reference {
                    referent: pointee, ..
                }
                | Kind::Pointer { pointee, .. }
                    if types.is_sized(pointee) != Some(true) =>
                {
                    self.prepare_metadata(types, pointee)?;
                }
                // Sized references/raw pointers do not require their referent's fields.
                // Function signatures and rigid parameters own no nested values.
                _ => {}
            }
        }
        Ok(())
    }
}
fn extend(types: &Store, list: ListId, work: &mut Vec<TypeId>) -> Result<(), &'static str> {
    let fields = types.list(list).ok_or("invalid prepared field list")?;
    if work.len().saturating_add(fields.len()) > 1_000_000 {
        return Err("field preparation pending work budget exceeded");
    }
    work.extend_from_slice(fields);
    Ok(())
}
