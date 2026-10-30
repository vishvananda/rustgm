//! Validate source optimization restrictions without inferring invocation permission.
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
mod restrictions;
use restrictions::Restriction;
#[cfg(test)]
#[path = "../../../tests/invariants/mir_boundaries.rs"]
mod tests;
#[cfg(test)]
#[path = "../../../tests/invariants/writable_boundaries.rs"]
mod writable_tests;
struct Entry {
    image: Arc<OriginalImage>,
    span: Span,
    bound: bool,
}
/// A once-per-declaration draft, consumed by the existing real definition walk.
/// There is deliberately no absence query or optimizer permission represented here.
#[derive(Default)]
pub(in crate::sema) struct Draft {
    entries: HashMap<(NodeKey, Restriction), Entry>,
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
        let error = |message| Error::at(declaration.file, attribute.span.start, message);
        reserve_source_contracts(facts, 1, 0)?;
        let key = (declaration, restriction);
        if self.entries.contains_key(&key) {
            return Err(error(restriction.duplicate_error()));
        }
        let growth = Growth::map::<(NodeKey, Restriction), Entry>(
            self.entries.len(),
            self.entries.capacity(),
            1,
        );
        reserve_source_contracts(facts, growth.work, growth.bytes)?;
        self.entries
            .try_reserve(1)
            .map_err(|_| error("optimization boundary draft allocation failed"))?;
        let unit = report.package.unit(declaration.file);
        self.entries.insert(
            key,
            Entry {
                image: Arc::clone(unit.source.original_image()),
                span: attribute.span,
                bound: closure,
            },
        );
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
                    .saturating_mul(Restriction::ALL.len())
                    .saturating_add(self.entries.len()),
                0,
            )?;
        }
        Ok(())
    }
    pub(in crate::sema) fn bind(
        &mut self,
        report: &Report,
        index: usize,
        definition: &Definition,
    ) -> Result<(), Error> {
        if self.entries.is_empty() {
            return Ok(());
        }
        let Some(key) = definition.node else {
            return Ok(());
        };
        for restriction in Restriction::ALL {
            let Some(entry) = self.entries.get_mut(&(key, restriction)) else {
                continue;
            };
            let actual = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::unlocated("too many optimization boundary definitions"))?,
            );
            if report
                .resolution
                .definitions
                .get(index)
                .is_none_or(|current| !std::ptr::eq(current, definition))
                || definition.kind != DefKind::Function
                || definition.namespace != Namespace::Value
                || report.resolution.foreign_owners.contains_key(&actual)
                || !Arc::ptr_eq(
                    &entry.image,
                    report.package.unit(key.file).source.original_image(),
                )
            {
                return Err(Error::at(
                    key.file,
                    entry.span.start,
                    restriction.binding_error(),
                ));
            }
            entry.bound = true;
        }
        Ok(())
    }
    pub(in crate::sema) fn finish(&self) -> Result<(), Error> {
        if self.entries.values().any(|entry| !entry.bound) {
            return Err(Error::unlocated(
                "optimization boundary lacks its actual function declaration",
            ));
        }
        Ok(())
    }
}
