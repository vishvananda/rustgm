//! Non-root item keys share the table's declaration ordinals and scope order.
use super::{BindingKey, BindingTable};
use crate::{expand::marks::ItemContext, support::hash::HashMap};

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
