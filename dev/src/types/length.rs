//! Fixed-array bounds retain rigid const identity until concrete instantiation.
use super::{
    Kind, Primitive, Store, TypeId,
    arguments::{Argument, ConstId, ConstKind},
    substitute::Bound,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Length {
    Known(u64),
    Parameter(ConstId),
}
impl Length {
    /// # Errors
    /// Rejects non-array types and invalid canonical length arguments.
    pub fn of(store: &Store, ty: TypeId) -> Result<Self, &'static str> {
        match store.kind(ty) {
            Some(Kind::Array { length, .. }) => Ok(Self::Known(length)),
            Some(Kind::ArrayTerm { length, .. }) => Self::argument(store, length),
            _ => Err("indexed type is not a fixed array"),
        }
    }
    fn argument(store: &Store, id: ConstId) -> Result<Self, &'static str> {
        if store.const_type(id) != Some(TypeId::primitive(Primitive::Usize)) {
            return Err("array length must have type usize");
        }
        match store.const_kind(id) {
            Some(ConstKind::Value(value)) => u64::try_from(value.bits)
                .map(Self::Known)
                .map_err(|_| "array length exceeds target usize"),
            Some(ConstKind::Parameter(_)) => Ok(Self::Parameter(id)),
            None => Err("missing array length argument"),
        }
    }
    /// # Errors
    /// Rejects missing or wrongly typed arguments; foreign rigid parameters remain symbolic.
    pub fn substitute(
        self,
        store: &mut Store,
        substitution: &mut Bound,
    ) -> Result<Self, &'static str> {
        let Self::Parameter(id) = self else {
            return Ok(self);
        };
        let Argument::Const(id) = substitution.apply_argument(store, Argument::Const(id))? else {
            return Err("array length substitution changed argument kind");
        };
        Self::argument(store, id)
    }
    #[must_use]
    pub const fn known(self) -> Option<u64> {
        match self {
            Self::Known(value) => Some(value),
            Self::Parameter(_) => None,
        }
    }
}
impl std::fmt::Display for Length {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Known(value) => value.fmt(out),
            Self::Parameter(id) => write!(out, "const:{}", id.0),
        }
    }
}
