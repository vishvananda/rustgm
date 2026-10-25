//! Authenticate retained source owners without parsing their attributes again.
use super::{Owner, Producer, Violation, Work, invalid};
use crate::resolve::{
    package::Package,
    unit::{FileId, NodeKey},
};
use std::cell::Cell;
use std::sync::Arc;

pub(super) fn configured(
    package: &Package,
    key: NodeKey,
) -> Result<&crate::expand::crate_syntax::Owner, Violation> {
    let unit = package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(|| invalid(key, "foreign lint source file"))?;
    let index = unit
        .owner_indices
        .get(key.node.get() as usize)
        .copied()
        .flatten()
        .ok_or_else(|| invalid(key, "lint source owner was not configured"))?;
    let owner = unit
        .configured
        .owners
        .get(index.get() as usize - 1)
        .ok_or_else(|| invalid(key, "lint source owner index changed"))?;
    if unit.include_context.is_some()
        || !unit
            .configured
            .ast
            .nodes
            .get(key.node.get() as usize)
            .is_some_and(|node| matches!(node.kind, crate::syntax::ast::Kind::Attributed { .. }))
        || owner.node != key.node
        || !owner.enabled
        || unit.configured.active.get(key.node.get() as usize) != Some(&true)
        || owner.attributes.start > owner.attributes.end
        || owner.attributes.end > unit.configured.effects.len()
    {
        return Err(invalid(
            key,
            "lint source owner is not active or has invalid effects",
        ));
    }
    Ok(owner)
}
impl Owner {
    pub(super) fn validate(&self, package: &Package, work: &Cell<Work>) -> Result<(), Violation> {
        let mut charged = work.get();
        let result = charged.charge(self.key, 1 + self.effects.len(), 0);
        work.set(charged);
        result?;
        let owner = configured(package, self.key)?;
        let unit = package.unit(self.key.file);
        if owner.attributes != self.effects
            || !Arc::ptr_eq(&self.image, unit.source.original_image())
        {
            return Err(invalid(
                self.key,
                "lint source incarnation or effect range changed",
            ));
        }
        if unit.configured.effects[self.effects.clone()]
            .iter()
            .any(|effect| effect.capture as usize >= unit.configured.ast.attributes.len())
        {
            return Err(invalid(self.key, "lint source capture is missing"));
        }
        let mut actual = unit.retained_indexed_attributes(self.effects.clone());
        for witness in &self.parsed.witnesses {
            let mut charged = work.get();
            let result = charged.charge(self.key, 1 + witness.trees.len(), 0);
            work.set(charged);
            result?;
            let expected = actual.next();
            if expected.is_none_or(|(index, attribute, forest)| {
                index != witness.effect
                    || attribute != witness.attribute
                    || witness
                        .trees
                        .iter()
                        .any(|(index, tree)| forest.tree(*index) != Some(*tree))
            }) || unit
                .configured
                .effects
                .get(witness.effect)
                .is_none_or(|effect| effect.capture != witness.capture)
            {
                return Err(invalid(self.key, "lint source attribute witness changed"));
            }
        }
        if actual.next().is_some() {
            return Err(invalid(
                self.key,
                "lint source gained an unparsed attribute",
            ));
        }
        Ok(())
    }
}
pub(super) fn validate(
    producer: &Producer,
    package: &Package,
) -> Result<(usize, usize, bool), Violation> {
    let mut attributes = 0;
    let mut entries = 0;
    let mut changes = false;
    for (file, unit) in package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file)
                .map_err(|_| invalid(package_key(package), "lint source file index overflow"))?,
        );
        for owner in &unit.configured.owners {
            if !owner.enabled
                || unit.configured.active.get(owner.node.get() as usize) != Some(&true)
                || owner.attributes.is_empty()
            {
                continue;
            }
            let key = NodeKey {
                file,
                node: owner.node,
            };
            let tail =
                producer.index.get(&key).copied().ok_or_else(|| {
                    invalid(key, "active lint source owner has no produced record")
                })?;
            let record = producer.record(tail, key)?;
            record.validate(package, &producer.reads)?;
            attributes += record.parsed.attributes;
            entries += record.parsed.entries;
            changes |= !record.parsed.changes.is_empty();
        }
    }
    Ok((attributes, entries, changes))
}
fn package_key(package: &Package) -> NodeKey {
    NodeKey {
        file: FileId(0),
        node: package.modules[0].root,
    }
}
