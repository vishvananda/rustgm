//! Calls retain their actual use category without inventing an AST fragment.
use crate::{expand::builtin::BuiltinKind, syntax::ast::MacroContext};

#[derive(Clone, Copy)]
pub(super) enum CallGrammar {
    Bang(MacroContext),
    Attribute,
    Derive,
}
impl CallGrammar {
    pub(super) const fn kind(self) -> BuiltinKind {
        match self {
            Self::Bang(_) => BuiltinKind::Bang,
            Self::Attribute => BuiltinKind::Attribute,
            Self::Derive => BuiltinKind::Derive,
        }
    }
    pub(super) fn bang(self) -> Result<MacroContext, &'static str> {
        match self {
            Self::Bang(context) => Ok(context),
            Self::Attribute | Self::Derive => {
                Err("non-bang invocation has no bang fragment grammar")
            }
        }
    }
}
