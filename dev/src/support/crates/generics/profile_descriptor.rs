//! A descriptor borrows completed source data; it is never a callable permission.
use super::SourceRows;
use crate::support::{crates::CrateIdentity, edition::Edition};

/// Supported semantic source family; numeric wire tags are assigned by the codec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Source2024,
}

/// Private construction will be added only with the complete source validator.
/// Raw draft rows cannot construct this data or enter an Interface.
#[derive(Debug)]
pub struct ValidatedRows {
    owner: CrateIdentity,
    profile: Profile,
    rows: SourceRows,
}
impl ValidatedRows {
    #[must_use]
    pub fn descriptor(&self) -> Descriptor<'_> {
        Descriptor { source: self }
    }
}

#[derive(Clone, Copy)]
pub struct Descriptor<'a> {
    source: &'a ValidatedRows,
}
impl<'a> Descriptor<'a> {
    pub(crate) const fn rows(self) -> &'a SourceRows {
        &self.source.rows
    }
    #[must_use]
    pub fn identity(&self) -> &CrateIdentity {
        &self.source.owner
    }
    /// This first source family accepts only an own-provider payload.
    #[must_use]
    pub fn provider(&self) -> &CrateIdentity {
        &self.source.owner
    }
    #[must_use]
    pub const fn edition(&self) -> Edition {
        self.source.rows.policy.edition
    }
    #[must_use]
    pub const fn profile(&self) -> Profile {
        self.source.profile
    }
}
