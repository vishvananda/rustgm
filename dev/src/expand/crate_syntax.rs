//! Attribute owners are evaluated top-down from parse records; only disabled subtrees are walked.
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
    configure(
        source,
        expression::parse_crate_in_edition(source, None, config.edition())?,
        config,
    )
}

/// Configure a newly parsed source or expansion without parsing its spelling again.
///
/// Parsed syntax has no unreachable nodes and every child precedes its parent,
/// so a subtree is the contiguous ID range ending at its root. Owners are
/// evaluated in preorder, merged with deferred literals by that range; only the
/// subtrees of disabled owners are walked. Arenas built without the parser
/// record no owners or deferred literals.
///
/// # Errors
/// Rejects demanded attribute predicates and literal values in enabled syntax.
pub fn configure<'s>(
    source: &'s str,
    ast: crate::syntax::ast::Ast<'s>,
    config: &Config,
) -> Result<ConfiguredCrate<'s>, LexError> {
    configure_with_inputs(source, ast, config, &config.edition())
}

/// Configure generated syntax using the same borrowed provenance as its parser.
/// # Errors
/// Rejects demanded attribute predicates, missing provenance and invalid literals.
pub fn configure_with_inputs<'s>(
    source: &'s str,
    mut ast: crate::syntax::ast::Ast<'s>,
    config: &Config,
    inputs: &dyn crate::syntax::interpolation::Inputs,
) -> Result<ConfiguredCrate<'s>, LexError> {
    let pending = std::mem::take(&mut ast.pending);
    let mut active = vec![true; ast.nodes.len()];
    let mut scratch = Vec::new();
    let mut order = Vec::with_capacity(pending.owners.len());
    for &owner in &pending.owners {
        order.push((subtree_start(&ast, owner, &mut scratch), owner));
    }
    // An ancestor's range starts no later than a descendant's and ends after it.
    order.sort_unstable_by_key(|&(start, owner)| (start, std::cmp::Reverse(owner.get())));
    // Usually every deferred literal is enabled: size its value arena once.
    ast.literals.reserve(pending.literals.len());
    let mut literals = pending.literals.iter().copied().peekable();
    let mut owners = Vec::with_capacity(order.len());
    let mut attributes = Attributes::default();
    let mut effects = Vec::new();
    for (start, node) in order {
        // A literal before the owner's range precedes the owner in preorder.
        while let Some(literal) = literals.next_if(|literal| literal.get() < start) {
            interpret(source, &mut ast, &active, literal)?;
        }
        if !active[node.get() as usize] {
            continue;
        }
        let Kind::Attributed {
            attributes: range, ..
        } = ast.nodes[node.get() as usize].kind
        else {
            return Err(crate::syntax::token_tree::error(
                ast.nodes[node.get() as usize].span.start,
                "attribute owner is not attributed syntax",
            ));
        };
        let first = effects.len();
        configure_owner(
            &ast,
            range,
            source,
            config,
            inputs,
            &mut attributes,
            &mut effects,
        )?;
        let enabled = attributes.enabled();
        attributes.clear();
        owners.push(Owner {
            node,
            enabled,
            attributes: first..effects.len(),
        });
        if !enabled {
            scratch.clear();
            scratch.push(node);
            while let Some(disabled) = scratch.pop() {
                active[disabled.get() as usize] = false;
                ast.push_children(disabled, &mut scratch);
            }
        }
    }
    for literal in literals {
        interpret(source, &mut ast, &active, literal)?;
    }
    Ok(ConfiguredCrate {
        ast,
        owners,
        active,
        effects,
    })
}

/// The first node created in `node`'s subtree: its smallest child's, recursively.
fn subtree_start<L>(ast: &Syntax<L>, mut node: NodeId, scratch: &mut Vec<NodeId>) -> u32 {
    loop {
        scratch.clear();
        ast.push_children(node, scratch);
        match scratch.iter().copied().min_by_key(|child| child.get()) {
            Some(child) => node = child,
            None => return node.get(),
        }
    }
}

/// Decode an enabled deferred literal into the next literal identity.
fn interpret<'s>(
    source: &'s str,
    ast: &mut crate::syntax::ast::Ast<'s>,
    active: &[bool],
    node: NodeId,
) -> Result<(), LexError> {
    let index = node.get() as usize;
    if !active[index] {
        return Ok(());
    }
    let Kind::DeferredLiteral(token) = ast.nodes[index].kind else {
        return Err(crate::syntax::token_tree::error(
            ast.nodes[index].span.start,
            "deferred literal is not literal syntax",
        ));
    };
    let value = crate::literal::interpret(source, token)?.ok_or(
        crate::syntax::token_tree::error(token.start, "expected literal token"),
    )?;
    let literal = u32::try_from(ast.literals.len())
        .map_err(|_| crate::syntax::token_tree::error(token.start, "too many literals"))?;
    ast.literals.push(value);
    ast.nodes[index].kind = Kind::Literal(literal);
    Ok(())
}

/// Evaluate one owner's attributes, appending their effects with parent
/// indices relative to the whole effect list.
fn configure_owner<L>(
    ast: &Syntax<L>,
    range: crate::syntax::item::AttributeRange,
    source: &str,
    config: &Config,
    inputs: &dyn crate::syntax::interpolation::Inputs,
    attributes: &mut Attributes,
    effects: &mut Vec<AttributeEffect>,
) -> Result<(), LexError> {
    let first = effects.len();
    for index in range.start..range.end {
        let capture = &ast.attributes[index as usize];
        let before = attributes.entries.len();
        attributes.push_owned_with_inputs(
            &capture.forest,
            source,
            capture.attribute,
            config,
            inputs,
        )?;
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
        writeln!(output, "configured {} {}", owner.node.get(), owner.enabled)?;
        for index in owner.attributes.clone() {
            let effect = &report.effects[index];
            let entry = effect.attribute;
            write!(
                output,
                "attribute-effect {} {index} {} ",
                owner.node.get(),
                effect.capture
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
