//! A second global allocator or allocation error handler is diagnosed as
//! rustc 1.98.0's crate loader does (`inject_allocator_crate`), not as a
//! plain duplicate export: both register the same internal symbol.
use super::{Checker, DefId};
use crate::support::linkage::Naming;

/// The registered items, each by the unmangled name of the function that its
/// attribute generates, with the attribute and what rustc calls the item.
const REGISTERED: [(&str, &str, &str); 2] = [
    ("__rust_alloc", "global_allocator", "global allocator"),
    (
        "__rust_alloc_error_handler",
        "alloc_error_handler",
        "allocation error handler",
    ),
];

impl Checker<'_> {
    /// rustc's diagnostic when `definition` exports the internal symbol that
    /// an earlier definition already registers: `cannot define multiple`
    /// within one crate, a conflict naming both crates otherwise. The
    /// session's program is rustc's `this crate`, which it names first.
    pub(super) fn allocator_conflict(&self, definition: DefId, naming: Naming) -> Option<String> {
        let Naming::Exact(id) = naming else {
            return None;
        };
        let registry = self.facts.foreign.as_deref()?;
        let spelling = registry.names().get(id)?;
        let (_, attribute, item) = REGISTERED.iter().find(|(name, _, _)| {
            crate::support::linkage::internal_symbol(name).is_ok_and(|mangled| mangled == spelling)
        })?;
        let earlier = registry
            .exporter(id)
            .filter(|&earlier| earlier != definition)?;
        let (earlier, later) = (self.crate_of(earlier)?, self.crate_of(definition)?);
        if earlier == later {
            return Some(format!("cannot define multiple {item}s"));
        }
        let package = &self.report.package;
        let program = package.crate_members().len().checked_sub(1);
        let name = |slot: usize| {
            if Some(slot) == program {
                "this crate".to_owned()
            } else {
                package
                    .crate_members()
                    .get(slot)
                    .map_or_else(String::new, |member| {
                        package.names.get(member.name).to_owned()
                    })
            }
        };
        // rustc meets this crate's own first, then its dependencies in order.
        let (first, second) = if Some(earlier) == program {
            (earlier, later)
        } else if Some(later) == program {
            (later, earlier)
        } else {
            (earlier.min(later), earlier.max(later))
        };
        Some(format!(
            "the `#[{attribute}]` in {} conflicts with {item} in: {}",
            name(first),
            name(second)
        ))
    }
    /// The session crate that declares `definition`: its index among the
    /// crate members, the host first, or 0 in a session of one crate.
    fn crate_of(&self, definition: DefId) -> Option<usize> {
        let node = self
            .report
            .resolution
            .definitions
            .get(definition.0 as usize)?
            .node?;
        Some(self.report.package.file_crate(node.file).0 as usize)
    }
}
