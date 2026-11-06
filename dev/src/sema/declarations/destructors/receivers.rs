//! Drop signatures retain their independent receiver lifetime and restricted method bounds.
use super::super::{BinderId, Checker, DefId, Error, List, NodeKey, Type, TypeId};
use crate::{sema::lifetimes::FunctionRegions, types::outlives::Pair};
impl Checker<'_> {
    pub(in crate::sema::declarations) fn drop_method_regions(
        &mut self,
        key: NodeKey,
        owner: DefId,
        binder: BinderId,
        declared: List,
        signature: TypeId,
        bounds: &[Pair],
    ) -> Result<(), Error> {
        let Some(Type::Function {
            parameters,
            result: TypeId::UNIT,
            unsafe_: false,
            ..
        }) = self.facts.types.kind(signature)
        else {
            return Err(self.error(key, "Drop method must return unit"));
        };
        let Some(&receiver) = self.facts.types.list(parameters).and_then(|p| p.first()) else {
            return Err(self.error(key, "missing typed Drop receiver"));
        };
        let receiver = self.drop_receiver(key, owner, receiver)?;
        let Some(Type::Reference { region, .. }) = self.facts.types.kind(receiver) else {
            return Err(self.error(key, "invalid typed Drop receiver"));
        };
        let declared = self.declared_regions(key, declared)?;
        let mut method_slots = crate::support::hash::HashSet::default();
        for &definition in &declared {
            method_slots.insert(
                self.regions
                    .named(Some(binder), definition)
                    .map_err(|e| self.error(key, e))?,
            );
        }
        if bounds
            .iter()
            .any(|(a, b)| method_slots.contains(a) || method_slots.contains(b))
            || self
                .facts
                .generics
                .bound_lifetimes(&self.facts.types, &self.facts.unreduced, owner)
                .map_err(|e| self.error(key, e))?
                .iter()
                .any(|&slot| {
                    crate::types::RegionId::bound(0, slot)
                        .is_ok_and(|region| method_slots.contains(&region))
                })
        {
            return Err(self.error(
                key,
                "Drop method cannot introduce early-bound lifetime parameters",
            ));
        }
        let (_, receiver_slot) = region
            .bound_parts()
            .ok_or_else(|| self.error(key, "invalid Drop receiver region"))?;
        self.facts.destructors.extend_bounds(owner, bounds);
        let implementation = self
            .facts
            .destructors
            .method(owner)
            .ok_or_else(|| self.error(key, "missing Drop contract"))?;
        let bounds = implementation.bounds.clone();
        let captures = self.source_drop_captures(key, owner)?;
        if u32::from(receiver_slot) < captures {
            return Err(self.error(
                key,
                "source Drop receiver requires an independent late lifetime",
            ));
        }
        let scheme = FunctionRegions {
            total: self.regions.count(binder),
            captures,
            early: Vec::new(),
            late: vec![receiver_slot],
            has_declared_late: !declared.is_empty(),
            bounds,
        };
        if self.facts.source_bindings.is_some() {
            self.callable_template(key, owner, signature, &scheme)?;
        }
        self.facts.lifetimes.functions.insert(owner, scheme);
        self.function_binders.insert(key, binder);
        Ok(())
    }
    pub(in crate::sema::declarations) fn drop_receiver(
        &self,
        key: NodeKey,
        method: DefId,
        receiver: TypeId,
    ) -> Result<TypeId, Error> {
        let definition = self
            .report
            .resolution
            .definitions
            .get(method.0 as usize)
            .ok_or_else(|| self.error(key, "Drop method lost its actual definition"))?;
        if self.report.package.names.get(definition.name) != "pin_drop" {
            return Ok(receiver);
        }
        let pin = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(crate::support::bindings::LangItem::Pin))
            .ok_or_else(|| self.error(key, "pinned destructor requires its actual source Pin"))?;
        let Some(Type::Adt(item, arguments)) = self.facts.types.kind(receiver) else {
            return Err(self.error(key, "pinned destructor requires its actual Pin application"));
        };
        let mut arguments = self
            .facts
            .types
            .arguments(arguments)
            .ok_or_else(|| self.error(key, "pinned destructor has invalid Pin arguments"))?;
        if item.0 != pin.definition.0 || arguments.len() != 1 {
            return Err(self.error(key, "pinned destructor changed its actual Pin owner"));
        }
        let Some(crate::types::arguments::Argument::Type(pointer)) = arguments.next() else {
            return Err(self.error(key, "pinned destructor requires its actual pointer input"));
        };
        let target = self
            .facts
            .destructors
            .method(method)
            .ok_or_else(|| self.error(key, "pinned destructor lost its registered nominal target"))?
            .target;
        if !matches!(self.facts.types.kind(pointer), Some(Type::Reference { mutable:true, referent, .. }) if referent==target)
        {
            return Err(self.error(
                key,
                "pinned destructor requires its exact unique Self borrow",
            ));
        }
        Ok(pointer)
    }
    /// Bootstrap cleanup supplies its own flattened argument mapping. An actual
    /// source Drop method also participates in ordinary trait conformance and
    /// must preserve its impl captures independently from the receiver borrow.
    fn source_drop_captures(&self, key: NodeKey, method: DefId) -> Result<u32, Error> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(0);
        };
        let role = catalog
            .validated_language(crate::support::bindings::LangItem::Drop)
            .map_err(|e| self.error(key, e))?
            .ok_or_else(|| {
                self.error(key, "registered source destructor has no actual Drop role")
            })?;
        let implementation = self
            .facts
            .destructors
            .method(method)
            .ok_or_else(|| self.error(key, "missing registered source Drop method"))?;
        let owner = self
            .facts
            .associated
            .owner(method)
            .and_then(|id| self.facts.associated.get(id))
            .ok_or_else(|| self.error(key, "registered source Drop method has no owner"))?;
        let schema = self
            .facts
            .associated
            .schema(method)
            .ok_or_else(|| self.error(key, "registered source Drop method has no schema"))?;
        let header = self
            .facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| registry.header(implementation.self_definition))
            .ok_or_else(|| {
                self.error(key, "registered source Drop requires its ordinary header")
            })?;
        let Some((application, constness)) = header.predicate.bound.trait_parts() else {
            return Err(self.error(
                key,
                "registered source Drop header lost its trait application",
            ));
        };
        let crate::sema::associated::OwnerKind::Impl { constant, .. } = owner.kind else {
            return Err(self.error(key, "registered source Drop lost its actual impl owner"));
        };
        let expected = if constant {
            crate::support::constness::Constness::Const
        } else {
            crate::support::constness::Constness::Ordinary
        };
        if constness != expected {
            return Err(self.error(
                key,
                "registered source Drop header changed its actual owner constness",
            ));
        }
        let application = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(key, "registered source Drop application is missing"))?;
        if owner.self_definition != implementation.self_definition
            || header.owner != implementation.self_definition
            || schema.self_type != implementation.target
            || schema.captures != implementation.parameters
            || application.item.0 != role.definition.0
            || self
                .facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err(self.error(
                key,
                "registered source Drop capture contract changed its owner",
            ));
        }
        Ok(implementation.parameters)
    }
}
