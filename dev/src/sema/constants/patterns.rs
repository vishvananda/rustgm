//! Translate shared constant values into selected patterns with use-site provenance.
use super::{Declarations, Error, NodeKey, Report, Type, Value, products};
use crate::{
    sema::{
        members::Selection,
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
    expansion.structural_type(ty).map_err(error)?;
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
    fn structural_type(&mut self, root: TypeId) -> Result<(), &'static str> {
        let mut work = vec![(root, false)];
        while let Some((ty, finish)) = work.pop() {
            self.charge(1)?;
            if self.facts.constant_products.structural.contains(&ty) {
                continue;
            }
            if finish {
                self.facts.constant_products.structural.insert(ty);
                continue;
            }
            work.push((ty, true));
            match self.facts.types.kind(ty) {
                Some(Type::Primitive(primitive)) if Domain::new(primitive).is_some() => {}
                Some(Type::Tuple(fields)) => {
                    work.extend(
                        self.facts
                            .types
                            .list(fields)
                            .ok_or("missing constant tuple types")?
                            .iter()
                            .map(|ty| (*ty, false)),
                    );
                }
                Some(Type::Array { element, .. }) => work.push((element, false)),
                _ => {
                    return Err(
                        "constant pattern type requires supported structural equality and PartialEq",
                    );
                }
            }
        }
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
                (Some(Type::Tuple(_) | Type::Array { .. }), Value::Product(product)) => {
                    self.product(ty, product)?
                }
                _ => return Err("constant pattern requires supported structural equality"),
            };
            self.facts.patterns.nodes[id.0 as usize].kind = kind;
        }
        Ok(())
    }
    fn product(&mut self, ty: TypeId, product: products::Id) -> Result<Kind, &'static str> {
        let length = self.facts.constant_products.length(product)?;
        let expected = match self.facts.types.kind(ty) {
            Some(Type::Tuple(fields)) => self
                .facts
                .types
                .list(fields)
                .ok_or("missing constant tuple types")?
                .len() as u64,
            Some(Type::Array { length, .. }) => length,
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
            let (ty, selection) = child(self.facts, ty, index)?;
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
            variant: None,
            fields: Fields { start, end },
        })
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
        _ => Err("missing constant product field type"),
    }
}
