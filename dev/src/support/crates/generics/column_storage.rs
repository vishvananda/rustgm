//! Frozen source columns own their payload once; `BodyCatalog` owns body rows.
use super::super::{
    Argument, Binder, BodyInputs, BodyUnit, Clause, Declaration, Formal, FunctionApplication,
    FunctionHeader, ImplHeader, MemberLink, Original, ParameterScope, ParameterSlot, Policy,
    Region, RegionBound, RoleBinding, SlotRef, SourceRows, TraitApplication, TraitHeader, Type,
};
use crate::support::crates::generics::TypeRef;
use crate::support::crates::owner_catalog::{BodyOwner, DeclarationRef};

/// Owned immutable-payload columns, exposed only through a checked borrowed view.
/// `SourceRows` remains the separate mutable staging/decode representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Columns {
    pub policy: Policy,
    pub declarations: Vec<Declaration>,
    pub slots: Vec<ParameterSlot>,
    pub scopes: Vec<ParameterScope>,
    pub binders: Vec<Binder>,
    pub types: Vec<Type>,
    pub roots: Vec<TypeRef>,
    pub regions: Vec<Region>,
    pub arguments: Vec<Argument>,
    pub clauses: Vec<Clause>,
    pub clause_indices: Vec<u32>,
    pub region_bounds: Vec<RegionBound>,
    pub region_slots: Vec<SlotRef>,
    pub trait_applications: Vec<TraitApplication>,
    pub function_applications: Vec<FunctionApplication>,
    pub traits: Vec<TraitHeader>,
    pub implementations: Vec<ImplHeader>,
    pub functions: Vec<FunctionHeader>,
    pub formals: Vec<Formal>,
    pub members: Vec<DeclarationRef>,
    pub member_links: Vec<MemberLink>,
    pub roles: Vec<RoleBinding>,
    pub originals: Vec<Original>,
    pub body_inputs: Vec<BodyInputs>,
    pub body_units: Vec<BodyUnit>,
}
impl Columns {
    pub(super) fn take(rows: SourceRows) -> (Vec<BodyOwner>, Self) {
        let SourceRows {
            body_owners,
            policy,
            declarations,
            slots,
            scopes,
            binders,
            types,
            roots,
            regions,
            arguments,
            clauses,
            clause_indices,
            region_bounds,
            region_slots,
            trait_applications,
            function_applications,
            traits,
            implementations,
            functions,
            formals,
            members,
            member_links,
            roles,
            originals,
            body_inputs,
            body_units,
        } = rows;
        (
            body_owners,
            Self {
                policy,
                declarations,
                slots,
                scopes,
                binders,
                types,
                roots,
                regions,
                arguments,
                clauses,
                clause_indices,
                region_bounds,
                region_slots,
                trait_applications,
                function_applications,
                traits,
                implementations,
                functions,
                formals,
                members,
                member_links,
                roles,
                originals,
                body_inputs,
                body_units,
            },
        )
    }
}

/// This view grants no body/caller/const proof and cannot mutate frozen data.
#[derive(Clone, Copy)]
pub struct ValidatedView<'a> {
    pub(super) source: &'a super::ValidatedRows,
}
impl std::ops::Deref for ValidatedView<'_> {
    type Target = Columns;
    fn deref(&self) -> &Self::Target {
        &self.source.columns
    }
}
impl ValidatedView<'_> {
    #[must_use]
    pub fn body_owners(&self) -> &[BodyOwner] {
        self.source.owners.rows()
    }
    /// # Errors
    /// Rejects a reference outside the same immutable declaration catalog.
    pub fn header(
        &self,
        declaration: DeclarationRef,
    ) -> Result<Option<super::super::HeaderRef>, &'static str> {
        self.source
            .headers
            .get(declaration.0 as usize)
            .copied()
            .ok_or("source declaration header is outside the completed ledger")
    }
}
