//! Anonymous specialization retains the entire enclosing identity, never a signature-only key.
#[cfg(test)]
#[path = "../../../tests/invariants/anonymous_contexts.rs"]
mod tests;
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::{model::Declarations, values},
    types::{Kind, instance::Instance},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Context {
    Callable(Instance),
    Initializer(values::Instance),
    SourceInitializer(values::execution::SourceComputation),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InlineInstance {
    source: NodeKey,
    enclosing: Context,
}

impl InlineInstance {
    /// Select a source-owned template in an exact checked enclosing context.
    /// Binding arity, substitution and every body proof remain provider gates;
    /// this identity carries no execution or const permission.
    /// # Errors
    /// Rejects missing templates, foreign owners and unresolved enclosing arguments.
    pub(crate) fn select(
        facts: &Declarations,
        source: NodeKey,
        enclosing: Context,
    ) -> Result<Self, &'static str> {
        let body = facts
            .inline_constants
            .as_deref()
            .and_then(|table| table.get(source))
            .ok_or("anonymous instance requires a retained typed template")?;
        if body.result.is_none() {
            return Err("anonymous instance result is not ready");
        }
        let closure_owner = facts.closures.get(body.context_owner).map(|c| c.identity);
        let owner = match enclosing {
            Context::Callable(instance) => {
                if facts.types.contains_parameters(instance.ty()) != Some(false)
                    || facts.types.contains_projections(instance.ty()) != Some(false)
                {
                    return Err("anonymous enclosing callable is not closed");
                }
                match facts.types.kind(instance.ty()) {
                    Some(Kind::FunctionItem { item, .. }) if closure_owner.is_none() => {
                        DefId(item.0)
                    }
                    Some(Kind::Closure { identity, .. }) if closure_owner == Some(identity) => {
                        facts
                            .closures
                            .identity(identity)
                            .and_then(|closure| closure.owner)
                            .ok_or("anonymous enclosing closure has no declaration owner")?
                    }
                    _ => return Err("anonymous context is not a selected callable"),
                }
            }
            Context::SourceInitializer(witness) => {
                if closure_owner.is_some() {
                    return Err("source initializer selects a closure-owned anonymous body");
                }
                let (instance, original) = witness.identity(facts)?;
                let initializer = facts
                    .initializer_index(instance.definition)
                    .and_then(|index| facts.initializers.get(index))
                    .ok_or("source anonymous context has no initializer")?;
                if initializer.semantic_source() != original {
                    return Err("source anonymous context changed its initializer owner");
                }
                let _ = instance.substitution(facts)?;
                instance.definition
            }
            Context::Initializer(instance) => {
                if closure_owner.is_some() {
                    return Err("anonymous initializer context selects a closure-owned body");
                }
                if !facts.types.arguments_are_concrete(instance.arguments)? {
                    return Err("anonymous initializer context is not closed");
                }
                let _ = instance.substitution(facts)?;
                instance.definition
            }
        };
        if body.declaration != Some(owner) {
            return Err("anonymous instance belongs to another enclosing declaration");
        }
        Ok(Self { source, enclosing })
    }
    #[must_use]
    pub const fn source(self) -> NodeKey {
        self.source
    }
    #[must_use]
    pub const fn enclosing(self) -> Context {
        self.enclosing
    }
}

impl std::fmt::Display for InlineInstance {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "{}:{}@", self.source.file.0, self.source.node.get())?;
        match self.enclosing {
            Context::SourceInitializer(_) => out.write_str("source-initializer"),
            Context::Callable(instance) => write!(out, "callable:{}", instance.ty().0),
            Context::Initializer(instance) => write!(
                out,
                "initializer:{}:{}",
                instance.definition.0, instance.arguments.0
            ),
        }
    }
}
impl InlineInstance {
    pub(crate) fn order(self, facts: &Declarations) -> Result<[u32; 11], &'static str> {
        let mut key = [0; 11];
        key[0] = self.source.file.0;
        key[1] = self.source.node.get();
        match self.enclosing {
            Context::Callable(instance) => key[3] = instance.ty().0,
            Context::Initializer(instance) => {
                key[2] = 1;
                key[3] = instance.definition.0;
                key[4] = instance.arguments.0;
            }
            Context::SourceInitializer(witness) => {
                key[2] = 2;
                key[3..].copy_from_slice(&witness.context_order(facts)?);
            }
        }
        Ok(key)
    }
}
#[derive(Default)]
pub(super) struct Values {
    pub(super) completed:
        crate::support::hash::HashMap<(InlineInstance, crate::types::TypeId), values::Constant>,
    pub(super) journals: Option<Box<super::journals::Journals>>,
}
impl super::Registry {
    #[must_use]
    pub fn instance_value(
        &self,
        instance: InlineInstance,
        result: crate::types::TypeId,
    ) -> Option<values::Constant> {
        self.instance_lookups
            .set(self.instance_lookups.get().saturating_add(1));
        self.instances
            .as_deref()?
            .completed
            .get(&(instance, result))
            .copied()
    }
    pub(crate) fn instance_values(
        &self,
    ) -> impl Iterator<Item = (InlineInstance, crate::types::TypeId, values::Constant)> + '_ {
        self.instances.iter().flat_map(|values| {
            values
                .completed
                .iter()
                .map(|(&(instance, result), &value)| (instance, result, value))
        })
    }
    pub(crate) fn publish_instance(
        &mut self,
        instance: InlineInstance,
        result: crate::types::TypeId,
        value: values::Constant,
    ) -> Result<(), &'static str> {
        let key = (instance, result);
        if let Some(previous) = self.instance_value(instance, result) {
            return if previous == value {
                Ok(())
            } else {
                Err("anonymous instance value changed after publication")
            };
        }
        let bytes = self
            .retained_bytes
            .checked_add(crate::support::retained::new_map::<
                (InlineInstance, crate::types::TypeId),
                values::Constant,
            >(1))
            .filter(|&bytes| bytes <= 64 * 1024 * 1024)
            .ok_or("anonymous instance values exceed 64 MiB budget")?;
        let values = self.instances.get_or_insert_with(Default::default);
        if values.completed.len() >= 1_000_000 {
            return Err("anonymous instance value limit exceeded");
        }
        if let Some(journals) = &mut values.journals {
            journals.record(key, value);
        }
        values.completed.insert(key, value);
        self.instance_publications += 1;
        self.retained_bytes = bytes;
        Ok(())
    }
}
