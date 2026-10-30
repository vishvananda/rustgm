//! Cold exact declaration/attribute joins; these rows grant no API availability.
mod reserved_messages;
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
    reservation: Option<reserved_messages::Message>,
    bound: Vec<BoundEffect>,
    bounds: Vec<Box<str>>,
}
#[derive(Clone, Copy)]
struct BoundEffect {
    effect: u32,
    start: u32,
    end: u32,
}
#[derive(Clone, Copy)]
pub(crate) struct DeclarationAnnotation<'a> {
    pub owner: NodeKey,
    pub unstable: Option<Attribute>,
    pub bound: Option<Attribute>,
    pub bounds: &'a [Box<str>],
    pub reservation: Option<(Attribute, &'a str)>,
}
pub(crate) type ImplAnnotation<'a> = DeclarationAnnotation<'a>;
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
    let accepts_bound = is_impl
        || item.is_some_and(|item| {
            matches!(
                item.kind,
                ItemKind::Trait { .. } | ItemKind::Function { .. }
            )
        });
    let mut unstable = None;
    let mut reservation = None;
    let mut bound = Vec::new();
    let mut bounds = Vec::new();
    for witness in &parsed.witnesses {
        work.charge(key, 1, 0)?;
        let capture = &unit.configured.ast.attributes[witness.capture as usize];
        let forest = &capture.forest;
        let attribute = witness.attribute;
        match attribute.name(forest, unit.source.text()) {
            Some("unstable") if accepts_bound => unstable = Some(witness.effect),
            Some("rustc_reservation_impl") => {
                if !is_impl || reservation.is_some() {
                    return Err(invalid(
                        key,
                        "rustc_reservation_impl requires one actual trait impl owner",
                    ));
                }
                reservation = Some(reserved_messages::capture(unit, key, witness, work)?);
            }
            Some("unstable_feature_bound") => {
                if !accepts_bound {
                    return Err(Violation {
                        key,
                        offset: attribute.span.start,
                        message: "unstable_feature_bound requires an actual trait, function or trait impl declaration",
                    });
                }
                capture_bound(unit, key, witness, &mut bound, &mut bounds, work)?;
            }
            _ => {}
        }
    }
    if reservation.is_none() && bound.is_empty() && (!is_impl || unstable.is_none()) {
        return Ok(None);
    }
    Ok(Some(Annotation {
        tail,
        key: declaration,
        node,
        item: item.ok_or_else(|| invalid(key, "declaration annotation lost actual item"))?,
        unstable,
        reservation,
        bound,
        bounds,
    }))
}
fn capture_bound(
    unit: &Unit,
    key: NodeKey,
    witness: &super::producer::Witness,
    effects: &mut Vec<BoundEffect>,
    names: &mut Vec<Box<str>>,
    work: &mut Work,
) -> Result<(), Violation> {
    let growth = Growth::vector::<BoundEffect>(effects.len(), effects.capacity(), 1);
    work.charge(key, 1 + growth.work, growth.bytes)?;
    let start = u32::try_from(names.len())
        .map_err(|_| invalid(key, "feature name coordinates exceed their arena"))?;
    let forest = &unit.configured.ast.attributes[witness.capture as usize].forest;
    feature_bounds(unit, key, witness.attribute, forest, names, work)?;
    let end = u32::try_from(names.len())
        .map_err(|_| invalid(key, "feature name coordinates exceed their arena"))?;
    effects.push(BoundEffect {
        effect: witness.effect,
        start,
        end,
    });
    Ok(())
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
        work.charge(key, name.len().saturating_mul(2).saturating_add(1), 0)?;
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
    /// Data-only enumeration; each key still needs its authenticated source query.
    /// The upper size hint counts all rows and prepays filtering traversal.
    pub(crate) fn feature_bound_keys(&self) -> impl Iterator<Item = NodeKey> + '_ {
        self.ineffective_headers
            .as_ref()
            .map(|column| column.rows.iter())
            .unwrap_or_default()
            .filter_map(|(&key, row)| (!row.bound.is_empty()).then_some(key))
    }
    pub(crate) fn has_impl_annotations(&self) -> bool {
        self.ineffective_headers.is_some()
    }
    pub(crate) fn impl_annotation(
        &self,
        package: &Package,
        key: NodeKey,
    ) -> Result<Option<ImplAnnotation<'_>>, Violation> {
        let Some(annotation) = self.declaration_annotation(package, key)? else {
            return Ok(None);
        };
        self.attribute_read(key, 1)?;
        let is_impl = self
            .ineffective_headers
            .as_ref()
            .and_then(|column| column.rows.get(&key))
            .is_some_and(|row| {
                matches!(
                    row.item.kind,
                    ItemKind::Impl {
                        trait_path: Some(_),
                        ..
                    }
                )
            });
        Ok(is_impl.then_some(annotation))
    }
    pub(crate) fn declaration_annotation(
        &self,
        package: &Package,
        key: NodeKey,
    ) -> Result<Option<DeclarationAnnotation<'_>>, Violation> {
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
        Ok(Some(DeclarationAnnotation {
            owner: owner.key,
            unstable: row
                .unstable
                .map(|effect| attribute(self, unit, owner, effect, "unstable"))
                .transpose()?,
            bound: bound_attributes(self, unit, owner, row)?,
            bounds: &row.bounds,
            reservation: row
                .reservation
                .as_ref()
                .map(|message| reserved_messages::current(self, unit, owner, message))
                .transpose()?,
        }))
    }
    pub(crate) fn require_impl_bound_transport(&self, package: &Package) -> Result<(), Violation> {
        self.require_impl_annotation_transport(package, false)
    }
    pub(crate) fn require_impl_annotation_transport(
        &self,
        package: &Package,
        feature_predicates: bool,
    ) -> Result<(), Violation> {
        if let Some(column) = &self.ineffective_headers {
            for (&key, row) in &column.rows {
                self.attribute_read(key, 1)?;
                if row.reservation.is_some() {
                    self.declaration_annotation(package, key)?;
                    return Err(invalid(
                        key,
                        "reservation implementation transport requires a wider metadata profile",
                    ));
                }
                if !feature_predicates && !row.bound.is_empty() {
                    self.declaration_annotation(package, key)?;
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
fn bound_attributes(
    producer: &Producer,
    unit: &Unit,
    owner: &super::producer::Owner,
    row: &Annotation,
) -> Result<Option<Attribute>, Violation> {
    // Parsed witnesses already fix the list profile and source payload. Match
    // every retained effect once; never parse feature names again at a query.
    producer.attribute_read(
        owner.key,
        owner
            .parsed
            .witnesses
            .len()
            .saturating_mul(25)
            .saturating_add(1),
    )?;
    let mut effects = row.bound.iter();
    let mut next = effects.next();
    let mut end = 0;
    let mut first = None;
    for witness in &owner.parsed.witnesses {
        let forest = &unit.configured.ast.attributes[witness.capture as usize].forest;
        if witness.attribute.name(forest, unit.source.text()) != Some("unstable_feature_bound") {
            continue;
        }
        let effect =
            next.ok_or_else(|| invalid(owner.key, "feature annotation lost an actual effect"))?;
        let Input::Delimited(group) = witness.attribute.input else {
            return Err(invalid(
                owner.key,
                "feature annotation lost its validated list",
            ));
        };
        let range = forest
            .children(group)
            .ok_or_else(|| invalid(owner.key, "feature annotation lost its list coordinates"))?;
        // The captured plain-list profile has one name token and at most one
        // comma per name. Coordinates prove its count without a token walk.
        let count = range
            .end
            .checked_sub(range.start)
            .ok_or_else(|| invalid(owner.key, "feature annotation list coordinates changed"))?
            .div_ceil(2);
        if effect.effect != witness.effect
            || effect.start != end
            || effect.end.checked_sub(effect.start) != Some(count)
        {
            return Err(invalid(
                owner.key,
                "feature annotation changed its effect/name partition",
            ));
        }
        first.get_or_insert(witness.attribute);
        end = effect.end;
        next = effects.next();
    }
    if next.is_some() || usize::try_from(end).ok() != Some(row.bounds.len()) {
        return Err(invalid(
            owner.key,
            "feature annotation has incomplete effect/name coverage",
        ));
    }
    Ok(first)
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
