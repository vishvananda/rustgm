//! Sparse feature sites share source identity, transfer and failed-attempt accounting.
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
    TraitAlias,
    ForeignType,
    FinalFunction,
}
impl Kind {
    pub(crate) const fn is_defaultness(self) -> bool {
        matches!(
            self,
            Self::Function | Self::Implementation | Self::Constant | Self::Type
        )
    }
    fn item(item: ItemKind) -> Option<Self> {
        Some(match item {
            ItemKind::Function { .. } => Self::Function,
            ItemKind::Impl { .. } => Self::Implementation,
            ItemKind::Const { .. } => Self::Constant,
            ItemKind::TypeAlias { .. } => Self::Type,
            ItemKind::TraitAlias { .. } => Self::TraitAlias,
            ItemKind::ForeignType => Self::ForeignType,
            _ => return None,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub item: u32,
    pub node: NodeId,
    pub span: Span,
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
        let invalid = || error(self.span.start, "defaultness actual item changed");
        let node = ast
            .nodes
            .get(self.node.get() as usize)
            .ok_or_else(invalid)?;
        if node.kind != NodeKind::Item(self.item)
            || ast
                .items
                .get(self.item as usize)
                .and_then(|item| Kind::item(item.kind))
                != Some(if self.kind == Kind::FinalFunction {
                    Kind::Function
                } else {
                    self.kind
                })
            || if self.kind == Kind::TraitAlias {
                self.span.end <= self.span.start || self.span != node.span
            } else {
                self.span.end.checked_sub(self.span.start)
                    != Some(match self.kind {
                        Kind::ForeignType => 4,
                        Kind::FinalFunction => 5,
                        _ => 7,
                    })
            }
            || node.span.start > self.span.start
            || self.span.end > node.span.end
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
            .map_err(|message| error(declaration.span.start, message))?;
        if self
            .entries
            .last()
            .is_some_and(|last| last.item >= declaration.item)
        {
            return Err(error(
                declaration.span.start,
                "defaultness item order changed",
            ));
        }
        if self.entries.len() >= 1_000_000 {
            return Err(error(
                declaration.span.start,
                "defaultness storage budget exceeded",
            ));
        }
        self.entries.try_reserve(1).map_err(|_| {
            error(
                declaration.span.start,
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
        self.gate_site(item)
            .filter(|site| site.kind.is_defaultness())
    }
    #[must_use]
    pub(crate) fn has_defaultness(&self) -> bool {
        self.gate_sites.as_ref().is_some_and(|entries| {
            entries
                .entries
                .iter()
                .any(|site| site.kind.is_defaultness())
        })
    }
    pub(crate) fn gate_site(&self, item: u32) -> Option<&Declaration> {
        let entries = &self.gate_sites.as_ref()?.entries;
        entries
            .binary_search_by_key(&item, |entry| entry.item)
            .ok()
            .map(|index| &entries[index])
    }
    pub(crate) fn record_defaultness(
        &mut self,
        item: u32,
        node: NodeId,
        span: Span,
    ) -> Result<(), LexError> {
        if matches!(
            self.items.get(item as usize).map(|item| item.kind),
            Some(ItemKind::TraitAlias { .. } | ItemKind::ForeignType)
        ) {
            return Err(error(
                span.start,
                "default modifier is not permitted on this item",
            ));
        }
        self.record_gate_site(item, node, span)
    }
    pub(crate) fn record_gate_site(
        &mut self,
        item: u32,
        node: NodeId,
        span: Span,
    ) -> Result<(), LexError> {
        if self
            .nodes
            .get(node.get() as usize)
            .is_none_or(|actual| actual.kind != NodeKind::Item(item))
        {
            return Err(error(
                span.start,
                "defaultness requires its actual item node",
            ));
        }
        let kind = self
            .items
            .get(item as usize)
            .and_then(|item| Kind::item(item.kind))
            .ok_or_else(|| error(span.start, "default modifier is not permitted on this item"))?;
        self.record_typed_gate_site(item, node, span, kind)
    }
    pub(crate) fn record_typed_gate_site(
        &mut self,
        item: u32,
        node: NodeId,
        span: Span,
        kind: Kind,
    ) -> Result<(), LexError> {
        let declaration = Declaration {
            item,
            node,
            span,
            kind,
            context: None,
        };
        declaration.validate(self)?;
        if let Some(entries) = &mut self.gate_sites {
            entries.push(declaration)?;
        } else {
            let mut entries = Declarations::default();
            entries
                .charge(1, size_of::<Declarations>())
                .map_err(|message| error(span.start, message))?;
            entries.push(declaration)?;
            self.gate_sites = Some(Box::new(entries));
        }
        Ok(())
    }
    pub(crate) fn bind_gate_sites_source(
        &mut self,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) -> Result<(), LexError> {
        if let Some(entries) = &mut self.gate_sites {
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
    pub(crate) fn fresh_gate_sites_source(
        &mut self,
        previous: &std::sync::Arc<crate::support::source::OriginalImage>,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) -> Result<(), LexError> {
        if let Some(entries) = &mut self.gate_sites {
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
    pub(crate) fn commit_gate_sites_source(
        &mut self,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) {
        if let Some(entries) = &mut self.gate_sites {
            entries.witness.commit(image);
        }
    }
    pub(crate) fn validate_gate_sites_source(
        &self,
        image: &std::sync::Arc<crate::support::source::OriginalImage>,
    ) -> Result<(), LexError> {
        if let Some(entries) = &self.gate_sites {
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
    pub(crate) fn move_gate_sites_root(&mut self, from: NodeId, to: NodeId) {
        let NodeKind::Item(item) = self.nodes[from.get() as usize].kind else {
            return;
        };
        if let Some(entries) = &mut self.gate_sites
            && let Ok(index) = entries
                .entries
                .binary_search_by_key(&item, |entry| entry.item)
        {
            // Transfer authenticated this exact incoming root before replacement.
            // Unit commits the same root on its existing macrocall identity.
            entries.entries[index].node = to;
        }
    }
    pub(crate) fn truncate_gate_sites(&mut self, count: usize) {
        if let Some(entries) = &mut self.gate_sites {
            // A failed default-bearing attempt owns debt even at a zero prefix.
            // Syntax which never attempted defaultness still has no owner.
            entries.entries.truncate(count);
            entries.completed = entries.completed.min(count);
        }
    }
    pub(crate) fn gate_sites_prefix(
        &self,
        count: usize,
    ) -> Result<Option<Box<Declarations>>, LexError> {
        match &self.gate_sites {
            Some(entries) => entries.prefix(count).map(Some),
            None if count == 0 => Ok(None),
            None => Err(error(0, "original defaultness disappeared")),
        }
    }
    pub(crate) fn gate_sites_bytes(&self) -> usize {
        self.gate_sites.as_ref().map_or(0, |entries| {
            size_of::<Declarations>() + entries.entries.capacity() * size_of::<Declaration>()
        })
    }
}
