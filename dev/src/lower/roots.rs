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
    pub startup: Option<crate::sema::entry::Startup>,
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
            startup: None,
        }
    }
}
/// A coarse runaway guard: a whole-graph session exports the roots of every
/// crate in it; this bounds only the sorted root lists.
const LIMIT: usize = 1 << 20;
const EXCEEDED: &str = "export roots exceed their whole-graph runaway limit";

pub(super) fn exports(facts: &Declarations, explicit: &[DefId]) -> Result<Exports, &'static str> {
    if explicit.len() > LIMIT {
        return Err(EXCEEDED);
    }
    let (mut functions, mut globals) = facts.foreign.as_deref().map_or_else(
        || (Vec::new(), Vec::new()),
        crate::sema::foreign::Registry::all_exports,
    );
    if functions.len().saturating_add(explicit.len()) > LIMIT {
        return Err(EXCEEDED);
    }
    let retained = facts.compiler_retained_statics()?;
    if functions
        .len()
        .saturating_add(explicit.len())
        .saturating_add(globals.len())
        .saturating_add(retained.len())
        > LIMIT
    {
        return Err(EXCEEDED);
    }
    functions.extend_from_slice(explicit);
    functions.sort_unstable_by_key(|id| id.0);
    functions.dedup();
    if !retained.is_empty() {
        globals
            .try_reserve(retained.len())
            .map_err(|_| "retained static roots allocation failed")?;
        globals.extend_from_slice(retained);
        globals.sort_unstable_by_key(|id| id.0);
        globals.dedup();
    }
    Ok(Exports { functions, globals })
}
