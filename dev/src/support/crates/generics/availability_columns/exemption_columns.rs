//! Versioned body exemptions never define API features or confer const callability.
use super::super::HeaderRef;
use super::{AvailabilityRows, Columns, DefinitionKind, ModuleContractRows};
use crate::support::{
    crates::{MAX_BYTES, MAX_COMPONENTS, generics::Work, owner_catalog::DeclarationRef},
    retained,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstBodyGrant {
    pub owner: DeclarationRef,
    pub feature: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConstBodyContracts {
    /// Actual unmarked const bodies checked under staged recursive stability.
    pub implicit: Vec<DeclarationRef>,
    /// Sorted exact (canonical function, feature) pairs, scoped to that body.
    pub grants: Vec<ConstBodyGrant>,
}
impl ConstBodyContracts {
    pub(super) fn storage_bytes(&self, work: &mut Work) -> Result<usize, &'static str> {
        work.charge(
            self.implicit
                .len()
                .saturating_add(self.grants.len())
                .saturating_add(1),
        )?;
        if self.implicit.len().max(self.grants.len()) > MAX_COMPONENTS {
            return Err("const body contracts exceed row budget");
        }
        let mut bytes = size_of::<Self>()
            .saturating_add(retained::vector(&self.implicit))
            .saturating_add(retained::vector(&self.grants));
        for grant in &self.grants {
            work.charge(grant.feature.len())?;
            bytes = bytes.saturating_add(grant.feature.capacity());
            if bytes > MAX_BYTES {
                return Err("const body contracts exceed storage budget");
            }
            if grant.feature.len() > super::super::MAX_REASON_BYTES || !feature(&grant.feature) {
                return Err("const body grant requires a canonical feature identifier");
            }
        }
        Ok(bytes)
    }
    pub(super) fn validate(
        &self,
        columns: &Columns,
        headers: &[Option<HeaderRef>],
        modules: &ModuleContractRows,
        rows: &AvailabilityRows,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if self.implicit.is_empty() && self.grants.is_empty() {
            return Err("empty const body extension has no actual producer");
        }
        if !self.implicit.is_empty()
            && rows
                .declarations
                .get(modules.namespace.root.0 as usize)
                .is_none_or(|row| row.regular.is_none())
        {
            return Err("implicit const body safety requires actual staged root policy");
        }
        let mut previous = None;
        for &owner in &self.implicit {
            work.charge(1)?;
            if previous.is_some_and(|before| before >= owner.0) {
                return Err("implicit const bodies must have unique ordered owners");
            }
            previous = Some(owner.0);
            function(columns, headers, owner)?;
            if rows
                .declarations
                .get(owner.0 as usize)
                .is_none_or(|row| row.constant.is_some())
            {
                return Err("implicit const body replaces an actual API policy");
            }
        }
        let mut previous: Option<(u32, &str)> = None;
        for grant in &self.grants {
            work.charge(grant.feature.len().saturating_add(1))?;
            let current = (grant.owner.0, grant.feature.as_str());
            if previous.is_some_and(|before| before >= current) {
                return Err("const body grants must be unique ordered owner/feature pairs");
            }
            previous = Some(current);
            let header = function(columns, headers, grant.owner)?;
            if !matches!(header.body, super::super::FunctionBody::Original(_)) {
                return Err("const body grant has no actual defining body");
            }
        }
        Ok(())
    }
}
fn function<'a>(
    columns: &'a Columns,
    headers: &[Option<HeaderRef>],
    owner: DeclarationRef,
) -> Result<&'a super::super::FunctionHeader, &'static str> {
    if columns
        .declarations
        .get(owner.0 as usize)
        .is_none_or(|row| row.definition.kind != DefinitionKind::Function)
    {
        return Err("const body contract requires an actual function owner");
    }
    let Some(HeaderRef::Function(index)) = headers.get(owner.0 as usize).copied().flatten() else {
        return Err("const body contract has no actual function header");
    };
    columns
        .functions
        .get(index as usize)
        .filter(|row| {
            row.declaration == owner
                && row.constness != crate::support::constness::Constness::Ordinary
        })
        .ok_or("const body contract cannot grant declaration constness")
}
fn feature(name: &str) -> bool {
    let mut characters = name.chars();
    name != "_"
        && characters
            .next()
            .is_some_and(|first| first == '_' || unicode_ident::is_xid_start(first))
        && characters.all(unicode_ident::is_xid_continue)
        && unicode_normalization::is_nfc(name)
}
