//! Structural subject keys exclude lifetimes and children; selection still checks complete inputs.
use super::{ItemId, Kind, Primitive};

/// Coarse candidate keys exclude lifetimes and children; complete equations still prove equality.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Head {
    Primitive(Primitive),
    Parameter(crate::types::parameters::ParameterId),
    Projection(ItemId),
    Nominal(ItemId),
    Foreign(ItemId),
    Opaque(super::opaque::OpaqueId),
    Dynamic(super::dynamic::ShapeId),
    FunctionItem(ItemId),
    Closure(super::closure::ClosureId),
    Reference(bool),
    Pointer(bool),
    Tuple,
    Array,
    Slice,
    Function,
    Never,
    Pattern(super::Constraint),
}
impl Head {
    pub const fn canonical(kind: Kind) -> Self {
        match kind {
            Kind::Primitive(p) => Self::Primitive(p),
            Kind::Parameter(id) => Self::Parameter(id),
            Kind::Projection(application) => Self::Projection(application.item),
            Kind::Adt(item, _) => Self::Nominal(item),
            Kind::Foreign(item) => Self::Foreign(item),
            Kind::Opaque(id, _) => Self::Opaque(id),
            Kind::Dynamic { shape, .. } => Self::Dynamic(shape),
            Kind::FunctionItem { item, .. } => Self::FunctionItem(item),
            Kind::Closure { identity, .. } => Self::Closure(identity),
            Kind::Reference { mutable, .. } => Self::Reference(mutable),
            Kind::Pointer { mutable, .. } => Self::Pointer(mutable),
            Kind::Tuple(_) => Self::Tuple,
            Kind::Array { .. } | Kind::ArrayTerm { .. } => Self::Array,
            Kind::Slice(_) => Self::Slice,
            Kind::Function { .. } => Self::Function,
            Kind::Never => Self::Never,
            Kind::Pattern { constraint, .. } => Self::Pattern(constraint),
        }
    }
}
