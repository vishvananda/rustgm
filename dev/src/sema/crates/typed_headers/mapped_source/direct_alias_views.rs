//! An alias shares its own imported clause slice and has no trait or value owner.
use super::{Declarations, DefId, DefKind, Graph, s};
use crate::{
    sema::{associated::Schema, declaration_inputs::Input},
    support::crates::owner_catalog::DeclarationRef,
    types::{Kind, TypeId, parameters::ParameterKind},
};
use std::sync::Arc;

pub(in crate::sema) struct ImportedTraitAlias {
    pub definition: DefId,
    pub self_definition: DefId,
    pub schema: Schema,
    pub inputs: Arc<[Input]>,
}

impl Graph {
    pub(in crate::sema) fn trait_alias_count(&self) -> Result<usize, &'static str> {
        Ok(self.descriptor(&self.origins)?.rows().trait_aliases().len())
    }

    pub(in crate::sema) fn trait_aliases(
        &self,
        facts: &Declarations,
    ) -> Result<Vec<ImportedTraitAlias>, &'static str> {
        self.require_store(&facts.types)?;
        let descriptor = self.descriptor(&self.origins)?;
        let mut aliases = Vec::with_capacity(descriptor.rows().trait_aliases().len());
        for alias in descriptor.rows().trait_aliases() {
            let definition = self
                .origins
                .definition_at(descriptor.identity(), alias.declaration)?;
            aliases.push(self.trait_alias(facts, alias.declaration, definition)?);
        }
        Ok(aliases)
    }

    pub(super) fn trait_alias(
        &self,
        facts: &Declarations,
        row: DeclarationRef,
        definition: DefId,
    ) -> Result<ImportedTraitAlias, &'static str> {
        let descriptor = self.descriptor(&self.origins)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::TraitAlias(index)) = rows.header(row)? else {
            return Err("imported trait alias has no actual typed declaration");
        };
        let alias = rows.trait_aliases()[index as usize];
        if self.origins.definition_at(descriptor.identity(), row)? != definition {
            return Err("imported trait alias changed its actual declaration address");
        }
        let id = self.origins.declaration(definition)?;
        self.origins
            .validate_declaration(id, definition, DefKind::TraitAlias)?;
        let self_definition = self
            .origins
            .definition_at(descriptor.identity(), alias.self_definition)?;
        let self_id = self.origins.declaration(self_definition)?;
        self.origins
            .validate_declaration(self_id, self_definition, DefKind::SelfType)?;
        let self_type = self
            .ty(alias.self_type)
            .map_err(|_| "imported trait alias lost its canonical Self")?;
        self.require_alias_self(facts, self_definition, self_type)?;
        self.require_input_parent(&self.origins, &facts.generics, definition)?;
        if facts.generics.parents.get(&definition) != Some(&self_definition)
            || facts
                .generics
                .parameters
                .get(&definition)
                .is_none_or(|formals| !formals.is_empty())
            || facts.definitions.get(definition.0 as usize) != Some(&None)
        {
            return Err("imported trait alias changed its actual installed scope");
        }
        let inputs = self
            .trait_alias_inputs
            .get(index as usize)
            .and_then(Option::as_ref)
            .ok_or("imported trait alias omitted its actual input slice")?;
        if inputs.len() != alias.inputs.slice(&rows.clauses)?.len() {
            return Err("imported trait alias changed its complete input range");
        }
        Ok(ImportedTraitAlias {
            definition,
            self_definition,
            schema: Schema {
                self_type,
                captures: 0,
                bounds: Vec::new(),
            },
            inputs: Arc::clone(inputs),
        })
    }

    fn require_alias_self(
        &self,
        facts: &Declarations,
        definition: DefId,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        if facts.definitions.get(definition.0 as usize) != Some(&Some(ty))
            || !matches!(facts.types.kind(ty), Some(Kind::Parameter(id)) if
            facts.types.parameter(id).is_some_and(|parameter| {
                parameter.owner.0 == definition.0 && parameter.index == 0
                    && parameter.kind == (ParameterKind::Type { sized: false })
            }))
        {
            return Err("imported trait alias changed its lexical unsized Self");
        }
        self.require_input_parent(&self.origins, &facts.generics, definition)
    }
}
