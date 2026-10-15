//! Shared rigid subject keys for caller assumptions and implementation headers.
use crate::types::{ItemId, Kind, Primitive, infer::Constructor};

#[derive(Clone, Copy)]
pub(super) enum Subject {
    Rigid(Head),
    Integer,
    Float,
}
impl Subject {
    pub fn query(
        store: &crate::types::Store,
        context: &mut crate::types::infer::Context,
        term: crate::types::infer::TermId,
    ) -> Result<Option<Self>, crate::types::infer::Error> {
        use crate::types::infer::Domain;
        Ok(if let Some(head) = context.head(store, term)? {
            Some(Self::Rigid(Head::inferred(head)))
        } else {
            match context.unresolved_domain(term)? {
                Some(Domain::Integer) => Some(Self::Integer),
                Some(Domain::Float) => Some(Self::Float),
                Some(Domain::Any) | None => None,
            }
        })
    }
    pub fn heads(self) -> impl Iterator<Item = Head> {
        use Primitive::{F32, F64, I8, I16, I32, I64, I128, Isize, U8, U16, U32, U64, U128, Usize};
        const INTEGERS: &[Primitive] = &[
            I8, I16, I32, I64, I128, Isize, U8, U16, U32, U64, U128, Usize,
        ];
        let (head, primitives) = match self {
            Self::Rigid(head) => (Some(head), &[][..]),
            Self::Integer => (None, INTEGERS),
            Self::Float => (None, &[F32, F64][..]),
        };
        head.into_iter()
            .chain(primitives.iter().copied().map(Head::Primitive))
    }
}

/// Coarse candidate keys exclude lifetimes and children; complete equations still prove equality.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Head {
    Primitive(Primitive),
    Parameter(crate::types::parameters::ParameterId),
    Projection(ItemId),
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
            Kind::Projection(application) => Self::Projection(application.item),
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
            Constructor::Projection(value) => Self::Projection(value.item()),
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
