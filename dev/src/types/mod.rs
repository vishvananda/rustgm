//! Immutable canonical type shapes and interned child lists; no source/text keys.
pub mod aggregate;
pub mod arguments;
pub mod auto_traits;
mod binder_shift;
pub mod bound_uses;
pub mod callable;
pub mod closure;
pub mod copy;
pub mod destruction;
pub mod dynamic;
pub mod equality;
pub mod infer;
pub mod instance;
pub mod interior_mutability;
pub mod length;
pub mod lifetime_proof;
pub mod opaque;
pub mod outlives;
pub mod parameters;
pub mod projection;
mod properties;
pub mod region_graph;
pub mod relations;
pub(crate) mod shape_key;
pub mod sizedness;
pub mod specialize;
pub(crate) mod structural;
pub(crate) mod subject_head;
pub mod substitute;
pub mod traits;
pub mod unsizing;
pub mod variance;

use crate::support::hash::HashMap;
use crate::support::intern_slice::Slices;
use properties::{Properties, Property};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ListId(pub u32);
/// Canonical lifetime/type/const arguments, in declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GenericArgs(pub u32);
impl GenericArgs {
    pub const EMPTY: Self = Self(0);
}
/// Semantic declaration identity supplied by the owning type-checking context.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemId(pub u32);
/// Regions retain identity independently of their later outlives constraints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegionId(pub u32);
impl RegionId {
    pub const STATIC: Self = Self(0);
    /// # Errors
    /// Rejects binder depth beyond the compact bound-region encoding.
    pub const fn bound(depth: u16, index: u16) -> Result<Self, &'static str> {
        if depth >= 0x8000 {
            return Err("region binder nesting exceeds 32767");
        }
        Ok(Self(0x8000_0000 | ((depth as u32) << 16) | index as u32))
    }
    #[must_use]
    pub fn bound_parts(self) -> Option<(u16, u16)> {
        if self.0 & 0x8000_0000 == 0 {
            return None;
        }
        let bytes = self.0.to_be_bytes();
        Some((
            u16::from_be_bytes([bytes[0] & 0x7f, bytes[1]]),
            u16::from_be_bytes([bytes[2], bytes[3]]),
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Primitive {
    Bool,
    Char,
    Str,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
}
impl Primitive {
    pub const ALL: [Self; 17] = [
        Self::Bool,
        Self::Char,
        Self::Str,
        Self::I8,
        Self::I16,
        Self::I32,
        Self::I64,
        Self::I128,
        Self::Isize,
        Self::U8,
        Self::U16,
        Self::U32,
        Self::U64,
        Self::U128,
        Self::Usize,
        Self::F32,
        Self::F64,
    ];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Char => "char",
            Self::Str => "str",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I128 => "i128",
            Self::Isize => "isize",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::Usize => "usize",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }
    /// Signedness and width in the initial x86-64 target profile.
    #[must_use]
    pub const fn integer(self) -> Option<(bool, u16)> {
        Some(match self {
            Self::I8 => (true, 8),
            Self::I16 => (true, 16),
            Self::I32 => (true, 32),
            Self::I64 | Self::Isize => (true, 64),
            Self::I128 => (true, 128),
            Self::U8 => (false, 8),
            Self::U16 => (false, 16),
            Self::U32 => (false, 32),
            Self::U64 | Self::Usize => (false, 64),
            Self::U128 => (false, 128),
            _ => return None,
        })
    }
    #[must_use]
    pub const fn is_float(self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }
}
impl TypeId {
    pub const NEVER: Self = Self(0);
    pub const UNIT: Self = Self(1);
    #[must_use]
    pub const fn primitive(value: Primitive) -> Self {
        Self(2 + value as u32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Never,
    Primitive(Primitive),
    Parameter(parameters::ParameterId),
    /// Nominal identity; field definitions and by-value cycles are checked separately.
    Adt(ItemId, GenericArgs),
    /// An associated type application is not injective in Self or its arguments.
    Projection(projection::Application),
    /// Declaration identity and captured inputs; representation is private to its body.
    Opaque(opaque::OpaqueId, GenericArgs),
    /// Trait inputs are inside one binder; the object lifetime is outside it.
    Dynamic {
        shape: dynamic::ShapeId,
        arguments: GenericArgs,
        bound_regions: u32,
        region: RegionId,
    },
    Tuple(ListId),
    Array {
        element: TypeId,
        length: u64,
    },
    ArrayTerm {
        element: TypeId,
        length: arguments::ConstId,
    },
    Slice(TypeId),
    Reference {
        mutable: bool,
        region: RegionId,
        referent: TypeId,
    },
    Pointer {
        mutable: bool,
        pointee: TypeId,
    },
    Function {
        bound_regions: u32,
        parameters: ListId,
        result: TypeId,
        unsafe_: bool,
        convention: callable::Convention,
    },
    FunctionItem {
        item: ItemId,
        arguments: GenericArgs,
        signature: TypeId,
    },
    /// Distinct expression identity, complete enclosing inputs and actual stored captures.
    Closure {
        identity: closure::ClosureId,
        arguments: GenericArgs,
        signature: TypeId,
        environment: TypeId,
    },
}
/// Child lists occupy one shared arena. Hash collisions compare their typed slices.
/// Type/list IDs are valid only in the store that produced them.
pub struct Store {
    properties: Vec<Properties>,
    kinds: Vec<Kind>,
    index: HashMap<Kind, TypeId>,
    children: Slices<TypeId>,
    regions: Slices<RegionId>,
    arguments: Option<Box<arguments::Storage>>,
    shapes: Option<Box<substitute::Bound>>,
    auto_shapes: Option<Box<substitute::Bound>>,
    opaques: Option<Vec<opaque::Declaration>>,
    opaque_outlives: Option<Box<opaque::bounds::Declarations>>,
    storage_heads: Option<HashMap<TypeId, TypeId>>,
    shifts: Option<Box<binder_shift::Shifts>>,
    dynamics: Option<Box<dynamic::Storage>>,
    closures: Option<Vec<closure::Declaration>>,
    nominal_sizes: Option<std::sync::Arc<sizedness::Rules>>,
    has_nominals: bool,
}
impl Default for Store {
    fn default() -> Self {
        let mut kinds = vec![Kind::Never, Kind::Tuple(ListId(0))];
        kinds.extend(Primitive::ALL.map(Kind::Primitive));
        let index = kinds
            .iter()
            .copied()
            .zip(0_u32..)
            .map(|(kind, id)| (kind, TypeId(id)))
            .collect();
        let properties = kinds
            .iter()
            .map(|kind| {
                let mut properties = Properties::default();
                properties.set(Property::Sized, *kind != Kind::Primitive(Primitive::Str));
                properties
            })
            .collect();
        Self {
            properties,
            kinds,
            index,
            children: Slices::default(),
            regions: Slices::default(),
            arguments: None,
            shapes: None,
            auto_shapes: None,
            opaques: None,
            opaque_outlives: None,
            storage_heads: None,
            shifts: None,
            dynamics: None,
            closures: None,
            nominal_sizes: None,
            has_nominals: false,
        }
    }
}
impl Store {
    pub(crate) fn find_kind(&self, kind: Kind) -> Option<TypeId> {
        self.index.get(&kind).copied()
    }
    pub(crate) fn find_list(&self, values: &[TypeId]) -> Option<ListId> {
        self.children.find(values).map(ListId)
    }
    #[must_use]
    pub fn kind(&self, id: TypeId) -> Option<Kind> {
        self.kinds.get(id.0 as usize).copied()
    }
    /// Structural sizedness, including a type parameter's declared Sized requirement.
    #[must_use]
    pub fn is_sized(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::Sized))
    }
    /// Canonical shapes may exist before a use demands their well-formedness.
    /// This does not discharge region, trait or generic-environment obligations.
    #[must_use]
    pub fn is_well_formed(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::WellFormed))
    }
    /// Number of enclosing binders needed by escaping bound lifetimes.
    #[must_use]
    pub fn escaping_binders(&self, id: TypeId) -> Option<u16> {
        self.properties.get(id.0 as usize).map(|p| p.escaping)
    }
    #[must_use]
    pub fn contains_regions(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::Regions))
    }
    /// Representation reveal visits opaque identities even when their captures are closed.
    #[must_use]
    pub fn contains_opaques(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::Opaques))
    }
    /// Normalization must visit projections even when all of their inputs are closed.
    #[must_use]
    pub fn contains_projections(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::Projections))
    }
    #[must_use]
    pub fn contains_late_callables(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::LateCallables))
    }
    #[must_use]
    pub(crate) fn contains_callables(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::Callables))
    }
    #[must_use]
    pub fn contains_parameters(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::Parameters))
    }
    #[must_use]
    pub fn has_free_regions(&self, id: TypeId, depth: u16) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::FreeRegions) || p.escaping > depth)
    }
    /// Static lifetimes and lifetimes bound inside the type need no outer scope.
    #[must_use]
    pub fn needs_region_context(&self, id: TypeId) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.has(Property::NonStaticFreeRegions) || p.escaping != 0)
    }
    #[must_use]
    pub fn list(&self, id: ListId) -> Option<&[TypeId]> {
        self.children.get(id.0)
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.kinds.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.children.edge_count()
    }
    /// # Errors
    /// Rejects out-of-range IDs and compact arena overflow without changing the store.
    pub fn intern_list(&mut self, values: &[TypeId]) -> Result<ListId, &'static str> {
        if values.iter().any(|id| self.kind(*id).is_none()) {
            return Err("invalid type in canonical child list");
        }
        self.children.intern(values).map(ListId)
    }
    /// Intern only types whose children already exist: canonical ownership is acyclic.
    ///
    /// # Errors
    /// Rejects invalid children, non-function item signatures and identity overflow.
    pub fn intern(&mut self, mut kind: Kind) -> Result<TypeId, &'static str> {
        if let Kind::ArrayTerm { element, length } = kind {
            if self.const_type(length) != Some(TypeId::primitive(Primitive::Usize)) {
                return Err("array length term must have type usize");
            }
            if let Some(value) = self.constant(length) {
                kind = Kind::Array {
                    element,
                    length: u64::try_from(value.bits)
                        .map_err(|_| "array length exceeds target usize")?,
                };
            }
        }
        if let Some(id) = self.index.get(&kind) {
            return Ok(*id);
        }
        let valid_type = |id: TypeId| self.kind(id).is_some();
        let valid = match kind {
            Kind::Never | Kind::Primitive(_) => true,
            Kind::Parameter(id) => self
                .parameter(id)
                .is_some_and(|p| matches!(p.kind, parameters::ParameterKind::Type { .. })),
            Kind::Adt(_, args) => self.arguments(args).is_some(),
            Kind::Opaque(id, args) => {
                self.opaque_declaration(id).is_some() && self.arguments(args).is_some()
            }
            Kind::Dynamic {
                shape, arguments, ..
            } => self.valid_dynamic_arguments(shape, arguments),
            Kind::Projection(application) => self.valid_projection(application),
            Kind::Tuple(list) => self.list(list).is_some(),
            Kind::Slice(element)
            | Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. } => valid_type(element),
            Kind::Reference { referent, .. } => valid_type(referent),
            Kind::Pointer { pointee, .. } => valid_type(pointee),
            Kind::Function {
                parameters,
                result,
                convention,
                ..
            } => {
                self.list(parameters).is_some()
                    && valid_type(result)
                    && (!convention.variadic || convention.abi == callable::Abi::C)
            }
            Kind::FunctionItem {
                signature,
                arguments,
                ..
            } => {
                self.arguments(arguments).is_some()
                    && matches!(self.kind(signature), Some(Kind::Function { .. }))
            }
            Kind::Closure {
                identity,
                arguments,
                signature,
                environment,
            } => {
                self.closure_kind(identity).is_some()
                    && self.arguments(arguments).is_some()
                    && matches!(
                        self.kind(signature),
                        Some(Kind::Function {
                            unsafe_: false,
                            convention: callable::Convention::RUST,
                            ..
                        })
                    )
                    && matches!(self.kind(environment), Some(Kind::Tuple(_)))
            }
        };
        if !valid {
            return Err("invalid canonical type child or signature");
        }
        let id = TypeId(u32::try_from(self.kinds.len()).map_err(|_| "too many canonical types")?);
        let properties = self.properties(kind)?;
        self.has_nominals |= matches!(kind, Kind::Adt(..));
        self.kinds.push(kind);
        self.properties.push(properties);
        self.index.insert(kind, id);
        Ok(id)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive constructor dispatch computes immutable structural summaries"
    )]
    fn properties(&self, kind: Kind) -> Result<Properties, &'static str> {
        let child = |ty: TypeId| {
            self.properties
                .get(ty.0 as usize)
                .copied()
                .ok_or("invalid structural type child")
        };
        let mut result = Properties::default();
        match kind {
            Kind::Never => {}
            Kind::Parameter(id) => {
                let parameter = self.parameter(id).ok_or("invalid type parameter")?;
                let parameters::ParameterKind::Type { sized } = parameter.kind else {
                    return Err("const parameter used as a type");
                };
                result.set(Property::Sized, sized);
                result.set(Property::Parameters, true);
            }
            Kind::Adt(item, args) => {
                result = self.nominal_properties(item, args)?;
            }
            Kind::Projection(application) => {
                result = self.projection_properties(application)?;
            }
            Kind::Opaque(id, arguments) => result = self.opaque_properties(id, arguments)?,
            Kind::Dynamic { .. } => result = self.dynamic_properties(kind)?,
            Kind::Closure {
                arguments,
                signature,
                environment,
                ..
            } => {
                result = self.closure_properties(arguments, signature, environment)?;
            }
            Kind::Primitive(p) => result.set(Property::Sized, p != Primitive::Str),
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => {
                let element = child(element)?;
                result.include(element);
                result.set(
                    Property::WellFormed,
                    element.has(Property::WellFormed) && element.has(Property::Sized),
                );
                result.set(
                    Property::Sized,
                    !matches!(kind, Kind::Slice(_)) && element.has(Property::Sized),
                );
                if matches!(kind, Kind::ArrayTerm { .. }) {
                    result.set(Property::Parameters, true);
                }
            }
            Kind::Tuple(list) => {
                let elements = self.list(list).ok_or("invalid structural tuple list")?;
                for (i, ty) in elements.iter().enumerate() {
                    let p = child(*ty)?;
                    result.include(p);
                    result.set(
                        Property::WellFormed,
                        result.has(Property::WellFormed)
                            && (p.has(Property::Sized) || i + 1 == elements.len()),
                    );
                    result.set(
                        Property::Sized,
                        result.has(Property::Sized) && p.has(Property::Sized),
                    );
                }
            }
            Kind::Reference { referent: ty, .. }
            | Kind::Pointer { pointee: ty, .. }
            | Kind::FunctionItem { signature: ty, .. } => {
                result.include(child(ty)?);
            }
            Kind::Function {
                parameters,
                result: ty,
                ..
            } => {
                result.include(child(ty)?);
                for ty in self
                    .list(parameters)
                    .ok_or("invalid structural parameter list")?
                {
                    result.include(child(*ty)?);
                }
            }
        }
        match kind {
            Kind::FunctionItem { arguments, .. } => {
                // Item identity is inferred atomically; reification selects its signature.
                self.argument_properties(arguments, &mut result)?;
                result.set(Property::LateCallables, false);
            }
            Kind::Reference { region, .. } => {
                result.region(region);
            }
            Kind::Function { bound_regions, .. } => {
                result.set(Property::Callables, true);
                result.escaping = result.escaping.saturating_sub(1);
                result.set(
                    Property::LateCallables,
                    result.has(Property::LateCallables) || bound_regions != 0,
                );
            }
            _ => {}
        }
        Ok(result)
    }
}
