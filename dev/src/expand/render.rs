//! PA3's diagnostic forest and root attribute facts, without syntax guessing.
use super::{
    config::Config,
    configured::{Attributes, Effect},
};
use crate::{
    lex::{render::RenderError, token::LexError},
    support::quoted,
    syntax::{
        attribute::{self, Input},
        token_tree::{Forest, TreeKind},
    },
};
use std::io::{self, Write};

pub struct Report {
    pub forest: Forest,
    pub attributes: Attributes,
}

/// Root attributes are at known diagnostic-stream positions. Nested trees stay
/// opaque until a syntax owner asks for their attributes, including macro input.
///
/// # Errors
/// Returns token-tree or root attribute/configuration failures.
pub fn analyze(source: &str, config: &Config) -> Result<Report, LexError> {
    let forest = Forest::parse(source)?;
    let mut attributes = Attributes::default();
    let mut index = 0;
    while index < forest.roots().end {
        if let Some((attribute, next)) = attribute::at(&forest, source, index)? {
            attributes.push(&forest, source, attribute, config)?;
            index = next;
        } else {
            index = forest.after(index);
        }
    }
    Ok(Report { forest, attributes })
}

fn trees(source: &str, forest: &Forest, output: &mut impl Write) -> io::Result<()> {
    let mut groups = Vec::new();
    for (index, tree) in forest.nodes().iter().enumerate() {
        while groups.last().is_some_and(|after| *after as usize == index) {
            groups.pop();
        }
        let depth = groups.len();
        let start = tree.span.start;
        let end = tree.span.end;
        match tree.kind {
            TreeKind::Group { delimiter, after } => {
                writeln!(
                    output,
                    "group {index} {depth} {} {start} {end} {after}",
                    delimiter.name()
                )?;
                groups.push(after);
            }
            TreeKind::Token(kind) => {
                write!(
                    output,
                    "token {index} {depth} {} {start} {end} ",
                    kind.name()
                )?;
                quoted::bytes(
                    output,
                    &source.as_bytes()[start as usize..end as usize],
                    false,
                )?;
                writeln!(output)?;
            }
            TreeKind::Doc { inner, body } => {
                write!(
                    output,
                    "doc {index} {depth} {} {start} {end} ",
                    if inner { "inner" } else { "outer" }
                )?;
                quoted::bytes(
                    output,
                    &source.as_bytes()[body.start as usize..body.end as usize],
                    false,
                )?;
                writeln!(output)?;
            }
        }
    }
    Ok(())
}

fn attributes(source: &str, report: &Report, output: &mut impl Write) -> io::Result<()> {
    for (index, entry) in report.attributes.entries.iter().enumerate() {
        let attribute = entry.attribute;
        write!(output, "attribute {index} ")?;
        if let Some(parent) = entry.parent {
            write!(output, "{parent}")?;
        } else {
            write!(output, "root")?;
        }
        write!(
            output,
            " {} {} {} {} ",
            attribute.style.name(),
            attribute.span.start,
            attribute.span.end,
            if attribute.unsafe_wrapper {
                "unsafe"
            } else {
                "safe"
            }
        )?;
        match entry.effect {
            Effect::Keep => write!(output, "keep")?,
            Effect::ExpandedDerive => write!(output, "expanded-derive")?,
            Effect::Cfg(value) => write!(output, "cfg={value}")?,
            Effect::CfgAttr(value) => write!(output, "cfg_attr={value}")?,
        }
        write!(output, " ")?;
        if matches!(attribute.input, Input::Doc(_)) {
            write!(output, "doc")?;
        } else {
            for position in attribute.path.start..attribute.path.end {
                if let Some(name) = report.forest.name(source, position) {
                    output.write_all(name.as_bytes())?;
                } else {
                    output.write_all(b"::")?;
                }
            }
        }
        match attribute.input {
            Input::Empty => writeln!(output, " word")?,
            Input::Delimited(group) => writeln!(output, " group {group}")?,
            Input::Equals(range) => writeln!(output, " value {} {}", range.start, range.end)?,
            Input::Doc(body) => {
                write!(output, " value ")?;
                quoted::bytes(
                    output,
                    &source.as_bytes()[body.start as usize..body.end as usize],
                    false,
                )?;
                writeln!(output)?;
            }
        }
    }
    Ok(())
}

/// Emit a complete view only after successful analysis; source errors are atomic.
///
/// # Errors
/// Returns syntax/configuration or output errors.
pub fn token_trees(
    source: &str,
    config: &Config,
    output: &mut impl Write,
) -> Result<(), RenderError> {
    let report = analyze(source, config).map_err(RenderError::Lex)?;
    trees(source, &report.forest, output).map_err(RenderError::Io)?;
    attributes(source, &report, output).map_err(RenderError::Io)?;
    writeln!(output, "eof {}", report.forest.source_end()).map_err(RenderError::Io)
}
