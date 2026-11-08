//! Required methods retain authenticated absence in the same immutable inventory.
use super::{Catalog, DefId, DefKind, Graph, SourceDescriptor, s};
use crate::resolve::body_origin::BodyOrigin;
use crate::sema::associated::{Owner, OwnerKind};

impl Graph {
    pub(in crate::sema) fn conversion_role(
        &self,
        store: &crate::types::Store,
        role: crate::sema::compiler_items::Role,
    ) -> Result<(), &'static str> {
        self.require_store(store)?;
        self.role_origin(role)?;
        if !self.descriptor(&self.origins)?.profile().conversion_roles() {
            return Err("imported From requires its explicit version-nine profile");
        }
        Ok(())
    }
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
        super::checked_rows::Mapping::require_owner(&super::checked_rows::Ordinary(self), owner)
    }

    pub(in crate::sema) fn implementation(
        &self,
        store: &crate::types::Store,
        owner: Owner,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<(bool, bool), &'static str> {
        self.require_store(store)?;
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
        let flags = self.applied_trait_flags(application)?;
        // rustc's impl safety rule, re-proved for every imported impl: an
        // impl of an unsafe trait is `unsafe impl` (E0200), an impl of a safe
        // trait is not (E0199), and a negative impl never is (E0198).
        let required_unsafe = flags & s::TraitFlags::UNSAFE != 0 && !raw.negative;
        if raw.unsafe_ != required_unsafe {
            return Err("selected impl safety/member kinds disagree with its actual trait");
        }
        Ok((raw.constant, raw.negative))
    }
    /// The flags of the trait row an imported impl's application names. The
    /// ledger admitted only this packet's own (artifact 0) trait headers.
    fn applied_trait_flags(
        &self,
        application: s::TraitApplicationRef,
    ) -> Result<u16, &'static str> {
        let rows = self.descriptor(&self.origins)?.rows();
        let application = rows
            .trait_applications
            .get(application.0 as usize)
            .ok_or("imported conformance lost its actual trait application")?;
        if application.declaration.artifact != s::ArtifactRef(0) {
            return Err("imported conformance requires its own packet's actual trait");
        }
        let Some(s::HeaderRef::Trait(index)) = rows.header(application.declaration.declaration)?
        else {
            return Err("imported conformance lost its actual trait row");
        };
        Ok(rows.traits[index as usize].flags.bits())
    }
    /// Version thirteen: an imported impl of its own packet's auto trait.
    /// `implementation` re-proved its safety, and the ledger its empty member
    /// range. Answers whether it is negative.
    pub(in crate::sema) fn automatic_implementation(
        &self,
        store: &crate::types::Store,
        owner: Owner,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<bool, &'static str> {
        let (_, negative) = self.implementation(store, owner, predicate)?;
        let OwnerKind::ImportedImpl(id) = owner.kind else {
            return Err("imported conformance requires its actual imported impl owner");
        };
        let descriptor = self.descriptor(&self.origins)?;
        if !descriptor.profile().body_nodes() {
            return Err("imported automatic implementation requires its version-thirteen profile");
        }
        let (_, row, _) = self.origins.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Implementation(index)) = rows.header(row)? else {
            return Err("imported conformance has no actual implementation header");
        };
        let application = rows.implementations[index as usize]
            .trait_application
            .ok_or("imported automatic implementation lost its actual trait application")?;
        if self.applied_trait_flags(application)? & s::TraitFlags::AUTO == 0 {
            return Err("imported automatic implementation requires its exact auto trait");
        }
        Ok(negative)
    }
    /// Version thirteen: an imported inherent impl has no trait application,
    /// no modifier and extends only a nominal of its own provider. The Self is
    /// re-read from the consumer's own installed type, never a spelling.
    pub(in crate::sema) fn inherent(
        &self,
        store: &crate::types::Store,
        owner: Owner,
        self_type: crate::types::TypeId,
    ) -> Result<(), &'static str> {
        self.require_store(store)?;
        self.owner(owner)?;
        let OwnerKind::ImportedInherent(id) = owner.kind else {
            return Err("imported inherent impl requires its actual imported owner");
        };
        let descriptor = self.descriptor(&self.origins)?;
        if !descriptor.profile().body_nodes() {
            return Err("inherent source impl requires its wider member profile");
        }
        let (_, row, _) = self.origins.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Implementation(index)) = rows.header(row)? else {
            return Err("imported inherent impl has no actual implementation header");
        };
        let raw = &rows.implementations[index as usize];
        if raw.trait_application.is_some()
            || raw.constant
            || raw.unsafe_
            || raw.negative
            || self
                .ty(raw.self_type)
                .map_err(|_| "imported inherent impl lost its actual Self type")?
                != self_type
        {
            return Err("imported inherent impl changed its actual Self or modifiers");
        }
        let Some(crate::types::Kind::Adt(nominal, _)) = store.kind(self_type) else {
            return Err("imported inherent impl Self is not a nominal of its own provider");
        };
        let declaration = self.origins.declaration(DefId(nominal.0))?;
        let (artifact, _, _) = self.origins.declaration_coordinate(declaration)?;
        if artifact != descriptor.identity() {
            return Err("imported inherent impl Self is not a nominal of its own provider");
        }
        Ok(())
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
            || catalog.definition_at(owner, row)? != definition
        {
            return Err("source header changed its actual inventory/catalog root");
        }
        if !descriptor.profile().ordinary_functions() {
            match descriptor.profile() {
                s::Profile::Source2024ModuleContracts
                | s::Profile::Source2024ModuleAvailability
                | s::Profile::Source2024ModuleFeaturePredicates
                | s::Profile::Source2024ModuleDefinitions
                | s::Profile::Source2024ModuleCompatibility
                | s::Profile::Source2024ModuleCallables
                | s::Profile::Source2024ModuleSafety
                | s::Profile::Source2024ModuleConversions
                | s::Profile::Source2024ModuleNominals
                | s::Profile::Source2024ModuleCompilerOperations
                | s::Profile::Source2024ModuleBranches
                | s::Profile::Source2024ModuleBodies => {
                    descriptor.module_contracts()?;
                }
                _ => return Err("source header profile is unsupported"),
            }
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
        require_function(catalog, descriptor, definition, body, universals)
    }
}
pub(in crate::sema::crates::typed_headers) fn require_function(
    catalog: &Catalog,
    descriptor: SourceDescriptor<'_>,
    definition: DefId,
    body: Option<BodyOrigin>,
    universals: u32,
) -> Result<bool, &'static str> {
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
        (s::FunctionBody::CompilerOperation(reference), Some(body)) => {
            let operation = rows
                .compiler_operations
                .get(reference.0 as usize)
                .ok_or("compiler operation lost its actual recipe")?;
            let id = catalog.body_at(owner, operation.body)?;
            if operation.declaration != ordinal || catalog.origin(id)? != body {
                return Err("compiler operation changed its actual body/declaration origin");
            }
            catalog.validate_function_origin(definition, body)?;
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
