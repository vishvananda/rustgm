//! A completed local trait row owns source identity even when the marker is absent.
use super::retention_work::Budget;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    sema::model::Declarations,
    support::{hash::HashMap, source::OriginalImage},
    syntax::{ast::Kind, item::ItemKind, token_tree::Span},
};
use std::sync::Arc;
#[derive(Clone, Copy)]
pub(super) enum Policy {
    Unsafe,
    Applicable,
}
impl Policy {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Unsafe => "rustc_unsafe_specialization_marker",
            Self::Applicable => "rustc_specialization_trait",
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct Marker {
    pub owner: NodeKey,
    pub effect: usize,
    pub span: Span,
    pub name: Span,
}
pub(super) struct Owner {
    pub key: NodeKey,
    pub syntax: Kind,
    pub effects: std::ops::Range<usize>,
}
pub(super) struct Row {
    pub image: Arc<OriginalImage>,
    pub syntax: ItemKind,
    pub owners: Vec<Owner>,
    pub marker: Option<Marker>,
    pub applicable: Option<Marker>,
}
#[derive(Default)]
pub(in crate::sema) struct LocalMarkers {
    pub(super) sources: HashMap<NodeKey, Row>,
    pub(super) definitions: HashMap<DefId, NodeKey>,
    pub(super) complete: bool,
    pub(super) has_applicable: bool,
    work: Budget,
}
impl LocalMarkers {
    pub(super) fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.work
            .charge(steps, bytes)
            .map_err(|_| "specialization marker work or storage budget exceeded")
    }
    pub(super) fn budget(&self) -> Result<Budget, &'static str> {
        self.work.alias()
    }
    pub(super) fn row<'a>(
        &'a mut self,
        report: &Report,
        key: NodeKey,
    ) -> Result<&'a mut Row, Error> {
        self.charge(1, 0)
            .map_err(|message| Error::at(key.file, 0, message))?;
        let syntax = trait_syntax(report, key)?;
        if !self.sources.contains_key(&key) {
            let growth = crate::support::retained::Growth::map::<NodeKey, Row>(
                self.sources.len(),
                self.sources.capacity(),
                1,
            );
            self.charge(growth.work, growth.bytes)
                .map_err(|message| Error::at(key.file, 0, message))?;
            self.sources.insert(
                key,
                Row {
                    syntax,
                    owners: Vec::new(),
                    marker: None,
                    applicable: None,
                    image: Arc::clone(report.package.unit(key.file).source.original_image()),
                },
            );
        }
        self.sources
            .get_mut(&key)
            .ok_or_else(|| Error::at(key.file, 0, "specialization trait row disappeared"))
    }
    #[cfg(test)]
    pub(in crate::sema) fn storage(&self) -> (usize, usize, usize, usize) {
        let (steps, bytes) = self.work.usage();
        (self.sources.len(), self.sources.capacity(), steps, bytes)
    }
}
pub(super) fn trait_syntax(report: &Report, key: NodeKey) -> Result<ItemKind, Error> {
    let invalid = || {
        Error::at(
            key.file,
            0,
            "specialization marker requires its actual local trait",
        )
    };
    let unit = report
        .package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(invalid)?;
    let node = unit
        .configured
        .ast
        .nodes
        .get(key.node.get() as usize)
        .ok_or_else(invalid)?;
    let Kind::Item(index) = node.kind else {
        return Err(invalid());
    };
    let item = unit
        .configured
        .ast
        .items
        .get(index as usize)
        .ok_or_else(invalid)?;
    if unit.include_context.is_some()
        || unit.configured.active.get(key.node.get() as usize) != Some(&true)
        || !matches!(item.kind, ItemKind::Trait { .. })
    {
        return Err(invalid());
    }
    Ok(item.kind)
}
pub(super) fn owner(facts: &mut Declarations) -> Result<&mut LocalMarkers, Error> {
    let common = facts.must_use_contracts.get_or_insert_default();
    if common.specialization.is_none() {
        // The outer cold contract owner retains failed preflight debt as well.
        common.work.contract_input_bytes = common
            .work
            .contract_input_bytes
            .saturating_add(size_of::<LocalMarkers>());
        if common.work.contract_input_bytes > 64 * 1024 * 1024 {
            return Err(Error::unlocated(
                "specialization marker owner exceeds storage budget",
            ));
        }
        common.specialization = Some(Box::default());
    }
    common
        .specialization
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("specialization marker owner disappeared"))
}
