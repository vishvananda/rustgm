//! Typed value operations shared by source constants and CFG promotion queries.
mod byte_codec;
mod relocations;
pub use relocations::{ImageFragment, WordTarget};
pub(crate) use relocations::{MetadataShape, WordAddress, WordShape};
mod callables;
mod computed;
mod footprints;
pub(crate) use footprints::{ProjectionStorage, StorageLayout, StoredShape};
pub mod execution;
mod memory;
mod mutation;
mod session;
mod structure;
mod tags;
pub use structure::FieldTypes;
mod array_storage;
mod inline;
mod union_storage;
mod validation;
pub(crate) use super::instances::Completed;
pub(crate) use super::instances::Dependency;
pub use super::instances::Instance;
pub use super::references::{Metadata, WritePermission};
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
pub(crate) use session::Scratch;
pub use union_storage::ImageView;

/// A value in the declaration result's shared constant arenas.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Constant(pub(super) Value);

/// Opaque location identity in the declaration result's flat reference arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocationId(references::Id);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Allocation,
    Static(DefId),
    Projection {
        base: LocationId,
        index: u64,
    },
    Subsequence {
        base: LocationId,
        from: u64,
        length: u64,
    },
    ByteOffset {
        base: LocationId,
        offset: u64,
    },
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
    Image {
        size: u64,
    },
    Function(crate::types::instance::Instance),
    Reference(LocationId),
    Address {
        bits: u64,
    },
    BytePointer {
        text: crate::sema::text::Id,
    },
    Text(crate::sema::text::Id),
    Bytes(crate::sema::text::Id),
    ByteRange {
        text: crate::sema::text::Id,
        from: u64,
        length: u64,
    },
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
    #[must_use]
    pub fn inline_instance(
        self,
        instance: crate::sema::inline_constants::InlineInstance,
        result: TypeId,
    ) -> Option<Constant> {
        self.facts
            .inline_constants
            .as_deref()?
            .instance_value(instance, result)
    }
    #[must_use]
    pub fn inline(self, source: crate::resolve::unit::NodeKey) -> Option<Constant> {
        self.facts.inline_constants.as_deref()?.get(source)?.value
    }
    #[must_use]
    pub fn associated(self, instance: crate::sema::values::Instance) -> Option<Constant> {
        self.facts
            .constant_instances
            .get(&instance)
            .map(|completed| completed.value)
    }
    pub(crate) fn associated_values(
        self,
    ) -> impl Iterator<Item = (Instance, Option<TypeId>, Constant)> + 'a {
        self.facts
            .constant_instances
            .iter()
            .map(|(&instance, value)| (instance, value.ty, value.value))
    }
    /// # Errors
    /// Rejects missing or malformed constant arena entries.
    pub fn view(self, constant: Constant) -> Result<View, &'static str> {
        Ok(match constant.0 {
            Value::Scalar(value) => View::Scalar(value),
            Value::Image(_) => View::Image {
                size: self.image(constant)?.bytes.len() as u64,
            },
            Value::Function(id) => {
                let target = self.facts.generics.constant_callable_targets.get(&id.ty());
                View::Function(if let Some(&ty) = target {
                    crate::types::instance::Instance::new(&self.facts.types, ty)?
                } else {
                    id
                })
            }
            Value::Reference(pointer) => View::Reference(LocationId(pointer.place)),
            Value::Address { bits, .. } => View::Address { bits },
            Value::BytePointer { text, .. } => View::BytePointer { text },
            Value::Text(id) => View::Text(id),
            Value::Bytes(id) => View::Bytes(id),
            Value::Variant(id) => View::Product {
                length: 0,
                variant: Some(id),
            },
            Value::Product(id) => self.facts.constant_products.view(id)?,
        })
    }
    /// A projected location retains its original base; equal values are not
    /// interchangeable storage. Following origins needs no expression evaluation.
    /// # Errors
    /// Rejects an invalid location identity.
    pub fn location(self, id: LocationId) -> Result<Location, &'static str> {
        let place = self
            .facts
            .constant_references
            .get(Value::Reference(references::Pointer::new(id.0)))?;
        let origin = match self.facts.constant_references.projection(place)? {
            Some(references::ProjectionView::Field { base, index }) => Origin::Projection {
                base: LocationId(base),
                index,
            },
            Some(references::ProjectionView::Sequence { base, from, length }) => {
                Origin::Subsequence {
                    base: LocationId(base),
                    from,
                    length,
                }
            }
            Some(references::ProjectionView::Byte { base, offset }) => Origin::ByteOffset {
                base: LocationId(base),
                offset,
            },
            None => {
                if let references::Access::Static(id) = place.access {
                    Origin::Static(id)
                } else {
                    Origin::Allocation
                }
            }
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
            Value::Product(id) => self
                .facts
                .constant_products
                .get(id, index, &self.facts.texts)
                .map(Constant),
            Value::Bytes(id) => self
                .facts
                .texts
                .get(id)?
                .get(usize::try_from(index).map_err(|_| "constant index exceeds host size")?)
                .copied()
                .map(|b| Constant::scalar(Scalar::Integer(u128::from(b))))
                .ok_or("constant index is out of bounds"),
            Value::Image(_) => self.array_image_field(constant, index),
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
    /// Selected wide-pointer metadata does not alter the backing location.
    #[must_use]
    pub const fn metadata(self) -> Option<Metadata> {
        self.0.metadata()
    }
    #[must_use]
    pub const fn length(self) -> Option<u64> {
        match self.metadata() {
            Some(Metadata::Length(length)) => Some(length),
            _ => None,
        }
    }
    /// Attach the already-proved terminal array length without allocating storage.
    /// # Errors
    /// Rejects values without an address representation.
    pub fn unsize(self, length: u64) -> Result<Self, Failure> {
        self.with_metadata(Metadata::Length(length))
    }
    /// Attach the selected pointer metadata, retaining backing storage identity.
    /// # Errors
    /// Rejects values without an address representation.
    pub fn with_metadata(self, metadata: Metadata) -> Result<Self, Failure> {
        super::address_values::unsize(self.0, metadata).map(Self)
    }
    /// Change a proved object's view while keeping its concrete backing identity.
    /// # Errors
    /// Rejects a pointer without object metadata.
    pub fn upcast(self, target: TypeId) -> Result<Self, Failure> {
        let Some(Metadata::Object { concrete, .. }) = self.metadata() else {
            return Err(Failure::Invalid("constant upcast requires object metadata"));
        };
        self.with_metadata(Metadata::Object { concrete, target })
    }
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
        if self
            .queries
            .typing
            .facts
            .crates
            .as_deref()
            .is_some_and(|linkage| linkage.imported_constant(id))
        {
            return Reader::new(self.queries.typing.facts)
                .definition(id)
                .ok_or_else(|| {
                    Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "imported constant is missing its validated value",
                    )
                });
        }
        self.named(id, false)
    }
    /// Evaluate one selected associated initializer in its complete argument context.
    /// # Errors
    /// Reports invalid instances, cycles and required constant evaluation faults.
    pub fn associated(
        &mut self,
        instance: crate::sema::values::Instance,
    ) -> Result<Constant, Error> {
        let query = Query::Instance(instance);
        self.queries.work.push(Work::Enter(query));
        self.queries.drain()?;
        let value = Constant(self.queries.value(query)?);
        if !self.facts().aggregates.has_shared_mutation() {
            return Ok(value);
        }
        let ty = self
            .facts()
            .definitions
            .get(instance.definition.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| {
                self.queries
                    .error(query, "missing associated constant result type")
            })?;
        let mut substitution = instance
            .substitution(self.facts())
            .map_err(|e| self.queries.error(query, e))?;
        let ty = substitution
            .apply(&mut self.queries.typing.facts.types, ty)
            .map_err(|e| self.queries.error(query, e))?;
        self.checked_reference_result(query, ty, value)
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
            if !self
                .queries
                .typing
                .facts
                .foreign
                .as_deref()
                .is_some_and(|r| r.imports_global(id))
            {
                self.initializer(id)?;
            }
            index += 1;
        }
        Ok(())
    }
    fn named(&mut self, id: DefId, static_: bool) -> Result<Constant, Error> {
        let index = self
            .queries
            .typing
            .facts
            .initializer_index(id)
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
        let scope = self
            .queries
            .value_scope
            .replace(super::instances::ValueScope::Initializer {
                body: self.queries.typing.facts.initializers[index].semantic_source(),
                index,
            });
        self.queries.work.push(Work::Enter(query));
        let result = self
            .queries
            .drain()
            .and_then(|()| self.queries.value(query).map(Constant));
        self.queries.value_scope = scope;
        let ty = self.queries.typing.facts.initializers[index].ty;
        self.checked_reference_result(query, ty, result?)
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
                    .get(id, index, &facts.texts)
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
            Value::Image(_) => Reader::new(facts)
                .array_image_field(value, index)
                .map_err(Failure::Invalid),
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
        if kind == Cast::FunctionPointer {
            return self.reify(source, value);
        }
        let primitive = match self.facts().types.kind(source) {
            Some(Type::Primitive(p)) => Some(p),
            _ => None,
        };
        if let Some(result) = super::address_values::cast(kind, primitive, value.0) {
            return result.map(Constant);
        }
        let (source, value, kind) = if kind == Cast::EnumToInteger {
            let Value::Variant(id) = value.0 else {
                return Err(Failure::Invalid("missing constant enum variant"));
            };
            let (tag, discriminant) = Reader::new(self.facts())
                .discriminant(id)?
                .ok_or("enum cast requires a completed discriminant")?;
            (tag, Scalar::Integer(discriminant), Cast::IntegerToInteger)
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
