//! Translate shared constant values into selected patterns with use-site provenance.
use super::{Declarations, Error, NodeKey, Report, Type, Value, VariantId, products};
use crate::{
    sema::{
        members::{FieldId, Selection},
        patterns::{Field, Fields, Interval, Kind, PatternId, Projection, RangeId, scalar::Domain},
    },
    types::TypeId,
};
use std::collections::HashMap;

struct Expansion<'a> {
    facts: &'a mut Declarations,
    source: NodeKey,
    work: Vec<(PatternId, Value)>,
    products: HashMap<(products::Id, TypeId), PatternId>,
    remaining: u64,
}
pub(super) fn expand(
    report: &Report,
    facts: &mut Declarations,
    root: PatternId,
    value: Value,
) -> Result<(), Error> {
    let source = facts.patterns.nodes[root.0 as usize].source;
    let error = |message| Declarations::error(report, source, message);
    let ty = facts.patterns.nodes[root.0 as usize]
        .ty
        .ok_or_else(|| error("missing constant pattern input type"))?;
    let mut expansion = Expansion {
        facts,
        source,
        work: vec![(root, value)],
        products: HashMap::new(),
        remaining: 1_000_000,
    };
    let facts = &mut *expansion.facts;
    let equality = facts
        .equality
        .demand(&facts.types, &facts.aggregates, &facts.equality_derives, ty)
        .map_err(error)?;
    if !equality.partial_eq {
        return Err(error("constant pattern type must implement PartialEq"));
    }
    if let Value::Product(product) = value {
        expansion.products.insert((product, ty), root);
    }
    expansion.drain().map_err(error)
}
impl Expansion<'_> {
    fn charge(&mut self, count: u64) -> Result<(), &'static str> {
        self.remaining = self
            .remaining
            .checked_sub(count)
            .ok_or("constant pattern expansion work budget exceeded")?;
        Ok(())
    }
    fn drain(&mut self) -> Result<(), &'static str> {
        while let Some((id, value)) = self.work.pop() {
            self.charge(1)?;
            let ty = self.facts.patterns.nodes[id.0 as usize]
                .ty
                .ok_or("missing constant pattern input type")?;
            let kind = match (self.facts.types.kind(ty), value) {
                (Some(Type::Primitive(primitive)), Value::Scalar(value)) => {
                    let domain = Domain::new(primitive)
                        .ok_or("constant pattern requires structural equality")?;
                    let rank = domain.rank(value, false)?;
                    let range = RangeId(
                        u32::try_from(self.facts.patterns.ranges.len())
                            .map_err(|_| "too many constant pattern ranges")?,
                    );
                    self.facts.patterns.ranges.push(Interval {
                        lower: rank,
                        upper: rank,
                    });
                    Kind::Range(range)
                }
                (
                    Some(Type::Tuple(_) | Type::Array { .. } | Type::Adt(_)),
                    Value::Product(product),
                ) => self.product(ty, product)?,
                (Some(Type::Adt(_)), Value::Variant(variant)) => {
                    if self.nominal(ty, variant)? != 0 {
                        return Err("fieldless constant has constructor fields");
                    }
                    let start = u32::try_from(self.facts.patterns.fields.len())
                        .map_err(|_| "too many constant pattern fields")?;
                    Kind::Product {
                        variant: Some(variant),
                        fields: Fields { start, end: start },
                    }
                }
                _ => return Err("constant pattern requires supported structural equality"),
            };
            self.facts.patterns.nodes[id.0 as usize].kind = kind;
        }
        Ok(())
    }
    fn product(&mut self, ty: TypeId, product: products::Id) -> Result<Kind, &'static str> {
        let length = self.facts.constant_products.length(product)?;
        let variant = self.facts.constant_products.variant(product)?;
        let expected = match self.facts.types.kind(ty) {
            Some(Type::Tuple(fields)) => self
                .facts
                .types
                .list(fields)
                .ok_or("missing constant tuple types")?
                .len() as u64,
            Some(Type::Array { length, .. }) => length,
            Some(Type::Adt(_)) => self.nominal(
                ty,
                variant.ok_or("constant product lacks a selected constructor")?,
            )?,
            _ => return Err("missing constant product type"),
        };
        if length != expected {
            return Err("constant product arity differs from its type");
        }
        // Reserve the complete edge budget before allocating siblings. Charging
        // only when popped would permit an unbounded number of pending nodes.
        self.charge(length)?;
        let start = u32::try_from(self.facts.patterns.fields.len())
            .map_err(|_| "too many constant pattern fields")?;
        let mark = self.work.len();
        for index in 0..length {
            let (ty, selection) = child(self.facts, ty, variant, index)?;
            let value = self.facts.constant_products.get(product, index)?;
            let pattern = self.select(ty, value)?;
            self.facts
                .patterns
                .fields
                .push(Field { selection, pattern });
        }
        self.work[mark..].reverse();
        let end = u32::try_from(self.facts.patterns.fields.len())
            .map_err(|_| "too many constant pattern fields")?;
        Ok(Kind::Product {
            variant,
            fields: Fields { start, end },
        })
    }
    fn nominal(&self, ty: TypeId, variant: VariantId) -> Result<u64, &'static str> {
        let Some(Type::Adt(item)) = self.facts.types.kind(ty) else {
            return Err("missing nominal constant type");
        };
        let member = self
            .facts
            .members
            .variants
            .get(variant.0 as usize)
            .ok_or("missing constant constructor")?;
        if item != member.owner {
            return Err("constant constructor differs from its type");
        }
        if !self
            .facts
            .equality_derives
            .get(&item)
            .is_some_and(|p| p.partial_eq)
        {
            return Err("nominal constant patterns require derived PartialEq");
        }
        Ok(u64::from(member.end - member.start))
    }
    fn select(&mut self, ty: TypeId, value: Value) -> Result<PatternId, &'static str> {
        if let Value::Product(product) = value
            && let Some(&id) = self.products.get(&(product, ty))
        {
            return Ok(id);
        }
        let id = self.facts.patterns.reserve(self.source)?;
        self.facts.patterns.nodes[id.0 as usize].ty = Some(ty);
        if let Value::Product(product) = value {
            self.products.insert((product, ty), id);
        }
        self.work.push((id, value));
        Ok(id)
    }
}
fn child(
    facts: &Declarations,
    ty: TypeId,
    variant: Option<VariantId>,
    index: u64,
) -> Result<(TypeId, Projection), &'static str> {
    match facts.types.kind(ty) {
        Some(Type::Tuple(fields)) => {
            let index = u32::try_from(index).map_err(|_| "constant tuple index overflow")?;
            let ty = facts
                .types
                .list(fields)
                .and_then(|fields| fields.get(index as usize))
                .copied()
                .ok_or("missing constant tuple field type")?;
            Ok((ty, Projection::Field(Selection::Tuple(index))))
        }
        Some(Type::Array { element, length }) if index < length => {
            Ok((element, Projection::Index(index)))
        }
        Some(Type::Adt(_)) => {
            let member = variant
                .and_then(|id| facts.members.variants.get(id.0 as usize))
                .ok_or("missing constant constructor")?;
            let ordinal = u32::try_from(index).map_err(|_| "constant field index overflow")?;
            if ordinal >= member.end - member.start {
                return Err("constant field exceeds constructor arity");
            }
            let field = FieldId(member.start + ordinal);
            Ok((
                facts.members.fields[field.0 as usize].ty,
                Projection::Field(Selection::Nominal(field)),
            ))
        }
        _ => Err("missing constant product field type"),
    }
}
