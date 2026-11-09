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
    /// Cold groups and the sparse original index; `None` when all are absent.
    cold: Option<Box<super::super::ColdColumns>>,
    pub associated_types: Vec<super::super::AssociatedTypeHeader>,
    pub projections: Vec<super::super::ProjectionRecipe>,
    pub opaque_futures: Vec<super::super::OpaqueFuture>,
    pub compiler_operations: Vec<super::super::CompilerOperation>,
    pub nominals: Vec<super::super::NominalHeader>,
    pub fields: Vec<super::super::NominalField>,
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
    pub exact_callables: Vec<super::super::ExactCallableExport>,
}
impl Columns {
    pub(crate) fn original_index(
        &self,
        body: super::super::BodyRef,
    ) -> Result<usize, &'static str> {
        super::super::bodymap::original(self.cold().bodies.as_deref(), body)
    }
    #[must_use]
    pub fn cold(&self) -> &super::super::ColdColumns {
        super::super::ColdColumns::of(self.cold.as_deref())
    }
    /// The raw-row form of the cold groups: without the validated sparse index.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn raw_cold(&self) -> Option<Box<super::super::ColdColumns>> {
        let cold = self.cold();
        (!cold.associated_definitions.is_empty()
            || !cold.enums.is_empty()
            || !cold.pattern_ranges.is_empty()
            || !cold.diagnostics.is_empty()
            || !cold.consts.is_empty()
            || !cold.array_lengths.is_empty()
            || !cold.constants.is_empty())
        .then(|| {
            Box::new(super::super::ColdColumns {
                associated_definitions: cold.associated_definitions.clone(),
                enums: cold.enums.clone(),
                pattern_ranges: cold.pattern_ranges.clone(),
                diagnostics: cold.diagnostics.clone(),
                consts: cold.consts.clone(),
                array_lengths: cold.array_lengths.clone(),
                constants: cold.constants.clone(),
                bodies: None,
            })
        })
    }
    #[must_use]
    pub fn enums(&self) -> &super::super::EnumColumns {
        &self.cold().enums
    }
    #[must_use]
    pub fn associated_definitions(&self) -> &[super::super::AssociatedDefinition] {
        &self.cold().associated_definitions
    }
    #[must_use]
    pub fn pattern_ranges(&self) -> &[super::super::PatternRange] {
        &self.cold().pattern_ranges
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[super::super::DiagnosticBinding] {
        &self.cold().diagnostics
    }
    #[must_use]
    pub fn consts(&self) -> &[super::super::ConstTerm] {
        &self.cold().consts
    }
    #[must_use]
    pub fn array_lengths(&self) -> &[super::super::ConstTerm] {
        &self.cold().array_lengths
    }
    #[must_use]
    pub fn constants(&self) -> &[super::super::ConstantHeader] {
        &self.cold().constants
    }
    /// Each constant declaration with its header row, in declaration order.
    pub fn constant_rows(
        &self,
    ) -> impl Iterator<Item = (DeclarationRef, &super::super::ConstantHeader)> {
        super::super::free_constants::declared(&self.declarations, self.constants())
    }
    pub(crate) fn original(&self, body: super::super::BodyRef) -> Result<&Original, &'static str> {
        self.originals
            .get(self.original_index(body)?)
            .filter(|row| row.body == body)
            .ok_or("source original has a foreign body owner")
    }
    pub(crate) fn body_input(
        &self,
        body: super::super::BodyRef,
    ) -> Result<&BodyInputs, &'static str> {
        self.body_inputs
            .get(self.original_index(body)?)
            .filter(|row| row.body == body)
            .ok_or("source body inputs have a foreign body owner")
    }

    pub(super) fn take(
        rows: SourceRows,
        bodies: Option<std::sync::Arc<super::super::bodymap::Lookup>>,
    ) -> (Vec<BodyOwner>, Self) {
        let SourceRows {
            associated_types,
            cold,
            projections,
            opaque_futures,
            compiler_operations,
            nominals,
            fields,
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
            exact_callables,
        } = rows;
        (
            body_owners,
            Self {
                cold: frozen_cold(cold, bodies),
                associated_types,
                projections,
                opaque_futures,
                compiler_operations,
                nominals,
                fields,
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
                exact_callables,
            },
        )
    }
}

/// The sparse index joins the raw cold groups; only a packet with compiler
/// operations and no cold group allocates here, which staging prepays.
fn frozen_cold(
    cold: Option<Box<super::super::ColdColumns>>,
    bodies: Option<std::sync::Arc<super::super::bodymap::Lookup>>,
) -> Option<Box<super::super::ColdColumns>> {
    if cold.is_none() && bodies.is_none() {
        return None;
    }
    let mut cold = cold.unwrap_or_default();
    cold.bodies = bodies;
    Some(cold)
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
    pub(crate) fn exact_callable(
        self,
        declaration: DeclarationRef,
    ) -> Option<&'a super::super::ExactCallableExport> {
        let rows = &self.source.columns.exact_callables;
        rows.binary_search_by_key(&declaration.0, |row| row.declaration.0)
            .ok()
            .map(move |index| &rows[index])
    }
    /// # Errors
    /// Older source profiles retain unknown availability.
    pub fn availability(self) -> Result<&'a super::super::AvailabilityRows, &'static str> {
        self.source.descriptor().availability()
    }
    pub(crate) fn declared_members(
        self,
        range: crate::support::crates::generics::Range,
    ) -> Result<&'a [DeclarationRef], &'static str> {
        range.slice(&self.source.columns.members)
    }
    pub(crate) fn implementation_members(
        self,
        range: crate::support::crates::generics::Range,
    ) -> Result<&'a [MemberLink], &'static str> {
        range.slice(&self.source.columns.member_links)
    }
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
    pub(crate) fn provided_contracts(
        self,
    ) -> Result<super::ProvidedContractView<'a>, &'static str> {
        self.source.descriptor().provided_contracts()
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
            Some(super::super::HeaderRef::Nominal(index)) => self
                .source
                .columns
                .nominals
                .get(index as usize)
                .map(|header| &header.contract),
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
        if !matches!(
            self.source.profile,
            super::Profile::Source2024Provided
                | super::Profile::Source2024ProvidedContracts
                | super::Profile::Source2024ProvidedCurrent
        ) || live_base < self.source.budget.bytes
        {
            return Err("frozen allocation inverse requires its actual Provided live owner");
        }
        self.allocation_inverse(body, live_base, extra, work)
    }
    /// Version-nine ordinary imports require a separate actual live owner.
    /// This returns only checked ordinals; semantic regions and body proof are absent.
    pub(crate) fn ordinary_allocation_ordinals(
        self,
        body: super::super::BodyRef,
        live_base: usize,
        extra: usize,
        work: &mut super::super::super::Work,
    ) -> Result<super::super::AllocationOrdinals<'a>, &'static str> {
        work.charge(1)?;
        if !self.source.profile.conversion_roles() || live_base < self.source.budget.bytes {
            return Err("ordinary allocation inverse requires its actual version-nine live owner");
        }
        self.allocation_inverse(body, live_base, extra, work)
    }
    fn allocation_inverse(
        self,
        body: super::super::BodyRef,
        live_base: usize,
        extra: usize,
        work: &mut super::super::super::Work,
    ) -> Result<super::super::AllocationOrdinals<'a>, &'static str> {
        let unit = self
            .source
            .columns
            .body_units
            .get(self.source.columns.original_index(body)?)
            .filter(|unit| unit.body == body)
            .ok_or("frozen allocation inverse lost its actual body unit")?;
        let original = self
            .source
            .columns
            .originals
            .get(self.source.columns.original_index(body)?)
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
    /// Version thirteen: whether a declaration is a record variant, whose one
    /// row names its type-namespace definition and which has no value.
    /// # Errors
    /// Rejects a reference outside the same immutable declaration catalog.
    pub fn record_variant(&self, declaration: DeclarationRef) -> Result<bool, &'static str> {
        Ok(match self.header(declaration)? {
            Some(super::super::HeaderRef::Variant(index)) => self
                .enums()
                .variants
                .get(index as usize)
                .is_some_and(|variant| variant.shape == super::super::VariantShape::Record),
            _ => false,
        })
    }
}
