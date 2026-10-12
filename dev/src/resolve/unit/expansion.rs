//! Install only newly configured expansion facts into a file's stable syntax arenas.
use super::Unit;
use crate::{
    expand::{crate_syntax::ConfiguredCrate, stream::Origin},
    lex::token::LexError,
    syntax::{
        ast::{Kind, NodeId},
        attribute::Input,
        token_tree::{Span, error},
        transfer,
    },
};
use std::num::NonZeroU32;

fn add(base: usize, count: usize) -> Result<u32, LexError> {
    base.checked_add(count)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| error(0, "expansion metadata exceeds 32-bit identities"))?;
    u32::try_from(base).map_err(|_| error(0, "invalid expansion metadata base"))
}
fn shift(span: Span, base: u32, length: usize) -> Result<Span, LexError> {
    if span.start > span.end || span.end as usize > length {
        return Err(error(
            span.start,
            "expansion span is outside its source image",
        ));
    }
    Ok(Span {
        start: span
            .start
            .checked_add(base)
            .ok_or_else(|| error(span.start, "expansion source offset overflow"))?,
        end: span
            .end
            .checked_add(base)
            .ok_or_else(|| error(span.start, "expansion source offset overflow"))?,
    })
}

impl Unit {
    /// Replace a pending macro with already parsed and configured syntax. No
    /// prior node IDs, decoded literals or source offsets change. Attribute
    /// ownership follows the replacement node, including disabled expansions.
    ///
    /// # Errors
    /// Rejects invalid targets, ranges, metadata or compact arena overflow.
    /// Failed installation leaves the unit unchanged.
    pub fn install_expansion(
        &mut self,
        target: NodeId,
        source: &str,
        mut generated: ConfiguredCrate<'_>,
        origins: &[Origin],
    ) -> Result<Span, LexError> {
        let target_node = self
            .configured
            .ast
            .nodes
            .get(target.0 as usize)
            .ok_or_else(|| error(0, "missing expansion target"))?;
        if !matches!(target_node.kind, Kind::Macro { .. })
            || !self.configured.active[target.0 as usize]
        {
            return Err(error(
                target_node.span.start,
                "expansion target is not an active macro",
            ));
        }
        let base = self.source.append_offset(source).map_err(|e| error(0, e))?;
        let indices = self.expansion_metadata(target, &mut generated, base, source.len())?;
        let mut previous_end = 0;
        for origin in origins {
            shift(origin.generated, base, source.len())?;
            if origin.generated.start < previous_end {
                return Err(error(
                    origin.generated.start,
                    "overlapping expansion origins",
                ));
            }
            previous_end = origin.generated.end;
        }
        let new_root = append_syntax(
            source,
            base,
            generated.ast,
            &mut self.configured.ast,
            &mut self.literals,
        )?;
        // All ranges were validated above; this is the transaction's commit point.
        self.source.append(source).map_err(|e| error(0, e))?;
        let mut obsolete = vec![target];
        while let Some(node) = obsolete.pop() {
            self.original.save(
                node,
                self.configured.ast.node(node),
                self.configured.active[node.0 as usize],
                self.owner_indices[node.0 as usize],
            );
            self.configured.active[node.0 as usize] = false;
            self.configured.ast.push_children(node, &mut obsolete);
        }
        self.configured.active.extend(generated.active);
        self.configured.active[target.0 as usize] = self.configured.active[new_root.0 as usize];
        self.configured.active[new_root.0 as usize] = false;
        self.configured.ast.nodes[target.0 as usize] =
            self.configured.ast.nodes[new_root.0 as usize];
        self.owner_indices
            .resize(self.configured.ast.nodes.len(), None);
        for (owner, index) in generated.owners.iter().zip(indices) {
            self.owner_indices[owner.node.0 as usize] = Some(index);
        }
        self.configured.owners.extend(generated.owners);
        self.configured.effects.extend(generated.effects);
        self.origins
            .extend(origins.iter().copied().map(|mut origin| {
                origin.generated.start += base;
                origin.generated.end += base;
                origin
            }));
        Ok(Span {
            start: base,
            end: base
                + u32::try_from(source.len())
                    .map_err(|_| error(base, "invalid expansion source length"))?,
        })
    }
    fn expansion_metadata(
        &self,
        target: NodeId,
        generated: &mut ConfiguredCrate<'_>,
        base: u32,
        length: usize,
    ) -> Result<Vec<NonZeroU32>, LexError> {
        let node_base = add(self.configured.ast.nodes.len(), generated.ast.nodes.len())?;
        let attribute_base = add(
            self.configured.ast.attributes.len(),
            generated.ast.attributes.len(),
        )?;
        let effect_base = add(self.configured.effects.len(), generated.effects.len())?;
        let owner_base = add(self.configured.owners.len(), generated.owners.len())?;
        if generated.active.len() != generated.ast.nodes.len()
            || generated.ast.root.0 as usize >= generated.ast.nodes.len()
        {
            return Err(error(0, "invalid configured expansion arena"));
        }
        let root = generated.ast.root;
        let mapped_node = |node: NodeId| {
            if node == root {
                target
            } else {
                NodeId(node.0 + node_base)
            }
        };
        for owner in &mut generated.owners {
            if owner.node.0 as usize >= generated.ast.nodes.len()
                || owner.attributes.start > owner.attributes.end
                || owner.attributes.end > generated.effects.len()
            {
                return Err(error(0, "invalid generated attribute owner"));
            }
            owner.node = mapped_node(owner.node);
            owner.attributes.start += effect_base as usize;
            owner.attributes.end += effect_base as usize;
        }
        let effect_count = generated.effects.len();
        for effect in &mut generated.effects {
            if effect.capture as usize >= generated.ast.attributes.len()
                || effect
                    .attribute
                    .parent
                    .is_some_and(|p| p as usize >= effect_count)
            {
                return Err(error(0, "invalid generated attribute effect"));
            }
            effect.capture += attribute_base;
            if let Some(parent) = &mut effect.attribute.parent {
                *parent += effect_base;
            }
            effect.attribute.attribute.span = shift(effect.attribute.attribute.span, base, length)?;
            if let Input::Doc(body) = &mut effect.attribute.attribute.input {
                *body = shift(*body, base, length)?;
            }
        }
        // Reserve nonzero owner indices before mutating either destination arena.
        (0..generated.owners.len())
            .map(|n| {
                u32::try_from(n)
                    .ok()
                    .and_then(|n| n.checked_add(owner_base))
                    .and_then(|n| n.checked_add(1))
                    .and_then(NonZeroU32::new)
                    .ok_or_else(|| error(0, "too many configured owners"))
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

fn append_syntax(
    source: &str,
    base: u32,
    ast: crate::syntax::ast::Ast<'_>,
    destination: &mut crate::syntax::ast::Syntax<crate::literal::storage::StoredValue>,
    literals: &mut crate::literal::storage::Storage,
) -> Result<NodeId, LexError> {
    let mark = literals.decoded_bytes();
    let stored = ast.map_literals(|value| {
        literals
            .store(source, value)
            .and_then(|value| value.offset_source(base))
            .map_err(|e| error(0, e))
    });
    let result = stored.and_then(|ast| {
        transfer::append(
            &ast,
            destination,
            |span| shift(span, base, source.len()),
            |value| Ok(*value),
        )
    });
    if result.is_err() {
        literals.truncate(mark);
    }
    result
}

impl Unit {
    /// Demand builtin arguments in this unit's existing source image. Their
    /// temporary root remains detached from the crate while its macros expand.
    ///
    /// # Errors
    /// Reports argument grammar/configuration errors and arena overflow.
    pub fn retain_arguments<'a>(
        &mut self,
        file: super::FileId,
        tokens: &[crate::lex::token::Token],
        registry: &crate::expand::registry::Registry,
        other: impl Fn(super::FileId) -> Option<&'a str>,
        config: &crate::expand::config::Config,
    ) -> Result<NodeId, LexError> {
        let source = self.source.text();
        let inputs = registry.view(|id| if id == file { Some(source) } else { other(id) });
        let ast = crate::syntax::expression::fragment::parse_arguments(source, tokens, &inputs)?;
        let mut generated = crate::expand::crate_syntax::configure(source, ast, config)?;
        let root = NodeId(
            add(self.configured.ast.nodes.len(), generated.ast.nodes.len())? + generated.ast.root.0,
        );
        let indices = self.expansion_metadata(root, &mut generated, 0, source.len())?;
        let root = append_syntax(
            source,
            0,
            generated.ast,
            &mut self.configured.ast,
            &mut self.literals,
        )?;
        self.configured.active.extend(generated.active);
        self.owner_indices
            .resize(self.configured.ast.nodes.len(), None);
        for (owner, index) in generated.owners.iter().zip(indices) {
            self.owner_indices[owner.node.0 as usize] = Some(index);
        }
        self.configured.owners.extend(generated.owners);
        self.configured.effects.extend(generated.effects);
        Ok(root)
    }

    pub(crate) fn retire_arguments(&mut self, root: NodeId) {
        let mut work = vec![root];
        while let Some(node) = work.pop() {
            self.configured.active[node.0 as usize] = false;
            self.configured.ast.push_children(node, &mut work);
        }
    }
}
