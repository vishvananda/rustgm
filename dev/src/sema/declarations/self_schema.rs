//! Rigid lexical Self preparation is independent of trait/impl registry ownership.
use super::{Checker, DefId, Error, ItemId, NodeKey, Type};
use crate::{
    sema::{associated::Schema, generics::Parameter, model::Declarations},
    types::{
        arguments::Argument,
        parameters::{Parameter as TypeParameter, ParameterKind},
    },
};
impl Checker<'_> {
    /// The actual declaration owner installs its schema before Self becomes complete.
    /// Installing an alias schema must not manufacture an associated trait owner.
    pub(super) fn reserve_lexical_self(
        &mut self,
        key: NodeKey,
        definition: DefId,
        self_definition: DefId,
        install: impl FnOnce(&mut Declarations, Schema) -> Result<(), &'static str>,
    ) -> Result<(), Error> {
        let declared = self.declared_regions(key, self.item(key)?.generics)?;
        let binder = self.regions.binder(None, declared.clone());
        self.regions.associate(binder, self_definition);
        for &parameter in &declared {
            self.regions
                .named(Some(binder), parameter)
                .map_err(|e| self.error(key, e))?;
        }
        let bounds = self.function_bounds(key, binder)?;
        let parameter = self
            .facts
            .types
            .intern_parameter(TypeParameter {
                owner: ItemId(self_definition.0),
                index: 0,
                kind: ParameterKind::Type { sized: false },
            })
            .map_err(|e| self.error(key, e))?;
        let self_type = self.intern(key, Type::Parameter(parameter))?;
        self.facts.generics.parameters.insert(
            self_definition,
            vec![Parameter {
                anonymous: false,
                binding: self_definition,
                argument: Argument::Type(self_type),
                default: None,
            }],
        );
        self.facts
            .generics
            .parents
            .insert(definition, self_definition);
        install(
            &mut self.facts,
            Schema {
                self_type,
                captures: self.regions.count(binder),
                bounds,
            },
        )
        .map_err(|e| self.error(key, e))?;
        self.owner_binders.insert(self_definition, binder);
        self.complete(self_definition, key, self_type)?;
        self.facts.lifetimes.parameters.insert(definition, declared);
        Ok(())
    }
    /// Capturing a lexical Self schema grants no associated member or trait identity.
    pub(super) fn lexical_self_type(
        &mut self,
        key: NodeKey,
        id: DefId,
        context: super::regions::Context,
        target: crate::types::TypeId,
        captures: u32,
    ) -> Result<crate::types::TypeId, Error> {
        if crate::sema::lifetimes::arguments(self.report, key, false)?.is_some() {
            return Err(self.error(key, "Self cannot take lifetime arguments"));
        }
        let binder = self.owner_binders[&id];
        let mut mapping = Vec::with_capacity(captures as usize);
        for index in 0..captures {
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

#[cfg(test)]
#[path = "../../../tests/invariants/self_formals.rs"]
mod tests;
