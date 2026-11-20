//! Authenticate the installed trait member links before re-proving conformance.
use super::{Graph, s};
use crate::{
    resolve::{
        body_origin::Catalog,
        model::{DefId, DefKind},
    },
    sema::{
        associated::{Inventory, Owner},
        model::Declarations,
    },
    support::crates::{CrateIdentity, owner_catalog::DeclarationRef},
    types::{Store, arena::Identity, parameters::predicates::Predicate},
};
use std::rc::Rc;
type OptionalLink = (DefId, Option<DefId>);

#[derive(Default)]
pub(in crate::sema) struct ImplMembers {
    pub methods: Vec<OptionalLink>,
    pub types: Vec<(DefId, DefId)>,
    pub constants: Vec<OptionalLink>,
}

pub(in crate::sema) struct ImportedMemberPlan {
    arena: Identity,
    catalog: Rc<Catalog>,
    owner: DefId,
    predicate: Predicate,
    pub members: ImplMembers,
}
impl ImportedMemberPlan {
    pub(in crate::sema) fn require(
        &self,
        facts: &Declarations,
        owner: DefId,
        predicate: Predicate,
    ) -> Result<(), &'static str> {
        if self.owner != owner
            || self.predicate != predicate
            || !facts.types.has_identity(&self.arena)
            || facts
                .crates
                .as_deref()
                .and_then(|linkage| linkage.origin_catalog())
                .is_none_or(|catalog| !std::ptr::eq(catalog, self.catalog.as_ref()))
        {
            return Err("imported member plan changed its actual arena/catalog/impl");
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(super) enum TraitMember {
    Method(s::FunctionBody),
    Type,
    Constant(s::ConstantBody),
}
impl TraitMember {
    pub(super) fn read(
        rows: s::ValidatedView<'_>,
        declaration: DeclarationRef,
    ) -> Result<Self, &'static str> {
        match rows.header(declaration)? {
            Some(s::HeaderRef::Function(index)) => {
                Ok(Self::Method(rows.functions[index as usize].body))
            }
            Some(s::HeaderRef::AssociatedType(_)) => Ok(Self::Type),
            Some(s::HeaderRef::Constant(index))
                if rows.descriptor().profile().declaration_policies() =>
            {
                Ok(Self::Constant(rows.constants()[index as usize].body))
            }
            _ => Err("imported link has no actual trait member header"),
        }
    }
    pub(super) const fn kind(self) -> DefKind {
        match self {
            Self::Method(_) => DefKind::Function,
            Self::Type => DefKind::TypeAlias,
            Self::Constant(_) => DefKind::Const,
        }
    }
    pub(super) const fn has_default(self) -> bool {
        matches!(
            self,
            Self::Method(s::FunctionBody::Original(_) | s::FunctionBody::CompilerOperation(_))
                | Self::Constant(s::ConstantBody::Original(_))
        )
    }
    pub(super) fn definition(
        self,
        catalog: &Catalog,
        provider: &CrateIdentity,
        row: DeclarationRef,
    ) -> Result<DefId, &'static str> {
        let definition = catalog.definition_at(provider, row)?;
        let id = catalog.declaration(definition)?;
        catalog.validate_declaration(id, definition, self.kind())?;
        Ok(definition)
    }
}
impl Graph {
    pub(in crate::sema::crates::typed_headers) fn implementation_member_plan(
        &self,
        store: &Store,
        associated: &Inventory,
        owner: Owner,
        predicate: Predicate,
        budget: &mut super::super::Headers,
    ) -> Result<Option<ImportedMemberPlan>, super::super::Error> {
        use super::super::error;
        self.require_store(store).map_err(error)?;
        self.owner(owner).map_err(error)?;
        let descriptor = self.descriptor(&self.origins).map_err(error)?;
        if !descriptor.profile().feature_predicates() {
            return Ok(None);
        }
        let id = self
            .origins
            .declaration(owner.self_definition)
            .map_err(error)?;
        let (_, row, _) = self.origins.declaration_coordinate(id).map_err(error)?;
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Implementation(index)) = rows.header(row).map_err(error)? else {
            return Err(error(
                "v4 method plan lost its actual implementation header",
            ));
        };
        let raw = &rows.implementations[index as usize];
        if raw.members.start == raw.members.end {
            return Ok(None);
        }
        self.implementation(store, owner, predicate)
            .map_err(error)?;
        let schema = associated
            .schema(owner.self_definition)
            .ok_or_else(|| error("v4 method plan lost its installed Self schema"))?;
        let members = associated
            .owner(owner.self_definition)
            .and_then(|id| associated.owner_members(id))
            .ok_or_else(|| error("v4 method plan lost its actual member inventory"))?;
        let mut steps = 1;
        let result = self.associated_row(store, owner, schema, members, &mut steps);
        budget.charge(steps)?;
        result.map_err(error)?;
        let members = self.remap_member_links(rows, descriptor.identity(), raw.members, budget)?;
        Ok(Some(ImportedMemberPlan {
            arena: self.store.clone(),
            catalog: Rc::clone(&self.origins),
            owner: owner.self_definition,
            predicate,
            members,
        }))
    }

    fn remap_member_links(
        &self,
        rows: s::ValidatedView<'_>,
        provider: &CrateIdentity,
        range: crate::support::crates::generics::Range,
        budget: &mut super::super::Headers,
    ) -> Result<ImplMembers, super::super::Error> {
        use super::super::error;
        let links = range.slice(&rows.member_links).map_err(error)?;
        budget.reserve(
            links
                .len()
                .saturating_mul(4 * size_of::<OptionalLink>())
                .saturating_add(size_of::<ImportedMemberPlan>()),
        )?;
        let mut members = ImplMembers::default();
        for link in links {
            budget.charge(3)?;
            let member = link
                .trait_member
                .filter(|member| member.artifact == s::ArtifactRef(0))
                .ok_or_else(|| error("v4 method requires its genuine trait member"))?;
            let kind = TraitMember::read(rows, member.declaration).map_err(error)?;
            let member = kind
                .definition(&self.origins, provider, member.declaration)
                .map_err(error)?;
            let actual = link
                .implementation
                .map(|row| kind.definition(&self.origins, provider, row))
                .transpose()
                .map_err(error)?;
            match (kind, actual) {
                (TraitMember::Constant(_), actual) if actual.is_some() || kind.has_default() => {
                    members.constants.push((member, actual));
                }
                (TraitMember::Type, Some(actual)) => members.types.push((member, actual)),
                (TraitMember::Method(_), Some(actual)) => {
                    members.methods.push((member, Some(actual)));
                }
                _ => {
                    return Err(error(
                        "v4 default method requires its wider body/conformance profile",
                    ));
                }
            }
        }
        Ok(members)
    }
}
