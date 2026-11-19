//! Non-root item keys share the table's declaration ordinals and scope order.
use super::{Binding, BindingKey, BindingTable, NameId, Namespace, ScopeId};
use crate::{
    expand::marks::{ItemContext, LocalContext},
    support::hash::HashMap,
};

#[derive(Default)]
pub(super) struct Index {
    forward: HashMap<BindingKey, u32>,
    reverse: HashMap<u32, ItemContext>,
}
impl Index {
    pub fn len(&self) -> usize {
        self.forward.len()
    }
}
impl BindingTable {
    /// Retained non-root keys cannot be represented by name-only metadata.
    #[must_use]
    pub fn has_contextual_items(&self) -> bool {
        self.contextual.is_some()
    }
    /// Bindings whose item namespace key carries a non-root context.
    pub(crate) fn contextual_items(
        &self,
    ) -> impl Iterator<Item = (ScopeId, NameId, Namespace, &Binding)> {
        self.contextual
            .iter()
            .flat_map(|index| index.forward.iter())
            .filter_map(|(&(scope, name, namespace, _), &index)| {
                let binding = self.entries.get(index as usize)?.binding.as_ref()?;
                Some((scope, name, namespace, binding))
            })
    }
    /// A local, parameter or label bound by a marked name token. Body-local
    /// declaration rows carry only a spelling, never this context.
    #[must_use]
    pub fn has_contextual_locals(&self) -> bool {
        self.locals
            .iter()
            .any(|local| local.context != LocalContext::ROOT)
    }
    pub(super) fn find_key(&self, key: BindingKey) -> Option<u32> {
        if key.3 == ItemContext::ROOT {
            return self.find(key.0, key.1, key.2);
        }
        self.contextual.as_ref()?.forward.get(&key).copied()
    }
    pub(super) fn item_context(&self, entry: u32) -> ItemContext {
        self.contextual
            .as_ref()
            .and_then(|index| index.reverse.get(&entry))
            .copied()
            .unwrap_or(ItemContext::ROOT)
    }
    pub(super) fn entry_key(&mut self, key: BindingKey) -> Result<u32, &'static str> {
        if key.3 == ItemContext::ROOT {
            return self.entry(key.0, key.1, key.2);
        }
        if let Some(entry) = self.find_key(key) {
            return Ok(entry);
        }
        let entry = self.new_entry(key.0, key.1, key.2)?;
        let index = self.contextual.get_or_insert_with(Box::default);
        index.forward.insert(key, entry);
        index.reverse.insert(entry, key.3);
        Ok(entry)
    }
}
