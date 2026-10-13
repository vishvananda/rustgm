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
#[derive(Clone, Copy)]
pub struct Constant(Value);
impl Constant {
    #[must_use]
    pub const fn scalar(value: Scalar) -> Self {
        Self(Value::Scalar(value))
    }
    #[must_use]
    pub const fn function(id: DefId) -> Self {
        Self(Value::Function(id))
    }
    #[must_use]
    pub const fn as_function(self) -> Option<DefId> {
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
            queries: Queries::new(report, facts, &[]),
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
        let query = Query::Constant(index);
        if !matches!(
            self.queries.typing.facts.initializers[index].kind,
            InitializerKind::Const
        ) {
            return Err(self
                .queries
                .error(query, "promotion cannot read static storage"));
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
                value: value.0,
                ty,
                access: references::Access::Shared,
                packed: None,
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
            .map(|p| Constant(p.value))
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
