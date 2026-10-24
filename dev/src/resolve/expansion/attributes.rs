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
        let mut transparency = false;
        let Some(owner) = owner
            .and_then(|node| unit.owner_indices[node.get() as usize])
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
                Some("rustc_macro_transparency") => {
                    crate::resolve::index::unavailable_macros::transparency_on_owner(
                        &self.loader.package,
                        file,
                        owner.node,
                        attribute,
                        forest,
                        &mut transparency,
                    )?;
                }
                _ => {}
            }
        }
        Ok(result)
    }
}

struct StringAttribute {
    span: crate::syntax::token_tree::Span,
    parent: Option<u32>,
    value: String,
}
fn strings<'a>(
    package: &'a super::Package,
    name: &'static str,
) -> impl Iterator<Item = Result<StringAttribute, Error>> + 'a {
    let unit = package.unit(FileId(0));
    let root = package.modules[0].root;
    let range = unit.owner_indices[root.get() as usize].map(|index| {
        unit.configured.owners[index.get() as usize - 1]
            .attributes
            .clone()
    });
    range.into_iter().flatten().filter_map(move |index| {
        let effect = &unit.configured.effects[index];
        let attribute = effect.attribute.attribute;
        let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
        if effect.attribute.effect != Effect::Keep
            || attribute.name(forest, unit.source.text()) != Some(name)
        {
            return None;
        }
        let failure = || {
            Error::at(
                FileId(0),
                attribute.span.start,
                format!("{name} requires a string literal"),
            )
        };
        Some((|| {
            let Input::Equals(range) = attribute.input else {
                return Err(failure());
            };
            if range.end != range.start + 1 {
                return Err(failure());
            }
            let token = forest.token(range.start).ok_or_else(failure)?;
            if !matches!(
                token.kind,
                crate::lex::token::Kind::String | crate::lex::token::Kind::RawString
            ) {
                return Err(failure());
            }
            let Some(crate::literal::Value::String(bytes)) =
                crate::literal::interpret(unit.source.text(), token)
                    .map_err(|e| Error::at(FileId(0), e.offset, e.message))?
            else {
                return Err(failure());
            };
            let value = String::from_utf8(bytes.into_owned()).map_err(|_| failure())?;
            Ok(StringAttribute {
                span: attribute.span,
                parent: effect.attribute.parent,
                value,
            })
        })())
    })
}

pub(super) fn recursion_limit(package: &super::Package) -> Result<usize, Error> {
    let Some(attribute) = strings(package, "recursion_limit").next().transpose()? else {
        return Ok(128);
    };
    attribute.value.parse().map_err(|_| {
        Error::at(
            FileId(0),
            attribute.span.start,
            "recursion_limit requires a string containing a non-negative integer",
        )
    })
}

pub(in crate::resolve) fn crate_name(
    package: &super::Package,
    requested: Option<&str>,
) -> Result<String, Error> {
    let mut first = None;
    for attribute in strings(package, "crate_name") {
        let attribute = attribute?;
        if attribute.parent.is_some() {
            return Err(Error::at(
                FileId(0),
                attribute.span.start,
                "crate_name within cfg_attr is forbidden",
            ));
        }
        if first.is_none() {
            first = Some(attribute);
        }
    }
    let offset = first.as_ref().map_or(0, |attribute| attribute.span.start);
    if let (Some(requested), Some(attribute)) = (requested, first.as_ref())
        && requested != attribute.value
    {
        return Err(Error::at(
            FileId(0),
            offset,
            "crate_name attribute and driver option must match",
        ));
    }
    let name = requested
        .map(str::to_owned)
        .or_else(|| first.map(|attribute| attribute.value))
        .unwrap_or_else(|| package.crate_name.clone());
    if name.is_empty() || !name.chars().all(|ch| ch.is_alphanumeric() || ch == '_') {
        return Err(Error::at(
            FileId(0),
            offset,
            "crate name must contain only letters, digits or underscores",
        ));
    }
    Ok(name)
}
