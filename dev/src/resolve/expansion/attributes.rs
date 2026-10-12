//! Expansion-owned built-in attributes consume configured captures.
use super::{Engine, Error, FileId, NodeId, Sources};
use crate::{
    expand::configured::Effect,
    syntax::{attribute::Input, token_tree::Delimiter},
};
#[derive(Default)]
pub(super) struct Flags {
    pub exported: bool,
    pub local_inner_macros: bool,
    pub macro_use: bool,
}
impl<S: Sources> Engine<'_, S> {
    pub(super) fn flags(&self, file: FileId, owner: Option<NodeId>) -> Result<Flags, Error> {
        let unit = self.loader.package.unit(file);
        let mut result = Flags::default();
        let Some(owner) = owner
            .and_then(|node| unit.owner_indices[node.0 as usize])
            .map(|index| &unit.configured.owners[index.get() as usize - 1])
        else {
            return Ok(result);
        };
        for index in owner.attributes.clone() {
            let effect = &unit.configured.effects[index];
            if effect.attribute.effect != Effect::Keep {
                continue;
            }
            let attribute = effect.attribute.attribute;
            let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
            match attribute.name(forest, unit.source.text()) {
                Some("macro_use") => {
                    if attribute.input != Input::Empty {
                        return Err(Error::at(
                            file,
                            attribute.span.start,
                            "macro_use arguments require an external crate",
                        ));
                    }
                    result.macro_use = true;
                }
                Some("macro_export") => {
                    result.exported = true;
                    match attribute.input {
                        Input::Empty => {}
                        Input::Delimited(group)
                            if forest.tree(group).is_some_and(|tree| {
                                matches!(
                                    tree.kind,
                                    crate::syntax::token_tree::TreeKind::Group {
                                        delimiter: Delimiter::Parenthesis,
                                        ..
                                    }
                                )
                            }) =>
                        {
                            let range = forest.children(group).ok_or_else(|| {
                                Error::at(
                                    file,
                                    attribute.span.start,
                                    "missing macro_export arguments",
                                )
                            })?;
                            if range.end == range.start {
                                continue;
                            }
                            if forest.name(unit.source.text(), range.start)
                                == Some("local_inner_macros")
                                && (range.end == range.start + 1
                                    || (range.end == range.start + 2
                                        && forest.punctuation(
                                            range.start + 1,
                                            crate::lex::token::Punctuation::Comma,
                                        )))
                            {
                                result.local_inner_macros = true;
                            } else {
                                return Err(Error::at(
                                    file,
                                    attribute.span.start,
                                    "unsupported macro_export argument",
                                ));
                            }
                        }
                        _ => {
                            return Err(Error::at(
                                file,
                                attribute.span.start,
                                "invalid macro_export input",
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }
}
