//! Authenticate retained source owners without parsing their attributes again.
use super::{Owner, Producer, Tail, Violation, Work, invalid};
use crate::resolve::{
    expansion::Report,
    package::Package,
    unit::{FileId, NodeKey},
};
use std::cell::Cell;
use std::sync::Arc;

const OWNER_RECEIPTS: usize = 32;

#[derive(Clone, Copy)]
struct Receipt {
    tail: Tail,
    steps: usize,
}

/// Completed reads belong to one borrow of the actual sealed producer and syntax.
pub(crate) struct OwnerReads<'a> {
    report: &'a Report,
    receipts: Vec<Option<Receipt>>,
}

impl<'a> OwnerReads<'a> {
    pub(crate) const fn new(report: &'a Report) -> Self {
        Self {
            report,
            receipts: Vec::new(),
        }
    }

    pub(crate) const fn report(&self) -> &'a Report {
        self.report
    }

    pub(crate) fn validate_owner(&mut self, key: NodeKey) -> Result<(), Violation> {
        let producer = self
            .report
            .lint_producer()
            .ok_or_else(|| invalid(key, "source header owner has no producer"))?;
        let package = &self.report.package;
        let (tail, owner) = producer.read_owner(package, key)?;
        if !producer.sealed {
            return owner.validate(package, &producer.reads);
        }
        let slot = tail.0.get() as usize % OWNER_RECEIPTS;
        let before = producer.reads.get();
        if let Some(receipt) = self.receipts.get(slot).copied().flatten()
            && receipt.tail == tail
        {
            let mut charged = before;
            if charged.charge(key, receipt.steps, 0).is_ok() {
                producer.reads.set(charged);
                return Ok(());
            }
            // Exhaustion must report the first ordered failed charge.
            return owner.validate(package, &producer.reads);
        }
        owner.validate(package, &producer.reads)?;
        let steps = producer.reads.get().steps.saturating_sub(before.steps);
        if self.receipts.is_empty() {
            if self.receipts.try_reserve_exact(OWNER_RECEIPTS).is_err() {
                return Ok(());
            }
            self.receipts.resize(OWNER_RECEIPTS, None);
        }
        self.receipts[slot] = Some(Receipt { tail, steps });
        Ok(())
    }
}

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
    pub(in crate::resolve::lint_policy) fn validate(
        &self,
        package: &Package,
        work: &Cell<Work>,
    ) -> Result<(), Violation> {
        let mut charged = work.get();
        let result = charged.charge(self.key, 1 + self.effects.len(), 0);
        work.set(charged);
        result?;
        let owner = configured(package, self.key)?;
        let unit = package.unit(self.key.file);
        let node = unit.configured.ast.node(self.key.node);
        if owner.attributes != self.effects
            || !Arc::ptr_eq(&self.image, unit.source.original_image())
            || !matches!(node.kind, crate::syntax::ast::Kind::Attributed {attributes,value} if (attributes,value,node.span) == self.header)
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
        let mut cursor = 0_usize;
        for witness in &self.parsed.witnesses {
            let mut charged = work.get();
            let result = charged.charge(self.key, 1 + witness.tree_count as usize, 0);
            work.set(charged);
            result?;
            let (index, attribute, forest) = actual
                .next()
                .ok_or_else(|| invalid(self.key, "lint source attribute witness changed"))?;
            let covered = super::attribute_changes::coverage(
                forest,
                attribute,
                unit.source.text(),
                self.key,
            )?;
            let end = cursor
                .checked_add(witness.tree_count as usize)
                .ok_or_else(|| invalid(self.key, "lint witness tree extent overflow"))?;
            let trees = self
                .parsed
                .trees
                .get(cursor..end)
                .ok_or_else(|| invalid(self.key, "lint witness trees are missing"))?;
            if index != witness.effect as usize
                || attribute != witness.attribute
                || covered.count != witness.tree_count
                || unit.configured.effects[index].capture != witness.capture
                || trees
                    .iter()
                    .zip(
                        covered
                            .ranges
                            .into_iter()
                            .flat_map(|range| range.start..range.end),
                    )
                    .any(|((index, tree), expected)| {
                        *index != expected || forest.tree(expected) != Some(*tree)
                    })
            {
                return Err(invalid(self.key, "lint source attribute witness changed"));
            }
            cursor = end;
        }
        if actual.next().is_some() || cursor != self.parsed.trees.len() {
            return Err(invalid(
                self.key,
                "lint source gained an unparsed attribute or tree",
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
            changes |= record.parsed.has_policy();
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
