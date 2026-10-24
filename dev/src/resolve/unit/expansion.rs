//! Install only newly configured expansion facts into a file's stable syntax arenas.
use super::Unit;
use crate::{
    expand::{
        crate_syntax::{AttributeEffect, ConfiguredCrate, ConfiguredSyntax, Owner},
        stream::Origin,
    },
    lex::token::LexError,
    literal::storage::{Storage, StoredValue},
    syntax::{
        ast::{Kind, NodeId, Syntax},
        attribute::Input,
        token_tree::{Span, error},
        transfer,
    },
};
use std::num::NonZeroU32;

struct Configuration {
    active: Vec<bool>,
    owners: Vec<Owner>,
    effects: Vec<AttributeEffect>,
}
impl Configuration {
    fn take<L>(generated: ConfiguredSyntax<L>) -> (Syntax<L>, Self) {
        (
            generated.ast,
            Self {
                active: generated.active,
                owners: generated.owners,
                effects: generated.effects,
            },
        )
    }
    fn copy<L>(generated: &ConfiguredSyntax<L>) -> Self {
        Self {
            active: generated.active.clone(),
            owners: generated.owners.clone(),
            effects: generated.effects.clone(),
        }
    }
}

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
        generated: ConfiguredCrate<'_>,
        origins: &[Origin],
    ) -> Result<Span, LexError> {
        let (ast, configuration) = Configuration::take(generated);
        self.install(
            target,
            source,
            &ast,
            configuration,
            origins,
            |storage, value, base| {
                storage
                    .store(source, value)
                    .and_then(|value| value.offset_source(base))
                    .map_err(|e| error(0, e))
            },
        )
    }
    /// Install cached included syntax by borrowing its arenas and copying only demanded facts.
    ///
    /// # Errors
    /// Rejects invalid targets, source mappings, literal storage or compact arena overflow.
    pub fn install_include(
        &mut self,
        target: NodeId,
        included: &Self,
        origins: &[Origin],
    ) -> Result<Span, LexError> {
        let source = included.source.text();
        self.install(
            target,
            source,
            &included.configured.ast,
            Configuration::copy(&included.configured),
            origins,
            |storage, value, base| {
                storage
                    .import(source, &included.literals, *value, base)
                    .map_err(|e| error(0, e))
            },
        )
    }
    fn install<L>(
        &mut self,
        target: NodeId,
        source: &str,
        ast: &Syntax<L>,
        mut configuration: Configuration,
        origins: &[Origin],
        mut literal: impl FnMut(&mut Storage, &L, u32) -> Result<StoredValue, LexError>,
    ) -> Result<Span, LexError> {
        let target_node = self
            .configured
            .ast
            .nodes
            .get(target.get() as usize)
            .ok_or_else(|| error(0, "missing expansion target"))?;
        if !matches!(target_node.kind, Kind::Macro { .. })
            || !self.configured.active[target.get() as usize]
        {
            return Err(error(
                target_node.span.start,
                "expansion target is not an active macro",
            ));
        }
        let base = self.source.append_offset(source).map_err(|e| error(0, e))?;
        let indices =
            self.expansion_metadata(target, ast, &mut configuration, base, source.len())?;
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
        let mark = self.literals.decoded_bytes();
        let result = transfer::append(
            ast,
            &mut self.configured.ast,
            |span| shift(span, base, source.len()),
            |value| literal(&mut self.literals, value, base),
        );
        let new_root = match result {
            Ok(root) => root,
            Err(error) => {
                self.literals.truncate(mark);
                return Err(error);
            }
        };
        // All ranges were validated above; this is the transaction's commit point.
        self.source.append(source).map_err(|e| error(0, e))?;
        self.replace_syntax(target, new_root, configuration, indices);
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
    /// Commit only a fully prepared replacement; original node IDs remain diagnostic keys.
    fn replace_syntax(
        &mut self,
        target: NodeId,
        new_root: NodeId,
        configuration: Configuration,
        indices: Vec<NonZeroU32>,
    ) {
        let mut obsolete = vec![target];
        while let Some(node) = obsolete.pop() {
            self.original.save(
                node,
                self.configured.ast.node(node),
                self.configured.active[node.get() as usize],
                self.owner_indices[node.get() as usize],
            );
            self.configured.active[node.get() as usize] = false;
            self.configured.ast.push_children(node, &mut obsolete);
        }
        self.retain_configuration(configuration, indices);
        self.configured.active[target.get() as usize] =
            self.configured.active[new_root.get() as usize];
        self.configured.active[new_root.get() as usize] = false;
        self.configured.ast.nodes[target.get() as usize] =
            self.configured.ast.nodes[new_root.get() as usize];
    }

    fn expansion_metadata<L>(
        &self,
        target: NodeId,
        ast: &Syntax<L>,
        generated: &mut Configuration,
        base: u32,
        length: usize,
    ) -> Result<Vec<NonZeroU32>, LexError> {
        let node_base = add(self.configured.ast.nodes.len(), ast.nodes.len())?;
        let attribute_base = add(self.configured.ast.attributes.len(), ast.attributes.len())?;
        let effect_base = add(self.configured.effects.len(), generated.effects.len())?;
        let owner_base = add(self.configured.owners.len(), generated.owners.len())?;
        if generated.active.len() != ast.nodes.len() || ast.root.get() as usize >= ast.nodes.len() {
            return Err(error(0, "invalid configured expansion arena"));
        }
        let root = ast.root;
        let mapped_node = |node: NodeId| {
            if node == root {
                target
            } else {
                NodeId::new(node.get() + node_base)
            }
        };
        for owner in &mut generated.owners {
            if owner.node.get() as usize >= ast.nodes.len()
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
            if effect.capture as usize >= ast.attributes.len()
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
            .store(source, &value)
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
        other: impl Fn(super::FileId) -> Option<&'a Unit>,
        config: &crate::expand::config::Config,
    ) -> Result<NodeId, LexError> {
        let source = self.source.text();
        let units = |id| if id == file { Some(&*self) } else { other(id) };
        let view = registry.view(|id| units(id).map(|unit| unit.source.text()));
        let inputs = crate::syntax::edition_context::WithEditions::new(&view, |token| {
            super::editions::token(file, token, units)
        });
        let ast = crate::syntax::expression::fragment::parse_arguments(source, tokens, &inputs)?;
        let (ast, mut configuration) = Configuration::take(
            crate::expand::crate_syntax::configure_with_inputs(source, ast, config, &inputs)?,
        );
        let root =
            NodeId::new(add(self.configured.ast.nodes.len(), ast.nodes.len())? + ast.root.get());
        let indices = self.expansion_metadata(root, &ast, &mut configuration, 0, source.len())?;
        let root = append_syntax(source, 0, ast, &mut self.configured.ast, &mut self.literals)?;
        self.retain_configuration(configuration, indices);
        Ok(root)
    }

    /// Select over this unit's real source/opaque registry, without a spelling replay.
    /// Every inactive RHS is parsed and discarded before only selected configuration
    /// and syntax are committed. Existing token origins and payload IDs stay intact.
    pub(crate) fn install_choice<'a>(
        &mut self,
        site: crate::expand::builtin::choice::Site,
        input: &crate::expand::macros::input::Input,
        registry: &crate::expand::registry::Registry,
        other: impl Fn(super::FileId) -> Option<&'a Unit>,
        config: &crate::expand::config::Config,
        charge: &crate::expand::builtin::choice::Charge<'_>,
    ) -> Result<crate::expand::builtin::choice::Installed, LexError> {
        let actual = self
            .configured
            .ast
            .nodes
            .get(site.target.get() as usize)
            .ok_or_else(|| error(site.original.start, "missing cfg_select call target"))?;
        if actual.span != site.original
            || self.configured.active.get(site.target.get() as usize) != Some(&true)
            || !matches!(actual.kind, Kind::Macro { context, .. } if context == site.grammar)
        {
            return Err(error(
                site.original.start,
                "cfg_select changed its actual call target or context",
            ));
        }
        let source = self.source.text();
        let units = |id| {
            if id == site.file {
                Some(&*self)
            } else {
                other(id)
            }
        };
        let view = registry.view(|id| units(id).map(|unit| unit.source.text()));
        let inputs = crate::syntax::edition_context::WithEditions::new(&view, |token| {
            super::editions::token(site.file, token, units)
        });
        let crate::expand::builtin::choice::Selected {
            ast,
            span,
            parser_calls,
        } = crate::expand::builtin::choice::select(
            source,
            input,
            config,
            site.grammar,
            &inputs,
            charge,
        )?;
        let (ast, mut configuration) = Configuration::take(
            crate::expand::crate_syntax::configure_with_inputs(source, ast, config, &inputs)?,
        );
        charge(
            ast.nodes.len().saturating_add(ast.edges.len()),
            ast.nodes
                .len()
                .saturating_mul(size_of::<crate::syntax::ast::Node>())
                .saturating_mul(4),
        )?;
        let indices =
            self.expansion_metadata(site.target, &ast, &mut configuration, 0, source.len())?;
        let new_root = append_syntax(source, 0, ast, &mut self.configured.ast, &mut self.literals)?;
        self.replace_syntax(site.target, new_root, configuration, indices);
        Ok(crate::expand::builtin::choice::Installed {
            generated: span,
            parser_calls,
        })
    }

    fn retain_configuration(&mut self, configuration: Configuration, indices: Vec<NonZeroU32>) {
        self.configured.active.extend(configuration.active);
        self.owner_indices
            .resize(self.configured.ast.nodes.len(), None);
        for (owner, index) in configuration.owners.iter().zip(indices) {
            self.owner_indices[owner.node.get() as usize] = Some(index);
        }
        self.configured.owners.extend(configuration.owners);
        self.configured.effects.extend(configuration.effects);
    }

    pub(crate) fn retire_arguments(&mut self, root: NodeId) {
        let mut work = vec![root];
        while let Some(node) = work.pop() {
            self.configured.active[node.get() as usize] = false;
            self.configured.ast.push_children(node, &mut work);
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/chosen_installation.rs"]
mod tests;
