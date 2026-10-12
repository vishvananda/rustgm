//! Selected pattern structure is shared by semantic checks and body construction.
pub mod scalar;
use super::members::{Selection, VariantId};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    types::{RegionId, TypeId},
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PatternId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RangeId(pub u32);
/// Inclusive ordered scalar ranks; signed integers and Unicode gaps are normalized.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    pub lower: u128,
    pub upper: u128,
}
#[derive(Clone, Copy)]
pub struct Arm {
    pub source: NodeKey,
    pub pattern: Option<PatternId>,
    pub guard: Option<NodeKey>,
    pub value: NodeKey,
    pub useful: bool,
}
pub struct Match {
    pub source: NodeKey,
    pub scrutinee: NodeKey,
    pub start: u32,
    pub end: u32,
    /// Logical pattern visits and interval-index operations, including guard checks.
    pub work: u64,
}
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
    Subslice { prefix: u64, suffix: u64 },
}
/// An occurrence's operation; alternatives can share a definition but differ here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capture {
    Value,
    Borrow { mutable: bool, region: RegionId },
}
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Pending,
    Wildcard,
    Range(RangeId),
    Binding {
        definition: DefId,
        mutable: bool,
        capture: Capture,
        subpattern: Option<PatternId>,
    },
    Dereference {
        mutable: bool,
        implicit: bool,
        pattern: PatternId,
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
    pub ranges: Vec<Interval>,
    pub matches: HashMap<NodeKey, u32>,
    pub decisions: Vec<Match>,
    pub arms: Vec<Arm>,
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
