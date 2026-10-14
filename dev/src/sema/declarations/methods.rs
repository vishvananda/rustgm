//! Associated owners share the ordinary generic and lifetime declaration worklist.
use super::{BinderId, Checker, DefId, Error, NodeKey, Type, TypeId, regions};
use crate::{
    resolve::model::DefKind,
    sema::associated::{OwnerKind, Schema},
};

impl Checker<'_> {
    pub(super) fn validate_method_receiver(
        &self,
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
            let OwnerKind::Impl {
                target,
                trait_type: None,
                unsafe_,
                negative,
            } = owner.kind
            else {
                continue;
            };
            let key = owner.source;
            if unsafe_ || negative {
                return Err(self.error(key, "inherent impls cannot be unsafe or negative"));
            }
            let declared = self.declared_regions(key, self.item(key)?.generics)?;
            let binder = self.regions.binder(None, declared.clone());
            for parameter in declared {
                self.regions
                    .named(Some(binder), parameter)
                    .map_err(|e| self.error(key, e))?;
            }
            let bounds = self.function_bounds(key, binder)?;
            let context = regions::Context {
                binder: Some(binder),
                position: regions::Position::Impl,
            };
            self.declaration_parameters(owner.self_definition, key, context)?;
            self.drain(steps, limit)?;
            self.node_work(target.file, target.node, context);
            self.drain(steps, limit)?;
            let self_type = self.ty(target)?;
            if !matches!(self.facts.types.kind(self_type), Some(Type::Adt(..))) {
                return Err(self.error(target, "inherent impl requires a local nominal type"));
            }
            self.inherent_parameters(owner.self_definition, key, self_type)?;
            self.facts
                .associated
                .declare(
                    owner.self_definition,
                    Schema {
                        self_type,
                        captures: self.regions.count(binder),
                        bounds,
                    },
                )
                .map_err(|e| self.error(key, e))?;
            self.owner_binders.insert(owner.self_definition, binder);
            self.complete(owner.self_definition, key, self_type)?;
            self.schedule_bounds(owner.self_definition, context)?;
            self.drain(steps, limit)?;
            self.facts
                .generics
                .complete_environment(&self.facts.types, owner.self_definition)
                .map_err(|e| self.error(key, e))?;
        }
        for member in self.facts.associated.members() {
            let owner = self.facts.associated.owners()[member.owner.0 as usize];
            if self
                .facts
                .associated
                .schema(owner.self_definition)
                .is_some()
                && self.report.resolution.definitions[member.definition.0 as usize].kind
                    == DefKind::Function
            {
                self.facts
                    .generics
                    .parents
                    .insert(member.definition, owner.self_definition);
            }
        }
        Ok(())
    }
    fn inherent_parameters(&self, owner: DefId, key: NodeKey, target: TypeId) -> Result<(), Error> {
        let mut uses = crate::types::parameters::uses::Uses::default();
        uses.collect(&self.facts.types, target)
            .map_err(|e| self.error(key, e))?;
        for id in self
            .facts
            .generics
            .parameter_ids(&self.facts.types, owner)
            .map_err(|e| self.error(key, e))?
        {
            if !uses.parameters.contains(&id) && !uses.constants.contains(&id) {
                return Err(self.error(
                    key,
                    "impl type and const parameters must constrain its target",
                ));
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
        if let Some(&source) = source {
            self.regions.capture(source, declared)
        } else {
            self.regions.binder(parent, declared)
        }
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
