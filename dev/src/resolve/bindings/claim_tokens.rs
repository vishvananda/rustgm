//! Attribute witnesses retain semantic identifier identity as well as tree shape.
use crate::expand::builtin::Builtin;

#[derive(Clone, Copy)]
pub(super) enum Spelling {
    BuiltinAttribute,
    Attributes,
    AllowanceAttribute,
    BuiltinRole(Builtin),
    EditionPanic,
}
impl Spelling {
    pub(super) fn read(name: &str, builtin: Option<Builtin>) -> Result<Self, &'static str> {
        match name {
            "rustc_builtin_macro" => Ok(Self::BuiltinAttribute),
            "attributes" => Ok(Self::Attributes),
            "allow_internal_unstable" => Ok(Self::AllowanceAttribute),
            "edition_panic" => Ok(Self::EditionPanic),
            _ => builtin
                .filter(|role| role.name() == name)
                .map(Self::BuiltinRole)
                .ok_or("macro attribute witness identifier changed"),
        }
    }
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::BuiltinAttribute => "rustc_builtin_macro",
            Self::Attributes => "attributes",
            Self::AllowanceAttribute => "allow_internal_unstable",
            Self::BuiltinRole(role) => role.name(),
            Self::EditionPanic => "edition_panic",
        }
    }
}
