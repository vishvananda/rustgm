//! Storage heads share complete capture substitution without rewriting callable identity.
use super::{StorageFailure, applications::Preparation};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    sema::model::Declarations,
    types::{Kind, TypeId},
};

/// Pending hidden types by application, the roots in order, and the first
/// failure bring-up set aside.
type Applications = (HashMap<TypeId, TypeId>, Vec<TypeId>, Option<StorageFailure>);

#[cfg(test)]
#[path = "../../../tests/invariants/undeclared_opaque_storage.rs"]
mod undeclared_tests;

impl Declarations {
    /// Prepare appended canonical applications once, independently of caller proofs.
    /// Bodies still pass ordinary region and ownership verification before execution.
    pub(crate) fn prepare_opaque_storage(&mut self) -> Result<(), &'static str> {
        self.prepare_opaque_storage_located()
            .map_err(|failure| failure.message)
    }
    /// Retain only the exact operation's diagnostic owner; never infer proof authority.
    pub(crate) fn prepare_opaque_storage_located(&mut self) -> Result<(), StorageFailure> {
        if self.opaques.declarations.is_empty() {
            return Ok(());
        }
        let mut preparation = self.opaques.storage_preparation.take().unwrap_or_default();
        let result = match preparation.failed {
            Some(error) => Err(error),
            None => self.prepare_opaque_heads(&mut preparation),
        };
        preparation.failed = result.as_ref().err().copied();
        self.opaques.storage_preparation = Some(preparation);
        result
    }
    fn prepare_opaque_heads(
        &mut self,
        preparation: &mut Preparation,
    ) -> Result<(), StorageFailure> {
        // Bring-up reports every root whose storage fails, then still fails.
        let (mut pending, roots, mut deferred) = self.collect_opaque_applications(preparation)?;
        let mut path = Vec::new();
        let mut active = HashSet::default();
        for &root in &roots {
            if !pending.contains_key(&root) {
                continue;
            }
            let result = (|| {
                let mut current = root;
                while let Some(&hidden) = pending.get(&current) {
                    preparation.charge(1)?;
                    if !active.insert(current) {
                        return Err("recursive opaque hidden type");
                    }
                    path.push(current);
                    current = hidden;
                }
                let head = self.types.storage_head(current)?;
                for ty in path.drain(..) {
                    self.types.publish_storage_head(ty, head)?;
                    pending.remove(&ty);
                }
                active.clear();
                Ok(())
            })();
            if let Err(message) = result {
                let failure = self.opaque_storage_failure(root, message);
                if !self.defer_storage_failure(failure) {
                    return Err(failure);
                }
                path.clear();
                active.clear();
                deferred.get_or_insert(failure);
            }
        }
        for root in roots {
            let result = (|| {
                let steps = preparation.layouts.steps;
                let result = preparation
                    .layouts
                    .demand(
                        &mut self.types,
                        &mut self.aggregates,
                        &mut self.layouts,
                        root,
                    )
                    .map_err(|e| e.message);
                preparation.charge(preparation.layouts.steps.saturating_sub(steps))?;
                result?;
                Ok(())
            })();
            if let Err(message) = result {
                let failure = self.opaque_storage_failure(root, message);
                if !self.defer_storage_failure(failure) {
                    return Err(failure);
                }
                deferred.get_or_insert(failure);
            }
        }
        deferred.map_or(Ok(()), Err)
    }
    /// Set a failed root aside at the function that defines its hidden type.
    /// Refused outside the bring-up profile and for a root without one.
    fn defer_storage_failure(&self, failure: StorageFailure) -> bool {
        let Some(function) = failure
            .owner
            .and_then(|owner| self.function_indices.get(&owner))
            .and_then(|&index| self.functions.get(index))
        else {
            return false;
        };
        crate::support::bringup::defer(
            crate::support::bringup::Stage::Opaques,
            function.node.file.0,
            crate::support::bringup::Position::Node(function.node.node.get()),
            failure.message,
        )
    }
    fn collect_opaque_applications(
        &mut self,
        preparation: &mut Preparation,
    ) -> Result<Applications, StorageFailure> {
        let mut pending = HashMap::default();
        let mut roots = Vec::new();
        let mut deferred = None;
        let replay = std::mem::take(&mut preparation.ready);
        let replay_capacity = replay.capacity();
        let mut replay = replay.into_iter();
        loop {
            let ty = if let Some(ty) = replay.next() {
                ty
            } else if self.opaques.storage_cursor < self.types.len() {
                let index = self.opaques.storage_cursor;
                self.opaques.storage_cursor += 1;
                TypeId(u32::try_from(index).map_err(|_| "too many storage types")?)
            } else {
                break;
            };
            preparation.charge(1)?;
            let Some(Kind::Opaque(id, arguments)) = self.types.kind(ty) else {
                continue;
            };
            let result = (|| {
                // An application is interned before its declaration is published, and
                // that declaration may wait for its clauses or fail. Either way it has
                // no hidden type yet: it waits for its own identity, like a declared
                // opaque whose defining body is unchecked, and never blames this caller.
                let Some((declaration, hidden)) =
                    self.opaques.declarations.get(&id).and_then(|declaration| {
                        declaration.hidden.map(|hidden| (declaration, hidden))
                    })
                else {
                    preparation.wait(id, ty)?;
                    return Ok(None);
                };
                preparation.scratch(roots.len().saturating_add(1), replay_capacity)?;
                let mut substitution = declaration.substitution(&self.types, arguments)?;
                let hidden = substitution.apply(&mut self.types, hidden)?;
                preparation.charge(substitution.steps)?;
                let hidden = self.normalized_hidden(preparation, hidden)?;
                self.aggregates.prepare(&mut self.types, hidden)?;
                self.hidden_storage(preparation, hidden)?;
                Ok(Some(hidden))
            })();
            let hidden = match result {
                Ok(Some(hidden)) => hidden,
                Ok(None) => continue,
                Err(message) => {
                    let failure = self.opaque_storage_failure(ty, message);
                    if !self.defer_storage_failure(failure) {
                        return Err(failure);
                    }
                    deferred.get_or_insert(failure);
                    continue;
                }
            };
            pending.insert(ty, hidden);
            roots.push(ty);
        }
        Ok((pending, roots, deferred))
    }
    /// A concrete hidden type is stored as rustc reveals it, normalized: a
    /// returned closure capturing `&mut Option<T::IntoIter>` stores
    /// `&mut Option<I>`, the type its caller's checked execution also has.
    /// Users still prove that normalization; storage grants no permission.
    /// The disposable proof is numbered past the hidden type's own lifetimes,
    /// which an instance's caller opened.
    fn normalized_hidden(
        &mut self,
        preparation: &mut Preparation,
        hidden: TypeId,
    ) -> Result<TypeId, &'static str> {
        use crate::types::lifetime_proof::{Obligations, ScopeId};
        if self.types.contains_projections(hidden) != Some(true)
            || self.types.contains_parameters(hidden) != Some(false)
        {
            return Ok(hidden);
        }
        let first = self.probe_first(preparation, hidden)?;
        let mut proof = Obligations::new(first);
        let mut session = crate::sema::projections::concrete(&mut proof, ScopeId::ROOT)?;
        let (normalized, steps) = session.normalize(self, hidden)?;
        preparation.charge(steps)?;
        Ok(normalized)
    }
    /// A caller stores an opaque value as its hidden type. Instantiated at
    /// the caller's lifetimes, a concrete hidden type is a different type
    /// from the one its defining body prepared, so its projected fields, as
    /// `Option<U::IntoIter>` in a returned `FlatMap`, need their normal
    /// storage here. The probe's lifetime facts are dropped: the defining
    /// body proves its hidden type, and field readiness grants nothing.
    fn hidden_storage(
        &mut self,
        preparation: &mut Preparation,
        hidden: TypeId,
    ) -> Result<(), &'static str> {
        use crate::types::lifetime_proof::{Obligations, ScopeId};
        if !self.aggregates.has_projected_templates()
            || self.types.contains_parameters(hidden) != Some(false)
            || self.types.contains_projections(hidden) != Some(false)
        {
            return Ok(());
        }
        let first = self.probe_first(preparation, hidden)?;
        let before = self.stored_field_work().steps;
        let mut proof = Obligations::new(first);
        let mut session = crate::sema::projections::concrete(&mut proof, ScopeId::ROOT)?;
        let mut storage = crate::sema::stored_fields::Preparation::default();
        let result = storage.prepare(self, hidden, &mut session);
        preparation.charge(
            self.stored_field_work()
                .steps
                .saturating_sub(before)
                .saturating_add(storage.steps())
                .saturating_add(session.nested_work()),
        )?;
        result
    }
    /// A probe's first lifetime identity. Disposable probes start beyond every
    /// numbered unit, but a caller instance's opened lifetimes, which its
    /// opaque applications carry, can lie beyond that frontier too.
    fn probe_first(
        &mut self,
        preparation: &mut Preparation,
        ty: TypeId,
    ) -> Result<u32, &'static str> {
        let mut observe = crate::types::substitute::Bound::observe_regions();
        let observed = observe.apply(&mut self.types, ty);
        preparation.charge(observe.steps)?;
        observed?;
        observe
            .observed_regions()
            .filter(|region| region.bound_parts().is_none())
            .try_fold(self.body_regions.probe_frontier()?, |first, region| {
                region
                    .0
                    .checked_add(1)
                    .map(|next| first.max(next))
                    .ok_or("opaque storage probe lifetime identity space exhausted")
            })
    }
    fn opaque_storage_failure(&self, ty: TypeId, message: &'static str) -> StorageFailure {
        let owner = match self.types.kind(ty) {
            Some(Kind::Opaque(id, _)) => self.opaque_defining_owner(id),
            _ => None,
        };
        StorageFailure { owner, message }
    }
}
