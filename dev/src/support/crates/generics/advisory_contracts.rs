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
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeaderAttributeColumns {
    pub functions: Vec<HeaderContract>,
    pub traits: Vec<HeaderContract>,
}
impl HeaderAttributeColumns {
    pub(crate) fn validate(
        &self,
        rows: &SourceRows,
        base: usize,
        work: &mut Work,
    ) -> Result<usize, &'static str> {
        work.charge(self.functions.len().saturating_add(self.traits.len()))?;
        if self.functions.len() != rows.functions.len() || self.traits.len() != rows.traits.len() {
            return Err("source header attribute columns are incomplete");
        }
        let mut bytes = size_of::<Self>()
            .saturating_add(retained::vector(&self.functions))
            .saturating_add(retained::vector(&self.traits));
        for (contract, header) in self.functions.iter().zip(&rows.functions) {
            Self::check_row(contract, header.declaration, rows, &mut bytes, work)?;
            if contract.inline.is_some_and(|inline| {
                (inline.applicability == InlineApplicability::FunctionBody)
                    != matches!(header.body, super::FunctionBody::Original(_))
            }) {
                return Err("source inline applicability differs from its actual function body");
            }
        }
        for (contract, header) in self.traits.iter().zip(&rows.traits) {
            Self::check_row(contract, header.declaration, rows, &mut bytes, work)?;
            if contract.inline.is_some() {
                return Err("source trait header cannot carry an effective inline hint");
            }
        }
        if base.saturating_add(bytes) > MAX_BYTES {
            return Err("source header attributes exceed aggregate retained storage");
        }
        Ok(bytes)
    }
    fn check_row(
        contract: &HeaderContract,
        expected: DeclarationRef,
        rows: &SourceRows,
        bytes: &mut usize,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let declaration = rows
            .declarations
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
