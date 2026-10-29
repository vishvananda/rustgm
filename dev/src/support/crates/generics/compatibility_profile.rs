//! Version-six and its explicit successors retain stable compatibility messages.
use super::{Profile, SourceRows, ValidatedRows, Work};
use crate::support::crates::{CrateIdentity, generics::source as s};

impl Profile {
    #[must_use]
    pub const fn unconditional_callables(self) -> bool {
        matches!(self, Self::Source2024ModuleCallables)
    }
    #[must_use]
    pub const fn definitions(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleDefinitions
                | Self::Source2024ModuleCompatibility
                | Self::Source2024ModuleCallables
        )
    }
    #[must_use]
    pub const fn module_compatibility(self) -> bool {
        matches!(
            self,
            Self::Source2024ModuleCompatibility | Self::Source2024ModuleCallables
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
