//! Constant-time membership and actual generated-child checks for formatting sites.
#[cfg(test)]
#[path = "../../../tests/invariants/formatting_sites.rs"]
mod tests;
use super::{Error, Report};
use crate::{
    resolve::unit::NodeKey,
    support::library_types::{FormatConstructor, LibraryValue},
    syntax::ast::{BlockFlavor, Kind, List, NodeId},
};
impl Report {
    pub(super) fn check_format_site(
        &self,
        key: NodeKey,
        root: NodeId,
        generated: crate::syntax::token_tree::Span,
    ) -> Result<(), Error> {
        let unit = self.package.unit(key.file);
        let error = |message| Error::at(key.file, 0, message);
        let source = self
            .package
            .source_core()
            .ok_or_else(|| error("missing formatting provider"))?;
        let kind = |node: NodeId| {
            source.macros.charge(1, 0).map_err(error)?;
            unit.configured
                .ast
                .nodes
                .get(node.get() as usize)
                .filter(|actual| {
                    unit.configured.active.get(node.get() as usize) == Some(&true)
                        && actual.span.start >= generated.start
                        && actual.span.end <= generated.end
                })
                .map(|node| node.kind)
                .ok_or_else(|| error("formatting lost an actual active child"))
        };
        let at = |list: List, index: usize| {
            source.macros.charge(1, 0).map_err(error)?;
            let values = unit
                .configured
                .ast
                .edges
                .get(list.start as usize..list.end as usize)
                .ok_or_else(|| error("formatting changed its actual site range"))?;
            values
                .get(index)
                .copied()
                .ok_or_else(|| error("formatting changed its actual site ordinal"))
        };
        let shape = shape(key, root, &kind, &at)?;
        let OutputShape {
            sites, statements, ..
        } = shape;
        match kind(key.node)? {
            Kind::LibraryValue {
                kind: LibraryValue::FormatOutput { .. },
                ..
            } if key.node == root => {}
            Kind::LibraryValue {
                kind: LibraryValue::FormatSuperLet { slot },
                ..
            } if slot < 2 => {
                if at(sites, slot as usize)? != key.node
                    || at(statements, slot as usize)? != key.node
                {
                    return Err(error("formatting super-let is not its recorded child"));
                }
            }
            Kind::LibraryValue {
                kind:
                    LibraryValue::FormatConstructor {
                        slot,
                        constructor,
                        name,
                    },
                value: None,
            } => {
                source
                    .macros
                    .charge(constructor.name().len().saturating_mul(2), 0)
                    .map_err(error)?;
                if at(sites, slot as usize)? != key.node
                    || name.0 as usize >= self.package.names.len()
                    || self.package.names.get(name) != constructor.name()
                    || unit
                        .spelling(unit.configured.ast.node(key.node).span)
                        .map_err(|failure| error(failure.message))?
                        != constructor.name()
                {
                    return Err(error("formatting changed its canonical constructor site"));
                }
                let call = shape.constructor_call(key, slot, constructor, &kind, &at)?;
                if !matches!(kind(call)?,Kind::Call {callee,..}if callee==key.node) {
                    return Err(error("formatting constructor is not its actual call child"));
                }
            }
            _ => {
                return Err(error(
                    "formatting special use is not its actual root or member",
                ));
            }
        }
        Ok(())
    }
}

// Derived coordinates only; no retained permission, cache, or alternate owner.
struct OutputShape {
    sites: List,
    statements: List,
    tail: NodeId,
    arguments: List,
    count: usize,
}
fn shape(
    key: NodeKey,
    root: NodeId,
    kind: &impl Fn(NodeId) -> Result<Kind, Error>,
    at: &impl Fn(List, usize) -> Result<NodeId, Error>,
) -> Result<OutputShape, Error> {
    let error = |message| Error::at(key.file, 0, message);
    let Kind::LibraryValue {
        kind: LibraryValue::FormatOutput { sites },
        value: Some(block),
    } = kind(root)?
    else {
        return Err(error("formatting changed its actual output root"));
    };
    let sites = List {
        start: sites.start,
        end: sites.end,
    };
    let Kind::Expansion {
        children: statements,
        tail: Some(tail),
    } = kind(block)?
    else {
        return Err(error("formatting changed its actual output child"));
    };
    if statements.end.checked_sub(statements.start) != Some(2) {
        return Err(error("formatting changed its generated bindings"));
    }
    let binding = |slot| {
        let marker = at(statements, slot)?;
        if marker != at(sites, slot)? {
            return Err(error("formatting binding differs from its recorded site"));
        }
        let Kind::LibraryValue {
            kind: LibraryValue::FormatSuperLet { slot: actual },
            value: Some(value),
        } = kind(marker)?
        else {
            return Err(error("formatting changed its super-let site"));
        };
        if actual as usize != slot {
            return Err(error("formatting changed its super-let ordinal"));
        }
        let Kind::Let {
            value: Some(value),
            otherwise: None,
            ty: None,
            ..
        } = kind(value)?
        else {
            return Err(error("formatting super-let changed its actual binding"));
        };
        Ok(value)
    };
    let tuple = binding(0)?;
    let array = binding(1)?;
    let Kind::Tuple(values) = kind(tuple)? else {
        return Err(error("formatting changed its evaluated input tuple"));
    };
    let Kind::Array(arguments) = kind(array)? else {
        return Err(error("formatting changed its actual argument array"));
    };
    let inputs = values
        .end
        .checked_sub(values.start)
        .filter(|count| *count > 0);
    let count = arguments
        .end
        .checked_sub(arguments.start)
        .filter(|count| *count > 0);
    let pair_limit = u32::try_from(FormatConstructor::PAIR_COUNT)
        .map_err(|_| error("formatting pair cardinality exceeds its site coordinates"))?;
    if inputs.zip(count).is_none_or(|(inputs, count)| {
        count < inputs
            || count > inputs.saturating_mul(pair_limit)
            || sites.end.checked_sub(sites.start)
                != inputs.checked_add(count).and_then(|n| n.checked_add(3))
    }) {
        return Err(error("formatting changed its actual argument cardinality"));
    }
    Ok(OutputShape {
        sites,
        statements,
        tail,
        arguments,
        count: count.ok_or_else(|| error("formatting lost its constructor count"))? as usize,
    })
}

impl OutputShape {
    fn constructor_call(
        &self,
        key: NodeKey,
        slot: u32,
        constructor: FormatConstructor,
        kind: &impl Fn(NodeId) -> Result<Kind, Error>,
        at: &impl Fn(List, usize) -> Result<NodeId, Error>,
    ) -> Result<NodeId, Error> {
        let error = |message| Error::at(key.file, 0, message);
        let call = match (slot, constructor) {
            (2, FormatConstructor::Arguments) => {
                let Kind::Block {
                    flavor: BlockFlavor::Unsafe,
                    statements,
                    tail: Some(call),
                } = kind(self.tail)?
                else {
                    return Err(error(
                        "formatting changed its narrow constructor unsafe block",
                    ));
                };
                if statements.start != statements.end {
                    return Err(error("formatting unsafe block contains extra statements"));
                }
                call
            }
            (slot, constructor)
                if constructor.owner() == crate::support::bindings::LangItem::FormatArgument
                    && slot >= 3
                    && (slot as usize) < 3 + self.count =>
            {
                at(self.arguments, slot as usize - 3)?
            }
            _ => {
                return Err(error(
                    "formatting constructor changed its actual protocol member",
                ));
            }
        };
        Ok(call)
    }
}
