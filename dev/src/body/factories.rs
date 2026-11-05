//! Only the corresponding checked factory validator can construct each permit.
use crate::types::instance::Instance;
pub(crate) enum Permit {
    Primitive(super::primitive_methods::Permit),
    FunctionPointer(super::pointers::Permit),
    Future(Box<super::coroutines::Permit>),
    Lending(super::lending::Permit),
}
impl Permit {
    pub(crate) const fn instance(&self) -> Option<Instance> {
        match self {
            Self::Primitive(permit) => Some(permit.instance()),
            Self::FunctionPointer(permit) => Some(permit.instance()),
            Self::Future(permit) => permit.instance(),
            Self::Lending(permit) => Some(permit.instance()),
        }
    }
    pub(crate) fn future(&self) -> Option<&super::coroutines::Permit> {
        match self {
            Self::Future(permit) => Some(permit),
            _ => None,
        }
    }
}

const _: [(); 16] = [(); size_of::<Permit>()];
