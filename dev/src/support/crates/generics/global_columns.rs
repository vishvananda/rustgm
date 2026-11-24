//! Module statics retain their exact type recipes and one completed initializer.
use super::{ConstTerm, ConstantBody, HeaderRef, SourceRows, Type};
use crate::support::crates::{
    self, Scalar, ScalarValue,
    generics::{self as g, Work},
    owner_catalog::DeclarationRef,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaticHeader {
    pub declaration: DeclarationRef,
    pub ty: g::TypeRef,
    pub initializer: StaticInitializer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticInitializer {
    /// Canonical little-endian scalar leaves in tuple or array element order.
    Scalars(g::Range),
    /// A raw pointer reached through zero or more single-field struct values.
    Promotion(DeclarationRef),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StaticColumns {
    pub headers: Vec<StaticHeader>,
    pub values: Vec<u8>,
}

pub(crate) enum StaticShape<'a> {
    Scalar(Scalar),
    WideFloat(usize),
    Tuple(&'a [g::TypeRef]),
    Array(g::TypeRef, usize),
}

impl<'a> StaticShape<'a> {
    pub(crate) fn new(
        types: &[Type],
        roots: &'a [g::TypeRef],
        lengths: &[ConstTerm],
        ty: g::TypeRef,
    ) -> Result<Self, &'static str> {
        match types.get(ty.0 as usize).copied() {
            Some(Type::Scalar(ty)) => Ok(Self::Scalar(ty)),
            Some(Type::F16) => Ok(Self::WideFloat(2)),
            Some(Type::F128) => Ok(Self::WideFloat(16)),
            Some(Type::Tuple(fields)) => Ok(Self::Tuple(fields.slice(roots)?)),
            Some(Type::Array { element, length }) => {
                let Some(ConstTerm::Value(value)) = lengths.get(length.0 as usize) else {
                    return Err("source static array requires a closed length");
                };
                if types.get(value.ty.0 as usize) != Some(&Type::Scalar(Scalar::Usize)) {
                    return Err("source static array length is not usize");
                }
                let count = usize::try_from(value.bits)
                    .ok()
                    .filter(|&count| count <= crates::MAX_COMPONENTS)
                    .ok_or("source static array length exceeds its value budget")?;
                Ok(Self::Array(element, count))
            }
            _ => Err("source static initializer requires its closed scalar aggregate type"),
        }
    }
}

enum Visit<'a> {
    Root(g::TypeRef, bool),
    Tuple(std::slice::Iter<'a, g::TypeRef>, bool),
    Array(g::TypeRef, usize, bool),
}

pub(super) fn validate(
    rows: &SourceRows,
    headers: &[Option<HeaderRef>],
    base: usize,
    work: &mut Work,
) -> Result<usize, &'static str> {
    let Some(statics) = rows.cold().statics.as_deref() else {
        return Ok(base);
    };
    if statics.headers.is_empty() {
        return Err("source static column requires an actual declaration");
    }
    let mut end = 0;
    let mut walk = Walk {
        pending: Vec::new(),
        peak: base,
        base,
    };
    for header in &statics.headers {
        work.charge(1)?;
        match header.initializer {
            StaticInitializer::Scalars(values) => {
                if values.start != end {
                    return Err("source static initializer ranges must cover the values in order");
                }
                walk.value(rows, header.ty, values.slice(&statics.values)?, work)?;
                end = values.end;
            }
            StaticInitializer::Promotion(anchor) => {
                let Some(Some(HeaderRef::Constant(index))) = headers.get(anchor.0 as usize) else {
                    return Err("source static relocation requires an actual promoted constant");
                };
                if !matches!(rows.constants()[*index as usize].body,
                    ConstantBody::PromotedReference { anchor: owner, .. } if owner == anchor)
                {
                    return Err("source static relocation changed its actual promotion owner");
                }
            }
        }
    }
    if end as usize != statics.values.len() {
        return Err("source static initializer bytes have no actual owner");
    }
    Ok(walk.peak)
}

struct Walk<'a> {
    pending: Vec<Visit<'a>>,
    peak: usize,
    base: usize,
}

impl<'a> Walk<'a> {
    fn reserve(&mut self, additional: usize) -> Result<(), &'static str> {
        let previous = self.pending.capacity();
        let needed = self.pending.len() + additional;
        if needed > previous {
            let capacity = needed.next_power_of_two();
            let peak = |size: usize| {
                self.base
                    .saturating_add((previous + size).saturating_mul(size_of::<Visit<'_>>()))
            };
            if peak(capacity) > crates::MAX_BYTES {
                return Err("source static validation exceeds the combined storage budget");
            }
            self.pending
                .try_reserve_exact(capacity - self.pending.len())
                .map_err(|_| "source static validation stack could not grow")?;
            self.peak = self.peak.max(peak(self.pending.capacity()));
            if self.peak > crates::MAX_BYTES {
                return Err("source static validation exceeds the combined storage budget");
            }
        }
        Ok(())
    }

    fn value(
        &mut self,
        rows: &'a SourceRows,
        ty: g::TypeRef,
        mut bytes: &[u8],
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.reserve(1)?;
        self.pending.push(Visit::Root(ty, true));
        while let Some(visit) = self.pending.pop() {
            work.charge(1)?;
            self.reserve(2)?;
            match visit {
                Visit::Root(ty, present) => self.root(rows, ty, present, &mut bytes)?,
                Visit::Tuple(mut fields, present) => {
                    if let Some(&field) = fields.next() {
                        self.pending.push(Visit::Tuple(fields, present));
                        self.pending.push(Visit::Root(field, present));
                    }
                }
                Visit::Array(element, count, present) => {
                    if count != 0 {
                        self.pending.push(Visit::Array(element, count - 1, present));
                        self.pending.push(Visit::Root(element, present));
                    }
                }
            }
        }
        if !bytes.is_empty() {
            return Err("source static initializer contains unused scalar bytes");
        }
        Ok(())
    }

    fn root(
        &mut self,
        rows: &'a SourceRows,
        ty: g::TypeRef,
        present: bool,
        bytes: &mut &[u8],
    ) -> Result<(), &'static str> {
        match StaticShape::new(&rows.types, &rows.roots, rows.array_lengths(), ty)? {
            StaticShape::Scalar(ty) if present => {
                let bits = read_bits(bytes, ty.bit_width().div_ceil(8) as usize)?;
                ScalarValue::new(ty, bits)?;
            }
            StaticShape::WideFloat(width) if present => {
                read_bits(bytes, width)?;
            }
            StaticShape::Scalar(_) | StaticShape::WideFloat(_) => {}
            StaticShape::Tuple(fields) => self.pending.push(Visit::Tuple(fields.iter(), present)),
            StaticShape::Array(element, count) if present && count != 0 => {
                self.pending.push(Visit::Array(element, count, true));
            }
            StaticShape::Array(element, _) => self.pending.push(Visit::Root(element, false)),
        }
        Ok(())
    }
}

pub(crate) fn read_bits(bytes: &mut &[u8], width: usize) -> Result<u128, &'static str> {
    let (field, remaining) = bytes
        .split_at_checked(width)
        .ok_or("source static initializer is missing scalar bytes")?;
    let mut bits = [0; 16];
    bits.get_mut(..width)
        .ok_or("source static primitive width exceeds sixteen bytes")?
        .copy_from_slice(field);
    *bytes = remaining;
    Ok(u128::from_le_bytes(bits))
}

impl super::Original {
    pub(crate) fn static_places(&self) -> impl Iterator<Item = DeclarationRef> + '_ {
        let projections = self.projections.iter().map(|projection| projection.base);
        let values = self.values.iter().filter_map(|value| match value.kind {
            super::ValueKind::Read(place)
            | super::ValueKind::Borrow(place)
            | super::ValueKind::TwoPhaseBorrow(place)
            | super::ValueKind::IsVariant { place, .. } => Some(place),
            _ => None,
        });
        let statements = self
            .statements
            .iter()
            .filter_map(|statement| match statement.action {
                super::Action::Store { place, .. }
                | super::Action::CheckInitialized(place)
                | super::Action::Mention(place)
                | super::Action::BindView { place, .. }
                | super::Action::Select { anchor: place, .. } => Some(place),
                _ => None,
            });
        let guards = self
            .guards
            .iter()
            .flat_map(|guards| guards.places.iter().copied());
        projections
            .chain(values)
            .chain(statements)
            .chain(guards)
            .filter_map(|place| {
                if let super::Place::Static(declaration) = place {
                    Some(declaration)
                } else {
                    None
                }
            })
    }
}
