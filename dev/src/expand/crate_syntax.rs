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
    pub(crate) cfg_uses: Option<Box<super::configured::ConfiguredUses>>,
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
    let mut cfg_uses = None;
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
        let uses = configure_owner(
            &ast,
            range,
            source,
            config,
            inputs,
            &mut attributes,
            &mut effects,
        )?;
        super::configured::retain_cfg_uses(&mut cfg_uses, uses)?;
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
    promote_tails(&mut ast, &active, &pending.tails)?;
    Ok(ConfiguredCrate {
        ast,
        owners,
        active,
        effects,
        cfg_uses,
    })
}

/// As in rustc, a block whose written tail is configured out takes its last
/// enabled statement as its tail when that statement is an expression written
/// without a semicolon, and otherwise has none. The promoted statement becomes a
/// transparent group around its value, keeping its attribute owner. The removed
/// tail stays inactive after the statements, so every node keeps its parent and
/// each subtree stays the contiguous ID range ending at its root.
fn promote_tails<L>(
    ast: &mut Syntax<L>,
    active: &[bool],
    blocks: &[NodeId],
) -> Result<(), LexError> {
    for &block in blocks {
        let (Kind::Block {
            statements,
            tail: Some(tail),
            ..
        }
        | Kind::Expansion {
            children: statements,
            tail: Some(tail),
        }) = ast.nodes[block.get() as usize].kind
        else {
            continue;
        };
        if !active[block.get() as usize] || active[tail.get() as usize] {
            continue;
        }
        let promoted = promoted_statement(ast, active, statements);
        let span = ast.nodes[block.get() as usize].span;
        let failure =
            || crate::syntax::token_tree::error(span.start, "configured block tail overflow");
        let first = statements.start as usize;
        let length = statements.end as usize - first;
        ast.edges.try_reserve(length + 1).map_err(|_| failure())?;
        let start = u32::try_from(ast.edges.len()).map_err(|_| failure())?;
        for index in first..first + length {
            if promoted.is_none_or(|(at, ..)| at != index - first) {
                let edge = ast.edges[index];
                ast.edges.push(edge);
            }
        }
        ast.edges.push(tail);
        let end = u32::try_from(ast.edges.len()).map_err(|_| failure())?;
        if let Some((_, _, statement, value)) = promoted {
            ast.nodes[statement.get() as usize].kind = Kind::Group(value);
        }
        if let Kind::Block {
            statements, tail, ..
        }
        | Kind::Expansion {
            children: statements,
            tail,
        } = &mut ast.nodes[block.get() as usize].kind
        {
            *statements = crate::syntax::ast::List { start, end };
            *tail = promoted.map(|(_, element, ..)| element);
        }
    }
    Ok(())
}

/// The last enabled statement when it is an expression without a semicolon:
/// its list position, list element, statement node and value. A braced macro
/// statement is not promoted; its expansion supplies its own statements.
fn promoted_statement<L>(
    ast: &Syntax<L>,
    active: &[bool],
    statements: crate::syntax::ast::List,
) -> Option<(usize, NodeId, NodeId, NodeId)> {
    let list = ast.list(statements);
    let at = list.iter().rposition(|node| active[node.get() as usize])?;
    let element = list[at];
    let statement = match ast.node(element).kind {
        Kind::Attributed { value, .. } => value,
        _ => element,
    };
    let Kind::Statement {
        value,
        semicolon: false,
    } = ast.node(statement).kind
    else {
        return None;
    };
    let mut written = value;
    while let Kind::Attributed { value, .. } = ast.node(written).kind {
        written = value;
    }
    (!matches!(ast.node(written).kind, Kind::Macro { .. }))
        .then_some((at, element, statement, value))
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
) -> Result<Option<Box<super::configured::ConfiguredUses>>, LexError> {
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
    let mut uses = attributes.take_cfg_uses();
    if attributes.enabled() {
        super::link_conditions::configure(
            ast,
            &mut effects[first..],
            source,
            config,
            inputs,
            &mut uses,
        )?;
    }
    let first = u32::try_from(first)
        .map_err(|_| crate::syntax::token_tree::error(0, "cfg effect overflow"))?;
    if let Some(uses) = &mut uses {
        uses.relocate(first, effects.len() - first as usize, Ok)?;
    }
    Ok(uses)
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
                Effect::ExpandedDerive => writeln!(output, " expanded-derive")?,
                Effect::Cfg(value) => writeln!(output, " cfg={value}")?,
                Effect::CfgAttr(value) => writeln!(output, " cfg_attr={value}")?,
                Effect::ConditionalLink(value) => writeln!(output, " link_cfg={value}")?,
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
