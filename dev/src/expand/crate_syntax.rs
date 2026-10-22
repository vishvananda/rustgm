//! Attribute owners are visited top-down; disabled subtrees leave the worklist.
use super::{
    config::Config,
    configured::{Attributes, ConfiguredAttribute, Effect},
};
use crate::{
    lex::{render::RenderError, token::LexError},
    syntax::{
        ast::{Kind, NodeId, Syntax},
        expression,
    },
};
use std::io::{self, Write};

#[derive(Clone)]
pub struct Owner {
    pub node: NodeId,
    pub enabled: bool,
    pub attributes: std::ops::Range<usize>,
}
#[derive(Clone)]
pub struct AttributeEffect {
    pub capture: u32,
    pub attribute: ConfiguredAttribute,
}
pub struct ConfiguredSyntax<L> {
    pub ast: Syntax<L>,
    pub owners: Vec<Owner>,
    pub active: Vec<bool>,
    pub effects: Vec<AttributeEffect>,
}

pub type ConfiguredCrate<'source> = ConfiguredSyntax<crate::literal::Value<'source>>;

/// Parse once, then evaluate only attributes of reachable enabled owners.
///
/// # Errors
/// Rejects malformed syntax or demanded attribute/configuration predicates.
pub fn analyze<'s>(source: &'s str, config: &Config) -> Result<ConfiguredCrate<'s>, LexError> {
    configure(source, expression::parse_crate(source)?, config)
}

/// Configure a newly parsed source or expansion without parsing its spelling again.
///
/// # Errors
/// Rejects demanded attribute predicates and literal values in enabled syntax.
pub fn configure<'s>(
    source: &'s str,
    mut ast: crate::syntax::ast::Ast<'s>,
    config: &Config,
) -> Result<ConfiguredCrate<'s>, LexError> {
    let mut active = vec![false; ast.nodes.len()];
    let mut work = Vec::with_capacity(64);
    work.push(ast.root);
    let mut owners = Vec::new();
    let mut attributes = Attributes::default();
    let mut effects = Vec::new();
    while let Some(node) = work.pop() {
        let index = node.0 as usize;
        if active[index] {
            continue;
        }
        if let Kind::Attributed {
            attributes: range, ..
        } = ast.nodes[index].kind
        {
            let first = effects.len();
            configure_owner(&ast, range, source, config, &mut attributes, &mut effects)?;
            let enabled = attributes.enabled();
            attributes.clear();
            owners.push(Owner {
                node,
                enabled,
                attributes: first..effects.len(),
            });
            if !enabled {
                continue;
            }
        }
        active[index] = true;
        match ast.nodes[index].kind {
            Kind::DeferredLiteral(token) => {
                let value = crate::literal::interpret(source, token)?.ok_or(
                    crate::syntax::token_tree::error(token.start, "expected literal token"),
                )?;
                let literal = u32::try_from(ast.literals.len()).map_err(|_| {
                    crate::syntax::token_tree::error(token.start, "too many literals")
                })?;
                ast.literals.push(value);
                ast.nodes[index].kind = Kind::Literal(literal);
            }
            // The most frequent leaves have no children to visit.
            Kind::Name(_) | Kind::Literal(_) | Kind::CrateRoot(_) => {}
            _ => {
                let mark = work.len();
                ast.push_children(node, &mut work);
                match work.len() - mark {
                    0 | 1 => {}
                    2 => work.swap(mark, mark + 1),
                    _ => work[mark..].reverse(),
                }
            }
        }
    }
    Ok(ConfiguredCrate {
        ast,
        owners,
        active,
        effects,
    })
}

/// Evaluate one owner's attributes, appending their effects with parent
/// indices relative to the whole effect list.
fn configure_owner<L>(
    ast: &Syntax<L>,
    range: crate::syntax::item::AttributeRange,
    source: &str,
    config: &Config,
    attributes: &mut Attributes,
    effects: &mut Vec<AttributeEffect>,
) -> Result<(), LexError> {
    let first = effects.len();
    for index in range.start..range.end {
        let capture = &ast.attributes[index as usize];
        let before = attributes.entries.len();
        attributes.push_owned(&capture.forest, source, capture.attribute, config)?;
        for entry in &attributes.entries[before..] {
            let mut entry = *entry;
            if let Some(parent) = entry.parent {
                entry.parent = Some(u32::try_from(first + parent as usize).map_err(|_| {
                    crate::syntax::token_tree::error(
                        capture.attribute.span.start,
                        "too many configured attributes",
                    )
                })?);
            }
            effects.push(AttributeEffect {
                capture: index,
                attribute: entry,
            });
        }
    }
    Ok(())
}

/// # Errors
/// Produces no source-error output until parsing and configuration both succeed.
pub fn render(source: &str, config: &Config, output: &mut impl Write) -> Result<(), RenderError> {
    let report = analyze(source, config).map_err(RenderError::Lex)?;
    write_configuration(&report, output).map_err(RenderError::Io)?;
    expression::render::write_ast(source, &report.ast, output).map_err(RenderError::Io)
}

/// Render configuration facts without changing their literal storage.
/// # Errors
/// Reports output failures.
pub fn write_configuration<L>(
    report: &ConfiguredSyntax<L>,
    output: &mut impl Write,
) -> io::Result<()> {
    for owner in &report.owners {
        writeln!(output, "configured {} {}", owner.node.0, owner.enabled)?;
        for index in owner.attributes.clone() {
            let effect = &report.effects[index];
            let entry = effect.attribute;
            write!(
                output,
                "attribute-effect {} {index} {} ",
                owner.node.0, effect.capture
            )?;
            if let Some(parent) = entry.parent {
                write!(output, "{parent}")?;
            } else {
                write!(output, "root")?;
            }
            match entry.effect {
                Effect::Keep => writeln!(output, " keep")?,
                Effect::Cfg(value) => writeln!(output, " cfg={value}")?,
                Effect::CfgAttr(value) => writeln!(output, " cfg_attr={value}")?,
            }
        }
    }
    write!(output, "active [")?;
    let mut first = true;
    for (index, active) in report.active.iter().enumerate() {
        if *active {
            if !first {
                write!(output, ",")?;
            }
            write!(output, "{index}")?;
            first = false;
        }
    }
    writeln!(output, "]")
}
