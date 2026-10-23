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
        let Some((application, crate::support::constness::Constness::Ordinary)) =
            header.predicate.bound.trait_parts()
        else {
            return Err(self.error(
                key,
                "registered source Drop header lost its trait application",
            ));
        };
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
