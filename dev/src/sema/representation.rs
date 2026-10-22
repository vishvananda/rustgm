//! Interpret retained nominal attributes without reparsing item source.
use super::model::Declarations;
use crate::{
    lex::token::Punctuation,
    literal::{self, Value},
    resolve::{
        expansion::Report,
        package::Error,
        unit::{FileId, NodeKey},
    },
    syntax::{
        ast::Kind,
        attribute::Input,
        item::ItemKind,
        token_tree::{Delimiter, Forest, TreeKind, TreeRange},
    },
    types::{
        Primitive,
        aggregate::{Repr, Representation},
        copy::Properties,
    },
};

#[derive(Clone, Copy, Default)]
pub(super) struct Attributes {
    pub repr: Repr,
    pub derives: Properties,
    pub equality: crate::types::equality::Properties,
}
pub(super) fn analyze(
    report: &Report,
) -> Result<crate::support::hash::HashMap<NodeKey, Attributes>, Error> {
    let mut representations = crate::support::hash::HashMap::default();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file).map_err(|_| Error::at(FileId(0), 0, "too many source files"))?,
        );
        for (index, owner) in unit.configured.owners.iter().enumerate() {
            if !owner.enabled {
                continue;
            }
            let mut node = owner.node;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            let Kind::Item(item) = unit.configured.ast.node(node).kind else {
                continue;
            };
            if matches!(
                unit.configured.ast.items[item as usize].kind,
                ItemKind::Struct { .. } | ItemKind::Enum { .. } | ItemKind::Union { .. }
            ) {
                let key = NodeKey { file, node };
                representations.insert(key, parse(report, key, index)?);
            }
        }
    }
    Ok(representations)
}
fn parse(report: &Report, key: NodeKey, index: usize) -> Result<Attributes, Error> {
    let unit = report.package.unit(key.file);
    let owner = &unit.configured.owners[index];
    let mut parser = Parser {
        repr: Repr::default(),
        base: None,
        hints: 0,
    };
    let mut derives = Properties::default();
    let mut equality = crate::types::equality::Properties::default();
    for index in owner.attributes.clone() {
        let effect = &unit.configured.effects[index];
        let attribute = effect.attribute.attribute;
        let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
        let name = attribute.name(forest, unit.source.text());
        if !matches!(name, Some("repr" | "derive")) {
            continue;
        }
        let error = |message| Error::at(key.file, attribute.span.start, message);
        let Input::Delimited(group) = attribute.input else {
            return Err(error("nominal attribute requires parenthesized arguments"));
        };
        let range = parentheses(forest, group).map_err(error)?;
        if name == Some("derive") {
            derive(
                forest,
                unit.source.text(),
                range,
                &mut derives,
                &mut equality,
            )
            .map_err(error)?;
        } else {
            parser
                .parse(forest, unit.source.text(), range)
                .map_err(error)?;
        }
    }
    if parser.base == Some(Representation::Transparent) && parser.hints != 1 {
        return Err(Declarations::error(
            report,
            key,
            "transparent representation cannot combine with other hints",
        ));
    }
    if derives.copy && !derives.clone_ {
        return Err(Declarations::error(
            report,
            key,
            "Copy requires a Clone implementation",
        ));
    }
    if equality.eq && !equality.partial_eq {
        return Err(Declarations::error(
            report,
            key,
            "Eq requires a PartialEq implementation",
        ));
    }
    Ok(Attributes {
        repr: parser.repr,
        derives,
        equality,
    })
}
fn derive(
    forest: &Forest,
    source: &str,
    range: TreeRange,
    selected: &mut Properties,
    equality: &mut crate::types::equality::Properties,
) -> Result<(), &'static str> {
    let mut index = range.start;
    while index < range.end {
        let field = match forest.name(source, index) {
            Some("Copy") => &mut selected.copy,
            Some("Clone") => &mut selected.clone_,
            Some("PartialEq") => &mut equality.partial_eq,
            Some("Eq") => &mut equality.eq,
            _ => return Err("this derive requires the trait or procedural macro assignment"),
        };
        if std::mem::replace(field, true) {
            return Err("conflicting derived trait implementations");
        }
        index += 1;
        if index < range.end {
            if !forest.punctuation(index, Punctuation::Comma) {
                return Err("expected comma between built-in derives");
            }
            index += 1;
        }
    }
    Ok(())
}
struct Parser {
    repr: Repr,
    base: Option<Representation>,
    hints: usize,
}
impl Parser {
    fn parse(
        &mut self,
        forest: &Forest,
        source: &str,
        range: TreeRange,
    ) -> Result<(), &'static str> {
        let mut index = range.start;
        while index < range.end {
            let name = forest
                .name(source, index)
                .ok_or("repr requires a named hint")?;
            index += 1;
            self.hints += 1;
            match name {
                "Rust" | "C" | "transparent" => {
                    let base = match name {
                        "C" => Representation::C,
                        "transparent" => Representation::Transparent,
                        _ => Representation::Rust,
                    };
                    if self.base.is_some_and(|prior| prior != base) {
                        return Err("conflicting representation hints");
                    }
                    self.base = Some(base);
                    self.repr.representation = base;
                }
                "align" | "packed" => {
                    let value = if index < range.end && forest.children(index).is_some() {
                        let value = alignment(forest, source, index)?;
                        index = forest.after(index);
                        value
                    } else if name == "packed" {
                        1
                    } else {
                        return Err("align requires one integer argument");
                    };
                    if name == "align" {
                        self.repr.align = Some(self.repr.align.unwrap_or(1).max(value));
                    } else {
                        if self.repr.packed.is_some_and(|prior| prior != value) {
                            return Err("conflicting packed representation hints");
                        }
                        self.repr.packed = Some(value);
                    }
                }
                _ => {
                    let tag = Primitive::ALL
                        .into_iter()
                        .find(|p| p.integer().is_some() && p.name() == name)
                        .ok_or("unrecognized representation hint")?;
                    if self.repr.tag.replace(tag).is_some() {
                        return Err("conflicting integer representation hints");
                    }
                }
            }
            if index < range.end {
                if !forest.punctuation(index, Punctuation::Comma) {
                    return Err("expected comma between representation hints");
                }
                index += 1;
            }
        }
        if self.base == Some(Representation::Rust) && self.repr.tag.is_some() {
            return Err("Rust and integer representation hints conflict");
        }
        Ok(())
    }
}
fn parentheses(forest: &Forest, index: u32) -> Result<TreeRange, &'static str> {
    if !forest.tree(index).is_some_and(|tree| {
        matches!(
            tree.kind,
            TreeKind::Group {
                delimiter: Delimiter::Parenthesis,
                ..
            }
        )
    }) {
        return Err("representation hint requires parentheses");
    }
    forest
        .children(index)
        .ok_or("missing representation arguments")
}
fn alignment(forest: &Forest, source: &str, index: u32) -> Result<u32, &'static str> {
    let range = parentheses(forest, index)?;
    if range.end != range.start + 1 {
        return Err("alignment requires exactly one integer literal");
    }
    let token = forest
        .token(range.start)
        .ok_or("alignment requires an integer literal")?;
    let Some(Value::Integer(value)) = literal::interpret(source, token).map_err(|e| e.message)?
    else {
        return Err("alignment requires an integer literal");
    };
    if value.suffix.is_some() {
        return Err("alignment literal must be unsuffixed");
    }
    let value = u32::try_from(value.magnitude).map_err(|_| "alignment exceeds target limit")?;
    if !value.is_power_of_two() || value > 1 << 29 {
        return Err("alignment must be a power of two no greater than 2^29");
    }
    Ok(value)
}
