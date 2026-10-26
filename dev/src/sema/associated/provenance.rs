//! Source elaboration cannot receive an imported owner or invented syntax site.
use super::{Inventory, Owner, OwnerId, OwnerKind};
use crate::resolve::{model::DefId, unit::NodeKey};

pub(super) fn imported_self(
    report: &crate::resolve::expansion::Report,
    index: usize,
) -> Result<bool, crate::resolve::package::Error> {
    use crate::resolve::{model::DefKind, package::Error};
    let client = report.package.source_client().ok_or_else(|| {
        Error::unlocated("imported Self declaration requires its authenticated source client")
    })?;
    let imported = report.resolution.crates.as_deref().ok_or_else(|| {
        Error::unlocated("imported Self declaration lacks its complete inventory")
    })?;
    let provider = client
        .require_inventory(imported.dependencies())
        .map_err(Error::unlocated)?;
    let catalog = imported
        .origin_catalog()
        .map_err(Error::unlocated)?
        .ok_or_else(|| Error::unlocated("imported Self declaration lacks its complete catalog"))?;
    let definition =
        DefId(u32::try_from(index).map_err(|_| Error::unlocated("imported Self index overflow"))?);
    let id = catalog.declaration(definition).map_err(Error::unlocated)?;
    catalog
        .validate_declaration(id, definition, DefKind::SelfType)
        .map_err(Error::unlocated)?;
    let (owner, ordinal, _) = catalog
        .declaration_coordinate(id)
        .map_err(Error::unlocated)?;
    let descriptor = imported
        .dependencies()
        .artifacts()
        .get(provider as usize)
        .and_then(crate::support::crates::Interface::source_descriptor)
        .ok_or_else(|| {
            Error::unlocated("imported Self requires its actual complete source descriptor")
        })?;
    if descriptor.identity() != owner
        || imported
            .generic_definition(owner, ordinal)
            .map_err(Error::unlocated)?
            != definition
    {
        return Err(Error::unlocated(
            "imported Self changed its exact provider/definition remap",
        ));
    }
    Ok(true)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) struct SourceOwner {
    pub source: NodeKey,
    pub self_definition: DefId,
    pub kind: OwnerKind,
}
impl From<SourceOwner> for Owner {
    fn from(owner: SourceOwner) -> Self {
        Self {
            source: Some(owner.source),
            self_definition: owner.self_definition,
            kind: owner.kind,
        }
    }
}
impl Owner {
    pub(in crate::sema) fn source_owner(self) -> Result<SourceOwner, &'static str> {
        let source = self
            .source
            .ok_or("source elaboration requires its actual source owner")?;
        if matches!(self.kind, OwnerKind::ImportedImpl(_)) {
            return Err("an imported impl cannot acquire a source syntax owner");
        }
        Ok(SourceOwner {
            source,
            self_definition: self.self_definition,
            kind: self.kind,
        })
    }
}
impl Inventory {
    fn imported_trait_owner(&self, definition: DefId) -> Result<Owner, &'static str> {
        let owner = self
            .owner(definition)
            .and_then(|id| self.get(id))
            .copied()
            .ok_or("imported implementation policy lacks its actual trait owner")?;
        if !matches!(owner.kind,OwnerKind::Trait{definition:actual,..} if actual==definition) {
            return Err("imported implementation policy changed its actual trait declaration");
        }
        Ok(owner)
    }
    /// Only complete v2 rows can describe an ordinary empty non-role trait.
    /// This authenticates its family; callers must still prove impl obligations.
    pub(in crate::sema) fn imported_empty_module_trait(
        &self,
        definition: DefId,
    ) -> Result<bool, &'static str> {
        let owner = self.imported_trait_owner(definition)?;
        let eligible = self
            .language
            .as_ref()
            .and_then(|l| l.imported.as_deref())
            .ok_or("imported trait conformance lacks its canonical source graph")?
            .empty_module_trait(owner)?;
        if eligible
            && self
                .owner(definition)
                .and_then(|id| self.owner_members(id))
                .is_none_or(|members| !members.is_empty())
        {
            return Err("ordinary module conformance changed its installed empty trait members");
        }
        Ok(eligible)
    }
    pub(in crate::sema) fn imported_denies_implementation(
        &self,
        definition: DefId,
    ) -> Result<bool, &'static str> {
        let owner = self.imported_trait_owner(definition)?;
        self.language
            .as_ref()
            .and_then(|l| l.imported.as_deref())
            .ok_or("imported implementation policy lacks its canonical source graph")?
            .denies_implementation(owner)
    }
    pub(in crate::sema) fn imported_implementation_domain(
        &self,
        definition: DefId,
    ) -> Result<crate::support::crates::generics::source::ImplementationDomain, &'static str> {
        let owner = self.imported_trait_owner(definition)?;
        self.language
            .as_ref()
            .and_then(|l| l.imported.as_deref())
            .ok_or("imported implementation domain lacks its canonical source graph")?
            .implementation_domain(owner)
    }

    pub(in crate::sema) fn imported_impl_request(
        &self,
        store: &crate::types::Store,
        owner: Owner,
    ) -> Result<crate::types::parameters::predicates::Predicate, &'static str> {
        if self
            .owner(owner.self_definition)
            .and_then(|id| self.get(id))
            .copied()
            != Some(owner)
        {
            return Err("imported candidate request changed its actual retained owner");
        }
        self.language
            .as_ref()
            .and_then(|l| l.imported.as_deref())
            .ok_or("imported candidate request lacks its exact canonical graph")?
            .implementation_request(store, owner)
    }
    pub(in crate::sema) fn require_imported_role(
        &self,
        role: crate::sema::compiler_items::Role,
    ) -> Result<(), &'static str> {
        self.language
            .as_ref()
            .and_then(|l| l.imported.as_deref())
            .ok_or("imported role lacks its exact canonical header graph")?
            .role_origin(role)
    }
    /// Source-only consumers (including early conformance) fail explicitly for
    /// imported owners instead of treating absence as a source identity.
    pub(in crate::sema) fn source_owner(&self, id: OwnerId) -> Result<SourceOwner, &'static str> {
        self.get(id)
            .copied()
            .ok_or("source associated owner is outside its inventory")?
            .source_owner()
    }

    /// Source AST passes skip only a fully authenticated imported owner. The
    /// immutable header correspondence supplies no conformance or body proof.
    pub(in crate::sema) fn source_at(
        &self,
        index: usize,
    ) -> Result<Option<SourceOwner>, &'static str> {
        let owner = self
            .owners
            .get(index)
            .copied()
            .ok_or("associated owner index is outside its actual inventory")?;
        if owner.source.is_some() {
            return owner.source_owner().map(Some);
        }
        self.require_imported(owner)?;
        Ok(None)
    }

    pub(in crate::sema) fn require_imported(&self, owner: Owner) -> Result<(), &'static str> {
        self.language
            .as_ref()
            .and_then(|l| l.imported.as_deref())
            .ok_or("imported associated owner lacks its authenticated canonical headers")?
            .owner(owner)
    }
    pub(in crate::sema) fn imported_implementation(
        &self,
        store: &crate::types::Store,
        owner: Owner,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<(bool, bool), &'static str> {
        self.language
            .as_ref()
            .and_then(|l| l.imported.as_deref())
            .ok_or("imported implementation lacks its exact canonical owner graph")?
            .implementation(store, owner, predicate)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/domain_consumers.rs"]
mod module_tests;
