//! Value-form Self retains the impl application while selecting a real constructor.
use super::{Checker, DefId, DefKind, Error, NodeKey, Target, Type, Work, regions};
use crate::{
    resolve::model::{Access, Namespace},
    sema::{associated::OwnerKind, generics::ConstructorUse},
};

impl Checker<'_> {
    pub(super) fn schedule_self_constructor(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<bool, Error> {
        let Some(Target::SelfConstructor(owner) | Target::InheritedSelfConstructor(owner)) =
            self.facts.node(key).and_then(|f| f.reference)
        else {
            return Ok(false);
        };
        let declaration = self
            .facts
            .associated
            .owner(owner)
            .and_then(|id| self.facts.associated.get(id))
            .ok_or_else(|| self.error(key, "Self constructor requires an implementation"))?;
        if !matches!(declaration.kind, OwnerKind::Impl { .. }) {
            return Err(self.error(key, "trait Self has no concrete constructor"));
        }
        self.work.push(Work::SelfConstructor(key, owner, context));
        self.work
            .push(Work::ImplHeader(owner, super::impl_headers::Phase::Begin));
        Ok(true)
    }

    pub(super) fn complete_self_constructor(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        let inherited = matches!(
            self.facts.node(key).and_then(|f| f.reference),
            Some(Target::InheritedSelfConstructor(_))
        );
        let ty = if inherited {
            if crate::sema::lifetimes::arguments(self.report, key, false)?.is_some() {
                return Err(self.error(key, "Self cannot take generic arguments"));
            }
            let ty = self
                .facts
                .associated
                .schema(owner)
                .ok_or_else(|| self.error(key, "missing inherited Self type"))?
                .self_type;
            if self.facts.types.contains_parameters(ty) != Some(false)
                || self.facts.types.has_free_regions(ty, 0) != Some(false)
            {
                return Err(self.error(
                    key,
                    "Self constructor cannot capture outer implementation parameters",
                ));
            }
            ty
        } else {
            self.associated_self_type(key, owner, context)?
        };
        let Some(Type::Adt(item, _)) = self.facts.types.kind(ty) else {
            return Err(self.error(key, "Self value requires a tuple or unit struct"));
        };
        let definition = &self.report.resolution.definitions[item.0 as usize];
        if definition.kind != DefKind::Struct {
            return Err(self.error(key, "Self value requires a tuple or unit struct"));
        }
        let binding = self
            .report
            .resolution
            .bindings
            .get(definition.scope, definition.name, Namespace::Value)
            .ok_or_else(|| self.error(key, "Self value requires a tuple or unit constructor"))?;
        if self.report.resolution.definitions[binding.target.0 as usize].kind
            != DefKind::Constructor
        {
            return Err(self.error(key, "Self value does not name its struct constructor"));
        }
        let module = self.report.resolution.scopes
            [self.report.resolution.definitions[owner.0 as usize].scope.0 as usize]
            .module;
        if binding.access == Access::ExternalPrivate
            || matches!(binding.access, Access::Within(ancestor)
                if !self.report.package.is_descendant(module, ancestor))
        {
            return Err(self.error(key, "Self constructor is private"));
        }
        let target = binding.target;
        self.constructor(target)?;
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing Self constructor fact"))?
            .reference = Some(Target::Definition(target));
        self.facts
            .generics
            .constructor_uses
            .insert(key, ConstructorUse::SelfType(ty));
        Ok(())
    }
}
