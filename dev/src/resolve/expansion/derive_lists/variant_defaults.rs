//! An enum choice belongs to its selected source Default and configured helper.
use super::{Builtin, Engine, Error, NodeKey, Sources};
use crate::{
    expand::configured::Effect,
    resolve::{package::Package, unit::Unit},
    syntax::{
        ast::{Kind, NodeId},
        attribute::Input,
        item::{ItemKind, Shape},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Choice {
    pub variant: NodeId,
    pub effect: usize,
    pub name: crate::syntax::token_tree::Span,
}

fn charge(package: &Package, owner: NodeKey, work: usize, bytes: usize) -> Result<(), Error> {
    package
        .source_core()
        .ok_or_else(|| Error::at(owner.file, 0, "Default lost its source provider"))?
        .macros
        .charge(work, bytes)
        .map_err(|message| Error::at(owner.file, 0, message))
}

fn underlying(unit: &Unit, mut node: NodeId) -> NodeId {
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
        node = value;
    }
    node
}

fn marker(package: &Package, key: NodeKey) -> Result<Option<usize>, Error> {
    let unit = package.unit(key.file);
    let Some(index) = unit
        .owner_indices
        .get(key.node.get() as usize)
        .copied()
        .flatten()
    else {
        return Ok(None);
    };
    let owner = unit
        .configured
        .owners
        .get(index.get() as usize - 1)
        .ok_or_else(|| Error::at(key.file, 0, "Default helper owner disappeared"))?;
    if owner.node != key.node
        || !owner.enabled
        || owner.attributes.end > unit.configured.effects.len()
    {
        return Err(Error::at(
            key.file,
            0,
            "Default helper configured owner changed",
        ));
    }
    let mut found = None;
    let mut non_exhaustive = false;
    for effect in owner.attributes.clone() {
        charge(package, key, 1, 0)?;
        let actual = &unit.configured.effects[effect];
        if actual.attribute.effect != Effect::Keep {
            continue;
        }
        let capture = unit
            .configured
            .ast
            .attributes
            .get(actual.capture as usize)
            .ok_or_else(|| Error::at(key.file, 0, "Default helper capture disappeared"))?;
        let attribute = actual.attribute.attribute;
        let name = attribute.name(&capture.forest, unit.source.text());
        charge(package, key, name.map_or(0, str::len), 0)?;
        non_exhaustive |= name == Some("non_exhaustive");
        if name != Some("default") {
            continue;
        }
        let source = unit.configured.ast.node(key.node).span;
        let token = capture
            .forest
            .tree(attribute.path.start)
            .ok_or_else(|| Error::at(key.file, 0, "default helper name disappeared"))?
            .span;
        if attribute.span.start < source.start
            || attribute.span.end > source.end
            || token.start < attribute.span.start
            || token.end > attribute.span.end
        {
            return Err(Error::at(
                key.file,
                attribute.span.start,
                "default helper capture belongs to another source owner",
            ));
        }
        if attribute.input != Input::Empty || attribute.unsafe_wrapper {
            return Err(Error::at(
                key.file,
                attribute.span.start,
                "default helper requires a word attribute",
            ));
        }
        if found.replace(effect).is_some() {
            return Err(Error::at(
                key.file,
                attribute.span.start,
                "default variant has multiple default attributes",
            ));
        }
    }
    if found.is_some() && non_exhaustive {
        return Err(Error::at(
            key.file,
            0,
            "default variant cannot be non_exhaustive",
        ));
    }
    Ok(found)
}

fn actual_item(package: &Package, owner: NodeKey) -> Result<(NodeId, ItemKind), Error> {
    charge(package, owner, 1, 0)?;
    let unit = package.unit(owner.file);
    let root = underlying(unit, owner.node);
    let Kind::Item(index) = unit.configured.ast.node(root).kind else {
        return Err(Error::at(owner.file, 0, "Default requires its actual item"));
    };
    Ok((root, unit.configured.ast.items[index as usize].kind))
}

pub(super) fn requires_choice(package: &Package, owner: NodeKey) -> Result<bool, Error> {
    Ok(matches!(
        actual_item(package, owner)?.1,
        ItemKind::Enum { .. }
    ))
}

/// A completed enum Default cannot erase its required choice to skip validation.
pub(super) fn replay(
    report: &crate::resolve::expansion::Report,
    entry: &super::Transaction,
    definition: &crate::resolve::expansion::Definition,
) -> Result<(), Error> {
    let failure = || {
        Error::at(
            entry.owner.file,
            entry.attribute.attribute.span.start,
            "source Default choice or helper receipt changed",
        )
    };
    if !matches!(definition.body, super::Body::Builtin(Builtin::Default))
        || requires_choice(&report.package, entry.owner)? != entry.enum_default.is_some()
    {
        return Err(failure());
    }
    if entry.enum_default.is_some() {
        let helpers = definition
            .header
            .as_ref()
            .ok_or_else(failure)?
            .derive_helpers(Builtin::Default)?;
        charge(
            &report.package,
            entry.owner,
            helpers.len().saturating_mul(8),
            0,
        )?;
        if !helpers.iter().any(|helper| helper.name() == "default") {
            return Err(failure());
        }
    }
    Ok(())
}

/// Walk only this actual item, retaining no second owner map or source parse.
pub(super) fn select(package: &Package, owner: NodeKey) -> Result<Option<Choice>, Error> {
    let unit = package.unit(owner.file);
    let (root, kind) = actual_item(package, owner)?;
    let enumeration = matches!(kind, ItemKind::Enum { .. });
    charge(package, owner, 1, size_of::<Vec<(NodeId, bool)>>())?;
    let mut pending = Vec::new();
    push(package, owner, &mut pending, owner.node, false)?;
    let mut selected = None;
    while let Some((node, direct)) = pending.pop() {
        charge(package, owner, 1, 0)?;
        if unit.configured.active.get(node.get() as usize) != Some(&true) {
            continue;
        }
        let actual = underlying(unit, node);
        if matches!(unit.configured.ast.node(node).kind, Kind::Attributed { .. })
            && let Some(effect) = marker(package, NodeKey { node, ..owner })?
        {
            match unit.configured.ast.node(actual).kind {
                Kind::Variant(index) if direct => {
                    let variant = unit.configured.ast.variants[index as usize];
                    if variant.shape != Shape::Unit {
                        return Err(Error::at(
                            owner.file,
                            0,
                            "source enum Default requires a unit variant; default_field_values needs its wider recipe",
                        ));
                    }
                    if selected
                        .replace(Choice {
                            variant: node,
                            effect,
                            name: variant.name,
                        })
                        .is_some()
                    {
                        return Err(Error::at(
                            owner.file,
                            0,
                            "source enum Default has multiple default variants",
                        ));
                    }
                }
                Kind::Variant(_) => {}
                _ => {
                    return Err(Error::at(
                        owner.file,
                        0,
                        "default helper requires an enum variant owner",
                    ));
                }
            }
        }
        unit.configured.ast.try_for_each_child(node, &mut |child| {
            let top = node == root
                && matches!(
                    unit.configured.ast.node(underlying(unit, child)).kind,
                    Kind::Variant(_)
                );
            let attributed = matches!(unit.configured.ast.node(node).kind, Kind::Attributed { .. });
            push(
                package,
                owner,
                &mut pending,
                child,
                top || direct && attributed,
            )
        })?;
    }
    if enumeration && selected.is_none() {
        return Err(Error::at(
            owner.file,
            0,
            "source enum Default requires exactly one default variant",
        ));
    }
    Ok(selected)
}

fn push(
    package: &Package,
    owner: NodeKey,
    pending: &mut Vec<(NodeId, bool)>,
    node: NodeId,
    direct: bool,
) -> Result<(), Error> {
    let growth = crate::support::retained::Growth::vector::<(NodeId, bool)>(
        pending.len(),
        pending.capacity(),
        1,
    );
    charge(package, owner, growth.work.saturating_add(1), growth.bytes)?;
    pending
        .try_reserve(1)
        .map_err(|_| Error::at(owner.file, 0, "Default selection allocation failed"))?;
    pending.push((node, direct));
    Ok(())
}

impl<S: Sources> Engine<'_, S> {
    pub(super) fn reserve_default_helper(
        &mut self,
        index: usize,
        choice: Choice,
    ) -> Result<(), Error> {
        let owner = self.derive_entry(index)?.owner;
        let transactions = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or_else(|| self.calls[index].diagnostic("Default helper ledger disappeared"))?;
        if transactions
            .index
            .contains_key(&(owner.file, choice.effect))
        {
            return Err(self.calls[index].diagnostic("Default helper effect is already owned"));
        }
        let growth = crate::support::retained::Growth::map::<(super::FileId, usize), usize>(
            transactions.index.len(),
            transactions.index.capacity(),
            1,
        );
        charge(
            &self.loader.package,
            owner,
            growth.work.saturating_add(1),
            growth.bytes,
        )?;
        self.attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.derives.as_deref_mut())
            .ok_or_else(|| Error::unlocated("Default helper ledger disappeared"))?
            .index
            .try_reserve(1)
            .map_err(|_| Error::at(owner.file, 0, "Default helper index allocation failed"))
    }

    pub(super) fn selected_default(
        &self,
        index: usize,
        definition: usize,
    ) -> Result<Option<Choice>, Error> {
        self.checked_derive_definition(definition, Builtin::Default)?;
        let entry = self.derive_entry(index)?;
        let choice = select(&self.loader.package, entry.owner)?;
        if choice.is_some() {
            let header = self.definitions[definition]
                .header
                .as_ref()
                .ok_or_else(|| {
                    self.calls[index].diagnostic("Default helper declaration is missing")
                })?;
            let helpers = header.derive_helpers(Builtin::Default)?;
            charge(
                &self.loader.package,
                entry.owner,
                helpers.len().saturating_mul(8),
                0,
            )?;
            if !helpers.iter().any(|helper| helper.name() == "default") {
                return Err(self.calls[index]
                    .diagnostic("selected Default does not declare the default helper"));
            }
        }
        Ok(choice)
    }

    pub(in crate::resolve::expansion) fn default_helper(
        &self,
        key: NodeKey,
        effect: usize,
    ) -> Result<bool, Error> {
        let Some(transactions) = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
        else {
            return Ok(false);
        };
        let Some(&index) = transactions.index.get(&(key.file, effect)) else {
            return Ok(false);
        };
        let entry = &transactions.entries[index];
        let Some(choice) = entry.enum_default else {
            return Ok(false);
        };
        if choice.effect != effect || choice.variant != key.node || entry.output.is_none() {
            return Ok(false);
        }
        super::receipt::current(&self.loader.package, &self.paths.state.facts, entry, true)?;
        Ok(true)
    }
}
