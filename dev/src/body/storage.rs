//! Storage endpoints are explicit CFG facts, independent of promotion and loans.
use super::{ir::BlockId, scopes};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Location {
    /// Immediately before a global statement index, including the block's end.
    At { block: BlockId, before: u32 },
    /// After selecting this successor, before executing any of its statements.
    Edge { block: BlockId, target: BlockId },
}
/// Close scopes from `inner` upward, excluding `outer`; None includes the body.
/// This persistent ancestor range does not copy live locals or temporary lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub inner: scopes::Id,
    pub outer: Option<scopes::Id>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Scopes(Range),
    /// A let-else failure ends its initializer's places, including extended ones,
    /// without ending earlier temporaries owned by the same surrounding block.
    Temporaries {
        start: u32,
        end: u32,
    },
}
#[derive(Clone, Copy, Debug)]
pub struct Exit {
    pub location: Location,
    pub end: End,
}
/// Body-owned lexical location; semantic scope construction retains its syntax IDs.
#[derive(Clone, Copy, Debug)]
pub struct Scope {
    pub parent: Option<scopes::Id>,
    pub source: super::ir::BodySite,
    pub kind: scopes::Kind,
}
impl From<scopes::Scope> for Scope {
    fn from(scope: scopes::Scope) -> Self {
        Self {
            parent: scope.parent,
            source: scope.source.into(),
            kind: scope.kind,
        }
    }
}
#[derive(Clone)]
pub struct Facts {
    pub scope_work: usize,
    pub scopes: Vec<Scope>,
    pub exits: Vec<Exit>,
}
impl Facts {
    pub(crate) fn new(scopes: scopes::Facts) -> Self {
        Self {
            scope_work: scopes.work.saturating_add(scopes.scopes().len()),
            scopes: scopes.into_scopes().into_iter().map(Scope::from).collect(),
            exits: Vec::new(),
        }
    }
}
