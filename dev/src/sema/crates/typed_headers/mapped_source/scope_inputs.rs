//! Full declaration assumptions share their existing typed input owners.
use super::{Catalog, DefId, Graph, s, slot_bindings};
use crate::sema::declaration_inputs::Input;
use std::sync::Arc;

impl Graph {
    pub(in crate::sema::crates::typed_headers) fn require_input_parent(
        &self,
        catalog: &Catalog,
        generics: &crate::sema::generics::Generics,
        definition: DefId,
    ) -> Result<(), &'static str> {
        let descriptor = self.descriptor(catalog)?;
        let Some(id) = catalog.declaration_if_imported(definition)? else {
            return Ok(());
        };
        let (owner, row, _) = catalog.declaration_coordinate(id)?;
        if owner != descriptor.identity() {
            return Ok(());
        }
        let rows = descriptor.rows();
        let raw = rows
            .declarations
            .get(row.0 as usize)
            .ok_or("source input scope is outside its actual declaration table")?;
        catalog.validate_declaration(id, definition, slot_bindings::kind(raw.definition.kind))?;
        let parent = match rows.header(row)? {
            Some(s::HeaderRef::Function(index)) => rows.functions[index as usize].parent,
            Some(s::HeaderRef::Trait(index)) => Some(rows.traits[index as usize].self_definition),
            Some(s::HeaderRef::Implementation(_)) => None,
            None if raw.definition.kind == s::DefinitionKind::SelfType => None,
            None => return Ok(()),
        };
        let expected = parent
            .map(|parent| catalog.definition_at(owner, parent))
            .transpose()?;
        if generics.parents.get(&definition).copied() != expected {
            return Err("source declaration inputs changed their actual parent scope");
        }
        Ok(())
    }

    pub(in crate::sema::crates::typed_headers) fn finish_inputs(
        &mut self,
        traits: Vec<Option<Arc<[Input]>>>,
        implementations: Vec<Option<Arc<[Input]>>>,
    ) -> Result<(), &'static str> {
        let rows = self.descriptor(&self.origins)?.rows();
        if !self.trait_inputs.is_empty()
            || !self.impl_inputs.is_empty()
            || rows.traits.len() != traits.len()
            || rows.implementations.len() != implementations.len()
            || traits.iter().chain(&implementations).any(Option::is_none)
        {
            return Err("source full declaration inputs have missing or duplicate typed ownership");
        }
        self.trait_inputs = traits;
        self.impl_inputs = implementations;
        Ok(())
    }

    pub(in crate::sema::crates::typed_headers) fn declaration_inputs(
        &self,
        catalog: &Catalog,
        definition: DefId,
        declared: bool,
    ) -> Result<Option<&[Input]>, &'static str> {
        let descriptor = self.descriptor(catalog)?;
        let Some(id) = catalog.declaration_if_imported(definition)? else {
            return Ok(None);
        };
        let (owner, row, _) = catalog.declaration_coordinate(id)?;
        if owner != descriptor.identity() {
            return Ok(None);
        }
        let rows = descriptor.rows();
        let raw = rows
            .declarations
            .get(row.0 as usize)
            .ok_or("source input declaration is outside its actual owner table")?;
        catalog.validate_declaration(id, definition, slot_bindings::kind(raw.definition.kind))?;
        let (inputs, prefix) = match rows.header(row)? {
            Some(s::HeaderRef::Trait(index)) => (
                self.trait_inputs.get(index as usize),
                rows.traits[index as usize].declared_inputs,
            ),
            Some(s::HeaderRef::Implementation(index)) => (
                self.impl_inputs.get(index as usize),
                rows.implementations[index as usize].declared_inputs,
            ),
            Some(s::HeaderRef::Function(_)) => {
                return Err(
                    "source function omitted its actual canonical input/header installation",
                );
            }
            None if raw.definition.kind == s::DefinitionKind::SelfType => {
                let parent = raw
                    .definition
                    .parent
                    .ok_or("source Self inputs lack their actual trait owner")?;
                let Some(s::HeaderRef::Trait(index)) = rows.header(parent)? else {
                    return Err("source Self inputs lack their actual trait header");
                };
                if rows.traits[index as usize].self_definition != row {
                    return Err("source Self input scope changed its actual trait owner");
                }
                return Ok(Some(&[]));
            }
            None => return Ok(None),
        };
        let inputs = inputs
            .and_then(Option::as_deref)
            .ok_or("source declaration omitted its actual full canonical input remap")?;
        Ok(Some(if declared {
            inputs
                .get(..prefix as usize)
                .ok_or("source declaration truncated its actual declared prefix")?
        } else {
            inputs
        }))
    }
}
