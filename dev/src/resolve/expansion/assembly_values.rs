//! Template formation consumes completed literal roots and publishes checked operand ordinals.
use crate::support::hash::HashMap;
use crate::{
    lex::token::LexError,
    resolve::unit::Unit,
    syntax::{
        assembly::{self, Direction},
        ast::{Kind, List, Node, NodeId},
        token_tree::error,
    },
};
/// Build template pieces in the existing arena; all operands remain actual parsed nodes.
pub(crate) fn complete(unit: &mut Unit, root: NodeId) -> Result<(), LexError> {
    let Kind::InlineAssembly(args) = unit.configured.ast.node(root).kind else {
        return Err(error(0, "missing parsed assembly"));
    };
    let rows = unit.configured.ast.list(args).to_vec();
    let Operands {
        names,
        registers,
        options,
    } = argument_layout(unit, &rows)?;
    let mut used = vec![false; registers.len()];
    let mut implicit = 0;
    for row in rows {
        let node = unit.configured.ast.node(row);
        let Kind::AssemblyTemplate { value, .. } = node.kind else {
            continue;
        };
        let Kind::Literal(index) = unit.configured.ast.node(value).kind else {
            return Err(error(
                node.span.start,
                "assembly template must expand to an actual string literal",
            ));
        };
        let crate::literal::Value::String(bytes) = unit.literal(index)? else {
            return Err(error(node.span.start, "assembly template must be a string"));
        };
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| error(node.span.start, "invalid assembly template string"))?;
        let pieces = if options & assembly::RAW != 0 {
            Vec::new()
        } else {
            placeholders(
                text,
                &names,
                &registers,
                &mut used,
                &mut implicit,
                node.span.start,
            )?
        };
        let start = u32::try_from(unit.configured.ast.edges.len())
            .map_err(|_| error(0, "assembly piece range overflow"))?;
        for (begin, end, operand, modifier) in pieces {
            let id = NodeId::checked(
                u32::try_from(unit.configured.ast.nodes.len())
                    .map_err(|_| error(0, "assembly piece identity overflow"))?,
            )
            .ok_or_else(|| error(0, "assembly piece identity overflow"))?;
            unit.configured.ast.nodes.push(Node {
                span: node.span,
                kind: Kind::AssemblyPlaceholder {
                    start: begin,
                    end,
                    operand,
                    modifier,
                },
            });
            unit.configured.active.push(true);
            unit.owner_indices.push(None);
            unit.configured.ast.edges.push(id);
        }
        let end = u32::try_from(unit.configured.ast.edges.len())
            .map_err(|_| error(0, "assembly piece range overflow"))?;
        unit.configured.ast.nodes[row.get() as usize].kind = Kind::AssemblyTemplate {
            value,
            pieces: List { start, end },
        };
    }
    if used
        .iter()
        .zip(registers)
        .any(|(&used, explicit)| !used && !explicit)
    {
        return Err(error(0, "assembly operand is never used in a template"));
    }
    Ok(())
}
struct Operands {
    names: HashMap<String, usize>,
    registers: Vec<bool>,
    options: u16,
}
fn argument_layout(unit: &Unit, rows: &[NodeId]) -> Result<Operands, LexError> {
    let mut names = HashMap::default();
    let mut registers = Vec::new();
    let mut options = 0;
    let mut outputs = 0;
    let mut real_outputs = 0;
    let mut positional_closed = false;
    for &row in rows {
        let node = unit.configured.ast.node(row);
        match node.kind {
            Kind::AssemblyOptions(bits) => {
                if options & bits != 0 {
                    return Err(error(node.span.start, "duplicate assembly option"));
                }
                options |= bits;
            }
            Kind::AssemblyOperand {
                register,
                name,
                output,
                direction,
                ..
            } => {
                let explicit = register.is_some_and(|r| {
                    matches!(
                        unit.configured.ast.node(r).kind,
                        Kind::AssemblyRegister { explicit: true, .. }
                    )
                });
                if explicit && name.is_some() {
                    return Err(error(
                        node.span.start,
                        "explicit assembly registers cannot be named",
                    ));
                }
                if let Some(name) = name {
                    let span = unit.configured.ast.node(name).span;
                    let key = unit
                        .source
                        .text()
                        .get(span.start as usize..span.end as usize)
                        .ok_or_else(|| error(span.start, "invalid assembly operand name"))?
                        .to_owned();
                    if names.insert(key, registers.len()).is_some() {
                        return Err(error(span.start, "duplicate assembly operand name"));
                    }
                    positional_closed = true;
                } else if !explicit && positional_closed {
                    return Err(error(
                        node.span.start,
                        "positional assembly operand follows named or explicit operand",
                    ));
                }
                if explicit {
                    positional_closed = true;
                }
                if matches!(
                    direction,
                    Direction::Out | Direction::LateOut | Direction::InOut | Direction::InLateOut
                ) {
                    outputs += 1;
                    real_outputs += usize::from(output.is_some());
                }
                registers.push(explicit);
            }
            _ => {}
        }
    }
    if options & assembly::NOMEM != 0 && options & assembly::READONLY != 0 {
        return Err(error(
            0,
            "assembly nomem and readonly are mutually exclusive",
        ));
    }
    if options & assembly::PURE != 0
        && (options & (assembly::NOMEM | assembly::READONLY) == 0
            || options & assembly::NORETURN != 0
            || real_outputs == 0)
    {
        return Err(error(
            0,
            "pure assembly requires real outputs and nomem or readonly, without noreturn",
        ));
    }
    if options & assembly::NORETURN != 0 && outputs != 0 {
        return Err(error(0, "noreturn assembly cannot have outputs"));
    }
    Ok(Operands {
        names,
        registers,
        options,
    })
}

fn placeholders(
    text: &str,
    names: &HashMap<String, usize>,
    explicit: &[bool],
    used: &mut [bool],
    implicit: &mut usize,
    offset: u32,
) -> Result<Vec<(u32, u32, u32, u8)>, LexError> {
    let mut result = Vec::new();
    let bytes = text.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        if matches!(bytes[at], b'{' | b'}') && bytes.get(at + 1) == Some(&bytes[at]) {
            at += 2;
            continue;
        }
        if bytes[at] == b'}' {
            return Err(error(offset, "unmatched assembly template brace"));
        }
        if bytes[at] != b'{' {
            at += 1;
            continue;
        }
        let start = at;
        at += 1;
        let rest = &text[at..];
        let end = rest
            .find('}')
            .ok_or_else(|| error(offset, "unclosed assembly placeholder"))?
            + at;
        let field = &text[at..end];
        let (name, modifier) = field
            .split_once(':')
            .map_or((field, None), |(n, m)| (n, Some(m)));
        let index = if name.is_empty() {
            let n = *implicit;
            *implicit += 1;
            n
        } else if let Ok(n) = name.parse::<usize>() {
            n
        } else {
            *names
                .get(name)
                .ok_or_else(|| error(offset, "unknown assembly placeholder operand"))?
        };
        if explicit.get(index) != Some(&false) {
            return Err(error(
                offset,
                "assembly placeholder names an explicit or unknown operand",
            ));
        }
        let modifier = match modifier {
            None => 0,
            Some(s) if s.len() == 1 => s.as_bytes()[0],
            _ => {
                return Err(error(
                    offset,
                    "assembly placeholder requires at most one modifier",
                ));
            }
        };
        used[index] = true;
        result.push((
            u32::try_from(start).map_err(|_| error(offset, "assembly template too large"))?,
            u32::try_from(end + 1).map_err(|_| error(offset, "assembly template too large"))?,
            u32::try_from(index).map_err(|_| error(offset, "too many assembly operands"))?,
            modifier,
        ));
        at = end + 1;
    }
    Ok(result)
}

impl super::Report {
    /// Pay actual selected assembly work only after checking its real source invocation.
    pub(crate) fn prepare_assembly(
        &self,
        key: crate::resolve::unit::NodeKey,
        steps: usize,
        bytes: usize,
    ) -> Result<(), crate::resolve::package::Error> {
        use super::{Body, Builtin, permission_frames::FrameView};
        use crate::{
            expand::marks::ExpansionId,
            resolve::{bindings::panic_contracts, package::Error},
        };
        self.package
            .source_core()
            .ok_or_else(|| Error::at(key.file, 0, "assembly requires its actual source provider"))?
            .macros
            .charge(1, 0)
            .map_err(|message| Error::at(key.file, 0, message))?;
        let (context, span) = self.assembly_context(key)?;
        let fail = |message| Error::at(key.file, span.start, message);
        let invocation = context
            .checked_sub(1)
            .and_then(|n| self.invocations.get(n as usize))
            .ok_or_else(|| fail("assembly invocation identity is invalid"))?;
        if invocation.context.0 != context || invocation.node != key || invocation.generated != span
        {
            return Err(fail("assembly belongs to another actual invocation"));
        }
        let view = FrameView {
            package: &self.package,
            definitions: &self.definitions,
            invocations: &self.invocations,
            facts: &self.resolution,
        };
        let frame = view.frame(ExpansionId(context))?;
        let definition = view.definition(frame.definition)?;
        if frame.definition != invocation.definition
            || !matches!(definition.body, Body::Builtin(Builtin::Asm))
        {
            return Err(fail("assembly invocation changed its actual source role"));
        }
        let source = self
            .package
            .source_core()
            .ok_or_else(|| fail("assembly requires its actual source provider"))?;
        let published = panic_contracts::published(
            &self.package,
            definition
                .node
                .ok_or_else(|| fail("assembly declaration has no actual source origin"))?,
            definition.header.as_ref(),
            1,
        )?;
        if published.builtin != Some(Builtin::Asm) {
            return Err(fail("assembly declaration has another published role"));
        }
        source.macros.charge(steps, bytes).map_err(fail)
    }
    fn assembly_context(
        &self,
        key: crate::resolve::unit::NodeKey,
    ) -> Result<(u32, crate::syntax::token_tree::Span), crate::resolve::package::Error> {
        use crate::resolve::package::Error;
        let unit = self
            .package
            .units
            .get(key.file.0 as usize)
            .ok_or_else(|| Error::at(key.file, 0, "missing assembly source unit"))?;
        let node = unit
            .configured
            .ast
            .nodes
            .get(key.node.get() as usize)
            .ok_or_else(|| Error::at(key.file, 0, "missing assembly source node"))?;
        let fail = |message| Error::at(key.file, node.span.start, message);
        let Kind::InlineAssembly(args) = node.kind else {
            return Err(fail("assembly changed its actual syntax kind"));
        };
        if unit.configured.active.get(key.node.get() as usize) != Some(&true) {
            return Err(fail("assembly source node is not active"));
        }
        let origin = unit
            .configured
            .ast
            .list(args)
            .last()
            .and_then(|n| unit.configured.ast.nodes.get(n.get() as usize))
            .ok_or_else(|| fail("assembly has no actual invocation origin"))?;
        let Kind::AssemblyOrigin(context) = origin.kind else {
            return Err(fail("assembly has no actual invocation origin"));
        };
        Ok((context, node.span))
    }
}

#[cfg(test)]
impl super::Report {
    pub(crate) fn assembly_work(&self) -> Option<(usize, usize, usize)> {
        self.package
            .source_core()
            .map(|source| source.macros.usage())
    }
}
