//! The current frontend never performs Rust MIR inlining, including imported bodies.
//! Validate its source prohibition without changing independent native inline hints.
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
        ast::Kind,
        attribute::{Attribute, Input},
        item::ItemKind,
        token_tree::{Forest, Span},
    },
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../tests/invariants/mir_boundaries.rs"]
mod tests;
struct Entry {
    image: Arc<OriginalImage>,
    span: Span,
    bound: bool,
}
/// A once-per-declaration draft, consumed by the existing real definition walk.
/// There is deliberately no absence query or optimizer permission represented here.
#[derive(Default)]
pub(in crate::sema) struct Draft {
    entries: HashMap<NodeKey, Entry>,
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
        facts.require_lint_context(report)?;
        facts.reserve_must_use_source(declaration, attribute.span)?;
        let error = |message| Error::at(declaration.file, attribute.span.start, message);
        if attribute.unsafe_wrapper || attribute.input != Input::Empty {
            return Err(error("rustc_no_mir_inline requires a plain word attribute"));
        }
        let name = forest
            .tree(attribute.path.start)
            .ok_or_else(|| error("MIR inline boundary lost its actual name"))?
            .span;
        if !report.package.requested_internal_attributes()
            && !allowances.allows_at(
                declaration.file,
                name,
                &crate::resolve::bindings::FeatureName::new("rustc_attrs").map_err(error)?,
            )?
        {
            return Err(error(
                "rustc_no_mir_inline requires its rustc_attrs request",
            ));
        }
        let unit = report.package.unit(declaration.file);
        if owner.file != declaration.file
            || unit
                .owner_indices
                .get(owner.node.get() as usize)
                .copied()
                .flatten()
                .and_then(|index| unit.configured.owners.get(index.get() as usize - 1))
                .is_none_or(|row| {
                    !row.enabled || row.node != owner.node || !row.attributes.contains(&effect)
                })
            || !matches!(unit.configured.ast.node(declaration.node).kind,Kind::Item(index)
                if matches!(unit.configured.ast.items[index as usize].kind,ItemKind::Function{..}))
        {
            return Err(error(
                "rustc_no_mir_inline requires its active function or method owner",
            ));
        }
        let actual = unit
            .retained_attributes(effect..effect + 1)
            .next()
            .ok_or_else(|| error("MIR inline boundary lost its configured effect"))?;
        if actual.0 != attribute
            || actual.0.name(actual.1, unit.source.text()) != Some("rustc_no_mir_inline")
        {
            return Err(error("MIR inline boundary changed its actual attribute"));
        }
        let mut node = owner.node;
        while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
            reserve_source_contracts(facts, 1, 0)?;
            node = value;
        }
        if node != declaration.node {
            return Err(error("MIR inline boundary changed its actual declaration"));
        }
        reserve_source_contracts(facts, 1, 0)?;
        if self.entries.contains_key(&declaration) {
            return Err(error("duplicate rustc_no_mir_inline attribute"));
        }
        let growth = Growth::map::<NodeKey, Entry>(self.entries.len(), self.entries.capacity(), 1);
        reserve_source_contracts(facts, growth.work, growth.bytes)?;
        self.entries
            .try_reserve(1)
            .map_err(|_| error("MIR inline boundary draft allocation failed"))?;
        self.entries.insert(
            declaration,
            Entry {
                image: Arc::clone(unit.source.original_image()),
                span: attribute.span,
                bound: false,
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
        let Some(key) = definition.node else {
            return Ok(());
        };
        let Some(entry) = self.entries.get_mut(&key) else {
            return Ok(());
        };
        let actual = DefId(
            u32::try_from(index)
                .map_err(|_| Error::unlocated("too many MIR inline boundary definitions"))?,
        );
        if definition.kind != DefKind::Function
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
                "rustc_no_mir_inline requires its actual non-foreign function or method",
            ));
        }
        entry.bound = true;
        Ok(())
    }
    pub(in crate::sema) fn finish(&self) -> Result<(), Error> {
        if self.entries.values().any(|entry| !entry.bound) {
            return Err(Error::unlocated(
                "MIR inline boundary lacks its actual function declaration",
            ));
        }
        Ok(())
    }
}
