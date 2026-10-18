//! Associated owners share the ordinary generic and lifetime declaration worklist.
use super::{BinderId, Checker, DefId, Error, NodeKey, Type, TypeId, regions};
use crate::{resolve::model::DefKind, sema::associated::OwnerKind};

impl Checker<'_> {
    pub(super) fn validate_method_receiver(
        &mut self,
        function: &crate::sema::model::Function,
    ) -> Result<(), Error> {
        let Some(schema) = self.facts.associated.schema(function.definition) else {
            return Ok(());
        };
        let unit = self.report.package.unit(function.node.file);
        let Some(node) = unit
            .configured
            .ast
            .list(function.parameters)
            .iter()
            .copied()
            .find(|node| unit.configured.active[node.0 as usize])
        else {
            return Ok(());
        };
        let key = self.unattributed(NodeKey {
            node,
            ..function.node
        });
        let Some(receiver) = self.facts.receivers.get(&key) else {
            return Ok(());
        };
        self.facts
            .methods
            .receivers
            .insert(function.definition, (receiver.ty, receiver.references));
        if receiver.base == schema.self_type {
            return Ok(());
        }
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
        for predicate in self.facts.generics.predicates_in_scope(function.definition) {
            if let crate::types::parameters::predicates::Bound::Outlives(shorter) =
                predicate.value.bound
            {
                constraints
                    .assume_type_outlives(&mut queries, predicate.value.ty, shorter)
                    .map_err(|e| self.error(key, e))?;
            }
        }
        constraints
            .coerce(receiver.base, schema.self_type, false, 0)
            .map_err(error)?;
        constraints
            .coerce(schema.self_type, receiver.base, false, 0)
            .map_err(error)?;
        constraints.solve().map_err(error)?;
        Ok(())
    }
    pub(super) fn prepare_inherent_owners(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        for index in 0..self.facts.associated.owners().len() {
            let owner = self.facts.associated.owners()[index];
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
    pub(super) fn reserve_member_parents(&mut self) {
        for member in self.facts.associated.members() {
            let owner = self.facts.associated.owners()[member.owner.0 as usize];
            if matches!(
                self.report.resolution.definitions[member.definition.0 as usize].kind,
                DefKind::Function | DefKind::Const
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
        if crate::sema::lifetimes::arguments(self.report, key, false)?.is_some() {
            return Err(self.error(key, "Self cannot take lifetime arguments"));
        }
        let schema = self
            .facts
            .associated
            .schema(id)
            .ok_or_else(|| self.error(key, "missing associated Self type"))?;
        let target = schema.self_type;
        let binder = self.owner_binders[&id];
        let mut mapping = Vec::with_capacity(schema.captures as usize);
        for index in 0..schema.captures {
            let slot = u16::try_from(index)
                .map_err(|_| self.error(key, "too many impl lifetime slots"))?;
            mapping.push(
                self.regions
                    .captured_region(context.binder, binder, slot)
                    .map_err(|e| self.error(key, e))?,
            );
        }
        let args = self
            .facts
            .types
            .intern_regions(&mapping)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .lifetimes
            .substitute(&mut self.facts.types, args, target)
            .map_err(|e| self.error(key, e))
    }
}
