//! Complete demanded field/argument instances before immutable property queries.
use super::Definitions;
use crate::types::{Kind, ListId, Store, TypeId};
use std::collections::HashSet;

impl Definitions {
    /// Prepare this root's fields and nominal type arguments without expanding
    /// pointer/reference referents. Type arguments can have derive obligations
    /// even when stored behind a pointer. This proves neither sizedness nor cycles.
    /// Declarations must already be complete and remain immutable afterwards.
    /// # Errors
    /// Rejects missing/malformed fields and bounded substitution/graph exhaustion.
    pub fn prepare(&mut self, types: &mut Store, root: TypeId) -> Result<(), &'static str> {
        self.prepare_with(types, root, false)
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
        self.prepare_with(types, root, true)
    }
    fn prepare_with(
        &mut self,
        types: &mut Store,
        root: TypeId,
        references: bool,
    ) -> Result<(), &'static str> {
        if let Some(result) = self
            .instances
            .as_ref()
            .and_then(|i| i.prepared.get(&(root, references)))
        {
            return *result;
        }
        let mut seen = HashSet::new();
        let result = self.prepare_graph(types, root, references, &mut seen);
        let instances = self.instances.get_or_insert_with(Default::default);
        if result.is_ok() {
            instances
                .prepared
                .extend(seen.into_iter().map(|ty| ((ty, references), Ok(()))));
        } else if seen.contains(&root) {
            // A failed query cannot publish unfinished descendants as prepared.
            instances.prepared.insert((root, references), result);
        }
        result
    }
    fn prepare_graph(
        &mut self,
        types: &mut Store,
        root: TypeId,
        references: bool,
        seen: &mut HashSet<TypeId>,
    ) -> Result<(), &'static str> {
        let mut work = vec![root];
        while let Some(ty) = work.pop() {
            let instances = self.instances.get_or_insert_with(Default::default);
            instances.charge(1)?;
            if let Some(result) = instances.prepared.get(&(ty, references)) {
                (*result)?;
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
                        extend(types, variant.fields, &mut work)?;
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
                Kind::Tuple(list) => {
                    let count = types
                        .list(list)
                        .ok_or("invalid field preparation tuple")?
                        .len();
                    instances.charge(count)?;
                    extend(types, list, &mut work)?;
                }
                Kind::Array { element, .. }
                | Kind::ArrayTerm { element, .. }
                | Kind::Slice(element) => work.push(element),
                Kind::Reference { referent, .. } if references => work.push(referent),
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
