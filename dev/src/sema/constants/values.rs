//! Typed value operations shared by source constants and CFG promotion queries.
use super::{
    Declarations, DefId, Error, InitializerKind, Queries, Query, Report, Type, TypeId, Value,
    VariantId, Work, arithmetic, references,
};
use crate::{
    lex::token::Punctuation,
    sema::model::{Cast, Scalar},
    syntax::ast::Unary,
    types::Primitive,
};

/// A value in the declaration result's shared constant arenas.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constant(Value);

/// Opaque location identity in the declaration result's flat reference arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocationId(references::Id);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Allocation,
    Static(DefId),
    Projection { base: LocationId, index: u64 },
}
#[derive(Clone, Copy)]
pub struct Location {
    /// Address formation need not read an immutable static's initializer.
    pub value: Option<Constant>,
    pub ty: TypeId,
    pub origin: Origin,
}
#[derive(Clone, Copy)]
pub enum View {
    Scalar(Scalar),
    Function(crate::types::instance::Instance),
    Reference(LocationId),
    Text(crate::sema::text::Id),
    Bytes(crate::sema::text::Id),
    Repeat {
        value: Constant,
        length: u64,
    },
    Product {
        length: u64,
        variant: Option<VariantId>,
    },
}
/// Read completed values without starting semantic queries or revisiting syntax.
#[derive(Clone, Copy)]
pub struct Reader<'a> {
    facts: &'a Declarations,
}
impl<'a> Reader<'a> {
    #[must_use]
    pub const fn new(facts: &'a Declarations) -> Self {
        Self { facts }
    }
    #[must_use]
    pub fn definition(self, id: DefId) -> Option<Constant> {
        self.facts.constant_values.get(&id).copied().map(Constant)
    }
    /// # Errors
    /// Rejects missing or malformed constant arena entries.
    pub fn view(self, constant: Constant) -> Result<View, &'static str> {
        Ok(match constant.0 {
            Value::Scalar(value) => View::Scalar(value),
            Value::Function(id) => View::Function(id),
            Value::Reference(id) => View::Reference(LocationId(id)),
            Value::Text(id) => View::Text(id),
            Value::Bytes(id) => View::Bytes(id),
            Value::Variant(id) => View::Product {
                length: 0,
                variant: Some(id),
            },
            Value::Product(id) => {
                if let Some((value, length)) = self.facts.constant_products.repeated(id)? {
                    View::Repeat {
                        value: Constant(value),
                        length,
                    }
                } else {
                    View::Product {
                        length: self.facts.constant_products.length(id)?,
                        variant: self.facts.constant_products.variant(id)?,
                    }
                }
            }
        })
    }
    /// A projected location retains its original base; equal values are not
    /// interchangeable storage. Following origins needs no expression evaluation.
    /// # Errors
    /// Rejects an invalid location identity.
    pub fn location(self, id: LocationId) -> Result<Location, &'static str> {
        let place = self.facts.constant_references.get(Value::Reference(id.0))?;
        let origin =
            if let Some((base, index)) = self.facts.constant_references.projection(place)? {
                Origin::Projection {
                    base: LocationId(base),
                    index,
                }
            } else if let references::Access::Static(id) = place.access {
                Origin::Static(id)
            } else {
                Origin::Allocation
            };
        Ok(Location {
            value: place.value.map(Constant),
            ty: place.ty,
            origin,
        })
    }
    /// # Errors
    /// Rejects non-products or out-of-bounds field selection.
    pub fn field(self, constant: Constant, index: u64) -> Result<Constant, &'static str> {
        match constant.0 {
            Value::Product(id) => self.facts.constant_products.get(id, index).map(Constant),
            Value::Bytes(id) => self
                .facts
                .texts
                .get(id)?
                .get(usize::try_from(index).map_err(|_| "constant index exceeds host size")?)
                .copied()
                .map(|b| Constant::scalar(Scalar::Integer(u128::from(b))))
                .ok_or("constant index is out of bounds"),
            _ => Err("constant projection requires a product"),
        }
    }
    /// # Errors
    /// Rejects an invalid interned byte sequence.
    pub fn bytes(self, id: crate::sema::text::Id) -> Result<&'a [u8], &'static str> {
        self.facts.texts.get(id)
    }
}
impl Constant {
    /// Read an evaluated generic scalar without confusing a rigid parameter with a definition.
    /// # Errors
    /// Rejects unknown constants, invalid scalar kinds or malformed character bits.
    pub fn argument(
        store: &crate::types::Store,
        id: crate::types::arguments::ConstId,
    ) -> Result<Option<Self>, &'static str> {
        let value = match store.const_kind(id) {
            Some(crate::types::arguments::ConstKind::Value(value)) => value,
            Some(crate::types::arguments::ConstKind::Parameter(_)) => return Ok(None),
            None => return Err("missing canonical const argument"),
        };
        let scalar = match store.kind(value.ty) {
            Some(Type::Primitive(Primitive::Bool)) => Scalar::Boolean(value.bits != 0),
            Some(Type::Primitive(Primitive::Char)) => Scalar::Character(
                u32::try_from(value.bits)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or("invalid canonical char argument")?,
            ),
            Some(Type::Primitive(primitive)) if primitive.integer().is_some() => {
                Scalar::Integer(value.bits)
            }
            _ => return Err("invalid canonical scalar argument type"),
        };
        Ok(Some(Self::scalar(scalar)))
    }
    #[must_use]
    pub const fn string(id: crate::sema::text::Id) -> Self {
        Self(Value::Text(id))
    }
    #[must_use]
    pub const fn bytes(id: crate::sema::text::Id) -> Self {
        Self(Value::Bytes(id))
    }
    #[must_use]
    pub const fn scalar(value: Scalar) -> Self {
        Self(Value::Scalar(value))
    }
    #[must_use]
    pub const fn function(id: crate::types::instance::Instance) -> Self {
        Self(Value::Function(id))
    }
    #[must_use]
    pub const fn as_function(self) -> Option<crate::types::instance::Instance> {
        if let Value::Function(id) = self.0 {
            Some(id)
        } else {
            None
        }
    }
    /// # Errors
    /// Rejects a non-scalar value.
    pub fn as_scalar(self) -> Result<Scalar, Failure> {
        self.0.scalar().map_err(Failure::Invalid)
    }
}
/// Promotion leaves overflow assertions in the original CFG. Only its value
/// computation wraps; required constant evaluation continues to diagnose overflow.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    Checked,
    Wrapping,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// A valid operation that would fail at evaluation time.
    Fault(&'static str),
    /// Missing or inconsistent typed facts, including arena/budget exhaustion.
    Invalid(&'static str),
}
impl Failure {
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::Fault(message) | Self::Invalid(message) => message,
        }
    }
}
impl From<&'static str> for Failure {
    fn from(message: &'static str) -> Self {
        Self::Invalid(message)
    }
}
impl std::fmt::Display for Failure {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str(self.message())
    }
}
impl std::error::Error for Failure {}

pub struct Evaluator<'a> {
    queries: Queries<'a>,
}
impl<'a> Evaluator<'a> {
    #[must_use]
    pub fn new(report: &'a Report, facts: &'a mut Declarations) -> Self {
        Self {
            queries: Queries::new(report, facts),
        }
    }
    #[must_use]
    pub fn facts(&self) -> &Declarations {
        self.queries.typing.facts
    }
    /// Demand a named constant once in this batch, reusing previously solved values.
    /// # Errors
    /// Preserves required-constant diagnostics; runtime faults are not ignored here.
    pub fn definition(&mut self, id: DefId) -> Result<Constant, Error> {
        self.named(id, false)
    }
    /// Demand a static's initializer without reading or moving its storage.
    /// # Errors
    /// Preserves constant-evaluation errors and rejects non-static definitions.
    pub fn initializer(&mut self, id: DefId) -> Result<Constant, Error> {
        self.named(id, true)
    }
    /// Complete backing for every static address discovered by this evaluation.
    /// # Errors
    /// Preserves initializer errors, including actual value dependency cycles.
    pub fn complete_static_storage(&mut self) -> Result<(), Error> {
        let mut index = 0;
        while let Some(&id) = self
            .queries
            .typing
            .facts
            .constant_references
            .static_order
            .get(index)
        {
            self.initializer(id)?;
            index += 1;
        }
        Ok(())
    }
    fn named(&mut self, id: DefId, static_: bool) -> Result<Constant, Error> {
        let index = self
            .queries
            .constants
            .get(id.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| {
                Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "missing constant definition",
                )
            })?;
        let is_static = matches!(
            self.queries.typing.facts.initializers[index].kind,
            InitializerKind::Static { .. }
        );
        let query = if static_ {
            Query::Static(index)
        } else {
            Query::Constant(index)
        };
        if is_static != static_ {
            return Err(self.queries.error(
                query,
                if static_ {
                    "initializer demand requires a static"
                } else {
                    "promotion cannot read static storage"
                },
            ));
        }
        self.queries.work.push(Work::Enter(query));
        self.queries.drain()?;
        self.queries.value(query).map(Constant)
    }
    /// # Errors
    /// Diagnoses exhausted constant storage.
    pub fn fields(
        &mut self,
        fields: &[Constant],
        variant: Option<VariantId>,
    ) -> Result<Constant, Failure> {
        if fields.is_empty()
            && let Some(variant) = variant
        {
            return Ok(Constant(Value::Variant(variant)));
        }
        self.queries
            .typing
            .facts
            .constant_products
            .sequence(fields.iter().map(|c| c.0), variant)
            .map(Constant)
            .map_err(Failure::Invalid)
    }
    /// # Errors
    /// Diagnoses exhausted constant storage.
    pub fn repeat(&mut self, value: Constant, length: u64) -> Result<Constant, Failure> {
        self.queries
            .typing
            .facts
            .constant_products
            .repeat(value.0, length)
            .map(Constant)
            .map_err(Failure::Invalid)
    }
    /// # Errors
    /// Separates out-of-bounds evaluation from malformed value storage.
    pub fn field(&self, value: Constant, index: u64) -> Result<Constant, Failure> {
        let facts = &self.queries.typing.facts;
        match value.0 {
            Value::Product(id) => {
                if index >= facts.constant_products.length(id)? {
                    return Err(Failure::Fault("constant index is out of bounds"));
                }
                facts
                    .constant_products
                    .get(id, index)
                    .map(Constant)
                    .map_err(Failure::Invalid)
            }
            Value::Bytes(id) => {
                let bytes = facts.texts.get(id)?;
                let byte = usize::try_from(index)
                    .ok()
                    .and_then(|i| bytes.get(i))
                    .copied()
                    .ok_or(Failure::Fault("constant index is out of bounds"))?;
                Ok(Constant::scalar(Scalar::Integer(u128::from(byte))))
            }
            _ => Err(Failure::Invalid("constant projection requires a product")),
        }
    }
    /// # Errors
    /// Diagnoses exhausted constant reference storage.
    pub fn borrow(&mut self, value: Constant, ty: TypeId) -> Result<Constant, Failure> {
        self.queries
            .typing
            .facts
            .constant_references
            .insert(references::Place {
                value: Some(value.0),
                ty,
                access: references::Access::Shared,
                packed: None,
                projection: None,
            })
            .map(Constant)
            .map_err(Failure::Invalid)
    }
    /// # Errors
    /// Rejects a missing shared reference value.
    pub fn referent(&self, value: Constant) -> Result<Constant, Failure> {
        self.queries
            .typing
            .facts
            .constant_references
            .get(value.0)
            .and_then(|p| {
                p.value
                    .map(Constant)
                    .ok_or("promotion cannot read static storage")
            })
            .map_err(Failure::Invalid)
    }
    /// # Errors
    /// Reports typed arithmetic faults separately from invalid facts.
    pub fn unary(
        &self,
        op: Unary,
        ty: TypeId,
        value: Constant,
        literal: bool,
        overflow: Overflow,
    ) -> Result<Constant, Failure> {
        arithmetic::unary(
            op,
            self.primitive(ty)?,
            value.as_scalar()?,
            literal,
            overflow,
        )
        .map(Constant::scalar)
    }
    /// # Errors
    /// Reports typed arithmetic faults separately from invalid facts.
    pub fn binary(
        &self,
        op: Punctuation,
        ty: TypeId,
        left: Constant,
        right: Constant,
        overflow: Overflow,
    ) -> Result<Constant, Failure> {
        arithmetic::binary(
            op,
            self.primitive(ty)?,
            left.as_scalar()?,
            right.as_scalar()?,
            overflow,
        )
        .map(Constant::scalar)
    }
    /// # Errors
    /// Requires the already selected scalar, enum or function-pointer cast.
    pub fn cast(
        &self,
        kind: Cast,
        source: TypeId,
        target: TypeId,
        value: Constant,
    ) -> Result<Constant, Failure> {
        if kind == Cast::FunctionPointer && value.as_function().is_some() {
            return Ok(value);
        }
        let (source, value, kind) = if kind == Cast::EnumToInteger {
            let Value::Variant(id) = value.0 else {
                return Err(Failure::Invalid("missing constant enum variant"));
            };
            let facts = self.facts();
            let member = facts
                .members
                .variants
                .get(id.0 as usize)
                .ok_or("missing constant variant")?;
            let shape = facts
                .aggregates
                .get(member.owner)
                .ok_or("missing enum representation")?;
            let variant = facts
                .aggregates
                .variants(shape)
                .get(member.ordinal as usize)
                .ok_or("missing enum discriminant")?;
            (
                shape.repr.tag.unwrap_or(Primitive::Isize),
                Scalar::Integer(variant.discriminant),
                Cast::IntegerToInteger,
            )
        } else {
            (self.primitive(source)?, value.as_scalar()?, kind)
        };
        arithmetic::cast(kind, source, self.primitive(target)?, value).map(Constant::scalar)
    }
    fn primitive(&self, ty: TypeId) -> Result<Primitive, Failure> {
        match self.facts().types.kind(ty) {
            Some(Type::Primitive(p)) => Ok(p),
            _ => Err(Failure::Invalid("missing selected constant scalar type")),
        }
    }
}
