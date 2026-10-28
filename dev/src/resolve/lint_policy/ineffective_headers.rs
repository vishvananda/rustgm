//! Cold exact impl/attribute joins; these rows grant no API availability.
use super::{
    Violation,
    producer::{Parsed, Producer, Tail, Work, invalid},
};
use crate::{
    lex::token::Punctuation,
    resolve::{
        package::Package,
        unit::{NodeKey, Unit},
    },
    support::{hash::HashMap, retained::Growth},
    syntax::{
        ast::{Kind, Node},
        attribute::{Attribute, Input},
        item::{Item, ItemKind},
    },
};
#[cfg(test)]
#[path = "../../../tests/invariants/ineffective_storage.rs"]
mod tests;
pub(super) struct Annotations {
    rows: HashMap<NodeKey, Annotation>,
}
pub(super) struct Annotation {
    tail: Tail,
    key: NodeKey,
    node: Node,
    item: Item,
    unstable: Option<u32>,
    bound: Option<u32>,
    bounds: Vec<Box<str>>,
}
#[derive(Clone, Copy)]
pub(crate) struct ImplAnnotation<'a> {
    pub owner: NodeKey,
    pub unstable: Option<Attribute>,
    pub bound: Option<Attribute>,
    pub bounds: &'a [Box<str>],
}
pub(super) fn capture(
    unit: &Unit,
    key: NodeKey,
    tail: Tail,
    parsed: &Parsed,
    work: &mut Work,
) -> Result<Option<Annotation>, Violation> {
    let mut declaration = key;
    loop {
        work.charge(key, 1, 0)?;
        let Kind::Attributed { value, .. } = unit.configured.ast.node(declaration.node).kind else {
            break;
        };
        declaration.node = value;
    }
    let node = unit.configured.ast.node(declaration.node);
    let item = match node.kind {
        Kind::Item(index) => unit.configured.ast.items.get(index as usize).copied(),
        _ => None,
    };
    let is_impl = item.is_some_and(|item| {
        matches!(
            item.kind,
            ItemKind::Impl {
                trait_path: Some(_),
                ..
            }
        )
    });
    let mut unstable = None;
    let mut bound = None;
    let mut bounds = Vec::new();
    for witness in &parsed.witnesses {
        work.charge(key, 1, 0)?;
        let capture = &unit.configured.ast.attributes[witness.capture as usize];
        let forest = &capture.forest;
        let attribute = witness.attribute;
        match attribute.name(forest, unit.source.text()) {
            Some("unstable") if is_impl => unstable = Some(witness.effect),
            Some("unstable_feature_bound") => {
                bound = Some(witness.effect);
                if !is_impl {
                    return Err(Violation {
                        key,
                        offset: attribute.span.start,
                        message: "unstable_feature_bound consumer requires an actual local trait impl; other owners remain required",
                    });
                }
                feature_bounds(unit, key, attribute, forest, &mut bounds, work)?;
            }
            _ => {}
        }
    }
    if unstable.is_none() && bound.is_none() {
        return Ok(None);
    }
    Ok(Some(Annotation {
        tail,
        key: declaration,
        node,
        item: item.ok_or_else(|| invalid(key, "impl annotation lost actual item"))?,
        unstable,
        bound,
        bounds,
    }))
}
fn feature_bounds(
    unit: &Unit,
    key: NodeKey,
    attribute: Attribute,
    forest: &crate::syntax::token_tree::Forest,
    bounds: &mut Vec<Box<str>>,
    work: &mut Work,
) -> Result<(), Violation> {
    if attribute.unsafe_wrapper {
        return Err(invalid(
            key,
            "unstable_feature_bound cannot use an unsafe wrapper",
        ));
    }
    let Input::Delimited(group) = attribute.input else {
        return Err(invalid(
            key,
            "unstable_feature_bound requires a list of features",
        ));
    };
    let range = forest
        .children(group)
        .ok_or_else(|| invalid(key, "unstable_feature_bound list is missing"))?;
    let mut index = range.start;
    while index < range.end {
        work.charge(key, 1, 0)?;
        let name = forest
            .name(unit.source.text(), index)
            .ok_or_else(|| invalid(key, "unstable_feature_bound requires feature identifiers"))?;
        if crate::resolve::bindings::FeatureName::new(name).is_err()
            || forest.after(index) != index + 1
        {
            return Err(invalid(
                key,
                "unstable_feature_bound feature is outside the local plain ASCII identifier profile",
            ));
        }
        let growth = Growth::vector::<Box<str>>(bounds.len(), bounds.capacity(), 1);
        work.charge(
            key,
            1 + growth.work,
            growth.bytes.saturating_add(name.len()),
        )?;
        bounds.push(name.into());
        index += 1;
        if index < range.end {
            if !forest.punctuation(index, Punctuation::Comma) {
                return Err(invalid(
                    key,
                    "unstable_feature_bound features require commas",
                ));
            }
            index += 1;
        }
    }
    Ok(())
}
impl Producer {
    pub(super) fn publish_impl_annotation(
        &mut self,
        key: NodeKey,
        row: Option<Annotation>,
    ) -> Result<(), Violation> {
        let Some(row) = row else {
            return Ok(());
        };
        self.reserve_diagnostic(
            key,
            crate::support::retained::new_map::<NodeKey, Annotation>(1).saturating_add(
                if self.ineffective_headers.is_none() {
                    size_of::<Annotations>()
                } else {
                    0
                },
            ),
        )?;
        let annotations = self.ineffective_headers.get_or_insert_with(|| {
            Box::new(Annotations {
                rows: HashMap::default(),
            })
        });
        if annotations.rows.contains_key(&row.key) {
            return Err(invalid(
                key,
                "nested impl annotation owners require a distinct authenticated join",
            ));
        }
        annotations.rows.insert(row.key, row);
        Ok(())
    }
    pub(crate) fn has_impl_annotations(&self) -> bool {
        self.ineffective_headers.is_some()
    }
    pub(crate) fn impl_annotation(
        &self,
        package: &Package,
        key: NodeKey,
    ) -> Result<Option<ImplAnnotation<'_>>, Violation> {
        self.attribute_read(key, 1)?;
        let Some(row) = self
            .ineffective_headers
            .as_ref()
            .and_then(|column| column.rows.get(&key))
        else {
            return Ok(None);
        };
        let owner = self.record(row.tail, key)?;
        owner.validate(package, &self.reads)?;
        let unit = package.unit(key.file);
        let mut actual = owner.key;
        loop {
            self.attribute_read(key, 1)?;
            let Kind::Attributed { value, .. } = unit.configured.ast.node(actual.node).kind else {
                break;
            };
            actual.node = value;
        }
        if actual != key
            || unit.configured.ast.node(key.node) != row.node
            || !matches!(row.node.kind,Kind::Item(index) if unit.configured.ast.items.get(index as usize)==Some(&row.item))
        {
            return Err(invalid(
                key,
                "impl annotation changed actual source declaration",
            ));
        }
        Ok(Some(ImplAnnotation {
            owner: owner.key,
            unstable: row
                .unstable
                .map(|effect| attribute(self, unit, owner, effect, "unstable"))
                .transpose()?,
            bound: row
                .bound
                .map(|effect| attribute(self, unit, owner, effect, "unstable_feature_bound"))
                .transpose()?,
            bounds: &row.bounds,
        }))
    }
    pub(crate) fn require_impl_bound_transport(&self, package: &Package) -> Result<(), Violation> {
        if let Some(column) = &self.ineffective_headers {
            for (&key, row) in &column.rows {
                self.attribute_read(key, 1)?;
                if row.bound.is_some() {
                    self.impl_annotation(package, key)?;
                    return Err(invalid(
                        key,
                        "unstable_feature_bound transport is not implemented for this metadata profile",
                    ));
                }
            }
        }
        Ok(())
    }
}
fn attribute(
    producer: &Producer,
    unit: &Unit,
    owner: &super::producer::Owner,
    effect: u32,
    name: &str,
) -> Result<Attribute, Violation> {
    producer.attribute_read(
        owner.key,
        1 + owner.parsed.witnesses.len().checked_ilog2().unwrap_or(0) as usize + name.len(),
    )?;
    let witness = owner
        .parsed
        .witnesses
        .binary_search_by_key(&effect, |witness| witness.effect)
        .ok()
        .and_then(|index| owner.parsed.witnesses.get(index))
        .ok_or_else(|| invalid(owner.key, "impl annotation lost its exact retained effect"))?;
    let capture = &unit.configured.ast.attributes[witness.capture as usize];
    if witness.attribute.name(&capture.forest, unit.source.text()) != Some(name) {
        return Err(invalid(
            owner.key,
            "impl annotation effect belongs to another attribute consumer",
        ));
    }
    Ok(witness.attribute)
}
