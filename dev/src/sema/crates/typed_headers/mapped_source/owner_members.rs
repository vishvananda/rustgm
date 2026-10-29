//! Associated inventory rows must match the complete actual immutable header graph.
use super::{DefKind, Graph, s};
use crate::resolve::model::DefId;
use crate::sema::associated::{Owner, OwnerKind, Schema};

impl Graph {
    pub(in crate::sema) fn implementation_domain(
        &self,
        owner: Owner,
    ) -> Result<s::ImplementationDomain, &'static str> {
        self.owner(owner)?;
        let OwnerKind::Trait { definition, .. } = owner.kind else {
            return Err("implementation domain requires its actual trait owner");
        };
        let descriptor = self.descriptor(&self.origins)?;
        let id = self.origins.declaration(definition)?;
        let (artifact, row, _) = self.origins.declaration_coordinate(id)?;
        let Some(s::HeaderRef::Trait(index)) = descriptor.rows().header(row)? else {
            return Err("implementation domain lacks its actual trait header");
        };
        match descriptor.profile() {
            s::Profile::Source2024Functions | s::Profile::Source2024FunctionContracts => {
                Ok(s::ImplementationDomain::Unrestricted)
            }
            s::Profile::Source2024ModuleContracts
            | s::Profile::Source2024ModuleAvailability
            | s::Profile::Source2024ModuleFeaturePredicates => {
                let domain = descriptor
                    .module_contracts()?
                    .trait_domain(index as usize, row)?;
                if let s::ImplementationDomain::RestrictedTo {
                    module,
                    clause_site,
                    path_site,
                } = domain.domain
                {
                    let definition = self.origins.definition_at(artifact, module)?;
                    let module_id = self.origins.declaration(definition)?;
                    self.origins
                        .validate_declaration(module_id, definition, DefKind::Module)?;
                    self.origins.declaration_site(id, clause_site)?;
                    self.origins.declaration_site(id, path_site)?;
                }
                Ok(domain.domain)
            }
            _ => Err("implementation domain has no complete admitted source profile"),
        }
    }
    pub(in crate::sema) fn empty_module_trait(&self, owner: Owner) -> Result<bool, &'static str> {
        self.owner(owner)?;
        let descriptor = self.descriptor(&self.origins)?;
        let OwnerKind::Trait { definition, .. } = owner.kind else {
            return Err("ordinary module conformance requires its actual trait owner");
        };
        let id = self.origins.declaration(definition)?;
        let (_, row, _) = self.origins.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Trait(index)) = rows.header(row)? else {
            return Err("ordinary module conformance lacks its exact trait header");
        };
        match descriptor.profile() {
            s::Profile::Source2024Functions | s::Profile::Source2024FunctionContracts => {
                return Ok(false);
            }
            s::Profile::Source2024ModuleContracts
            | s::Profile::Source2024ModuleAvailability
            | s::Profile::Source2024ModuleFeaturePredicates => {}
            _ => return Err("imported conformance has no complete admitted source profile"),
        }
        descriptor
            .module_contracts()?
            .trait_domain(index as usize, row)?;
        let header = &rows.traits[index as usize];
        if header.flags.bits() & !s::TraitFlags::RESTRICTED_IMPL != 0
            || !header.members.slice(&rows.members)?.is_empty()
        {
            return Err("imported impl family requires its wider complete conformance consumer");
        }
        Ok(true)
    }
    pub(in crate::sema) fn denies_implementation(
        &self,
        owner: Owner,
    ) -> Result<bool, &'static str> {
        self.owner(owner)?;
        let OwnerKind::Trait { definition, .. } = owner.kind else {
            return Err("imported implementation policy requires its actual trait owner");
        };
        // Deny-explicit-impl remains independent of module implementation domains.
        // A restricted provider-owned implementation can still be a valid candidate.
        let descriptor = self.descriptor(&self.origins)?;
        let id = self.origins.declaration(definition)?;
        let (_, row, _) = self.origins.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Trait(index)) = rows.header(row)? else {
            return Err("imported implementation policy lacks its completed actual trait header");
        };
        Ok(rows.traits[index as usize].flags.bits() & s::TraitFlags::DENY_IMPL != 0)
    }
    pub(in crate::sema) fn implementation_request(
        &self,
        store: &crate::types::Store,
        owner: Owner,
    ) -> Result<crate::types::parameters::predicates::Predicate, &'static str> {
        self.require_store(store)?;
        self.owner(owner)?;
        let OwnerKind::ImportedImpl(id) = owner.kind else {
            return Err("imported candidate reservation requires its actual impl owner");
        };
        let descriptor = self.descriptor(&self.origins)?;
        let (_, row, _) = self.origins.declaration_coordinate(id)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Implementation(index)) = rows.header(row)? else {
            return Err("imported candidate reservation lacks its complete actual header");
        };
        let application = rows.implementations[index as usize]
            .trait_application
            .ok_or("imported inherent reservation requires its wider candidate profile")?;
        let predicate = self.application(application)?;
        self.implementation(store, owner, predicate)?;
        Ok(predicate)
    }
    /// Header classification only; this neither checks nor completes conformance.
    pub(in crate::sema) fn feature_trait_implementation(
        &self,
        store: &crate::types::Store,
        owner: Owner,
    ) -> Result<Option<crate::types::parameters::predicates::Predicate>, &'static str> {
        self.require_store(store)?;
        self.owner(owner)?;
        if self.descriptor(&self.origins)?.profile()
            != s::Profile::Source2024ModuleFeaturePredicates
        {
            return Ok(None);
        }
        self.implementation_request(store, owner).map(Some)
    }
    pub(in crate::sema) fn associated_count(&self) -> Result<usize, &'static str> {
        let rows = self.descriptor(&self.origins)?.rows();
        Ok(rows.traits.len().saturating_add(rows.implementations.len()))
    }
    pub(in crate::sema) fn associated_row(
        &self,
        store: &crate::types::Store,
        owner: Owner,
        schema: &Schema,
        members: &[DefId],
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        super::checked_rows::Mapping::require_schema(
            &mut super::checked_rows::Ordinary(self),
            store,
            owner,
            schema,
            members,
            steps,
        )
    }
}
