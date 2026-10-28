//! Item lookup follows exact identifiers and actual macro definition scopes.
use super::{
    index::{Context, Resolver},
    lookup::{BindingKey, Observations},
    model::{Binding, ItemAdjustment, Namespace, ScopeId},
    names::NameId,
};
use crate::expand::marks::ItemContext;

pub(super) struct Selected {
    pub key: BindingKey,
    pub binding: Binding,
    pub privacy: super::package::ModuleId,
}

#[cfg(test)]
#[path = "../../tests/invariants/contextual_lookup.rs"]
mod tests;

impl Resolver<'_> {
    /// `self` and each `super` identifier follow their own retained context.
    pub(super) fn self_module(
        &self,
        mut module: super::package::ModuleId,
        context: &mut ItemContext,
    ) -> Result<super::package::ModuleId, &'static str> {
        let mut remaining = self
            .package
            .marks
            .context_count()
            .saturating_mul(self.package.modules.len() + 1);
        loop {
            remaining = remaining
                .checked_sub(1)
                .ok_or("module context work budget exceeded")?;
            let item = self
                .package
                .modules
                .get(module.0 as usize)
                .ok_or("unknown path module")?;
            let own = if let Some(key) = item.declaration {
                let span = self
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .node(key.node)
                    .span;
                self.package
                    .item_context(key.file, span.start)
                    .map_err(|_| "invalid module token context")?
            } else {
                ItemContext::ROOT
            };
            if own == *context {
                return Ok(module);
            }
            if let Some(parent) = item.parent {
                module = parent;
            } else {
                let (parent, expansion) = self
                    .package
                    .marks
                    .item_parent(*context)?
                    .ok_or("module context cannot reach its definition")?;
                let scope = self.facts.introductions.definition_scope(expansion)?;
                module = self
                    .facts
                    .scopes
                    .get(scope.0 as usize)
                    .ok_or("unknown macro definition scope")?
                    .module;
                *context = parent;
            }
        }
    }

    pub(super) fn scope_adjustment(
        &self,
        scope: ScopeId,
        context: ItemContext,
        caller: super::package::ModuleId,
    ) -> Result<ItemAdjustment, &'static str> {
        self.facts
            .scopes
            .get(scope.0 as usize)
            .ok_or("unknown item lookup scope")?;
        let introductions = &self.facts.introductions;
        let adjusted = self.package.marks.adjust_item(
            context,
            introductions.scope(scope),
            &introductions.ancestry,
        )?;
        let privacy = if let Some(expansion) = adjusted.definition {
            let scope = introductions.definition_scope(expansion)?;
            self.facts
                .scopes
                .get(scope.0 as usize)
                .ok_or("unknown macro definition scope")?
                .module
        } else {
            caller
        };
        Ok(ItemAdjustment {
            context: adjusted.context,
            privacy,
        })
    }

    pub(super) fn item_in_scope(
        &self,
        c: Context,
        key: BindingKey,
        observed: Option<&mut Observations>,
    ) -> Result<Option<Selected>, &'static str> {
        let (scope, name, namespace, context) = key;
        let adjusted = self.scope_adjustment(scope, context, c.module)?;
        let key = (scope, name, namespace, adjusted.context);
        if let Some(observed) = observed {
            observed.dependency(key);
        }
        Ok(self
            .facts
            .bindings
            .get_key(key)
            .copied()
            .map(|binding| Selected {
                key,
                binding,
                privacy: adjusted.privacy,
            }))
    }

    pub(super) fn item_lexical(
        &self,
        c: Context,
        name: NameId,
        namespace: Namespace,
        context: ItemContext,
        observed: Option<&mut Observations>,
    ) -> Result<Option<Selected>, &'static str> {
        self.item_lexical_filtered(c, (name, namespace, context), observed, None)
    }

    pub(super) fn item_lexical_filtered(
        &self,
        c: Context,
        requested: (NameId, Namespace, ItemContext),
        mut observed: Option<&mut Observations>,
        filter: Option<super::lookup::MacroFilter<'_>>,
    ) -> Result<Option<Selected>, &'static str> {
        let (name, namespace, mut context) = requested;
        let mut current = Some(c.scope);
        if namespace == Namespace::Type && self.package.names.get(name) == "Self" {
            context = ItemContext::ROOT;
        }
        let parameter_context = context;
        let mut privacy = c.module;
        let mut barriers = 0_u32;
        let mut remaining = self
            .package
            .marks
            .context_count()
            .saturating_mul(self.facts.scopes.len() + 1);
        while let Some(scope) = current {
            remaining = remaining
                .checked_sub(1)
                .ok_or("item lookup work budget exceeded")?;
            let frame = self
                .facts
                .scopes
                .get(scope.0 as usize)
                .ok_or("unknown item lookup scope")?;
            let key = (scope, name, namespace, context);
            if let Some(observed) = observed.as_deref_mut() {
                observed.dependency(key);
            }
            if let Some(target) = frame.nominal_self
                && barriers == 0
                && namespace == Namespace::Type
                && self.package.names.get(name) == "Self"
            {
                return Ok(Some(Selected {
                    key,
                    binding: Binding {
                        target,
                        access: self.facts.definitions[target.0 as usize].access,
                        origin: super::model::Origin::Direct,
                        ambiguous: false,
                    },
                    privacy,
                }));
            }
            if let Some(binding) = self.facts.bindings.get_key(key)
                && filter.is_none_or(|filter| filter.permits(binding.target))
                && binding.permits_lexical_lookup(
                    &self.facts.definitions,
                    frame,
                    barriers,
                    context == parameter_context,
                )
            {
                return Ok(Some(Selected {
                    key,
                    binding: *binding,
                    privacy,
                }));
            }
            if namespace == Namespace::Label && frame.label_barrier {
                return Ok(None);
            }
            if let Some((parent, expansion)) = self.package.marks.item_parent(context)? {
                let definition = self.facts.introductions.definition_scope(expansion)?;
                if definition == scope
                    || (frame.parent.is_none()
                        && !self
                            .facts
                            .introductions
                            .ancestor(expansion, self.facts.introductions.scope(scope)))
                {
                    privacy = self
                        .facts
                        .scopes
                        .get(definition.0 as usize)
                        .ok_or("unknown macro definition scope")?
                        .module;
                    context = parent;
                    current = Some(definition);
                    continue;
                }
            }
            barriers += u32::from(frame.barrier);
            current = frame.parent;
        }
        // Primitive and library preludes have no contextual names. Reuse only
        // the existing terminal fallback, including inside a generated module
        // whose item context remains opaque at its lexical boundary.
        Ok(self
            .lexical_prelude(privacy, name, namespace)
            .filter(|(_, binding)| filter.is_none_or(|filter| filter.permits(binding.target)))
            .map(|(scope, binding)| Selected {
                key: (scope, name, namespace, ItemContext::ROOT),
                binding,
                privacy,
            }))
    }
}
