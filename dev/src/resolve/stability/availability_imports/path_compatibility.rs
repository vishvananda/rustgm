//! Real path groups preserve terminal identity before checking module prefixes.
use super::{
    Declaration, Error, Imports, NodeKey, Package, PathEntry, Resolution, Selection, Target, owner,
    retained, s,
};
use crate::resolve::{
    expansion::Report,
    model::{DefId, DefKind},
};

pub(super) struct PathUse {
    site: NodeKey,
    terminal: DefId,
    entries: std::ops::Range<usize>,
    kind: PathKind,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::resolve) enum PathKind {
    Import,
    Body(usize),
    Associated(usize),
}
pub(crate) struct Warning<'a> {
    pub site: NodeKey,
    pub prefix: DefId,
    pub message: &'a str,
}
impl Imports {
    pub(in crate::resolve) fn record(
        &mut self,
        site: NodeKey,
        terminal: DefId,
        kind: PathKind,
        selected: &[Selection],
    ) -> Result<(), Error> {
        self.charge(selected.len().saturating_mul(2), 0)?;
        if self.paths_complete {
            return Err(Error::unlocated("availability paths are already completed"));
        }
        let count = selected.iter().try_fold(0usize, |count, selection| {
            let Some(binding) = self.bindings.get(&selection.key) else {
                return Ok(count);
            };
            if selection.binding.target != binding.target {
                return Err(Error::unlocated(
                    "availability path changed its actual selected target",
                ));
            }
            Ok(count + 1)
        })?;
        if count == 0 {
            return Ok(());
        }
        let mut growth = retained::Growth::vector::<PathEntry>(
            self.entries.len(),
            self.entries.capacity(),
            count,
        );
        growth.include(retained::Growth::vector::<PathUse>(
            self.paths.len(),
            self.paths.capacity(),
            1,
        ));
        self.charge(growth.work, growth.bytes)?;
        self.entries
            .try_reserve(count)
            .map_err(|_| Error::unlocated("availability path allocation failed"))?;
        self.paths
            .try_reserve(1)
            .map_err(|_| Error::unlocated("availability group allocation failed"))?;
        let start = self.entries.len();
        for selection in selected {
            let Some(binding) = self.bindings.get(&selection.key).copied() else {
                continue;
            };
            self.entries.push(PathEntry {
                selected: selection.binding,
                key: selection.key,
                binding,
            });
        }
        self.paths.push(PathUse {
            site,
            terminal,
            kind,
            entries: start..self.entries.len(),
        });
        Ok(())
    }
    fn declared_policy(
        &self,
        definition: DefId,
    ) -> Result<Option<(Declaration, &s::DeclarationAvailability)>, Error> {
        self.charge(1, 0)?;
        let Some(mapped) = self.declarations.get(&definition) else {
            return Ok(None);
        };
        let mapped = mapped.ok_or_else(|| {
            Error::unlocated("imported API availability is unknown in its metadata profile")
        })?;
        self.catalog
            .validate_declaration(mapped.handle, definition, mapped.kind)
            .map_err(Error::unlocated)?;
        let coordinate = self
            .catalog
            .declaration_coordinate(mapped.handle)
            .map_err(Error::unlocated)?;
        let artifact = &self.inventory.artifacts()[mapped.artifact];
        let descriptor = artifact
            .source_descriptor()
            .ok_or_else(|| Error::unlocated("availability descriptor disappeared"))?;
        if coordinate.0 != artifact.identity()
            || coordinate.1 != mapped.reference
            || coordinate.2 != descriptor.edition()
        {
            return Err(Error::unlocated(
                "availability declaration changed its canonical origin",
            ));
        }
        let row = self
            .rows(mapped.artifact)?
            .declarations
            .get(mapped.reference.0 as usize)
            .ok_or_else(|| Error::unlocated("availability declaration coverage is missing"))?;
        Ok(Some((mapped, row)))
    }
    fn compatibility_message(&self, definition: DefId) -> Result<Option<&str>, Error> {
        let Some((mapped, row)) = self.declared_policy(definition)? else {
            return Ok(None);
        };
        let Some(policy) = row.regular else {
            return Ok(None);
        };
        let policy = self
            .rows(mapped.artifact)?
            .policies
            .get(policy.0 as usize)
            .ok_or_else(|| Error::unlocated("compatibility policy is absent"))?;
        Ok(match &policy.level {
            s::AvailabilityLevel::StableWithCompatibility { message, .. } => Some(message),
            s::AvailabilityLevel::Stable(_) | s::AvailabilityLevel::Unstable { .. } => None,
        })
    }
    fn path(
        &self,
        package: &Package,
        facts: &Resolution,
        path: &PathUse,
        visit: &mut impl FnMut(Warning<'_>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.charge(1, 0)?;
        if let PathKind::Body(index) | PathKind::Associated(index) = path.kind {
            let reference = facts
                .references
                .get(index)
                .ok_or_else(|| Error::unlocated("availability body reference disappeared"))?;
            if reference.node != path.site
                || !matches!(reference.target,Target::Definition(id)|Target::Associated(id) if id==path.terminal)
            {
                return Err(Error::unlocated(
                    "availability path changed its actual terminal reference",
                ));
            }
        }
        // A note never waives the terminal declaration's independent API gate.
        self.declaration(package, path.site, path.terminal, false)?;
        let message = if matches!(path.kind, PathKind::Associated(_)) {
            None
        } else {
            self.compatibility_message(path.terminal)?
        };
        let entries = self
            .entries
            .get(path.entries.clone())
            .ok_or_else(|| Error::unlocated("availability path range changed"))?;
        for entry in entries {
            self.charge(1, 0)?;
            if facts.bindings.get_key(entry.key) != Some(&entry.selected)
                || entry.selected.target != entry.binding.target
            {
                return Err(Error::unlocated(
                    "availability selected namespace changed after resolution",
                ));
            }
            let module = self
                .declarations
                .get(&entry.binding.target)
                .and_then(Option::as_ref)
                .is_some_and(|mapped| mapped.kind == DefKind::Module);
            if !module {
                self.require_prefix(package, path.site, entry)?;
                continue;
            }
            if self.prefix_allowed(package, path.site, entry)? {
                continue;
            }
            if let Some(message) = message {
                visit(Warning {
                    site: path.site,
                    prefix: entry.binding.target,
                    message,
                })?;
            } else if matches!(path.kind, PathKind::Associated(_)) {
                return Err(Error::at(
                    path.site.file,
                    package
                        .unit(path.site.file)
                        .configured
                        .ast
                        .node(path.site.node)
                        .span
                        .start,
                    "associated module compatibility requires its actual selected-leaf decision",
                ));
            } else {
                self.require_prefix(package, path.site, entry)?;
            }
        }
        Ok(())
    }
    fn prefix_allowed(
        &self,
        package: &Package,
        site: NodeKey,
        entry: &PathEntry,
    ) -> Result<bool, Error> {
        let row = self
            .rows(entry.binding.artifact)?
            .bindings
            .get(entry.binding.index)
            .ok_or_else(|| Error::unlocated("availability binding coverage is missing"))?;
        let direct = entry.binding.source != s::BindingSource::Declaration
            || self.policy_allowed(package, site, entry.binding.artifact, row.regular)?;
        let declared = self.declared_policy(entry.binding.target)?;
        let allowed = match declared {
            Some((mapped, row)) => {
                self.policy_allowed(package, site, mapped.artifact, row.regular)?
            }
            None => true,
        };
        Ok(direct && allowed)
    }
    fn require_prefix(
        &self,
        package: &Package,
        site: NodeKey,
        entry: &PathEntry,
    ) -> Result<(), Error> {
        let row = self
            .rows(entry.binding.artifact)?
            .bindings
            .get(entry.binding.index)
            .ok_or_else(|| Error::unlocated("availability binding coverage is missing"))?;
        if entry.binding.source == s::BindingSource::Declaration {
            self.policy(package, site, entry.binding.artifact, row.regular)?;
        }
        self.declaration(package, site, entry.binding.target, false)
    }
    pub(super) fn complete_paths(&mut self) -> Result<(), Error> {
        if self.paths_complete {
            return Err(Error::unlocated("availability paths completed twice"));
        }
        let count = self.paths.len();
        let log = usize::BITS as usize - count.leading_zeros() as usize;
        self.charge(count.saturating_mul(log.saturating_add(1)), 0)?;
        self.paths.sort_unstable_by_key(|path| {
            (
                !matches!(path.kind, PathKind::Import),
                path.site.file.0,
                path.site.node.get(),
            )
        });
        self.paths_complete = true;
        Ok(())
    }
    pub(in crate::resolve) fn check_uses(
        &self,
        package: &Package,
        facts: &Resolution,
    ) -> Result<(), Error> {
        for path in &self.paths {
            self.path(package, facts, path, &mut |_| Ok(()))?;
        }
        for reference in &facts.references {
            self.charge(1, 0)?;
            if let Target::Definition(definition) | Target::Associated(definition) =
                reference.target
            {
                self.declaration(package, reference.node, definition, false)?;
            }
        }
        Ok(())
    }
    pub(super) fn declaration_deprecation_absent(&self, definition: DefId) -> Result<bool, Error> {
        let Some((mapped, _)) = self.declared_policy(definition)? else {
            return Ok(false);
        };
        // This guarantee belongs to the actual leaf's origin, including reexports.
        Ok(self.inventory.artifacts()[mapped.artifact]
            .source_descriptor()
            .is_some_and(|descriptor| descriptor.profile().module_compatibility()))
    }
}
pub(crate) fn visit_warnings(
    report: &Report,
    site: Option<(NodeKey, DefId)>,
    mut visit: impl FnMut(Warning<'_>) -> Result<(), Error>,
) -> Result<(), Error> {
    let Some(imports) = owner(&report.package, report.resolution.crates.as_deref())? else {
        return Ok(());
    };
    if !imports.paths_complete {
        return Err(Error::unlocated("availability paths are not complete"));
    }
    let (start, end) = if let Some((site, _)) = site {
        let key = (true, site.file.0, site.node.get());
        let start = imports.paths.partition_point(|path| {
            (
                !matches!(path.kind, PathKind::Import),
                path.site.file.0,
                path.site.node.get(),
            ) < key
        });
        let end = start
            + imports.paths[start..].partition_point(|path| {
                path.site == site && !matches!(path.kind, PathKind::Import)
            });
        (start, end)
    } else {
        (
            0,
            imports
                .paths
                .partition_point(|path| matches!(path.kind, PathKind::Import)),
        )
    };
    let log = usize::BITS as usize - imports.paths.len().leading_zeros() as usize;
    imports.charge(log.saturating_mul(2).saturating_add(1), 0)?;
    for path in &imports.paths[start..end] {
        if let Some((site, terminal)) = site
            && path.terminal != terminal
        {
            if matches!(path.kind, PathKind::Associated(_)) {
                // Enabled prefixes retain the old ordinary member route. A denied
                // module already requires its genuine selected-leaf consumer.
                imports.declaration(&report.package, site, terminal, false)?;
                continue;
            }
            return Err(Error::unlocated(
                "checked availability use selected another terminal",
            ));
        }
        imports.path(&report.package, &report.resolution, path, &mut visit)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/compatibility_routes.rs"]
mod tests;
