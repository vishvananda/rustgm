//! Required methods retain authenticated absence in the same immutable inventory.
use super::{Catalog, DefId, DefKind, Graph, SourceDescriptor, s};
use crate::resolve::body_origin::BodyOrigin;
use crate::sema::associated::{Owner, OwnerKind};

impl Graph {
    pub(in crate::sema) fn role_origin(
        &self,
        role: crate::sema::compiler_items::Role,
    ) -> Result<(), &'static str> {
        let crate::resolve::body_origin::DeclarationOrigin::Imported(id) = role.origin else {
            return Err("imported role use requires its actual imported declaration origin");
        };
        self.origins
            .validate_declaration(id, role.definition, DefKind::Trait)?;
        let (owner, _, _) = self.origins.declaration_coordinate(id)?;
        if owner != self.descriptor(&self.origins)?.identity() {
            return Err("imported role belongs to a different actual descriptor");
        }
        Ok(())
    }
    pub(in crate::sema) fn owner(&self, owner: Owner) -> Result<(), &'static str> {
        if owner.source.is_some() {
            return Err("imported owner authentication cannot accept source syntax provenance");
        }
        let descriptor = self.descriptor(&self.origins)?;
        let rows = descriptor.rows();
        match owner.kind {
            OwnerKind::Trait {
                definition,
                auto_,
                unsafe_,
                constant,
            } => {
                let id = self.origins.declaration(definition)?;
                self.origins
                    .validate_declaration(id, definition, DefKind::Trait)?;
                let (artifact, row, _) = self.origins.declaration_coordinate(id)?;
                if artifact != descriptor.identity() {
                    return Err("imported trait owner belongs to another actual artifact");
                }
                let Some(s::HeaderRef::Trait(index)) = rows.header(row)? else {
                    return Err("imported trait owner lacks its exact completed header");
                };
                let header = &rows.traits[index as usize];
                let flags = header.flags.bits();
                if header.declaration != row
                    || self
                        .origins
                        .definition_at(artifact, header.self_definition)?
                        != owner.self_definition
                    || auto_ != (flags & s::TraitFlags::AUTO != 0)
                    || unsafe_ != (flags & s::TraitFlags::UNSAFE != 0)
                    || constant != (flags & s::TraitFlags::CONST != 0)
                {
                    return Err("imported trait changed its actual Self/flags/owner mapping");
                }
                let id = self.origins.declaration(owner.self_definition)?;
                self.origins
                    .validate_declaration(id, owner.self_definition, DefKind::SelfType)?;
            }
            OwnerKind::ImportedImpl(id) => {
                let definition = self.origins.declaration_identity(id, DefKind::SelfType)?;
                let (artifact, row, _) = self.origins.declaration_coordinate(id)?;
                if definition != owner.self_definition || artifact != descriptor.identity() {
                    return Err("imported impl owner changed its exact Self/catalog mapping");
                }
                let Some(s::HeaderRef::Implementation(index)) = rows.header(row)? else {
                    return Err("imported impl has no actual completed header");
                };
                let header = &rows.implementations[index as usize];
                if header.self_definition != row || header.scope != row {
                    return Err("imported impl changed its actual declaration/header scope");
                }
            }
            OwnerKind::Impl { .. } => {
                return Err("imported impl cannot carry source syntax targets");
            }
        }
        Ok(())
    }

    pub(in crate::sema) fn implementation(
        &self,
        owner: Owner,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<(bool, bool), &'static str> {
        self.owner(owner)?;
        let OwnerKind::ImportedImpl(id) = owner.kind else {
            return Err("imported conformance requires its actual imported impl owner");
        };
        let (_, row, _) = self.origins.declaration_coordinate(id)?;
        let descriptor = self.descriptor(&self.origins)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Implementation(index)) = rows.header(row)? else {
            return Err("imported conformance has no actual implementation header");
        };
        let raw = &rows.implementations[index as usize];
        let application = raw
            .trait_application
            .ok_or("imported inherent conformance requires its wider actual member profile")?;
        if self.application(application)? != predicate
            || self
                .ty(raw.self_type)
                .map_err(|_| "imported impl lost its actual Self type")?
                != predicate.ty
        {
            return Err("imported conformance changed its complete actual trait/Self predicate");
        }
        Ok((raw.constant, raw.negative))
    }
    pub(super) fn descriptor(
        &self,
        catalog: &Catalog,
    ) -> Result<SourceDescriptor<'_>, &'static str> {
        if !std::ptr::eq(self.origins.as_ref(), catalog) {
            return Err("source header belongs to another actual catalog allocation");
        }
        let definition = catalog.declaration_identity(self.root, DefKind::Module)?;
        let (owner, row, edition) = catalog.declaration_coordinate(self.root)?;
        let descriptor = self
            .inventory
            .artifacts()
            .get(self.provider_index as usize)
            .and_then(crate::support::crates::Interface::source_descriptor)
            .ok_or("source header lost its complete immutable inventory")?;
        if descriptor.identity() != owner
            || row != self.root_row
            || descriptor.edition() != edition
            || descriptor.profile() != s::Profile::Source2024Functions
            || catalog.definition_at(owner, row)? != definition
        {
            return Err("source header changed its actual inventory/catalog root");
        }
        Ok(descriptor)
    }

    pub(in crate::sema::crates::typed_headers) fn function(
        &self,
        catalog: &Catalog,
        definition: DefId,
        body: Option<BodyOrigin>,
        universals: u32,
    ) -> Result<bool, &'static str> {
        let descriptor = self.descriptor(catalog)?;
        let declaration = catalog.declaration(definition)?;
        catalog.validate_declaration(declaration, definition, DefKind::Function)?;
        let (owner, ordinal, _) = catalog.declaration_coordinate(declaration)?;
        if owner != descriptor.identity() {
            return Ok(false);
        }
        if catalog.definition_at(owner, ordinal)? != definition {
            return Err("source function header belongs to another actual artifact");
        }
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(ordinal)? else {
            return Err("source function lacks its actual completed header row");
        };
        let header = &rows.functions[index as usize];
        if header.declaration != ordinal || header.regions.total != universals {
            return Err("source function changed its actual declaration or original universals");
        }
        match (header.body, body) {
            (s::FunctionBody::Absent, None) => {
                let parent = header
                    .parent
                    .ok_or("required source method lacks its actual trait owner")?;
                let raw = rows
                    .declarations
                    .get(ordinal.0 as usize)
                    .ok_or("required source declaration is outside its table")?;
                if raw.definition.parent != Some(parent)
                    || rows
                        .declarations
                        .get(parent.0 as usize)
                        .is_none_or(|row| row.definition.kind != s::DefinitionKind::Trait)
                    || !matches!(rows.header(parent)?, Some(s::HeaderRef::Trait(_)))
                {
                    return Err("body absence is not an actual required trait method");
                }
                let actual = catalog.definition_at(owner, parent)?;
                let parent = catalog.declaration(actual)?;
                catalog.validate_declaration(parent, actual, DefKind::Trait)?;
            }
            (s::FunctionBody::Original(reference), Some(body)) => {
                let id = catalog.body_at(owner, reference)?;
                if catalog.origin(id)? != body {
                    return Err("provided source function changed its exact original body");
                }
                catalog.validate_function_origin(definition, body)?;
            }
            _ => return Err("source function omitted or invented an actual provided body"),
        }
        Ok(true)
    }
}
