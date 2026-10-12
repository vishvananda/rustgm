//! Immutable canonical type shapes and interned child lists; no source/text keys.
pub mod infer;

use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ListId(pub u32);
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
        signature: TypeId,
    },
}
struct List {
    start: u32,
    end: u32,
    next: Option<ListId>,
}
/// Child lists occupy one shared arena. Hash collisions compare their typed slices.
/// Type/list IDs are valid only in the store that produced them.
#[derive(Clone, Copy)]
struct Properties {
    sized: bool,
    well_formed: bool,
}
pub struct Store {
    properties: Vec<Properties>,
    kinds: Vec<Kind>,
    index: HashMap<Kind, TypeId>,
    edges: Vec<TypeId>,
    lists: Vec<List>,
    list_index: HashMap<u64, ListId>,
}
fn fingerprint(values: &[TypeId]) -> u64 {
    let mut hasher = DefaultHasher::new();
    values.hash(&mut hasher);
    hasher.finish()
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
            })
            .collect();
        Self {
            properties,
            kinds,
            index,
            edges: Vec::new(),
            lists: vec![List {
                start: 0,
                end: 0,
                next: None,
            }],
            list_index: HashMap::from([(fingerprint(&[]), ListId(0))]),
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
    #[must_use]
    pub fn list(&self, id: ListId) -> Option<&[TypeId]> {
        let list = self.lists.get(id.0 as usize)?;
        self.edges.get(list.start as usize..list.end as usize)
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
        self.edges.len()
    }
    /// # Errors
    /// Rejects out-of-range IDs and compact arena overflow without changing the store.
    pub fn intern_list(&mut self, values: &[TypeId]) -> Result<ListId, &'static str> {
        let hash = fingerprint(values);
        let head = self.list_index.get(&hash).copied();
        let mut next = head;
        while let Some(id) = next {
            if self.list(id) == Some(values) {
                return Ok(id);
            }
            next = self.lists[id.0 as usize].next;
        }
        if values.iter().any(|id| self.kind(*id).is_none()) {
            return Err("invalid type in canonical child list");
        }
        let id = ListId(u32::try_from(self.lists.len()).map_err(|_| "too many type lists")?);
        let start = u32::try_from(self.edges.len()).map_err(|_| "too many type edges")?;
        let end = self
            .edges
            .len()
            .checked_add(values.len())
            .and_then(|size| u32::try_from(size).ok())
            .ok_or("too many type edges")?;
        self.edges.extend_from_slice(values);
        self.lists.push(List {
            start,
            end,
            next: head,
        });
        self.list_index.insert(hash, id);
        Ok(id)
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
            Kind::Tuple(list) => self.list(list).is_some(),
            Kind::Slice(element) | Kind::Array { element, .. } => valid_type(element),
            Kind::Reference { referent, .. } => valid_type(referent),
            Kind::Pointer { pointee, .. } => valid_type(pointee),
            Kind::Function {
                parameters, result, ..
            } => self.list(parameters).is_some() && valid_type(result),
            Kind::FunctionItem { signature, .. } => {
                matches!(self.kind(signature), Some(Kind::Function { .. }))
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
        };
        match kind {
            Kind::Never => {}
            Kind::Primitive(p) => result.sized = p != Primitive::Str,
            Kind::Array { element, .. } | Kind::Slice(element) => {
                let element = child(element)?;
                result.well_formed = element.well_formed && element.sized;
                result.sized = matches!(kind, Kind::Array { .. }) && element.sized;
            }
            Kind::Tuple(list) => {
                let elements = self.list(list).ok_or("invalid structural tuple list")?;
                for (i, ty) in elements.iter().enumerate() {
                    let p = child(*ty)?;
                    result.well_formed &= p.well_formed && (p.sized || i + 1 == elements.len());
                    result.sized &= p.sized;
                }
            }
            Kind::Reference { referent: ty, .. }
            | Kind::Pointer { pointee: ty, .. }
            | Kind::FunctionItem { signature: ty, .. } => {
                result.well_formed = child(ty)?.well_formed;
            }
            Kind::Function {
                parameters,
                result: ty,
                ..
            } => {
                result.well_formed = child(ty)?.well_formed;
                for ty in self
                    .list(parameters)
                    .ok_or("invalid structural parameter list")?
                {
                    result.well_formed &= child(*ty)?.well_formed;
                }
            }
        }
        Ok(result)
    }
}
