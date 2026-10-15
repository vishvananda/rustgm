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
        let mut method_slots = std::collections::HashSet::new();
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
        self.facts.lifetimes.functions.insert(
            owner,
            FunctionRegions {
                total: self.regions.count(binder),
                captures: 0,
                early: Vec::new(),
                late: vec![receiver_slot],
                has_declared_late: !declared.is_empty(),
                bounds: implementation.bounds.clone(),
            },
        );
        self.function_binders.insert(key, binder);
        Ok(())
    }
}
