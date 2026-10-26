//! Attribute witnesses retain semantic identifier identity as well as tree shape.
use crate::expand::builtin::Builtin;
use std::rc::Rc;

#[derive(Clone)]
pub(super) enum Spelling {
    BuiltinAttribute,
    Attributes,
    AllowanceAttribute,
    UnsafeLintAttribute,
    BuiltinRole(Builtin),
    Feature(Rc<str>),
    Helper(Rc<str>),
}
impl Spelling {
    pub(super) fn read(name: &str, builtin: Option<Builtin>) -> Result<Self, &'static str> {
        match name {
            "rustc_builtin_macro" => Ok(Self::BuiltinAttribute),
            "attributes" => Ok(Self::Attributes),
            "allow_internal_unstable" => Ok(Self::AllowanceAttribute),
            "allow_internal_unsafe" => Ok(Self::UnsafeLintAttribute),
            _ => builtin
                .filter(|role| role.name() == name)
                .map(Self::BuiltinRole)
                .ok_or("macro attribute witness identifier changed"),
        }
    }
    pub(super) fn name(&self) -> &str {
        match self {
            Self::BuiltinAttribute => "rustc_builtin_macro",
            Self::Attributes => "attributes",
            Self::AllowanceAttribute => "allow_internal_unstable",
            Self::UnsafeLintAttribute => "allow_internal_unsafe",
            Self::BuiltinRole(role) => role.name(),
            Self::Feature(lexeme) | Self::Helper(lexeme) => lexeme,
        }
    }
    pub(super) fn matches(
        &self,
        source: &str,
        forest: &crate::syntax::token_tree::Forest,
        index: u32,
    ) -> bool {
        if let Self::Feature(lexeme) | Self::Helper(lexeme) = self {
            forest.tree(index).is_some_and(|tree| {
                source.get(tree.span.start as usize..tree.span.end as usize)
                    == Some(lexeme.as_ref())
            })
        } else {
            forest.name(source, index) == Some(self.name())
        }
    }
}
