//! Saved ordinals bind borrowed uses to exact configured effects, trees and origins.
use super::{
    Error, NodeKey, Report,
    demand::{self, Work},
};
use crate::{
    expand::{
        configured::{ConfiguredUse, Effect},
        crate_syntax::AttributeEffect,
        stream::Origin,
    },
    resolve::unit::{FileId, Unit},
    support::source::OriginalImage,
    syntax::token_tree::{Span, Tree, TreeKind},
};
use std::{cell::Cell, ops::Range, sync::Arc};

pub(crate) struct CheckedConfiguredUse<'report> {
    report: &'report Report,
    file: FileId,
    ordinal: usize,
    used: &'report ConfiguredUse,
    owner: NodeKey,
    range: Range<usize>,
    effect: &'report AttributeEffect,
    enabled: bool,
}
impl CheckedConfiguredUse<'_> {
    pub(crate) fn location(&self) -> crate::expand::stream::Location {
        crate::expand::stream::Location {
            file: self.file,
            span: self.used.predicate.span,
        }
    }
    pub(super) fn file(&self) -> FileId {
        self.file
    }
    pub(super) fn effect(&self) -> usize {
        self.used.effect as usize
    }
    pub(super) fn belongs_to(&self, report: &Report) -> bool {
        std::ptr::eq(self.report, report)
    }
    pub(crate) fn in_attribute_span(&self) -> bool {
        let span = self.effect.attribute.attribute.span;
        self.used.predicate.span.start >= span.start && self.used.predicate.span.end <= span.end
    }
}
pub(super) fn checked<'report>(
    report: &'report Report,
    file: FileId,
    ordinal: usize,
    work: &Cell<Work>,
) -> Result<CheckedConfiguredUse<'report>, Error> {
    demand::charge(work, file, 1, 0)?;
    let fail = || {
        Error::at(
            file,
            0,
            "cfg use lost its actual saved ordinal/effect/capture/owner",
        )
    };
    let unit = report.package.units.get(file.0 as usize).ok_or_else(fail)?;
    let used = unit
        .configured
        .cfg_uses
        .as_ref()
        .and_then(|uses| uses.get(ordinal))
        .ok_or_else(fail)?;
    let index = used.effect as usize;
    let effect = unit.configured.effects.get(index).ok_or_else(fail)?;
    let owner = owner(unit, index, file, work)?;
    if !matches!(effect.attribute.effect, Effect::Cfg(_) | Effect::CfgAttr(_))
        || unit
            .configured
            .ast
            .attributes
            .get(effect.capture as usize)
            .is_none()
        || used.predicate.span.start > used.predicate.span.end
        || used.predicate.span.end as usize > unit.source.text().len()
    {
        return Err(fail());
    }
    Ok(CheckedConfiguredUse {
        report,
        file,
        ordinal,
        used,
        owner: NodeKey {
            file,
            node: owner.node,
        },
        range: owner.attributes.clone(),
        effect,
        enabled: owner.enabled,
    })
}
fn owner<'unit>(
    unit: &'unit Unit,
    effect: usize,
    file: FileId,
    work: &Cell<Work>,
) -> Result<&'unit crate::expand::crate_syntax::Owner, Error> {
    demand::charge(
        work,
        file,
        1 + unit.configured.owners.len().checked_ilog2().unwrap_or(0) as usize,
        0,
    )?;
    let index = unit
        .configured
        .owners
        .partition_point(|owner| owner.attributes.end <= effect);
    let owner = unit
        .configured
        .owners
        .get(index)
        .filter(|owner| owner.attributes.contains(&effect))
        .ok_or_else(|| Error::at(file, 0, "cfg effect has no actual configured owner"))?;
    if unit
        .owner_indices
        .get(owner.node.get() as usize)
        .copied()
        .flatten()
        .is_none_or(|saved| saved.get() as usize != index + 1)
    {
        return Err(Error::at(file, 0, "cfg configured owner index changed"));
    }
    Ok(owner)
}
pub(super) struct Witness {
    pub body: usize,
    pub path: usize,
    pub file: FileId,
    pub effect: usize,
    owner: NodeKey,
    range: Range<usize>,
    capture: u32,
    attribute: crate::expand::configured::ConfiguredAttribute,
    trees: Vec<Tree>,
    forest_end: u32,
    uses: Vec<Use>,
}
struct Use {
    ordinal: usize,
    value: ConfiguredUse,
    tree: u32,
    origins: Vec<OriginWitness>,
}
pub(super) struct OriginWitness {
    file: FileId,
    offset: u32,
    image: Arc<OriginalImage>,
    origin: Option<(usize, Origin)>,
}
impl Witness {
    #[cfg(test)]
    pub(super) fn retained_payload(&self) -> usize {
        crate::support::retained::vector(&self.trees)
            + crate::support::retained::vector(&self.uses)
            + self
                .uses
                .iter()
                .map(|used| crate::support::retained::vector(&used.origins))
                .sum::<usize>()
    }
    pub(super) fn capture<'unit>(
        unit: &'unit Unit,
        file: FileId,
        effect: usize,
        path: usize,
        work: &Cell<Work>,
        other: impl Fn(FileId) -> Option<&'unit Unit>,
    ) -> Result<Self, Error> {
        let owner = owner(unit, effect, file, work)?;
        let entry = &unit.configured.effects[effect];
        if !matches!(entry.attribute.effect, Effect::Cfg(_) | Effect::CfgAttr(_)) {
            return Err(Error::at(
                file,
                0,
                "removed cfg use has a different actual effect kind",
            ));
        }
        let forest = unit
            .configured
            .ast
            .attributes
            .get(entry.capture as usize)
            .ok_or_else(|| Error::at(file, 0, "removed cfg capture disappeared"))?;
        demand::charge(
            work,
            file,
            forest.forest.nodes().len(),
            size_of::<Self>() + size_of_val(forest.forest.nodes()),
        )?;
        let trees = forest.forest.nodes().to_vec();
        let leaves = predicate_leaves(unit, &forest.forest, file, work)?;
        let mut uses = Vec::new();
        if let Some(saved) = &unit.configured.cfg_uses {
            demand::charge(
                work,
                file,
                1 + 2 * saved.len().checked_ilog2().unwrap_or(0) as usize,
                0,
            )?;
            for ordinal in saved.effect_range(effect) {
                demand::charge(work, file, 1, 4 * size_of::<Use>())?;
                let value = *saved
                    .get(ordinal)
                    .ok_or_else(|| Error::at(file, 0, "cfg saved ordinal disappeared"))?;
                let origins = origin_chain(unit, file, value.predicate.span, work, &other)?;
                let span = value.predicate.span;
                let tree = *leaves.get(&(span.start, span.end)).ok_or_else(|| {
                    Error::at(
                        file,
                        span.start,
                        "saved cfg use lost its exact typed predicate leaf",
                    )
                })?;
                uses.push(Use {
                    ordinal,
                    value,
                    tree,
                    origins,
                });
            }
        }
        Ok(Self {
            body: 0,
            path,
            file,
            effect,
            owner: NodeKey {
                file,
                node: owner.node,
            },
            range: owner.attributes.clone(),
            capture: entry.capture,
            attribute: entry.attribute,
            trees,
            forest_end: forest.forest.source_end(),
            uses,
        })
    }
    pub(super) fn validate_use(&self, used: &CheckedConfiguredUse<'_>) -> Result<(), Error> {
        let saved = self
            .uses
            .first()
            .and_then(|first| used.ordinal.checked_sub(first.ordinal))
            .and_then(|index| self.uses.get(index));
        if used.enabled
            || self.file != used.file
            || self.effect != used.effect()
            || self.owner != used.owner
            || self.range != used.range
            || self.capture != used.effect.capture
            || self.attribute != used.effect.attribute
            || saved.is_none_or(|saved| {
                saved.ordinal != used.ordinal || !same_use(saved.value, *used.used)
            })
        {
            return Err(Error::at(
                used.file,
                0,
                "removed Test cfg use/effect/owner identity changed",
            ));
        }
        Ok(())
    }
    pub(super) fn validate(
        &self,
        used: &CheckedConfiguredUse<'_>,
        work: &Cell<Work>,
    ) -> Result<(), Error> {
        self.validate_use(used)?;
        let unit = used.report.package.unit(self.file);
        let forest = &unit.configured.ast.attributes[self.capture as usize].forest;
        demand::charge(work, self.file, 1 + self.trees.len(), 0)?;
        if forest.nodes() != self.trees.as_slice() || forest.source_end() != self.forest_end {
            return Err(Error::at(
                self.file,
                0,
                "removed Test cfg typed capture trees changed",
            ));
        }
        for saved in &self.uses {
            demand::charge(work, self.file, 1, 0)?;
            if forest.tree(saved.tree).is_none_or(|tree| {
                tree.span != saved.value.predicate.span
                    || !matches!(
                        tree.kind,
                        TreeKind::Token(crate::lex::token::Kind::Identifier)
                    )
            }) || unit
                .configured
                .cfg_uses
                .as_ref()
                .and_then(|uses| uses.get(saved.ordinal))
                .is_none_or(|actual| !same_use(saved.value, *actual))
            {
                return Err(Error::at(
                    self.file,
                    0,
                    "removed Test saved cfg ordinal changed",
                ));
            }
            for origin in &saved.origins {
                origin.validate(used.report, work)?;
            }
        }
        Ok(())
    }
}
fn predicate_leaves(
    unit: &Unit,
    forest: &crate::syntax::token_tree::Forest,
    file: FileId,
    work: &Cell<Work>,
) -> Result<crate::support::hash::HashMap<(u32, u32), u32>, Error> {
    demand::charge(
        work,
        file,
        forest.nodes().len(),
        crate::support::retained::new_map::<(u32, u32), u32>(forest.nodes().len()),
    )?;
    let mut leaves = crate::support::hash::HashMap::default();
    for (index, tree) in forest.nodes().iter().enumerate() {
        let index =
            u32::try_from(index).map_err(|_| Error::at(file, 0, "cfg tree index exceeds32bits"))?;
        if matches!(
            tree.kind,
            TreeKind::Token(crate::lex::token::Kind::Identifier)
        ) && forest.name(unit.source.text(), index) == Some("ub_checks")
        {
            leaves.insert((tree.span.start, tree.span.end), index);
        }
    }
    Ok(leaves)
}
fn same_use(left: ConfiguredUse, right: ConfiguredUse) -> bool {
    left.effect == right.effect
        && left.predicate.span == right.predicate.span
        && left.predicate.gate == right.predicate.gate
}
fn origin_at(unit: &Unit, offset: u32) -> Option<(usize, Origin)> {
    if (offset as usize) < unit.source.physical_len() {
        return None;
    }
    let index = unit
        .origins
        .partition_point(|entry| entry.generated.end <= offset && entry.generated.start != offset);
    unit.origins
        .get(index)
        .copied()
        .filter(|entry| entry.generated.start <= offset && offset <= entry.generated.end)
        .map(|entry| (index, entry))
}
pub(super) fn origin_chain<'unit>(
    unit: &'unit Unit,
    file: FileId,
    span: Span,
    work: &Cell<Work>,
    other: &impl Fn(FileId) -> Option<&'unit Unit>,
) -> Result<Vec<OriginWitness>, Error> {
    let mut result = Vec::new();
    let (mut current, mut offset) = (file, span.start);
    for _ in 0..4096 {
        let source = if current == file {
            unit
        } else {
            other(current).ok_or_else(|| Error::at(file, 0, "cfg origin source disappeared"))?
        };
        demand::charge(
            work,
            file,
            1 + source.origins.len().checked_ilog2().unwrap_or(0) as usize,
            4 * size_of::<OriginWitness>(),
        )?;
        let origin = origin_at(source, offset);
        if origin.is_none() && offset as usize >= source.source.physical_len() {
            return Err(Error::at(
                file,
                offset,
                "cfg use has no current token origin",
            ));
        }
        result.push(OriginWitness {
            file: current,
            offset,
            image: Arc::clone(source.source.original_image()),
            origin,
        });
        let Some((_, origin)) = origin else {
            return Ok(result);
        };
        let length = origin
            .original
            .span
            .end
            .checked_sub(origin.original.span.start)
            .ok_or_else(|| Error::at(file, offset, "cfg origin range reversed"))?;
        offset = origin.original.span.start + (offset - origin.generated.start).min(length);
        current = origin.original.file;
    }
    Err(Error::at(file, span.start, "cfg origin chain exceeds4096"))
}
impl OriginWitness {
    pub(super) fn validate(&self, report: &Report, work: &Cell<Work>) -> Result<(), Error> {
        let unit = report
            .package
            .units
            .get(self.file.0 as usize)
            .ok_or_else(|| Error::at(self.file, self.offset, "cfg origin source changed"))?;
        demand::charge(
            work,
            self.file,
            1 + unit.origins.len().checked_ilog2().unwrap_or(0) as usize,
            0,
        )?;
        let actual = origin_at(unit, self.offset);
        let equal = match (self.origin, actual) {
            (None, None) => (self.offset as usize) < unit.source.physical_len(),
            (Some((a, left)), Some((b, right))) => {
                a == b
                    && left.generated == right.generated
                    && left.original == right.original
                    && left.context == right.context
                    && left.introduction == right.introduction
                    && left.joint == right.joint
            }
            _ => false,
        };
        if !Arc::ptr_eq(&self.image, unit.source.original_image()) || !equal {
            return Err(Error::at(
                self.file,
                self.offset,
                "cfg use current source origin changed",
            ));
        }
        Ok(())
    }
}
