//! A loaded source owns immutable text and movable, source-indexed syntax facts.
use crate::{
    expand::{
        config::Config,
        crate_syntax::{self, ConfiguredSyntax},
    },
    lex::token::LexError,
    literal::{
        Value,
        storage::{Storage, StoredValue},
    },
    support::source::SourceFile,
    syntax::{
        ast::NodeId,
        token_tree::{Span, error},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeKey {
    pub file: FileId,
    pub node: NodeId,
}

pub struct Unit {
    pub source: SourceFile,
    pub configured: ConfiguredSyntax<StoredValue>,
    pub literals: Storage,
    pub owner_indices: Vec<Option<std::num::NonZeroU32>>,
}

impl Unit {
    /// Parse/configure once. Loading another file cannot invalidate these facts.
    ///
    /// # Errors
    /// Propagates syntax/configuration errors and storage-range overflows.
    pub fn parse(source: SourceFile, config: &Config) -> Result<Self, LexError> {
        let report = crate_syntax::analyze(source.text(), config)?;
        let mut literals = Storage::default();
        let ast = report.ast.map_literals(|value| {
            literals
                .store(source.text(), value)
                .map_err(|message| error(0, message))
        })?;
        let configured = ConfiguredSyntax {
            ast,
            owners: report.owners,
            active: report.active,
            effects: report.effects,
        };
        let mut owner_indices = vec![None; configured.ast.nodes.len()];
        for (index, owner) in configured.owners.iter().enumerate() {
            let index = u32::try_from(index)
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(std::num::NonZeroU32::new)
                .ok_or_else(|| error(0, "too many attribute owners"))?;
            owner_indices[owner.node.0 as usize] = Some(index);
        }
        Ok(Self {
            source,
            configured,
            literals,
            owner_indices,
        })
    }

    /// # Errors
    /// Rejects invalid manually constructed spans.
    pub fn spelling(&self, span: Span) -> Result<&str, LexError> {
        self.source
            .text()
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| error(span.start, "invalid source span"))
    }

    /// # Errors
    /// Rejects invalid manually supplied literal IDs or storage ranges.
    pub fn literal(&self, index: u32) -> Result<Value<'_>, LexError> {
        let value = self
            .configured
            .ast
            .literals
            .get(index as usize)
            .copied()
            .ok_or_else(|| error(0, "invalid literal ID"))?;
        self.literals
            .view(self.source.text(), value)
            .map_err(|message| error(0, message))
    }
}
