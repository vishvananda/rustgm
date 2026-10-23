//! Interned token marks keep expansion identity separate from normalized names.
use crate::support::hash::HashMap;

/// Actual invocation identity; zero denotes the physical source ancestry root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExpansionId(pub u32);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SyntaxContextId(u32);
impl SyntaxContextId {
    pub const ROOT: Self = Self(0);
}
/// Only the mark arena can construct an item-normalized context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemContext(SyntaxContextId);
/// Only the mark arena can construct a local-normalized context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalContext(SyntaxContextId);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Transparency {
    Transparent,
    SemiOpaque,
    Opaque,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    parent: SyntaxContextId,
    expansion: ExpansionId,
    transparency: Transparency,
}
struct Node {
    key: Key,
    item: SyntaxContextId,
    local: SyntaxContextId,
}
#[derive(Default)]
struct Data {
    nodes: Vec<Node>,
    interned: HashMap<Key, SyntaxContextId>,
    /// A rebased definition prefix excludes the new invocation's final mark.
    rebased: HashMap<(SyntaxContextId, SyntaxContextId), SyntaxContextId>,
    #[cfg(test)]
    rebase_steps: usize,
}
/// Physical source needs neither a node nor a heap allocation.
#[derive(Default)]
pub struct Marks(Option<Box<Data>>);
impl Marks {
    /// Apply one invocation to an actual definition token's retained context.
    /// Captured tokens preserve their existing context and do not call this.
    /// # Errors
    /// Rejects root invocation marks, unknown contexts and identity overflow.
    pub fn apply(
        &mut self,
        original: SyntaxContextId,
        expansion: ExpansionId,
        transparency: Transparency,
        call_site: SyntaxContextId,
    ) -> Result<SyntaxContextId, &'static str> {
        if expansion.0 == 0 {
            return Err("a token mark requires an actual expansion");
        }
        self.projections(original)?;
        let (item, local) = self.projections(call_site)?;
        let base = match transparency {
            Transparency::Opaque => SyntaxContextId::ROOT,
            Transparency::SemiOpaque => item,
            Transparency::Transparent => local,
        };
        let data = self.0.get_or_insert_with(Box::default);
        let parent = if base == SyntaxContextId::ROOT {
            original
        } else {
            data.rebase(original, base)?
        };
        data.intern(Key {
            parent,
            expansion,
            transparency,
        })
    }
    /// # Errors
    /// Rejects a context outside the arena.
    pub fn item(&self, context: SyntaxContextId) -> Result<ItemContext, &'static str> {
        self.projections(context).map(|(item, _)| ItemContext(item))
    }
    /// # Errors
    /// Rejects a context outside the arena.
    pub fn local(&self, context: SyntaxContextId) -> Result<LocalContext, &'static str> {
        self.projections(context)
            .map(|(_, local)| LocalContext(local))
    }
    /// # Errors
    /// Rejects a context outside the arena.
    pub fn outer(&self, context: SyntaxContextId) -> Result<Option<ExpansionId>, &'static str> {
        self.node(context)
            .map(|node| node.map(|node| node.key.expansion))
    }
    /// Remove one local-normalized mark, preserving that token's own parent.
    /// # Errors
    /// Rejects a context outside the arena.
    pub fn local_parent(
        &self,
        context: LocalContext,
    ) -> Result<Option<(LocalContext, ExpansionId)>, &'static str> {
        self.node(context.0)?
            .map(|node| Ok((self.local(node.key.parent)?, node.key.expansion)))
            .transpose()
    }
    /// # Errors
    /// Rejects a context outside the arena.
    pub fn item_parent(
        &self,
        context: ItemContext,
    ) -> Result<Option<(ItemContext, ExpansionId)>, &'static str> {
        self.node(context.0)?
            .map(|node| Ok((self.item(node.key.parent)?, node.key.expansion)))
            .transpose()
    }
    fn node(&self, context: SyntaxContextId) -> Result<Option<&Node>, &'static str> {
        if context == SyntaxContextId::ROOT {
            return Ok(None);
        }
        self.0
            .as_ref()
            .and_then(|data| data.nodes.get(context.0 as usize - 1))
            .map(Some)
            .ok_or("unknown token syntax context")
    }
    fn projections(
        &self,
        context: SyntaxContextId,
    ) -> Result<(SyntaxContextId, SyntaxContextId), &'static str> {
        Ok(self
            .node(context)?
            .map_or((context, context), |node| (node.item, node.local)))
    }
}
impl Data {
    fn intern(&mut self, key: Key) -> Result<SyntaxContextId, &'static str> {
        if let Some(context) = self.interned.get(&key) {
            return Ok(*context);
        }
        let (parent_item, parent_local) = if key.parent == SyntaxContextId::ROOT {
            (key.parent, key.parent)
        } else {
            let parent = &self.nodes[key.parent.0 as usize - 1];
            (parent.item, parent.local)
        };
        // None means the new node itself is already normalized. Construct any
        // distinct projections first; their parents are normalized, so this
        // recursion has constant depth rather than following the token chain.
        let item = if key.transparency == Transparency::Opaque {
            self.projected(key, parent_item)?
        } else {
            Some(parent_item)
        };
        let local = if key.transparency == Transparency::Transparent {
            Some(parent_local)
        } else {
            self.projected(key, parent_local)?
        };
        let id = self
            .nodes
            .len()
            .checked_add(1)
            .and_then(|length| u32::try_from(length).ok())
            .map(SyntaxContextId)
            .ok_or("token syntax context space exhausted")?;
        self.nodes.push(Node {
            key,
            item: item.unwrap_or(id),
            local: local.unwrap_or(id),
        });
        self.interned.insert(key, id);
        Ok(id)
    }
    fn projected(
        &mut self,
        key: Key,
        parent: SyntaxContextId,
    ) -> Result<Option<SyntaxContextId>, &'static str> {
        if parent == key.parent {
            Ok(None)
        } else {
            self.intern(Key { parent, ..key }).map(Some)
        }
    }
    fn rebase(
        &mut self,
        original: SyntaxContextId,
        base: SyntaxContextId,
    ) -> Result<SyntaxContextId, &'static str> {
        let mut pending = Vec::new();
        let mut context = original;
        let mut prefix = loop {
            if context == SyntaxContextId::ROOT {
                break base;
            }
            if let Some(prefix) = self.rebased.get(&(context, base)) {
                break *prefix;
            }
            #[cfg(test)]
            {
                self.rebase_steps += 1;
            }
            pending.push(context);
            context = self.nodes[context.0 as usize - 1].key.parent;
        };
        for original in pending.into_iter().rev() {
            let key = self.nodes[original.0 as usize - 1].key;
            prefix = self.intern(Key {
                parent: prefix,
                ..key
            })?;
            self.rebased.insert((original, base), prefix);
        }
        Ok(prefix)
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/mark_contexts.rs"]
mod tests;
