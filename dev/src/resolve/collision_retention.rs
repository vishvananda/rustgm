//! Inherent collisions retain distinct declarations for typed owner coherence.
use super::{
    index::Resolver,
    model::{DefKind, Definition, Namespace, Origin},
    package::Error,
    table::BindingKey,
    unit::FileId,
};
use crate::syntax::{ast::Kind, item::ItemKind};

impl Resolver<'_> {
    /// Resolution cannot decide whether an impl's actual target is structural.
    /// Preserve every member ID without replacing the existing lexical binding.
    pub(super) fn retains_inherent_collision(
        &mut self,
        definition: &Definition,
        key: BindingKey,
    ) -> Result<bool, Error> {
        let scope = &self.facts.scopes[definition.scope.0 as usize];
        if !scope.associated
            || scope.trait_access.is_some()
            || definition.namespace != Namespace::Value
            || !matches!(definition.kind, DefKind::Function | DefKind::Const)
        {
            return Ok(false);
        }
        let Some(binding) = self.facts.bindings.get_key(key) else {
            return Ok(false);
        };
        let previous = &self.facts.definitions[binding.target.0 as usize];
        if binding.origin != Origin::Direct
            || binding.ambiguous
            || previous.scope != definition.scope
            || !matches!(previous.kind, DefKind::Function | DefKind::Const)
        {
            return Ok(false);
        }
        let self_name = self
            .package
            .names
            .intern("Self")
            .map_err(|message| Error::at(FileId(0), 0, message))?;
        let Some(binding) = self
            .facts
            .bindings
            .get(definition.scope, self_name, Namespace::Type)
        else {
            return Ok(false);
        };
        let owner = &self.facts.definitions[binding.target.0 as usize];
        if binding.origin != Origin::Direct
            || binding.ambiguous
            || owner.kind != DefKind::SelfType
            || owner.scope != definition.scope
            || owner.members != Some(definition.scope)
        {
            return Ok(false);
        }
        let Some(node) = owner.node else {
            return Ok(false);
        };
        let ast = &self.package.unit(node.file).configured.ast;
        Ok(matches!(
            ast.node(node.node).kind,
            Kind::Item(index)
                if matches!(ast.items[index as usize].kind, ItemKind::Impl { trait_path: None, .. })
        ))
    }
}
