//! Immutable canonical type shapes and interned child lists; no source/text keys.
pub mod aggregate;
pub mod arguments;
pub mod bound_uses;
pub mod callable;
pub mod copy;
pub mod destruction;
pub mod equality;
pub mod infer;
pub mod outlives;
pub mod region_graph;
pub mod specialize;
pub mod substitute;
pub mod variance;

use crate::support::intern_slice::Slices;
use std::collections::HashMap;

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
    /// Nominal identity; field definitions and by-value cycles are checked separately.
    Adt(ItemId, GenericArgs),
    Tuple(ListId),
    Array {
        element: TypeId,
        length: u64,
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
    },
    FunctionItem {
        item: ItemId,
        arguments: GenericArgs,
        signature: TypeId,
    },
}
/// Child lists occupy one shared arena. Hash collisions compare their typed slices.
/// Type/list IDs are valid only in the store that produced them.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RegionPresence {
    None,
    Bound,
    Unbound,
}
impl RegionPresence {
    fn of(region: RegionId) -> Self {
        if region.bound_parts().is_some() {
            Self::Bound
        } else {
            Self::Unbound
        }
    }
}
#[derive(Clone, Copy)]
struct Properties {
    sized: bool,
    well_formed: bool,
    escaping: u16,
    regions: RegionPresence,
    late_callables: bool,
}
pub struct Store {
    properties: Vec<Properties>,
    kinds: Vec<Kind>,
    index: HashMap<Kind, TypeId>,
    children: Slices<TypeId>,
    regions: Slices<RegionId>,
    arguments: Option<Box<arguments::Storage>>,
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
            .map(|kind| Properties {
                sized: *kind != Kind::Primitive(Primitive::Str),
                well_formed: true,
                escaping: 0,
                regions: RegionPresence::None,
                late_callables: false,
            })
            .collect();
        Self {
            properties,
            kinds,
            index,
            children: Slices::default(),
            regions: Slices::default(),
            arguments: None,
        }
    }
}
impl Store {
    #[must_use]
    pub fn kind(&self, id: TypeId) -> Option<Kind> {
        self.kinds.get(id.0 as usize).copied()
    }
    /// Structural sizedness for the closed, nongeneric type vocabulary.
    #[must_use]
    pub fn is_sized(&self, id: TypeId) -> Option<bool> {
        self.properties.get(id.0 as usize).map(|p| p.sized)
    }
    /// Canonical shapes may exist before a use demands their well-formedness.
    /// This does not discharge region, trait or generic-environment obligations.
    #[must_use]
    pub fn is_well_formed(&self, id: TypeId) -> Option<bool> {
        self.properties.get(id.0 as usize).map(|p| p.well_formed)
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
            .map(|p| p.regions != RegionPresence::None)
    }
    #[must_use]
    pub fn contains_late_callables(&self, id: TypeId) -> Option<bool> {
        self.properties.get(id.0 as usize).map(|p| p.late_callables)
    }
    #[must_use]
    pub fn has_free_regions(&self, id: TypeId, depth: u16) -> Option<bool> {
        self.properties
            .get(id.0 as usize)
            .map(|p| p.regions == RegionPresence::Unbound || p.escaping > depth)
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
    pub fn intern(&mut self, kind: Kind) -> Result<TypeId, &'static str> {
        if let Some(id) = self.index.get(&kind) {
            return Ok(*id);
        }
        let valid_type = |id: TypeId| self.kind(id).is_some();
        let valid = match kind {
            Kind::Never | Kind::Primitive(_) => true,
            Kind::Adt(_, args) => self.arguments(args).is_some(),
            Kind::Tuple(list) => self.list(list).is_some(),
            Kind::Slice(element) | Kind::Array { element, .. } => valid_type(element),
            Kind::Reference { referent, .. } => valid_type(referent),
            Kind::Pointer { pointee, .. } => valid_type(pointee),
            Kind::Function {
                parameters, result, ..
            } => self.list(parameters).is_some() && valid_type(result),
            Kind::FunctionItem {
                signature,
                arguments,
                ..
            } => {
                self.arguments(arguments).is_some()
                    && matches!(self.kind(signature), Some(Kind::Function { .. }))
            }
        };
        if !valid {
            return Err("invalid canonical type child or signature");
        }
        let id = TypeId(u32::try_from(self.kinds.len()).map_err(|_| "too many canonical types")?);
        let properties = self.properties(kind)?;
        self.kinds.push(kind);
        self.properties.push(properties);
        self.index.insert(kind, id);
        Ok(id)
    }
    fn properties(&self, kind: Kind) -> Result<Properties, &'static str> {
        let child = |ty: TypeId| {
            self.properties
                .get(ty.0 as usize)
                .copied()
                .ok_or("invalid structural type child")
        };
        let mut result = Properties {
            sized: true,
            well_formed: true,
            escaping: 0,
            regions: RegionPresence::None,
            late_callables: false,
        };
        match kind {
            // The closed aggregate profile requires sized, well-formed fields at
            // declaration completion. Nominal recursion is not a structural child edge.
            Kind::Never => {}
            Kind::Adt(_, args) => {
                self.argument_properties(args, &mut result)?;
            }
            Kind::Primitive(p) => result.sized = p != Primitive::Str,
            Kind::Array { element, .. } | Kind::Slice(element) => {
                let element = child(element)?;
                result.escaping = element.escaping;
                result.regions = element.regions;
                result.late_callables = element.late_callables;
                result.well_formed = element.well_formed && element.sized;
                result.sized = matches!(kind, Kind::Array { .. }) && element.sized;
            }
            Kind::Tuple(list) => {
                let elements = self.list(list).ok_or("invalid structural tuple list")?;
                for (i, ty) in elements.iter().enumerate() {
                    let p = child(*ty)?;
                    result.well_formed &= p.well_formed && (p.sized || i + 1 == elements.len());
                    result.sized &= p.sized;
                    result.escaping = result.escaping.max(p.escaping);
                    result.regions = result.regions.max(p.regions);
                    result.late_callables |= p.late_callables;
                }
            }
            Kind::Reference { referent: ty, .. }
            | Kind::Pointer { pointee: ty, .. }
            | Kind::FunctionItem { signature: ty, .. } => {
                let properties = child(ty)?;
                result.well_formed = properties.well_formed;
                result.escaping = properties.escaping;
                result.regions = properties.regions;
                result.late_callables = properties.late_callables;
            }
            Kind::Function {
                parameters,
                result: ty,
                ..
            } => {
                let properties = child(ty)?;
                result.well_formed = properties.well_formed;
                result.escaping = properties.escaping;
                result.regions = properties.regions;
                result.late_callables = properties.late_callables;
                for ty in self
                    .list(parameters)
                    .ok_or("invalid structural parameter list")?
                {
                    let properties = child(*ty)?;
                    result.well_formed &= properties.well_formed;
                    result.escaping = result.escaping.max(properties.escaping);
                    result.regions = result.regions.max(properties.regions);
                    result.late_callables |= properties.late_callables;
                }
            }
        }
        match kind {
            Kind::FunctionItem { arguments, .. } => {
                // Item identity is inferred atomically; reification selects its signature.
                self.argument_properties(arguments, &mut result)?;
                result.late_callables = false;
            }
            Kind::Reference { region, .. } => {
                result.escaping = result.escaping.max(escaping_region(region));
                result.regions = result.regions.max(RegionPresence::of(region));
            }
            Kind::Function { bound_regions, .. } => {
                result.escaping = result.escaping.saturating_sub(1);
                result.late_callables |= bound_regions != 0;
            }
            _ => {}
        }
        Ok(result)
    }
}
fn escaping_region(region: RegionId) -> u16 {
    region.bound_parts().map_or(0, |(depth, _)| depth + 1)
}
