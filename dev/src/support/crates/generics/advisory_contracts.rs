//! Complete version-one header facts, separate from executable body authority.
use super::super::Work;
use super::SourceRows;
use crate::support::{
    crates::{MAX_BYTES, owner_catalog::DeclarationRef},
    retained,
};

pub const MAX_REASON_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineHint {
    Hint,
    Always,
    Never,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineApplicability {
    FunctionBody,
    DiagnosedNoBody,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MustUseContract {
    pub owner_site: u32,
    pub reason: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InlineContract {
    pub owner_site: u32,
    pub hint: InlineHint,
    pub applicability: InlineApplicability,
}
/// Within a complete row, None is positively checked absence. An older profile
/// has no columns at all and therefore cannot claim that absence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderContract {
    pub declaration: DeclarationRef,
    pub declaration_site: u32,
    pub must_use: Option<MustUseContract>,
    pub inline: Option<InlineContract>,
    /// Version thirteen only, on function rows: the written cold advisory.
    pub cold: bool,
    /// Version thirteen only, on function rows: the written caller ABI policy.
    pub caller_tracking: bool,
}
impl HeaderContract {
    #[must_use]
    pub const fn callable_advisories(&self) -> bool {
        self.cold || self.caller_tracking
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeaderAttributeColumns {
    pub functions: Vec<HeaderContract>,
    pub traits: Vec<HeaderContract>,
}
impl HeaderAttributeColumns {
    /// `callable_advisories` is true exactly for the version-thirteen profile.
    pub(crate) fn validate(
        &self,
        rows: &SourceRows,
        base: usize,
        callable_advisories: bool,
        work: &mut Work,
    ) -> Result<usize, &'static str> {
        self.validate_headers(
            (&rows.declarations, &rows.functions, &rows.traits),
            base,
            callable_advisories,
            work,
        )
    }
    pub(crate) fn validate_columns(
        &self,
        rows: &super::Columns,
        base: usize,
        callable_advisories: bool,
        work: &mut Work,
    ) -> Result<usize, &'static str> {
        self.validate_headers(
            (&rows.declarations, &rows.functions, &rows.traits),
            base,
            callable_advisories,
            work,
        )
    }
    fn validate_headers(
        &self,
        (declarations, functions, traits): (
            &[super::Declaration],
            &[super::FunctionHeader],
            &[super::TraitHeader],
        ),
        base: usize,
        callable_advisories: bool,
        work: &mut Work,
    ) -> Result<usize, &'static str> {
        work.charge(self.functions.len().saturating_add(self.traits.len()))?;
        if self.functions.len() != functions.len() || self.traits.len() != traits.len() {
            return Err("source header attribute columns are incomplete");
        }
        let mut bytes = size_of::<Self>()
            .saturating_add(retained::vector(&self.functions))
            .saturating_add(retained::vector(&self.traits));
        for (contract, header) in self.functions.iter().zip(functions) {
            Self::check_row(contract, header.declaration, declarations, &mut bytes, work)?;
            if contract.inline.is_some_and(|inline| {
                (inline.applicability == InlineApplicability::FunctionBody)
                    != matches!(header.body, super::FunctionBody::Original(_))
            }) {
                return Err("source inline applicability differs from its actual function body");
            }
            if contract.callable_advisories() && !callable_advisories {
                return Err(
                    "source cold and caller tracking rows require the version-thirteen header profile",
                );
            }
            // Caller tracking changes the call ABI of a defined Rust function;
            // consumers re-prove its convention on their installed signature.
            if contract.caller_tracking && !matches!(header.body, super::FunctionBody::Original(_))
            {
                return Err("source caller tracking requires an exported original body");
            }
        }
        for (contract, header) in self.traits.iter().zip(traits) {
            Self::check_row(contract, header.declaration, declarations, &mut bytes, work)?;
            if contract.inline.is_some() {
                return Err("source trait header cannot carry an effective inline hint");
            }
            if contract.callable_advisories() {
                return Err("source trait header cannot carry a callable advisory");
            }
        }
        if base.saturating_add(bytes) > MAX_BYTES {
            return Err("source header attributes exceed aggregate retained storage");
        }
        Ok(bytes)
    }
    pub(super) fn check_row(
        contract: &HeaderContract,
        expected: DeclarationRef,
        declarations: &[super::Declaration],
        bytes: &mut usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let declaration = declarations
            .get(expected.0 as usize)
            .ok_or("source attribute header declaration is absent")?;
        if contract.declaration != expected || contract.declaration_site >= declaration.sites {
            return Err("source header attribute belongs to another declaration/site");
        }
        if contract
            .must_use
            .as_ref()
            .is_some_and(|value| value.owner_site >= declaration.sites)
            || contract
                .inline
                .is_some_and(|value| value.owner_site >= declaration.sites)
        {
            return Err("source attribute owner site is outside its actual declaration");
        }
        if let Some(reason) = contract
            .must_use
            .as_ref()
            .and_then(|value| value.reason.as_ref())
        {
            work.charge(reason.len())?;
            if reason.len() > MAX_REASON_BYTES {
                return Err("source must_use reason exceeds transport profile limit");
            }
            *bytes = bytes.saturating_add(reason.capacity());
        }
        Ok(())
    }
}
