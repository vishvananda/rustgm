//! Executable adapters and library exports share the same checked demand roots.
use crate::{resolve::model::DefId, sema::model::Declarations};

#[derive(Clone, Copy)]
pub struct Roots<'a> {
    pub entry: Option<DefId>,
    pub exports: &'a [DefId],
}
impl Roots<'_> {
    #[must_use]
    pub const fn executable(entry: DefId) -> Self {
        Self {
            entry: Some(entry),
            exports: &[],
        }
    }
    pub(crate) fn iter(self) -> impl DoubleEndedIterator<Item = DefId> {
        self.entry.into_iter().chain(self.exports.iter().copied())
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Selected<'a> {
    pub functions: Roots<'a>,
    pub globals: &'a [DefId],
}
pub(super) struct Exports {
    functions: Vec<DefId>,
    globals: Vec<DefId>,
}
impl Exports {
    pub fn selected(&self, entry: Option<DefId>) -> Selected<'_> {
        Selected {
            functions: Roots {
                entry,
                exports: &self.functions,
            },
            globals: &self.globals,
        }
    }
}
pub(super) fn exports(facts: &Declarations, explicit: &[DefId]) -> Result<Exports, &'static str> {
    const LIMIT: usize = 100_000;
    if explicit.len() > LIMIT {
        return Err("export roots exceed 100000 function budget");
    }
    let (mut functions, globals) = facts.foreign.as_deref().map_or_else(
        || (Vec::new(), Vec::new()),
        crate::sema::foreign::Registry::all_exports,
    );
    if functions.len().saturating_add(explicit.len()) > LIMIT {
        return Err("export roots exceed 100000 function budget");
    }
    if functions
        .len()
        .saturating_add(explicit.len())
        .saturating_add(globals.len())
        > LIMIT
    {
        return Err("export roots exceed 100000 definition budget");
    }
    functions.extend_from_slice(explicit);
    functions.sort_unstable_by_key(|id| id.0);
    functions.dedup();
    Ok(Exports { functions, globals })
}
