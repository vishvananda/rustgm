//! A public glob import of a dependency's module, such as the shim
//! `rustc_std_workspace_core`'s `pub use core::*`, publishes one glob row of
//! the macro table instead of one binding per name. Its consumer binds the
//! names again from the owner's own module. The names that glob gave are
//! therefore left out of the public-name and macro rows, unless another
//! public glob in the same scope could have given one of them.
use super::{Error, Report, error};
use crate::{
    resolve::model::{Access, Binding, Origin, ScopeId},
    support::{
        crates::{CrateIdentity, exported_macros::GlobReexport},
        hash::{HashMap, HashSet},
    },
};
use std::rc::Rc;

/// The recorded dependency-module globs of one publication.
#[derive(Default)]
pub(super) struct Globs {
    imports: HashSet<u32>,
    /// Scopes that also hold a public glob of a local module or an enum.
    mixed: HashSet<ScopeId>,
}
impl Globs {
    pub(super) fn new(report: &Report) -> Self {
        let Some(recorded) = report
            .resolution
            .crates
            .as_deref()
            .map(crate::resolve::crates::Imported::module_globs)
            .filter(|recorded| !recorded.is_empty())
        else {
            return Self::default();
        };
        let imports: HashSet<u32> = recorded.iter().map(|&(index, _)| index).collect();
        let mixed = report
            .resolution
            .imports
            .iter()
            .enumerate()
            .filter(|&(index, import)| {
                import.glob
                    && import.access == Access::Public
                    && u32::try_from(index).is_ok_and(|index| !imports.contains(&index))
            })
            .map(|(_, import)| import.scope)
            .collect();
        Self { imports, mixed }
    }
    /// Whether a glob row carries this public binding of `scope`: a recorded
    /// glob gave it, and if it is ambiguous every glob of its scope is one.
    pub(super) fn covered(&self, scope: ScopeId, binding: &Binding) -> bool {
        matches!(binding.origin, Origin::Glob(index) if self.imports.contains(&index))
            && (!binding.ambiguous || !self.mixed.contains(&scope))
    }
}

/// The glob row of every recorded glob in a public placement. A glob in a
/// private module binds nothing outside this crate; one in a module that only
/// a re-export reaches has no placement and is rejected by name.
pub(super) fn rows(
    report: &Report,
    placements: &HashMap<ScopeId, Rc<[String]>>,
    aliased: &HashSet<ScopeId>,
    dependencies: &[CrateIdentity],
) -> Result<Vec<GlobReexport>, Error> {
    let Some(crates) = report.resolution.crates.as_deref() else {
        return Ok(Vec::new());
    };
    let recorded = crates.module_globs();
    let modules: Vec<_> = recorded.iter().map(|&(_, module)| module).collect();
    let identities = crates.module_identities(&modules).map_err(error)?;
    let mut rows = Vec::new();
    for (&(index, _), identity) in recorded.iter().zip(identities) {
        let scope = report.resolution.imports[index as usize].scope;
        let Some(placement) = placements.get(&scope) else {
            if aliased.contains(&scope) {
                return Err(error(
                    "glob re-exports in a module reached only through a re-export are outside the macro metadata profile",
                ));
            }
            continue;
        };
        let (owner, target) = identity.ok_or_else(|| {
            error("glob re-export of a module outside every dependency is outside the macro metadata profile")
        })?;
        let ordinal = dependencies
            .binary_search(owner)
            .map_err(|_| error("glob re-export target's crate is not a direct dependency"))?;
        rows.push(
            GlobReexport::new(
                placement.to_vec(),
                u32::try_from(ordinal + 1).map_err(|_| error("too many macro owners"))?,
                target,
            )
            .map_err(error)?,
        );
    }
    rows.sort_unstable_by(|left, right| {
        (left.placement(), left.owner(), left.target()).cmp(&(
            right.placement(),
            right.owner(),
            right.target(),
        ))
    });
    rows.dedup();
    Ok(rows)
}
