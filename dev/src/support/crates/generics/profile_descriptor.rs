//! A descriptor borrows completed source data; it is never callable permission.
mod binding_order;
mod compatibility_profile;
mod safety_profile;
pub(crate) use binding_order::sort as canonical_order;
#[path = "namespace_paths.rs"]
mod callable_paths;
#[path = "column_storage.rs"]
mod columns;
#[path = "verbatim_exports.rs"]
mod exact_exports;
#[path = "ordinary_profile.rs"]
mod functions;
#[path = "implementation_domains.rs"]
mod module_contracts;
#[path = "module_exports.rs"]
mod module_exports;
mod predicate_contracts;
#[path = "supplied_profile.rs"]
mod provided;
#[path = "supplied_contracts.rs"]
mod provided_contracts;
use super::{HeaderIndex, SourceRows};
use crate::support::{
    crates::{
        CrateIdentity,
        generics::Work,
        owner_catalog::{BodyCatalog, DeclarationBody},
    },
    edition::Edition,
    retained,
};
pub(crate) use callable_paths::callable_path;
pub use columns::{Columns, ValidatedView};
pub use module_contracts::{
    BindingSource, ImplementationDomain, ModuleBindingRow, ModuleContractRows, ModuleContractView,
    ModuleNamespace, ModuleRow, ModuleVisibility, TraitDomainRow,
};
pub(crate) use module_exports::{ExportBinding, NamespaceColumns, UnpublishedNamespace};
pub(crate) use provided_contracts::{
    HeaderModifierColumns, HeaderModifierPolicy, HeaderModifierRecord, ProvidedContractRows,
    ProvidedContractView,
};

/// Existing source-mode facade; no source descriptor has been published yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Source2024,
    /// Ordinary function/move bodies; no const trait/default execution profile.
    Source2024Functions,
    /// Ordinary functions with complete version-one diagnostic/advisory facts.
    Source2024FunctionContracts,
    /// Complete unpaid provided-body data; canonical import and proof stay gated.
    Source2024Provided,
    /// Complete numbered source data with namespace, advisories and explicit plain modifiers.
    Source2024ProvidedContracts,
    /// Complete ordinary module namespace, implementation domains and advisory facts.
    Source2024ModuleContracts,
    /// Complete staged regular/const API availability, independently of body proof.
    Source2024ModuleAvailability,
    /// Complete version-three data plus real non-type feature requirements.
    Source2024ModuleFeaturePredicates,
    /// Version-five exact native definitions; nominal transport remains unsupported.
    Source2024ModuleDefinitions,
    /// Version-six stable API compatibility notes preserve ordinary path policy.
    Source2024ModuleCompatibility,
    /// Current unpaid provided rows; auto traits have no implicit Self sizing.
    Source2024ProvidedCurrent,
    /// Version-seven unconditional const/unsafe free callable originals.
    Source2024ModuleCallables,
    /// Version-eight exact source call safety contexts; no implicit unsafe permission.
    Source2024ModuleSafety,
    /// Version-nine From identity and complete ordinary conversion method bodies.
    Source2024ModuleConversions,
}
impl Profile {
    #[must_use]
    pub const fn feature_predicates(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleFeaturePredicates
                | Self::Source2024ModuleDefinitions
                | Self::Source2024ModuleCompatibility
                | Self::Source2024ModuleCallables
                | Self::Source2024ModuleSafety
                | Self::Source2024ModuleConversions
        )
    }
    #[must_use]
    pub const fn module_contracts(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleContracts
                | Self::Source2024ModuleAvailability
                | Self::Source2024ModuleFeaturePredicates
                | Self::Source2024ModuleDefinitions
                | Self::Source2024ModuleCompatibility
                | Self::Source2024ModuleCallables
                | Self::Source2024ModuleSafety
                | Self::Source2024ModuleConversions
        )
    }
    #[must_use]
    pub const fn ordinary_functions(self) -> bool {
        matches!(
            self,
            Self::Source2024Functions | Self::Source2024FunctionContracts
        )
    }
    #[must_use]
    pub const fn availability(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleAvailability
                | Self::Source2024ModuleFeaturePredicates
                | Self::Source2024ModuleDefinitions
                | Self::Source2024ModuleCompatibility
                | Self::Source2024ModuleCallables
                | Self::Source2024ModuleSafety
                | Self::Source2024ModuleConversions
        )
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Budget {
    pub bytes: usize,
    pub peak: usize,
    pub atoms: usize,
    pub names: usize,
}

/// Raw draft rows cannot construct an Interface or grant imported proof.
#[derive(Debug)]
pub struct ValidatedRows {
    owners: BodyCatalog,
    profile: Profile,
    columns: Box<Columns>,
    headers: Box<[Option<super::HeaderRef>]>,
    attributes: Option<Box<super::HeaderAttributeColumns>>,
    provided: Option<Box<provided_contracts::ProvidedContractColumns>>,
    modules: Option<Box<module_contracts::ModuleContractColumns>>,
    availability: Option<Box<super::AvailabilityColumns>>,
    budget: Budget,
}
impl PartialEq for ValidatedRows {
    fn eq(&self, other: &Self) -> bool {
        self.owners == other.owners
            && self.profile == other.profile
            && self.columns == other.columns
            && self.attributes == other.attributes
            && self.provided == other.provided
            && self.modules == other.modules
            && self.availability == other.availability
    }
}
impl Eq for ValidatedRows {}
impl ValidatedRows {
    pub(crate) fn stage_module_availability(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: super::HeaderAttributeColumns,
        modules: ModuleContractRows,
        availability: super::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleAvailability),
            base,
            work,
        )
    }
    pub(crate) fn stage_module_feature_predicates(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: super::HeaderAttributeColumns,
        modules: ModuleContractRows,
        availability: super::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleFeaturePredicates),
            base,
            work,
        )
    }
    pub(crate) fn stage_module_definitions(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: super::HeaderAttributeColumns,
        modules: ModuleContractRows,
        availability: super::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleDefinitions),
            base,
            work,
        )
    }
    fn stage_module_availability_with(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: super::HeaderAttributeColumns,
        modules: ModuleContractRows,
        available: (super::AvailabilityRows, Profile),
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let (availability, profile) = available;
        compatibility_profile::require_profile(&availability, profile, work)?;
        let features = profile.feature_predicates();
        let start = work.steps;
        let bytes = availability.storage_bytes(work)?;
        let mut staged = Self::stage_module_contracts_with(
            owner,
            rows,
            attributes,
            modules,
            base.saturating_add(bytes),
            work,
            profile,
        )?;
        let modules = staged
            .modules
            .as_deref()
            .ok_or("availability lost its actual module columns")?
            .view();
        let available = availability.validate(
            &staged.columns,
            &staged.headers,
            modules.rows(),
            base.saturating_add(staged.budget.bytes),
            work,
        )?;
        if features {
            predicate_contracts::complete(
                &staged.columns,
                &staged.headers,
                available.rows(),
                work,
            )?;
        }
        work.charge(available.rows().policies.len())?;
        staged.budget.names = staged.budget.names.saturating_add(
            available
                .rows()
                .policies
                .iter()
                .map(|policy| {
                    policy.feature.len()
                        + match &policy.level {
                            super::AvailabilityLevel::Stable(_) => 0,
                            super::AvailabilityLevel::StableWithCompatibility {
                                message, ..
                            } => message.len(),
                            super::AvailabilityLevel::Unstable { reason, .. } => {
                                reason.as_ref().map_or(0, String::len)
                            }
                        }
                })
                .sum::<usize>(),
        );
        staged.budget.bytes = staged.budget.bytes.saturating_add(bytes);
        staged.budget.peak = staged.budget.peak.saturating_add(bytes);
        staged.budget.atoms = work.steps.saturating_sub(start);
        staged.availability = Some(Box::new(available));
        staged.profile = profile;
        Ok(staged)
    }
    /// Structural staging only. Public payload activation remains gated until
    /// source serialization, canonical remapping and fresh proof all exist.
    pub(crate) fn stage(
        owner: &CrateIdentity,
        rows: SourceRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_profile(owner, rows, base, work, Profile::Source2024Functions)
    }
    pub(crate) fn stage_provided(
        owner: &CrateIdentity,
        rows: SourceRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_profile(owner, rows, base, work, Profile::Source2024ProvidedCurrent)
    }
    pub(crate) fn stage_legacy_provided(
        owner: &CrateIdentity,
        rows: SourceRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_profile(owner, rows, base, work, Profile::Source2024Provided)
    }
    /// Cost observation cannot alter metadata validation or grant permission.
    pub(crate) fn stage_provided_observed(
        owner: &CrateIdentity,
        rows: SourceRows,
        base: usize,
        work: &mut Work,
        observe: &mut impl FnMut(usize, usize),
    ) -> Result<Self, &'static str> {
        Self::stage_profile_observed(
            owner,
            rows,
            base,
            work,
            Profile::Source2024ProvidedCurrent,
            observe,
        )
    }
    pub(crate) fn stage_contracts(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: super::HeaderAttributeColumns,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let start = work.steps;
        let bytes = attributes.validate(&rows, base, work)?;
        let attribute_steps = work.steps.saturating_sub(start);
        let mut staged = Self::stage_profile(
            owner,
            rows,
            base.saturating_add(bytes),
            work,
            Profile::Source2024FunctionContracts,
        )?;
        staged.budget.bytes = staged.budget.bytes.saturating_add(bytes);
        staged.budget.peak = staged.budget.peak.saturating_add(bytes);
        staged.budget.atoms = staged.budget.atoms.saturating_add(attribute_steps);
        staged.attributes = Some(Box::new(attributes));
        Ok(staged)
    }
    pub(crate) fn stage_provided_contracts(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: super::HeaderAttributeColumns,
        contracts: ProvidedContractRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_complete(
            owner,
            rows,
            base,
            work,
            Profile::Source2024ProvidedContracts,
            &mut |_, _| {},
            Some((attributes, contracts)),
        )
    }
    pub(crate) fn stage_module_contracts(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: super::HeaderAttributeColumns,
        modules: ModuleContractRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_contracts_with(
            owner,
            rows,
            attributes,
            modules,
            base,
            work,
            Profile::Source2024ModuleContracts,
        )
    }
    fn stage_module_contracts_with(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: super::HeaderAttributeColumns,
        modules: ModuleContractRows,
        base: usize,
        work: &mut Work,
        profile: Profile,
    ) -> Result<Self, &'static str> {
        let start = work.steps;
        let features = profile.feature_predicates();
        safety_profile::require_profile(&rows, profile, work)?;
        if !profile.definitions() {
            exact_exports::legacy(&rows, work)?;
        }
        if !features {
            predicate_contracts::legacy(&rows, work)?;
        }
        let attribute_bytes = attributes.validate(&rows, base, work)?;
        let module_bytes = modules.storage_bytes(rows.declarations.len(), work)?;
        let extension = attribute_bytes.saturating_add(module_bytes);
        let extended_base = base.saturating_add(extension);
        let indexed = HeaderIndex::new(&rows, extended_base, work)?;
        if profile.definitions() {
            exact_exports::validate(&rows, &indexed, work)?;
            if profile.conversion_roles() {
                functions::validate_module_conversions(&rows, &indexed, work)?;
            } else if profile.call_safety() {
                functions::validate_module_safety(&rows, &indexed, work)?;
            } else if profile.unconditional_callables() {
                functions::validate_module_callables(&rows, &indexed, work)?;
            } else {
                functions::validate_module_definitions(&rows, &indexed, work)?;
            }
        } else if features {
            functions::validate_module_feature_predicates(&rows, &indexed, work)?;
        } else {
            functions::validate_module_contracts(&rows, &indexed, work)?;
        }
        let (modules, _, module_peak) = module_contracts::ModuleContractColumns::validate(
            &rows,
            &indexed,
            modules,
            indexed.retained_bytes(),
            work,
        )?;
        let domains = if profile.conversion_roles() {
            indexed.conversion_domains(work)?
        } else {
            indexed.domains(work)?
        };
        let inputs = domains.original_inputs(work)?;
        let peak = inputs
            .peak_bytes()
            .max(indexed.retained_bytes().saturating_add(module_peak));
        drop(inputs);
        drop(domains);
        let indexed = indexed.into_header_rows()?;
        let (owners, columns, headers, mut budget) =
            freeze_columns(owner, rows, indexed, extended_base, peak, work)?;
        budget.bytes = budget.bytes.saturating_add(extension);
        budget.peak = budget.peak.saturating_add(extension);
        budget.atoms = work.steps.saturating_sub(start);
        budget.names = budget.names.saturating_add(
            modules
                .view()
                .rows()
                .namespace
                .bindings
                .iter()
                .map(|row| row.name.len())
                .sum::<usize>(),
        );
        Ok(Self {
            owners,
            profile: Profile::Source2024ModuleContracts,
            columns,
            headers,
            attributes: Some(Box::new(attributes)),
            provided: None,
            modules: Some(Box::new(modules)),
            availability: None,
            budget,
        })
    }
    fn stage_profile(
        owner: &CrateIdentity,
        rows: SourceRows,
        base: usize,
        work: &mut Work,
        profile: Profile,
    ) -> Result<Self, &'static str> {
        Self::stage_profile_observed(owner, rows, base, work, profile, &mut |_, _| {})
    }
    fn stage_profile_observed(
        owner: &CrateIdentity,
        rows: SourceRows,
        base: usize,
        work: &mut Work,
        profile: Profile,
        observe: &mut impl FnMut(usize, usize),
    ) -> Result<Self, &'static str> {
        Self::stage_complete(owner, rows, base, work, profile, observe, None)
    }
    fn stage_complete(
        owner: &CrateIdentity,
        rows: SourceRows,
        base: usize,
        work: &mut Work,
        profile: Profile,
        observe: &mut impl FnMut(usize, usize),
        extension: Option<(super::HeaderAttributeColumns, ProvidedContractRows)>,
    ) -> Result<Self, &'static str> {
        let start = work.steps;
        safety_profile::require_profile(&rows, profile, work)?;
        exact_exports::legacy(&rows, work)?;
        predicate_contracts::legacy(&rows, work)?;
        if extension.is_some() != (profile == Profile::Source2024ProvidedContracts) {
            return Err(
                "provided contracts require their complete namespace/modifier/advisory data",
            );
        }
        let (attribute_bytes, provided_bytes) = if let Some((attributes, contracts)) = &extension {
            (
                attributes.validate(&rows, base, work)?,
                contracts.storage_bytes(work)?,
            )
        } else {
            (0, 0)
        };
        let extension_bytes = attribute_bytes.saturating_add(provided_bytes);
        let extended_base = base.saturating_add(extension_bytes);
        let indexed = HeaderIndex::new(&rows, extended_base, work)?;
        predicate_contracts::legacy_calls(&rows, &indexed, work)?;
        let numbering_peak = match profile {
            Profile::Source2024Functions | Profile::Source2024FunctionContracts => {
                functions::validate(&rows, &indexed, work)?;
                0
            }
            Profile::Source2024Provided
            | Profile::Source2024ProvidedContracts
            | Profile::Source2024ProvidedCurrent => provided::validate(&rows, &indexed, work)?,
            Profile::Source2024 => return Err("unpublished source facade has no data profile"),
            Profile::Source2024ModuleContracts
            | Profile::Source2024ModuleAvailability
            | Profile::Source2024ModuleFeaturePredicates
            | Profile::Source2024ModuleDefinitions
            | Profile::Source2024ModuleCompatibility
            | Profile::Source2024ModuleCallables
            | Profile::Source2024ModuleSafety
            | Profile::Source2024ModuleConversions => {
                return Err(
                    "module contracts require their complete namespace/domain/attribute columns",
                );
            }
        };
        let (attributes, provided, extension_peak) =
            complete_extension(extension, &rows, &indexed, provided_bytes, work)?;
        let domains = indexed.domains(work)?;
        let plan = domains.original_inputs(work)?;
        let previous_peak = plan.peak_bytes().max(numbering_peak).max(extension_peak);
        drop(plan);
        drop(domains);
        let indexed = indexed.into_header_rows()?;
        let (owners, columns, headers, mut budget) = freeze_columns_observed(
            owner,
            rows,
            indexed,
            extended_base,
            previous_peak,
            work,
            observe,
        )?;
        budget.bytes = budget.bytes.saturating_add(extension_bytes);
        budget.peak = budget.peak.saturating_add(extension_bytes);
        if let Some(contracts) = &provided {
            work.charge(contracts.view().rows().namespace.bindings.len())?;
            budget.names = budget.names.saturating_add(
                contracts
                    .view()
                    .rows()
                    .namespace
                    .bindings
                    .iter()
                    .map(|row| row.name.len())
                    .sum::<usize>(),
            );
        }
        budget.atoms = work.steps.saturating_sub(start);
        Ok(Self {
            owners,
            profile,
            columns,
            headers,
            attributes,
            provided,
            modules: None,
            availability: None,
            budget,
        })
    }
    #[must_use]
    pub fn descriptor(&self) -> Descriptor<'_> {
        Descriptor { source: self }
    }
    pub(crate) fn owners(&self) -> &BodyCatalog {
        &self.owners
    }
    pub(crate) const fn budget(&self) -> Budget {
        self.budget
    }
    pub(crate) const fn view(&self) -> ValidatedView<'_> {
        ValidatedView { source: self }
    }
}
type FrozenExtension = (
    Option<Box<super::HeaderAttributeColumns>>,
    Option<Box<provided_contracts::ProvidedContractColumns>>,
    usize,
);
fn complete_extension(
    extension: Option<(super::HeaderAttributeColumns, ProvidedContractRows)>,
    rows: &SourceRows,
    indexed: &HeaderIndex<'_>,
    provided_bytes: usize,
    work: &mut Work,
) -> Result<FrozenExtension, &'static str> {
    let Some((attributes, contracts)) = extension else {
        return Ok((None, None, 0));
    };
    let base = indexed.retained_bytes().saturating_sub(provided_bytes);
    let (contracts, peak) = contracts.validate(rows, indexed, &attributes, base, work)?;
    Ok((
        Some(Box::new(attributes)),
        Some(Box::new(contracts)),
        base.saturating_add(peak),
    ))
}

type FrozenColumns = (
    BodyCatalog,
    Box<Columns>,
    Box<[Option<super::HeaderRef>]>,
    Budget,
);
fn freeze_columns(
    owner: &CrateIdentity,
    rows: SourceRows,
    indexed: (Vec<Option<super::HeaderRef>>, usize),
    base: usize,
    previous_peak: usize,
    work: &mut Work,
) -> Result<FrozenColumns, &'static str> {
    freeze_columns_observed(
        owner,
        rows,
        indexed,
        base,
        previous_peak,
        work,
        &mut |_, _| {},
    )
}
fn freeze_columns_observed(
    owner: &CrateIdentity,
    rows: SourceRows,
    indexed: (Vec<Option<super::HeaderRef>>, usize),
    base: usize,
    previous_peak: usize,
    work: &mut Work,
    observe: &mut impl FnMut(usize, usize),
) -> Result<FrozenColumns, &'static str> {
    let (headers, raw_bytes) = indexed;
    let mut budget = freeze_budget(
        &rows,
        &headers,
        raw_bytes,
        base,
        previous_peak,
        work,
        observe,
    )?;
    let mut kinds = Vec::with_capacity(rows.declarations.len());
    let mut names = rows
        .exact_callables
        .iter()
        .map(|row| row.symbol.len())
        .sum::<usize>();
    for declaration in &rows.declarations {
        work.charge(1)?;
        names = names.saturating_add(declaration.name.as_ref().map_or(0, String::len));
        if declaration.definition.parent.is_none()
            && declaration.name.as_deref() != Some(owner.name())
        {
            return Err("source module root differs from its actual artifact identity");
        }
        kinds.push(
            if declaration.definition.kind == super::DefinitionKind::Function {
                DeclarationBody::Function
            } else {
                DeclarationBody::Other
            },
        );
    }
    work.charge(rows.body_owners.len().saturating_mul(2))?;
    let (body_owners, columns) = Columns::take(rows);
    let owners = BodyCatalog::new(owner.clone(), &kinds, body_owners)?;
    budget.names = names;
    Ok((
        owners,
        Box::new(columns),
        headers.into_boxed_slice(),
        budget,
    ))
}
fn freeze_budget(
    rows: &SourceRows,
    headers: &Vec<Option<super::HeaderRef>>,
    raw_bytes: usize,
    base: usize,
    previous_peak: usize,
    work: &mut Work,
    observe: &mut impl FnMut(usize, usize),
) -> Result<Budget, &'static str> {
    work.charge(1)?;
    let owner_bytes = rows
        .body_owners
        .len()
        .saturating_mul(size_of::<crate::support::crates::owner_catalog::BodyOwner>());
    let raw_header = size_of::<SourceRows>().saturating_add(retained::vector(&rows.body_owners));
    let payload = raw_bytes
        .checked_sub(base)
        .and_then(|bytes| bytes.checked_sub(raw_header))
        .ok_or("source frozen column accounting underflow")?;
    let bytes = payload
        .saturating_add(size_of::<ValidatedRows>())
        .saturating_add(size_of::<Columns>())
        .saturating_add(owner_bytes)
        .saturating_add(2 * size_of::<usize>())
        .saturating_add(
            headers
                .len()
                .saturating_mul(size_of::<Option<super::HeaderRef>>()),
        );
    // Kind scratch and both old/new row, header and Box storage coexist during
    // ownership transfer; reserve the complete envelope before those allocations.
    let transfer = raw_bytes
        .saturating_add(retained::vector(headers))
        .saturating_add(
            rows.declarations
                .len()
                .saturating_mul(4 * size_of::<DeclarationBody>()),
        )
        .saturating_add(size_of::<ValidatedRows>() + size_of::<Columns>())
        .saturating_add(owner_bytes)
        .saturating_add(2 * size_of::<usize>())
        .saturating_add(
            headers
                .len()
                .saturating_mul(size_of::<Option<super::HeaderRef>>()),
        );
    let combined_peak = previous_peak.max(transfer).max(base.saturating_add(bytes));
    // Report the owning preflight before either rejection or allocation. The
    // observer cannot change these bounds or the mandatory validation result.
    observe(base.saturating_add(bytes), combined_peak);
    if combined_peak > crate::support::crates::MAX_BYTES {
        return Err("source freeze exceeds combined retained and transfer budget");
    }
    Ok(Budget {
        bytes,
        peak: combined_peak.saturating_sub(base),
        atoms: 0,
        names: 0,
    })
}

#[derive(Clone, Copy)]
pub struct Descriptor<'a> {
    source: &'a ValidatedRows,
}
impl<'a> Descriptor<'a> {
    pub(crate) const fn rows(self) -> ValidatedView<'a> {
        self.source.view()
    }
    #[must_use]
    pub fn identity(&self) -> &CrateIdentity {
        self.source.owners.owner()
    }
    #[must_use]
    pub fn provider(&self) -> &CrateIdentity {
        self.source.owners.owner()
    }
    #[must_use]
    pub fn edition(&self) -> Edition {
        self.source.columns.policy.edition
    }
    /// # Errors
    /// Rejects a different profile or incomplete module/domain completion.
    pub fn module_contracts(self) -> Result<ModuleContractView<'a>, &'static str> {
        if !self.source.profile.module_contracts() {
            return Err("descriptor has no version-two module contracts");
        }
        self.source
            .modules
            .as_deref()
            .map(module_contracts::ModuleContractColumns::view)
            .ok_or("version-two module contract columns are missing")
    }
    /// # Errors
    /// Older profiles have unknown availability, never checked absence.
    pub fn availability(self) -> Result<&'a super::AvailabilityRows, &'static str> {
        if !self.source.profile.availability() {
            return Err("descriptor has no complete source API availability");
        }
        self.source
            .availability
            .as_deref()
            .map(super::AvailabilityColumns::rows)
            .ok_or("source availability completion columns are missing")
    }
    pub(crate) fn provided_contracts(self) -> Result<ProvidedContractView<'a>, &'static str> {
        if self.source.profile != Profile::Source2024ProvidedContracts {
            return Err("descriptor has no complete provided contracts");
        }
        self.source
            .provided
            .as_deref()
            .map(provided_contracts::ProvidedContractColumns::view)
            .ok_or("provided namespace/modifier completion is missing")
    }
    #[must_use]
    pub const fn profile(&self) -> Profile {
        self.source.profile
    }
}
