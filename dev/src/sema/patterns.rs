//! Selected pattern structure is shared by semantic checks and body construction.
use super::members::{Selection, VariantId};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::TypeId,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PatternId(pub u32);
#[derive(Clone, Copy, Debug)]
pub struct Fields {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct Alternatives {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub selection: Projection,
    pub pattern: PatternId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Projection {
    Field(Selection),
    Index(u64),
    Subarray { start: u64, end: u64 },
}
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Pending,
    Wildcard,
    Binding {
        definition: DefId,
        mutable: bool,
        subpattern: Option<PatternId>,
    },
    Product {
        variant: Option<VariantId>,
        fields: Fields,
    },
    Or(Alternatives),
}
pub struct Pattern {
    pub source: NodeKey,
    /// Completed once inference has solved the containing body.
    pub ty: Option<TypeId>,
    pub kind: Kind,
}
#[derive(Default)]
pub struct Patterns {
    pub roots: HashMap<NodeKey, PatternId>,
    pub nodes: Vec<Pattern>,
    pub fields: Vec<Field>,
    pub alternatives: Vec<PatternId>,
}
impl Patterns {
    /// Reserve a compact identity before elaborating nested pattern structure.
    ///
    /// # Errors
    /// Rejects exhausted pattern identities.
    pub(super) fn reserve(&mut self, source: NodeKey) -> Result<PatternId, &'static str> {
        let id = PatternId(u32::try_from(self.nodes.len()).map_err(|_| "too many patterns")?);
        self.nodes.push(Pattern {
            source,
            ty: None,
            kind: Kind::Pending,
        });
        Ok(id)
    }
    #[must_use]
    pub fn fields(&self, fields: Fields) -> &[Field] {
        &self.fields[fields.start as usize..fields.end as usize]
    }
    #[must_use]
    pub fn alternatives(&self, alternatives: Alternatives) -> &[PatternId] {
        &self.alternatives[alternatives.start as usize..alternatives.end as usize]
    }
}
