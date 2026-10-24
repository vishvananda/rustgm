//! Calls retain their actual use category without inventing an AST fragment.
use crate::{expand::builtin::BuiltinKind, syntax::ast::MacroContext};

#[derive(Clone, Copy)]
pub(super) enum CallGrammar {
    Bang(MacroContext),
    Attribute,
}
impl CallGrammar {
    pub(super) const fn kind(self) -> BuiltinKind {
        match self {
            Self::Bang(_) => BuiltinKind::Bang,
            Self::Attribute => BuiltinKind::Attribute,
        }
    }
    pub(super) fn bang(self) -> Result<MacroContext, &'static str> {
        match self {
            Self::Bang(context) => Ok(context),
            Self::Attribute => Err("attribute invocation has no bang fragment grammar"),
        }
    }
}
