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
pub(super) fn exports(
    facts: &Declarations,
    explicit: &[DefId],
) -> Result<Vec<DefId>, &'static str> {
    const LIMIT: usize = 100_000;
    if explicit.len() > LIMIT {
        return Err("export roots exceed 100000 function budget");
    }
    let mut exports = facts
        .foreign
        .as_deref()
        .map_or_else(Vec::new, crate::sema::foreign::Registry::exports);
    if exports.len().saturating_add(explicit.len()) > LIMIT {
        return Err("export roots exceed 100000 function budget");
    }
    exports.extend_from_slice(explicit);
    exports.sort_unstable_by_key(|definition| definition.0);
    exports.dedup();
    Ok(exports)
}
