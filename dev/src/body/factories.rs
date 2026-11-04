//! Only the corresponding checked factory validator can construct each permit.
use crate::types::instance::Instance;
#[derive(Clone, Copy)]
pub(crate) enum Permit {
    Primitive(super::primitive_methods::Permit),
    FunctionPointer(super::pointers::Permit),
}
impl Permit {
    pub(crate) const fn instance(self) -> Instance {
        match self {
            Self::Primitive(permit) => permit.instance(),
            Self::FunctionPointer(permit) => permit.instance(),
        }
    }
}
