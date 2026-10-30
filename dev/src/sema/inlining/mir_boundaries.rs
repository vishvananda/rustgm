//! Validate transient source restrictions/advisories without invocation permission.
//! Neither Rust MIR inlining nor LLVM Writable assumptions exist in this backend.
use crate::{
    resolve::{
        expansion::{MacroAllowanceSession, Report},
        model::{DefId, DefKind, Definition, Namespace},
        package::Error,
        unit::NodeKey,
    },
    sema::{header_attributes::reserve_source_contracts, model::Declarations},
    support::{hash::HashMap, retained::Growth, source::OriginalImage},
    syntax::{
        attribute::Attribute,
        token_tree::{Forest, Span},
    },
};
use std::sync::Arc;
mod advisory_inputs;
mod boundary_entries;
mod restrictions;
use boundary_entries::Entry;
use restrictions::Restriction;
#[cfg(test)]
#[path = "../../../tests/invariants/cell_annotation_cases.rs"]
mod cell_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/advisory_methods.rs"]
mod const_item_tests;
#[cfg(test)]
#[path = "../../../tests/invariants/mir_boundaries.rs"]
mod tests;
#[cfg(test)]
#[path = "../../../tests/invariants/writable_boundaries.rs"]
mod writable_tests;
/// One lookup per reached definition; kinds are visited only on touched owners.
/// No absence query, diagnostic alias or optimizer permission survives this draft.
#[derive(Default)]
pub(in crate::sema) struct Draft {
    entries: HashMap<NodeKey, Entry>,
    annotations: usize,
}
impl Draft {
    pub(in crate::sema) fn consume(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        (owner, declaration): (NodeKey, NodeKey),
        (attribute, effect): (Attribute, usize),
        forest: &Forest,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let (restriction, closure) = restrictions::authenticate(
            report,
            facts,
            (owner, declaration),
            (attribute, effect),
            forest,
            allowances,
        )?;
        let error = |message: &str| Error::at(declaration.file, attribute.span.start, message);
        let image = report
            .package
            .unit(declaration.file)
            .source
            .original_image();
        reserve_source_contracts(facts, 1, 0)?;
        if let Some(entry) = self.entries.get_mut(&declaration) {
            if !Arc::ptr_eq(&entry.image, image) {
                return Err(error("source advisory draft incarnation changed"));
            }
            let count = entry.annotation_count();
            reserve_source_contracts(facts, count, 0)?;
            if entry
                .annotations()
                .any(|row| row.restriction == restriction)
            {
                if restriction == Restriction::Confusables {
                    // Each repeated list was fully validated above. Its names
                    // combine for future diagnostics, never successful lookup.
                    entry.bound = false;
                    return Ok(());
                }
                return Err(error(restriction.duplicate_error()));
            }
            entry.append(facts, declaration, restriction, attribute.span)?;
        } else {
            let growth =
                Growth::map::<NodeKey, Entry>(self.entries.len(), self.entries.capacity(), 1);
            reserve_source_contracts(facts, growth.work, growth.bytes)?;
            self.entries
                .try_reserve(1)
                .map_err(|_| error("source advisory draft allocation failed"))?;
            self.entries.insert(
                declaration,
                Entry::new(Arc::clone(image), restriction, attribute.span, closure),
            );
        }
        self.annotations += 1;
        Ok(())
    }
    pub(in crate::sema) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub(in crate::sema) fn prepare_binding(
        &self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), Error> {
        if !self.entries.is_empty() {
            facts.require_lint_context(report)?;
            reserve_source_contracts(
                facts,
                report
                    .resolution
                    .definitions
                    .len()
                    .saturating_add(self.annotations.saturating_mul(16))
                    .saturating_add(self.entries.capacity()),
                0,
            )?;
        }
        Ok(())
    }
    pub(in crate::sema) fn bind(
        &mut self,
        report: &Report,
        facts: &Declarations,
        index: usize,
        definition: &Definition,
    ) -> Result<(), Error> {
        let Some(key) = definition.node else {
            return Ok(());
        };
        let Some(entry) = self.entries.get_mut(&key) else {
            return Ok(());
        };
        facts.require_lint_context(report)?;
        let actual = DefId(
            u32::try_from(index)
                .map_err(|_| Error::unlocated("too many source advisory definitions"))?,
        );
        if report
            .resolution
            .definitions
            .get(index)
            .is_none_or(|row| !std::ptr::eq(row, definition))
            || !Arc::ptr_eq(
                &entry.image,
                report.package.unit(key.file).source.original_image(),
            )
        {
            return Err(Error::at(
                key.file,
                0,
                "source advisory actual definition or incarnation changed",
            ));
        }
        // Constructors and trait Self share a source item but do not name the
        // canonical type declaration. They cannot complete its annotation.
        if entry.is_nominal()
            && restrictions::nominal_auxiliary(report, facts, actual, definition, key)?
        {
            return Ok(());
        }
        entry.bound = false;
        for annotation in entry.annotations() {
            annotation
                .restriction
                .validate_definition(report, facts, actual, definition, key)
                .map_err(|mut error| {
                    error.file = crate::resolve::package::DiagnosticFile::Source(key.file);
                    error.span = annotation.span;
                    error
                })?;
        }
        entry.bound = true;
        Ok(())
    }
    pub(in crate::sema) fn finish(&self) -> Result<(), Error> {
        if self.entries.values().any(|entry| !entry.bound) {
            return Err(Error::unlocated(
                "source advisory lacks its actual declaration",
            ));
        }
        Ok(())
    }
}
