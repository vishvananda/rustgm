//! Shared rigid subject keys for caller assumptions and implementation headers.
use crate::types::{ItemId, Kind, Primitive, infer::Constructor};

/// Coarse candidate keys exclude lifetimes and children; complete equations still prove equality.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Head {
    Primitive(Primitive),
    Parameter(crate::types::parameters::ParameterId),
    Nominal(ItemId),
    FunctionItem(ItemId),
    Reference(bool),
    Pointer(bool),
    Tuple,
    Array,
    Slice,
    Function,
    Never,
}
impl Head {
    pub const fn canonical(kind: Kind) -> Self {
        match kind {
            Kind::Primitive(p) => Self::Primitive(p),
            Kind::Parameter(id) => Self::Parameter(id),
            Kind::Adt(item, _) => Self::Nominal(item),
            Kind::FunctionItem { item, .. } => Self::FunctionItem(item),
            Kind::Reference { mutable, .. } => Self::Reference(mutable),
            Kind::Pointer { mutable, .. } => Self::Pointer(mutable),
            Kind::Tuple(_) => Self::Tuple,
            Kind::Array { .. } | Kind::ArrayTerm { .. } => Self::Array,
            Kind::Slice(_) => Self::Slice,
            Kind::Function { .. } => Self::Function,
            Kind::Never => Self::Never,
        }
    }
    pub const fn inferred(kind: Constructor) -> Self {
        match kind {
            Constructor::Primitive(p) => Self::Primitive(p),
            Constructor::Parameter(id) => Self::Parameter(id),
            Constructor::Adt(item, _) | Constructor::AdtArgs(item, _) => Self::Nominal(item),
            Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _) => {
                Self::FunctionItem(item)
            }
            Constructor::Reference { mutable, .. } => Self::Reference(mutable),
            Constructor::Pointer { mutable } => Self::Pointer(mutable),
            Constructor::Tuple => Self::Tuple,
            Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_) => {
                Self::Array
            }
            Constructor::Slice => Self::Slice,
            Constructor::Function { .. } => Self::Function,
            Constructor::Never => Self::Never,
        }
    }
}
