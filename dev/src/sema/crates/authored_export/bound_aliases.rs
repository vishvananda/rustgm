//! Publish declared clauses under the genuine independent alias and lexical Self.
use super::{
    DeclarationRef, Declarations, DefId, DefKind, Report, Stage, canonical_nodes::Context, g, s,
};
use crate::{
    support::constness::Constness,
    syntax::{ast::Kind, item::ItemKind},
    types::parameters::{clauses::Clause, predicates::Bound},
};

impl Stage {
    pub(super) fn trait_alias_headers(
        &mut self,
        report: &Report,
        facts: &Declarations,
        aliases: Vec<(DeclarationRef, DefId)>,
    ) -> Result<(), &'static str> {
        if aliases.is_empty() {
            return Ok(());
        }
        self.reserve(
            aliases
                .len()
                .saturating_mul(size_of::<s::TraitAliasHeader>()),
        )?;
        self.cold()?.trait_aliases.reserve_exact(aliases.len());
        for (declaration, definition) in aliases {
            let owner = &report.resolution.definitions[definition.0 as usize];
            let source = owner
                .node
                .ok_or("source trait alias lacks its actual declaration")?;
            let ast = &report.package.unit(source.file).configured.ast;
            let Kind::Item(item) = ast.node(source.node).kind else {
                return Err("source trait alias lacks its actual item");
            };
            let item = &ast.items[item as usize];
            if !matches!(
                item.kind,
                ItemKind::TraitAlias {
                    constant: false,
                    ..
                }
            ) || !ast.list(item.generics).is_empty()
            {
                return Err("source trait alias requires its direct ordinary declaration profile");
            }
            let self_id = facts
                .generics
                .parents
                .get(&definition)
                .copied()
                .ok_or("source trait alias lacks its actual lexical Self parent")?;
            let lexical = &report.resolution.definitions[self_id.0 as usize];
            if lexical.kind != DefKind::SelfType || lexical.node != Some(source) {
                return Err("source trait alias changed its actual lexical Self item");
            }
            let source_self = facts.definitions[self_id.0 as usize]
                .ok_or("source trait alias lacks its completed Self type")?;
            let context = Context {
                owner: declaration,
                frame: None,
                body: None,
            };
            let self_type = self.ty(facts, source_self, context)?;
            let start = self.trait_alias_input_offset()?;
            for input in facts.generics.declared_predicates(definition) {
                let Clause::Predicate(predicate) = input.value else {
                    return Err("source trait alias requires direct ordinary clauses");
                };
                if !matches!(predicate.bound, Bound::Sized | Bound::ProjectionEqual(_))
                    && !predicate
                        .bound
                        .trait_parts()
                        .is_some_and(|(_, effect)| effect == Constness::Ordinary)
                {
                    return Err("source trait alias requires direct ordinary clauses");
                }
                self.clause(facts, input.into(), context, None)?;
            }
            let inputs = g::Range {
                start,
                end: self.trait_alias_input_offset()?,
            };
            let self_definition = self.declaration(self_id)?;
            self.cold()?.trait_aliases.push(s::TraitAliasHeader {
                declaration,
                self_definition,
                self_type,
                inputs,
            });
        }
        Ok(())
    }

    fn trait_alias_input_offset(&self) -> Result<u32, &'static str> {
        u32::try_from(self.rows.clauses.len())
            .map_err(|_| "source trait alias clause range overflow")
    }
}
