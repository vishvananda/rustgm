//! Retained foreign assertions never replace call safety or constant evaluation.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::{MacroAllowanceSession, Report},
        model::{DefId, DefKind, Namespace},
        package::{Error, ModuleId},
        unit::NodeKey,
    },
    sema::{foreign::MemoryEffect, model::Declarations},
    support::{hash::HashMap, retained::Growth},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::{Forest, Span},
    },
};

#[derive(Default)]
pub(super) struct Draft {
    requests: HashMap<(ModuleId, MemoryEffect), bool>,
    pending: HashMap<NodeKey, (Span, MemoryEffect)>,
}

impl Draft {
    pub(super) fn consume(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        (owner, declaration): (NodeKey, NodeKey),
        (attribute, retained): (Attribute, usize),
        forest: &Forest,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        facts.reserve_must_use_source(declaration, attribute.span)?;
        let error = |message| Error::at(declaration.file, attribute.span.start, message);
        if !attribute.unsafe_wrapper {
            return Err(error("unsafe attribute used without unsafe"));
        }
        if attribute.style != Style::Outer || attribute.input != Input::Empty {
            return Err(error("foreign effect requires a word attribute"));
        }
        let unit = report.package.unit(declaration.file);
        let effect = match attribute.name(forest, unit.source.text()) {
            Some("ffi_const") => MemoryEffect::Const,
            Some("ffi_pure") => MemoryEffect::Pure,
            _ => return Err(error("foreign effect changed its actual attribute name")),
        };
        let ast = &unit.configured.ast;
        if !matches!(ast.node(declaration.node).kind, Kind::Item(index)
            if matches!(ast.items[index as usize].kind, ItemKind::Function { body: None, .. }))
        {
            return Err(error(
                "foreign effect requires an actual foreign function declaration",
            ));
        }
        if !self.requested(report, facts, declaration, effect)?
            && !allowances.allows_attribute(
                owner,
                retained,
                attribute,
                &FeatureName::new(effect.feature()).map_err(error)?,
            )?
        {
            return Err(Error::at(
                declaration.file,
                attribute.span.start,
                format!(
                    "foreign effect requires this crate's {} feature",
                    effect.feature()
                ),
            ));
        }
        if let Some((_, previous)) = self.pending.get(&declaration) {
            return Err(error(if *previous == effect {
                "duplicate foreign effect attribute"
            } else {
                "`#[ffi_const]` function cannot be `#[ffi_pure]`"
            }));
        }
        let growth = Growth::map::<NodeKey, (Span, MemoryEffect)>(
            self.pending.len(),
            self.pending.capacity(),
            1,
        );
        crate::sema::header_attributes::reserve_source_contracts(facts, growth.work, growth.bytes)?;
        self.pending
            .try_reserve(1)
            .map_err(|_| error("foreign effect draft allocation failed"))?;
        self.pending.insert(declaration, (attribute.span, effect));
        Ok(())
    }

    fn requested(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        declaration: NodeKey,
        effect: MemoryEffect,
    ) -> Result<bool, Error> {
        let root = (report.package.file_crate_root(declaration.file), effect);
        if let Some(&requested) = self.requests.get(&root) {
            return Ok(requested);
        }
        let requested = report
            .package
            .requested_ffi_effect(declaration.file, effect.feature())?;
        let growth = Growth::map::<(ModuleId, MemoryEffect), bool>(
            self.requests.len(),
            self.requests.capacity(),
            1,
        );
        crate::sema::header_attributes::reserve_source_contracts(facts, growth.work, growth.bytes)?;
        self.requests
            .try_reserve(1)
            .map_err(|_| Error::unlocated("foreign effect feature cache allocation failed"))?;
        self.requests.insert(root, requested);
        Ok(requested)
    }

    pub(super) fn bind(mut self, report: &Report, facts: &mut Declarations) -> Result<(), Error> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let owner = &report.resolution;
        let count = self.pending.len();
        let growth = Growth::map::<DefId, MemoryEffect>(0, 0, count);
        crate::sema::header_attributes::reserve_source_contracts(
            facts,
            owner.foreign_owners.len().saturating_add(growth.work),
            growth
                .bytes
                .saturating_add(size_of::<HashMap<DefId, MemoryEffect>>()),
        )?;
        let mut assertions = HashMap::default();
        assertions
            .try_reserve(count)
            .map_err(|_| Error::unlocated("foreign effect assertion allocation failed"))?;
        for &id in owner.foreign_owners.keys() {
            let actual = &owner.definitions[id.0 as usize];
            let Some(key) = actual.node else {
                continue;
            };
            let Some((span, effect)) = self.pending.remove(&key) else {
                continue;
            };
            if actual.kind != DefKind::Function || actual.namespace != Namespace::Value {
                return Err(Error::at(
                    key.file,
                    span.start,
                    "foreign effect requires its actual foreign function",
                ));
            }
            assertions.insert(id, effect);
        }
        if let Some((key, (span, _))) = self
            .pending
            .into_iter()
            .min_by_key(|(key, (span, _))| (key.file.0, span.start))
        {
            return Err(Error::at(
                key.file,
                span.start,
                "foreign effect requires its actual foreign function",
            ));
        }
        facts.must_use_contracts.get_or_insert_default().ffi_effects = Some(Box::new(assertions));
        Ok(())
    }
}

impl Declarations {
    /// The effect assertion on this actual local foreign declaration.
    #[must_use]
    pub fn foreign_memory_effect(&self, definition: DefId) -> Option<MemoryEffect> {
        self.must_use_contracts
            .as_deref()?
            .ffi_effects
            .as_deref()?
            .get(&definition)
            .copied()
    }
}
