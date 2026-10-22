//! Translate shared constant values into selected patterns with use-site provenance.
use super::{Declarations, Error, NodeKey, Report, Type, Value, VariantId, products};
use crate::support::hash::HashMap;
use crate::{
    sema::{
        members::{FieldId, Selection},
        patterns::{Field, Fields, Interval, Kind, PatternId, Projection, RangeId, scalar::Domain},
    },
    types::{Primitive, TypeId},
};

struct Expansion<'a> {
    facts: &'a mut Declarations,
    source: NodeKey,
    work: Vec<(PatternId, Value)>,
    products: HashMap<(products::Id, TypeId), PatternId>,
    references: HashMap<(super::references::Pointer, TypeId), PatternId>,
    texts: HashMap<(crate::sema::text::Id, TypeId), PatternId>,
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
        products: HashMap::default(),
        references: HashMap::default(),
        texts: HashMap::default(),
        remaining: 1_000_000,
    };
    let facts = &mut *expansion.facts;
    let equality = facts
        .equality
        .demand_with_copy(
            &facts.types,
            &facts.aggregates,
            &facts.equality_derives,
            ty,
            |ty| {
                facts
                    .copies
                    .demand(&facts.types, &facts.aggregates, &facts.derives, ty)
                    .map(|p| p.copy)
            },
        )
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
            let value = self.literal_pointer(ty, value)?;
            let kind = match (self.facts.types.kind(ty), value) {
                (
                    Some(Type::Reference {
                        mutable: false,
                        referent,
                        ..
                    }),
                    Value::Text(_) | Value::Bytes(_),
                ) => {
                    let pattern = self.select(referent, value)?;
                    Kind::Dereference {
                        mutable: false,
                        implicit: false,
                        pattern,
                    }
                }
                (
                    Some(Type::Reference {
                        mutable: false,
                        referent,
                        ..
                    }),
                    Value::Reference(_),
                ) => {
                    let value = self
                        .facts
                        .constant_references
                        .get(value)?
                        .value
                        .ok_or("constant pattern requires completed location contents")?;
                    let pattern = self.select(referent, value)?;
                    Kind::Dereference {
                        mutable: false,
                        implicit: false,
                        pattern,
                    }
                }
                (Some(Type::Primitive(Primitive::Str)), Value::Text(text)) => Kind::Text(text),
                (Some(Type::Array { .. } | Type::Slice(_)), Value::Bytes(text)) => {
                    self.bytes(ty, text)?
                }
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
                    Some(Type::Tuple(_) | Type::Array { .. } | Type::Slice(_) | Type::Adt(..)),
                    Value::Product(product),
                ) => self.product(ty, product)?,
                (Some(Type::Adt(..)), Value::Variant(variant)) => {
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
            Some(Type::Slice(_)) => length,
            Some(Type::Adt(..)) => self.nominal(
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
            let value = self
                .facts
                .constant_products
                .get(product, index, &self.facts.texts)?;
            let pattern = self.select(ty, value)?;
            self.facts
                .patterns
                .fields
                .push(Field { selection, pattern });
        }
        self.work[mark..].reverse();
        let end = u32::try_from(self.facts.patterns.fields.len())
            .map_err(|_| "too many constant pattern fields")?;
        let fields = Fields { start, end };
        Ok(
            if matches!(self.facts.types.kind(ty), Some(Type::Slice(_))) {
                Kind::Slice {
                    minimum: length,
                    exact: true,
                    fields,
                }
            } else {
                Kind::Product { variant, fields }
            },
        )
    }
    fn bytes(&mut self, ty: TypeId, text: crate::sema::text::Id) -> Result<Kind, &'static str> {
        let bytes = self.facts.texts.get(text)?.len();
        let length = bytes as u64;
        let byte = TypeId::primitive(Primitive::U8);
        let slice = match self.facts.types.kind(ty) {
            Some(Type::Array {
                element,
                length: expected,
            }) if element == byte && length == expected => false,
            Some(Type::Slice(element)) if element == byte => true,
            _ => return Err("byte string differs from its selected pattern type"),
        };
        self.charge(length)?;
        let start = u32::try_from(self.facts.patterns.fields.len())
            .map_err(|_| "too many byte pattern fields")?;
        let mark = self.work.len();
        for index in 0..bytes {
            let value = self.facts.texts.get(text)?[index];
            let pattern = self.select(
                byte,
                Value::Scalar(super::Scalar::Integer(u128::from(value))),
            )?;
            self.facts.patterns.fields.push(Field {
                selection: Projection::Index(index as u64),
                pattern,
            });
        }
        self.work[mark..].reverse();
        let end = u32::try_from(self.facts.patterns.fields.len())
            .map_err(|_| "too many byte pattern fields")?;
        let fields = Fields { start, end };
        Ok(if slice {
            Kind::Slice {
                minimum: length,
                exact: true,
                fields,
            }
        } else {
            Kind::Product {
                variant: None,
                fields,
            }
        })
    }
    fn nominal(&self, ty: TypeId, variant: VariantId) -> Result<u64, &'static str> {
        let Some(Type::Adt(item, _)) = self.facts.types.kind(ty) else {
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
        let value = self.literal_pointer(ty, value)?;
        if let Value::Reference(reference) = value
            && let Some(&id) = self.references.get(&(reference, ty))
        {
            return Ok(id);
        }
        if let Value::Product(product) = value
            && let Some(&id) = self.products.get(&(product, ty))
        {
            return Ok(id);
        }
        if let Value::Text(text) | Value::Bytes(text) = value
            && let Some(&id) = self.texts.get(&(text, ty))
        {
            return Ok(id);
        }
        let id = self.facts.patterns.reserve(self.source)?;
        self.facts.patterns.nodes[id.0 as usize].ty = Some(ty);
        if let Value::Reference(reference) = value {
            self.references.insert((reference, ty), id);
        }
        if let Value::Text(text) | Value::Bytes(text) = value {
            self.texts.insert((text, ty), id);
        }
        if let Value::Product(product) = value {
            self.products.insert((product, ty), id);
        }
        self.work.push((id, value));
        Ok(id)
    }
    fn literal_pointer(&self, ty: TypeId, value: Value) -> Result<Value, &'static str> {
        let Value::BytePointer {
            text,
            metadata: super::references::Metadata::Length(length),
        } = value
        else {
            return Ok(value);
        };
        let Some(Type::Reference {
            mutable: false,
            referent,
            ..
        }) = self.facts.types.kind(ty)
        else {
            return Ok(value);
        };
        if self.facts.types.kind(referent) == Some(Type::Slice(TypeId::primitive(Primitive::U8)))
            && self.facts.texts.get(text)?.len() as u64 == length
        {
            // Reuse the canonical literal and its structural expansion cache.
            return Ok(Value::Bytes(text));
        }
        Ok(value)
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/literal_pointers.rs"]
mod tests;
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
        Some(Type::Slice(element)) => Ok((element, Projection::Index(index))),
        Some(Type::Array { element, length }) if index < length => {
            Ok((element, Projection::Index(index)))
        }
        Some(Type::Adt(..)) => {
            let member = variant
                .and_then(|id| facts.members.variants.get(id.0 as usize))
                .ok_or("missing constant constructor")?;
            let ordinal = u32::try_from(index).map_err(|_| "constant field index overflow")?;
            if ordinal >= member.end - member.start {
                return Err("constant field exceeds constructor arity");
            }
            let field = FieldId(member.start + ordinal);
            let shape = facts.aggregates.shape(&facts.types, ty)?;
            let fields = shape
                .variant(member.ordinal as usize)
                .ok_or("missing constant variant shape")?
                .fields;
            let ty = facts
                .types
                .list(fields)
                .and_then(|fields| fields.get(ordinal as usize))
                .copied()
                .ok_or("missing instantiated constant field type")?;
            Ok((ty, Projection::Field(Selection::Nominal(field))))
        }
        _ => Err("missing constant product field type"),
    }
}
