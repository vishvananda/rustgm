//! Cold declaration requirements are vocabulary, never builtin semantic evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclarationTarget {
    Trait,
    Function,
    ForeignFunction,
    Struct,
    Enum,
    Variant,
    RequiredMethod,
    InherentMethod,
    AssociatedType,
    AssociatedConst,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GenericRequirement {
    Any,
    Exact(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeclarationProfile {
    pub target: DeclarationTarget,
    pub generics: GenericRequirement,
}
