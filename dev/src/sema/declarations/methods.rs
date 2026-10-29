//! Associated owners share the ordinary generic and lifetime declaration worklist.
use super::{BinderId, Checker, DefId, Error, NodeKey, Type, TypeId, regions};
use crate::{resolve::model::DefKind, sema::associated::OwnerKind};

impl Checker<'_> {
    fn actual_method_receiver(
        &self,
        function: &crate::sema::model::Function,
    ) -> Option<(NodeKey, crate::sema::associated::Receiver, TypeId)> {
        let target = self.facts.associated.schema(function.definition)?.self_type;
        let unit = self.report.package.unit(function.node.file);
        let node = unit
            .configured
            .ast
            .list(function.parameters)
            .iter()
            .copied()
            .find(|node| unit.configured.active[node.get() as usize])?;
        let key = self.unattributed(NodeKey {
            node,
            ..function.node
        });
        Some((key, *self.facts.receivers.get(&key)?, target))
    }
    pub(super) fn publish_exact_method_receiver(
        &mut self,
        function: &crate::sema::model::Function,
    ) -> Result<(), Error> {
        if self
            .actual_method_receiver(function)
            .is_some_and(|(_, receiver, target)| receiver.base == target)
        {
            self.validate_method_receiver(function)?;
        }
        Ok(())
    }
    pub(super) fn validate_method_receiver(
        &mut self,
        function: &crate::sema::model::Function,
    ) -> Result<(), Error> {
        let Some((key, receiver, target)) = self.actual_method_receiver(function) else {
            return Ok(());
        };
        if receiver.base != target {
            self.validate_distinct_receiver(function, key, receiver.base, target)?;
        }
        self.facts
            .methods
            .receivers
            .insert(function.definition, (receiver.ty, receiver.references));
        Ok(())
    }
    fn validate_distinct_receiver(
        &self,
        function: &crate::sema::model::Function,
        key: NodeKey,
        base: TypeId,
        target: TypeId,
    ) -> Result<(), Error> {
        let error = |e| self.error(key, format!("receiver type does not match Self: {e}"));
        let lifetimes = self.facts.lifetimes.functions.get(&function.definition);
        let mut constraints = crate::types::relations::Constraints::new(
            &self.facts.types,
            &self.facts.lifetimes.variances,
            lifetimes.map_or(0, |s| s.total),
            1_000_000,
        )
        .map_err(error)?;
        for parameter in self.facts.generics.parameters_in_scope(function.definition) {
            if let crate::types::arguments::Argument::Type(ty) = parameter.argument
                && let Some(Type::Parameter(id)) = self.facts.types.kind(ty)
            {
                constraints.parameter(id).map_err(error)?;
            }
        }
        for &(a, b) in lifetimes.into_iter().flat_map(|s| &s.bounds) {
            constraints.assume(a, b).map_err(error)?;
        }
        let mut queries = crate::types::outlives::Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        constraints
            .assume_signature(&mut queries, function.signature)
            .map_err(|e| self.error(key, e))?;
        constraints
            .assume_outlives_clauses(
                &mut queries,
                self.facts
                    .generics
                    .predicates_in_scope(function.definition)
                    .map(|p| p.value),
            )
            .map_err(|e| self.error(key, e))?;
        constraints.coerce(base, target, false, 0).map_err(error)?;
        constraints.coerce(target, base, false, 0).map_err(error)?;
        constraints.solve().map_err(error)
    }
    pub(super) fn prepare_inherent_owners(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            if matches!(
                owner.kind,
                OwnerKind::Impl {
                    trait_type: None,
                    ..
                }
            ) {
                self.work.push(super::Work::ImplHeader(
                    owner.self_definition,
                    super::impl_headers::Phase::Begin,
                ));
                self.drain(steps, limit)?;
            }
        }
        Ok(())
    }

    /// Resolved ownership does not imply that the owner's environment is ready.
    pub(super) fn reserve_member_parents(&mut self) -> Result<(), Error> {
        for member in self.facts.associated.members() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(member.owner.0 as usize)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            if matches!(
                self.report.resolution.definitions[member.definition.0 as usize].kind,
                DefKind::Function | DefKind::Const | DefKind::TypeAlias
            ) {
                self.facts.generics.parents.insert(
                    member.definition,
                    self.facts
                        .associated
                        .trait_owner(member.definition)
                        .unwrap_or(owner.self_definition),
                );
            }
        }
        Ok(())
    }
    pub(super) fn signature_binder(
        &mut self,
        owner: Option<DefId>,
        parent: Option<BinderId>,
        declared: Vec<DefId>,
    ) -> BinderId {
        let source = owner
            .and_then(|id| self.facts.associated.owner(id))
            .and_then(|id| self.facts.associated.get(id))
            .and_then(|owner| self.owner_binders.get(&owner.self_definition));
        let binder = if let Some(&source) = source {
            self.regions.capture(source, declared)
        } else {
            self.regions.binder(parent, declared)
        };
        if let Some(owner) = owner {
            self.regions.associate(binder, owner);
        }
        binder
    }
    pub(super) fn associated_self_type(
        &mut self,
        key: NodeKey,
        id: DefId,
        context: regions::Context,
    ) -> Result<TypeId, Error> {
        let schema = self
            .facts
            .associated
            .schema(id)
            .ok_or_else(|| self.error(key, "missing associated Self type"))?;
        self.lexical_self_type(key, id, context, schema.self_type, schema.captures)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/early_receiver_proofs.rs"]
mod receiver_tests;
