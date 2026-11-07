//! Version-six and its explicit successors retain stable compatibility messages.
use super::{Profile, SourceRows, ValidatedRows, Work};
use crate::support::crates::{CrateIdentity, generics::source as s};

impl Profile {
    #[must_use]
    pub const fn compiler_operations(self) -> bool {
        matches!(self, Self::Source2024ModuleCompilerOperations)
    }
    #[must_use]
    pub const fn nominals(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleNominals
                | Self::Source2024ModuleCompilerOperations
                | Self::Source2024ModuleBranches
                | Self::Source2024ModuleBodies
        )
    }
    /// Versions twelve and thirteen admit two-way Branch and checked Panic CFG exits.
    #[must_use]
    pub const fn branches(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleBranches | Self::Source2024ModuleBodies
        )
    }
    /// Only version thirteen encodes body operations as stable generic IR nodes.
    #[must_use]
    pub const fn body_nodes(self) -> bool {
        matches!(self, Self::Source2024ModuleBodies)
    }
    /// The complete ordinary conversion role/header protocol is explicit v9 data.
    #[must_use]
    pub const fn conversion_roles(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleConversions
                | Self::Source2024ModuleNominals
                | Self::Source2024ModuleCompilerOperations
                | Self::Source2024ModuleBranches
                | Self::Source2024ModuleBodies
        )
    }
    #[must_use]
    pub const fn unconditional_callables(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleCallables
                | Self::Source2024ModuleSafety
                | Self::Source2024ModuleConversions
                | Self::Source2024ModuleNominals
                | Self::Source2024ModuleCompilerOperations
                | Self::Source2024ModuleBranches
                | Self::Source2024ModuleBodies
        )
    }
    #[must_use]
    pub const fn call_safety(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleSafety
                | Self::Source2024ModuleConversions
                | Self::Source2024ModuleNominals
                | Self::Source2024ModuleCompilerOperations
                | Self::Source2024ModuleBranches
                | Self::Source2024ModuleBodies
        )
    }
    #[must_use]
    pub const fn definitions(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleDefinitions
                | Self::Source2024ModuleCompatibility
                | Self::Source2024ModuleCallables
                | Self::Source2024ModuleSafety
                | Self::Source2024ModuleConversions
                | Self::Source2024ModuleNominals
                | Self::Source2024ModuleCompilerOperations
                | Self::Source2024ModuleBranches
                | Self::Source2024ModuleBodies
        )
    }
    #[must_use]
    pub const fn module_compatibility(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleCompatibility
                | Self::Source2024ModuleCallables
                | Self::Source2024ModuleSafety
                | Self::Source2024ModuleConversions
                | Self::Source2024ModuleNominals
                | Self::Source2024ModuleCompilerOperations
                | Self::Source2024ModuleBranches
                | Self::Source2024ModuleBodies
        )
    }
}

pub(super) fn require_profile(
    rows: &s::AvailabilityRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    if !profile.module_compatibility() {
        for policy in &rows.policies {
            work.charge(1)?;
            if matches!(
                policy.level,
                s::AvailabilityLevel::StableWithCompatibility { .. }
            ) {
                return Err("stable module compatibility requires the version-six profile");
            }
        }
    }
    Ok(())
}

impl ValidatedRows {
    pub(crate) fn stage_module_conversions(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: s::HeaderAttributeColumns,
        modules: s::ModuleContractRows,
        availability: s::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleConversions),
            base,
            work,
        )
    }

    pub(crate) fn stage_compiler_operations(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: s::HeaderAttributeColumns,
        modules: s::ModuleContractRows,
        availability: s::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleCompilerOperations),
            base,
            work,
        )
    }
    pub(crate) fn stage_module_nominals(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: s::HeaderAttributeColumns,
        modules: s::ModuleContractRows,
        availability: s::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleNominals),
            base,
            work,
        )
    }
    pub(crate) fn stage_module_branches(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: s::HeaderAttributeColumns,
        modules: s::ModuleContractRows,
        availability: s::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleBranches),
            base,
            work,
        )
    }

    pub(crate) fn stage_module_bodies(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: s::HeaderAttributeColumns,
        modules: s::ModuleContractRows,
        availability: s::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleBodies),
            base,
            work,
        )
    }

    pub(crate) fn stage_module_safety(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: s::HeaderAttributeColumns,
        modules: s::ModuleContractRows,
        availability: s::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleSafety),
            base,
            work,
        )
    }
    pub(crate) fn stage_module_callables(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: s::HeaderAttributeColumns,
        modules: s::ModuleContractRows,
        availability: s::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleCallables),
            base,
            work,
        )
    }

    pub(crate) fn stage_module_compatibility(
        owner: &CrateIdentity,
        rows: SourceRows,
        attributes: s::HeaderAttributeColumns,
        modules: s::ModuleContractRows,
        availability: s::AvailabilityRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        Self::stage_module_availability_with(
            owner,
            rows,
            attributes,
            modules,
            (availability, Profile::Source2024ModuleCompatibility),
            base,
            work,
        )
    }
}
