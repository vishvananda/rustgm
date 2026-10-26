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
impl<'a> ValidatedView<'a> {
    #[must_use]
    pub fn descriptor(self) -> super::Descriptor<'a> {
        self.source.descriptor()
    }
    /// # Errors
    /// Rejects an absent or different module contract profile.
    pub fn module_contracts(
        self,
    ) -> Result<super::module_contracts::ModuleContractView<'a>, &'static str> {
        self.source.descriptor().module_contracts()
    }
    #[must_use]
    pub fn header_attributes(self) -> Option<&'a super::super::HeaderAttributeColumns> {
        self.source.attributes.as_deref()
    }
    /// # Errors
    /// Rejects a missing/mismatched row in a completed version-one column.
    pub fn header_contract(
        self,
        declaration: DeclarationRef,
    ) -> Result<Option<&'a super::super::HeaderContract>, &'static str> {
        let Some(attributes) = self.header_attributes() else {
            return Ok(None);
        };
        let contract = match self.header(declaration)? {
            Some(super::super::HeaderRef::Function(index)) => {
                attributes.functions.get(index as usize)
            }
            Some(super::super::HeaderRef::Trait(index)) => attributes.traits.get(index as usize),
            _ => return Err("source attribute query does not name an admitted header"),
        }
        .ok_or("source header contract completion row is missing")?;
        if contract.declaration != declaration {
            return Err("source header contract changed its actual declaration");
        }
        Ok(Some(contract))
    }
    /// Only the actual frozen Provided seal may reuse its checked unit inverse.
    /// The live base includes these retained rows, rather than paying them twice.
    pub(crate) fn allocation_ordinals(
        self,
        body: super::super::BodyRef,
        live_base: usize,
        extra: usize,
        work: &mut super::super::super::Work,
    ) -> Result<super::super::AllocationOrdinals<'a>, &'static str> {
        work.charge(1)?;
        if self.source.profile != super::Profile::Source2024Provided
            || live_base < self.source.budget.bytes
        {
            return Err("frozen allocation inverse requires its actual Provided live owner");
        }
        let unit = self
            .source
            .columns
            .body_units
            .get(body.0 as usize)
            .filter(|unit| unit.body == body)
            .ok_or("frozen allocation inverse lost its actual body unit")?;
        let original = self
            .source
            .columns
            .originals
            .get(body.0 as usize)
            .filter(|original| original.body == body)
            .ok_or("frozen allocation inverse lost its actual original")?;
        if unit.allocation_order.len() != original.variables.len() {
            return Err("frozen allocation inverse changed its complete allocation domain");
        }
        super::super::validation::frozen_ordinals(unit, live_base, extra, work)
    }
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
