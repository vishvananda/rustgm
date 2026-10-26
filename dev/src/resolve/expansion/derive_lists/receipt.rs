//! Consumed configuration is authenticated data, never an implementation proof.
use super::*;
use crate::resolve::expansion::Report;

pub(super) fn current(
    package: &super::super::Package,
    entry: &Transaction,
    expanded: bool,
) -> Result<(), Error> {
    if let Some(source) = package.source_core() {
        source
            .macros
            .charge(1 + entry.trees.len() + entry.spelling.len(), 0)
            .map_err(|message| {
                Error::at(
                    entry.owner.file,
                    entry.attribute.attribute.span.start,
                    message,
                )
            })?;
    }
    let failure = || {
        Error::at(
            entry.owner.file,
            entry.attribute.attribute.span.start,
            "source derive configured owner or capture changed",
        )
    };
    let unit = package
        .units
        .get(entry.owner.file.0 as usize)
        .ok_or_else(failure)?;
    let owner = unit
        .owner_indices
        .get(entry.owner.node.get() as usize)
        .copied()
        .flatten()
        .and_then(|owner| unit.configured.owners.get(owner.get() as usize - 1))
        .ok_or_else(failure)?;
    let effect = unit
        .configured
        .effects
        .get(entry.effect)
        .ok_or_else(failure)?;
    let mut expected = entry.attribute;
    expected.effect = if expanded {
        Effect::ExpandedDerive
    } else {
        Effect::Keep
    };
    if !Arc::ptr_eq(unit.source.original_image(), &entry.image)
        || !owner.enabled
        || owner.node != entry.owner.node
        || !owner.attributes.contains(&entry.effect)
        || unit.configured.active.get(entry.owner.node.get() as usize) != Some(&true)
        || unit.source.text().get(
            entry.attribute.attribute.span.start as usize
                ..entry.attribute.attribute.span.end as usize,
        ) != Some(entry.spelling.as_ref())
        || effect.capture != entry.capture
        || effect.attribute != expected
        || unit
            .configured
            .ast
            .attributes
            .get(entry.capture as usize)
            .is_none_or(|capture| capture.forest.nodes() != entry.trees.as_slice())
    {
        return Err(failure());
    }
    Ok(())
}
impl Report {
    pub(crate) fn expanded_derive(&self, owner: NodeKey, effect: usize) -> Result<(), Error> {
        let failure = || {
            Error::at(
                owner.file,
                0,
                "consumed derive lacks its actual source invocation receipt",
            )
        };
        let transactions = self
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or_else(failure)?;
        let source = self.package.source_core().ok_or_else(failure)?;
        source
            .macros
            .charge(1, 0)
            .map_err(|message| Error::at(owner.file, 0, message))?;
        let position = *transactions
            .index
            .get(&(owner.file, effect))
            .ok_or_else(failure)?;
        let entry = &transactions.entries[position];
        current(&self.package, entry, true)?;
        if entry.owner != owner || entry.output.is_none() {
            return Err(failure());
        }
        let invocation = self.invocations.get(entry.invocation).ok_or_else(failure)?;
        let definition = self
            .definitions
            .get(invocation.definition)
            .ok_or_else(failure)?;
        let node = definition.node.ok_or_else(failure)?;
        let header =
            panic_contracts::published(&self.package, node, definition.header.as_ref(), 1)?;
        if invocation.node != owner
            || invocation.original != entry.attribute.attribute.span
            || invocation.parser_calls != 1
            || !matches!(definition.body, Body::Builtin(Builtin::Derive))
            || header.builtin != Some(Builtin::Derive)
            || header.edition != definition.edition
        {
            return Err(failure());
        }
        for index in entry.contexts.clone() {
            source
                .macros
                .charge(1, 0)
                .map_err(|message| Error::at(owner.file, 0, message))?;
            let member = self.invocations.get(index).ok_or_else(failure)?;
            if member.node != owner
                || member.parent != invocation.context
                || member.original != invocation.original
                || member.generated != invocation.generated
            {
                return Err(failure());
            }
            let view = super::super::permission_frames::FrameView {
                package: &self.package,
                definitions: &self.definitions,
                invocations: &self.invocations,
                facts: &self.resolution,
            };
            let definition = self
                .definitions
                .get(member.definition)
                .ok_or_else(failure)?;
            let Body::Builtin(role @ (Builtin::Copy | Builtin::Clone)) = definition.body else {
                return Err(failure());
            };
            let node = definition.node.ok_or_else(failure)?;
            let header =
                panic_contracts::published(&self.package, node, definition.header.as_ref(), 1)?;
            if header.builtin != Some(role)
                || header.edition != definition.edition
                || !view.executed(member.context, definition)
            {
                return Err(failure());
            }
            // No non-Bang allowance frame or feature grant is created here.
        }
        Ok(())
    }
}
