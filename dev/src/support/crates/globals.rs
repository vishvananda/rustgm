//! Declared storage retains linkage and identity independently of initializer values.
mod validity;
use super::{
    ExternalDef, ExternalKind, MAX_COMPONENTS, MAX_FUNCTIONS, ScalarValue, Symbol, TypeRef,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ValueRef(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StaticRef(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoredKind {
    Scalar(ScalarValue),
    Product(Box<[ValueRef]>),
    Repeat(ValueRef),
    Bytes(Box<[u8]>),
    Reference {
        target: StaticRef,
        fields: Box<[u32]>,
    },
    ProjectedReference {
        target: StaticRef,
        path: Box<[StoredProjection]>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoredProjection {
    Field(u32),
    Element(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredValue {
    pub ty: TypeRef,
    pub kind: StoredKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageOrigin {
    Defined(ValueRef),
    Foreign { unsafe_: bool },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Static {
    definition: ExternalDef,
    ty: TypeRef,
    symbol: Symbol,
    public: bool,
    mutable: bool,
    origin: StorageOrigin,
}
impl Static {
    /// Preserve the real storage identity, type, permissions and initializer root.
    /// # Errors
    /// Rejects nonstatic identities and invalid exact symbol spellings.
    pub fn new(
        definition: ExternalDef,
        ty: TypeRef,
        symbol: Symbol,
        public: bool,
        mutable: bool,
        origin: StorageOrigin,
    ) -> Result<Self, &'static str> {
        if definition.kind() != ExternalKind::Static {
            return Err("metadata storage entry is not a static");
        }
        symbol.validate()?;
        if matches!(origin, StorageOrigin::Foreign { .. }) && symbol == Symbol::Rust {
            return Err("foreign metadata storage requires an exact symbol");
        }
        Ok(Self {
            definition,
            ty,
            symbol,
            public,
            mutable,
            origin,
        })
    }
    #[must_use]
    pub const fn definition(&self) -> &ExternalDef {
        &self.definition
    }
    #[must_use]
    pub const fn ty(&self) -> TypeRef {
        self.ty
    }
    #[must_use]
    pub const fn symbol(&self) -> &Symbol {
        &self.symbol
    }
    #[must_use]
    pub const fn public(&self) -> bool {
        self.public
    }
    #[must_use]
    pub const fn mutable(&self) -> bool {
        self.mutable
    }
    #[must_use]
    pub const fn origin(&self) -> StorageOrigin {
        self.origin
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Storage {
    statics: Box<[Static]>,
    values: Box<[StoredValue]>,
}
impl Storage {
    /// Retain canonical declaration ordinals and postorder value nodes.
    /// # Errors
    /// Rejects empty, unbounded or noncanonical tables before further validation.
    pub fn new(statics: Vec<Static>, values: Vec<StoredValue>) -> Result<Self, &'static str> {
        if statics.is_empty() || statics.len() > MAX_FUNCTIONS || values.len() > MAX_COMPONENTS {
            return Err("metadata storage table exceeds profile budget");
        }
        if statics
            .windows(2)
            .any(|pair| pair[0].definition >= pair[1].definition)
        {
            return Err("metadata static identities are duplicate or noncanonical");
        }
        Ok(Self {
            statics: statics.into_boxed_slice(),
            values: values.into_boxed_slice(),
        })
    }
    #[must_use]
    pub fn statics(&self) -> &[Static] {
        &self.statics
    }
    #[must_use]
    pub fn values(&self) -> &[StoredValue] {
        &self.values
    }
}
