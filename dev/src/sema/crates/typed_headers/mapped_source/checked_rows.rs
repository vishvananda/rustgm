//! One canonical owner/schema comparison kernel serves actual typed mappings.
use super::{Catalog, DefId, DefKind, Graph, SourceDescriptor, s};
use crate::{
    sema::associated::{Owner, OwnerKind, Schema},
    support::crates::generics::TypeRef,
    types::{RegionId, Store, TypeId},
};
pub(in crate::sema::crates::typed_headers) mod sealed {
    pub trait Sealed {}
}

/// Implementations are phase-owned adapters, never caller-supplied remap tables.
pub(in crate::sema::crates::typed_headers) trait Mapping<'rows>:
    sealed::Sealed
{
    fn catalog(&self) -> &'rows Catalog;
    fn descriptor(&self) -> Result<SourceDescriptor<'rows>, &'static str>;
    fn require_store(&mut self, store: &Store) -> Result<(), &'static str>;
    fn ty(&mut self, root: TypeRef) -> Result<TypeId, &'static str>;
    fn region(&mut self, root: s::RegionRef) -> Result<RegionId, &'static str>;
    fn members(
        &self,
        rows: s::ValidatedView<'rows>,
        header: s::HeaderRef,
    ) -> Result<MemberRows<'rows>, &'static str>;
    fn require_owner(&self, owner: Owner) -> Result<(), &'static str> {
        require_owner(self.catalog(), || self.descriptor(), owner)
    }

    fn require_schema(
        &mut self,
        store: &Store,
        owner: Owner,
        schema: &Schema,
        members: &[DefId],
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        *steps = steps.saturating_add(1);
        self.require_store(store)?;
        self.require_owner(owner)?;
        let descriptor = self.descriptor()?;
        let rows = descriptor.rows();
        let declaration = match owner.kind {
            OwnerKind::Trait { definition, .. } => definition,
            OwnerKind::ImportedImpl(_) => owner.self_definition,
            OwnerKind::Impl { .. } => {
                return Err("imported associated inventory cannot carry source syntax");
            }
        };
        let id = self.catalog().declaration(declaration)?;
        let (artifact, row, _) = self.catalog().declaration_coordinate(id)?;
        let header = rows
            .header(row)?
            .ok_or("imported associated inventory lacks its actual owner header")?;
        let (self_type, captures, bounds) = match header {
            s::HeaderRef::Trait(index) => {
                let raw = &rows.traits[index as usize];
                (raw.self_type, raw.region_captures, raw.region_bounds)
            }
            s::HeaderRef::Implementation(index) => {
                let raw = &rows.implementations[index as usize];
                (raw.self_type, raw.region_captures, raw.region_bounds)
            }
            s::HeaderRef::Function(_) => {
                return Err("imported associated inventory lacks its actual owner header");
            }
        };
        let expected_members = self.members(rows, header)?;
        if artifact != descriptor.identity()
            || schema.captures != captures
            || self
                .ty(self_type)
                .map_err(|_| "missing imported actual Self type")?
                != schema.self_type
        {
            return Err("imported associated schema changed its actual owner/Self/captures");
        }
        let bounds = bounds.slice(&rows.region_bounds)?;
        if bounds.len() != schema.bounds.len()
            || expected_members
                .declared_len()
                .is_some_and(|length| length != members.len())
        {
            return Err("imported associated schema omitted members or full region relations");
        }
        expected_members.require(
            self.catalog(),
            descriptor.identity(),
            rows,
            row,
            members,
            steps,
        )?;
        for (raw, &actual) in bounds.iter().zip(&schema.bounds) {
            *steps = steps.saturating_add(1);
            if (
                self.region(raw.longer)
                    .map_err(|_| "invalid imported schema bound")?,
                self.region(raw.shorter)
                    .map_err(|_| "invalid imported schema bound")?,
            ) != actual
            {
                return Err("imported associated schema changed its actual region relation");
            }
        }
        Ok(())
    }
}

pub(in crate::sema::crates::typed_headers) fn require_owner<'a>(
    catalog: &Catalog,
    descriptor: impl FnOnce() -> Result<SourceDescriptor<'a>, &'static str>,
    owner: Owner,
) -> Result<(), &'static str> {
    if owner.source.is_some() {
        return Err("imported owner authentication cannot accept source syntax provenance");
    }
    let descriptor = descriptor()?;
    let rows = descriptor.rows();
    match owner.kind {
        OwnerKind::Trait {
            definition,
            auto_,
            unsafe_,
            constant,
        } => {
            let id = catalog.declaration(definition)?;
            catalog.validate_declaration(id, definition, DefKind::Trait)?;
            let (artifact, row, _) = catalog.declaration_coordinate(id)?;
            if artifact != descriptor.identity() {
                return Err("imported trait owner belongs to another actual artifact");
            }
            let Some(s::HeaderRef::Trait(index)) = rows.header(row)? else {
                return Err("imported trait owner lacks its exact completed header");
            };
            let header = &rows.traits[index as usize];
            let flags = header.flags.bits();
            if header.declaration != row
                || catalog.definition_at(artifact, header.self_definition)? != owner.self_definition
                || auto_ != (flags & s::TraitFlags::AUTO != 0)
                || unsafe_ != (flags & s::TraitFlags::UNSAFE != 0)
                || constant != (flags & s::TraitFlags::CONST != 0)
            {
                return Err("imported trait changed its actual Self/flags/owner mapping");
            }
            let id = catalog.declaration(owner.self_definition)?;
            catalog.validate_declaration(id, owner.self_definition, DefKind::SelfType)?;
        }
        OwnerKind::ImportedImpl(id) => {
            let definition = catalog.declaration_identity(id, DefKind::SelfType)?;
            let (artifact, row, _) = catalog.declaration_coordinate(id)?;
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

/// Declaration correspondence only. Complete impl links retain separate actual
/// conformance obligations; inherited None is visited and never paid here.
pub(in crate::sema::crates::typed_headers) enum MemberRows<'a> {
    Declared(&'a [crate::support::crates::owner_catalog::DeclarationRef]),
    Implementation(&'a [s::MemberLink]),
}
impl MemberRows<'_> {
    fn declared_len(&self) -> Option<usize> {
        match self {
            Self::Declared(rows) => Some(rows.len()),
            Self::Implementation(_) => None,
        }
    }
    fn require(
        &self,
        catalog: &Catalog,
        provider: &crate::support::crates::CrateIdentity,
        rows: s::ValidatedView<'_>,
        owner: crate::support::crates::owner_catalog::DeclarationRef,
        actual: &[DefId],
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        let mut actual = actual.iter();
        match self {
            Self::Declared(declarations) => {
                for &raw in *declarations {
                    *steps = steps.saturating_add(1);
                    require_member(catalog, provider, rows, owner, raw, actual.next().copied())?;
                }
            }
            Self::Implementation(links) => {
                for link in *links {
                    *steps = steps.saturating_add(1);
                    if let Some(member) = link.trait_member {
                        if member.artifact != rows.policy.provider {
                            return Err("imported member requires its actual external trait owner");
                        }
                        let definition = catalog.definition_at(provider, member.declaration)?;
                        let id = catalog.declaration(definition)?;
                        catalog.validate_declaration(id, definition, DefKind::Function)?;
                    }
                    if let Some(raw) = link.implementation {
                        require_member(
                            catalog,
                            provider,
                            rows,
                            owner,
                            raw,
                            actual.next().copied(),
                        )?;
                    } else {
                        // The checked membership ledger already established the
                        // complete actual default/body link. Authenticate the
                        // current catalog counterpart without inventing a DefId.
                        let member = link
                            .trait_member
                            .ok_or("imported inherited member lost its actual trait declaration")?;
                        let Some(s::HeaderRef::Function(index)) =
                            rows.header(member.declaration)?
                        else {
                            return Err("imported inherited member has no actual callable header");
                        };
                        if rows.functions[index as usize].body == s::FunctionBody::Absent {
                            return Err(
                                "imported required member omitted its actual implementation",
                            );
                        }
                    }
                }
            }
        }
        if actual.next().is_some() {
            return Err("imported associated schema omitted members or full region relations");
        }
        Ok(())
    }
}
fn require_member(
    catalog: &Catalog,
    provider: &crate::support::crates::CrateIdentity,
    rows: s::ValidatedView<'_>,
    owner: crate::support::crates::owner_catalog::DeclarationRef,
    raw: crate::support::crates::owner_catalog::DeclarationRef,
    actual: Option<DefId>,
) -> Result<(), &'static str> {
    let expected = catalog.definition_at(provider, raw)?;
    let actual =
        actual.ok_or("imported associated schema omitted members or full region relations")?;
    let id = catalog.declaration(actual)?;
    catalog.validate_declaration(id, actual, DefKind::Function)?;
    if expected != actual
        || rows
            .declarations
            .get(raw.0 as usize)
            .is_none_or(|member| member.definition.parent != Some(owner))
    {
        return Err("imported associated member changed its exact lexical owner/identity");
    }
    Ok(())
}

pub(super) struct Ordinary<'a>(pub(super) &'a Graph);
impl sealed::Sealed for Ordinary<'_> {}
impl<'a> Mapping<'a> for Ordinary<'a> {
    fn catalog(&self) -> &'a Catalog {
        &self.0.origins
    }
    fn descriptor(&self) -> Result<SourceDescriptor<'a>, &'static str> {
        self.0.descriptor(&self.0.origins)
    }
    fn require_store(&mut self, store: &Store) -> Result<(), &'static str> {
        self.0.require_store(store)
    }
    fn ty(&mut self, root: TypeRef) -> Result<TypeId, &'static str> {
        self.0.ty(root).map_err(|_| "missing imported actual type")
    }
    fn members(
        &self,
        rows: s::ValidatedView<'a>,
        header: s::HeaderRef,
    ) -> Result<MemberRows<'a>, &'static str> {
        match header {
            s::HeaderRef::Trait(index) => Ok(MemberRows::Declared(
                rows.declared_members(rows.traits[index as usize].members)?,
            )),
            s::HeaderRef::Implementation(index) => {
                if rows.descriptor().profile() == s::Profile::Source2024ModuleFeaturePredicates {
                    return Ok(MemberRows::Implementation(rows.implementation_members(
                        rows.implementations[index as usize].members,
                    )?));
                }
                if rows.implementations[index as usize].members.start
                    != rows.implementations[index as usize].members.end
                {
                    return Err(
                        "imported impl members require their wider exact conformance profile",
                    );
                }
                Ok(MemberRows::Declared(&[]))
            }
            s::HeaderRef::Function(_) => {
                Err("imported associated inventory lacks its actual owner header")
            }
        }
    }
    fn region(&mut self, root: s::RegionRef) -> Result<RegionId, &'static str> {
        self.0
            .region(root)
            .map_err(|_| "invalid imported actual region")
    }
}
