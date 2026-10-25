//! Sparse keyword facts belong to actual item rows, never expression-node flags.
mod retention;
use super::{
    ast::{Kind as NodeKind, NodeId, Syntax},
    item::ItemKind,
    token_tree::{Span, error},
};
use crate::lex::token::LexError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Function,
    Implementation,
    Constant,
    Type,
}
impl Kind {
    fn item(item: ItemKind) -> Option<Self> {
        Some(match item {
            ItemKind::Function { .. } => Self::Function,
            ItemKind::Impl { .. } => Self::Implementation,
            ItemKind::Const { .. } => Self::Constant,
            ItemKind::TypeAlias { .. } => Self::Type,
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub item: u32,
    pub node: NodeId,
    pub keyword: Span,
    pub kind: Kind,
    context: Option<std::num::NonZeroU32>,
}
impl Declaration {
    pub(crate) fn context_key(&self) -> Option<std::num::NonZeroU32> {
        self.context
    }
    pub(crate) fn context_matches(&self, actual: std::num::NonZeroU32) -> bool {
        self.context == Some(actual)
    }
    pub(crate) fn validate<L>(&self, ast: &Syntax<L>) -> Result<(), LexError> {
        let invalid = || error(self.keyword.start, "defaultness actual item changed");
        let node = ast
            .nodes
            .get(self.node.get() as usize)
            .ok_or_else(invalid)?;
        if node.kind != NodeKind::Item(self.item)
            || ast
                .items
                .get(self.item as usize)
                .and_then(|item| Kind::item(item.kind))
                != Some(self.kind)
            || self.keyword.end.checked_sub(self.keyword.start) != Some(7)
            || node.span.start > self.keyword.start
            || self.keyword.end > node.span.end
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Declarations {
    entries: Vec<Declaration>,
    witness: retention::Witness,
    completed: usize,
}
impl Declarations {
    #[cfg(test)]
    pub(crate) fn storage(&self) -> (usize, usize, usize, usize) {
        let (steps, bytes) = self.witness.usage();
        (self.entries.len(), self.entries.capacity(), steps, bytes)
    }
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
    pub(crate) fn entries(&self) -> &[Declaration] {
        &self.entries
    }
    fn push(&mut self, declaration: Declaration) -> Result<(), LexError> {
        self.charge(1, 4 * size_of::<Declaration>())
            .map_err(|message| error(declaration.keyword.start, message))?;
        if self
            .entries
            .last()
            .is_some_and(|last| last.item >= declaration.item)
        {
            return Err(error(
                declaration.keyword.start,
                "defaultness item order changed",
            ));
        }
        if self.entries.len() >= 1_000_000 {
            return Err(error(
                declaration.keyword.start,
                "defaultness storage budget exceeded",
            ));
        }
        self.entries.try_reserve(1).map_err(|_| {
            error(
                declaration.keyword.start,
                "defaultness storage allocation failed",
            )
        })?;
        self.entries.push(declaration);
        Ok(())
    }
    pub(crate) fn prefix(&self, count: usize) -> Result<Box<Self>, LexError> {
        let entries = self
            .entries
            .get(..count)
            .ok_or_else(|| error(0, "invalid original defaultness prefix"))?;
        self.charge(
            1 + count,
            size_of::<Self>() + count * size_of::<Declaration>(),
        )
        .map_err(|message| error(0, message))?;
        let mut copied = Self {
            witness: self.witness.clone(),
            completed: self.completed.min(count),
            ..Self::default()
        };
        copied
            .entries
            .try_reserve_exact(count)
            .map_err(|_| error(0, "original defaultness allocation failed"))?;
        copied.entries.extend_from_slice(entries);
        Ok(Box::new(copied))
    }
}
impl Declarations {
    pub(crate) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.witness.charge(steps, bytes)
    }
    pub(crate) fn exhausted(&self) -> bool {
        self.witness.exhausted()
    }
    pub(crate) fn validate_source(
        &self,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) -> Result<(), &'static str> {
        self.witness.validate(image)
    }
    pub(crate) fn pending_context(&self) -> Option<Declaration> {
        self.entries.get(self.completed).copied()
    }
    pub(crate) fn seal_context(
        &mut self,
        expected: Declaration,
        actual: std::num::NonZeroU32,
    ) -> Result<(), &'static str> {
        self.charge(1, 0)?;
        let entry = self
            .entries
            .get_mut(self.completed)
            .ok_or("defaultness context prefix disappeared")?;
        if *entry != expected || entry.context.is_some() {
            return Err("defaultness context producer changed its pending row");
        }
        entry.context = Some(actual);
        self.completed += 1;
        Ok(())
    }
}
impl<L> Syntax<L> {
    #[must_use]
    pub fn defaultness(&self, item: u32) -> Option<&Declaration> {
        let entries = &self.defaultness.as_ref()?.entries;
        entries
            .binary_search_by_key(&item, |entry| entry.item)
            .ok()
            .map(|index| &entries[index])
    }
    pub(crate) fn record_defaultness(
        &mut self,
        item: u32,
        node: NodeId,
        keyword: Span,
    ) -> Result<(), LexError> {
        if self
            .nodes
            .get(node.get() as usize)
            .is_none_or(|actual| actual.kind != NodeKind::Item(item))
        {
            return Err(error(
                keyword.start,
                "defaultness requires its actual item node",
            ));
        }
        let kind = self
            .items
            .get(item as usize)
            .and_then(|item| Kind::item(item.kind))
            .ok_or_else(|| {
                error(
                    keyword.start,
                    "default modifier is not permitted on this item",
                )
            })?;
        let declaration = Declaration {
            item,
            node,
            keyword,
            kind,
            context: None,
        };
        declaration.validate(self)?;
        if let Some(entries) = &mut self.defaultness {
            entries.push(declaration)?;
        } else {
            let mut entries = Declarations::default();
            entries
                .charge(1, size_of::<Declarations>())
                .map_err(|message| error(keyword.start, message))?;
            entries.push(declaration)?;
            self.defaultness = Some(Box::new(entries));
        }
        Ok(())
    }
    pub(crate) fn bind_defaultness_source(
        &mut self,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) -> Result<(), LexError> {
        if let Some(entries) = &mut self.defaultness {
            entries
                .witness
                .bind(image)
                .map_err(|message| error(0, message))?;
            entries
                .charge(entries.entries.len(), 0)
                .map_err(|message| error(0, message))?;
            for entry in &mut entries.entries {
                entry.context = Some(std::num::NonZeroU32::MIN);
            }
            entries.completed = entries.entries.len();
        }
        Ok(())
    }
    pub(crate) fn fresh_defaultness_source(
        &mut self,
        previous: &std::sync::Arc<crate::support::source::OriginalImage>,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) -> Result<(), LexError> {
        if let Some(entries) = &mut self.defaultness {
            if entries.entries.is_empty() && !entries.witness.bound() {
                // Genuine original replay may carry a prior failed transfer's debt.
                entries
                    .witness
                    .bind(previous)
                    .map_err(|message| error(0, message))?;
            }
            entries
                .witness
                .fresh(previous, image)
                .map_err(|message| error(0, message))?;
        }
        Ok(())
    }
    pub(crate) fn commit_defaultness_source(
        &mut self,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) {
        if let Some(entries) = &mut self.defaultness {
            entries.witness.commit(image);
        }
    }
    pub(crate) fn validate_defaultness_source(
        &self,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) -> Result<(), LexError> {
        if let Some(entries) = &self.defaultness {
            if entries.entries.is_empty() && !entries.witness.bound() {
                // This is retained failed-attempt debt, never token authority.
                entries.charge(1, 0).map_err(|message| error(0, message))?;
            } else {
                entries
                    .validate_source(image)
                    .map_err(|message| error(0, message))?;
            }
        }
        Ok(())
    }
    pub(crate) fn move_defaultness_root(&mut self, from: NodeId, to: NodeId) {
        let NodeKind::Item(item) = self.nodes[from.get() as usize].kind else {
            return;
        };
        if let Some(entries) = &mut self.defaultness
            && let Ok(index) = entries
                .entries
                .binary_search_by_key(&item, |entry| entry.item)
        {
            // Transfer authenticated this exact incoming root before replacement.
            // Unit commits the same root on its existing macrocall identity.
            entries.entries[index].node = to;
        }
    }
    pub(crate) fn truncate_defaultness(&mut self, count: usize) {
        if let Some(entries) = &mut self.defaultness {
            // A failed default-bearing attempt owns debt even at a zero prefix.
            // Syntax which never attempted defaultness still has no owner.
            entries.entries.truncate(count);
            entries.completed = entries.completed.min(count);
        }
    }
    pub(crate) fn defaultness_prefix(
        &self,
        count: usize,
    ) -> Result<Option<Box<Declarations>>, LexError> {
        match &self.defaultness {
            Some(entries) => entries.prefix(count).map(Some),
            None if count == 0 => Ok(None),
            None => Err(error(0, "original defaultness disappeared")),
        }
    }
    pub(crate) fn defaultness_bytes(&self) -> usize {
        self.defaultness.as_ref().map_or(0, |entries| {
            size_of::<Declarations>() + entries.entries.capacity() * size_of::<Declaration>()
        })
    }
}
