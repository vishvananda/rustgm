//! Complete namespace and written-modifier data grants no imported body proof.
use super::{HeaderIndex, NamespaceColumns, SourceRows};
use crate::support::{
    crates::{MAX_BYTES, MAX_COMPONENTS, generics::Work, owner_catalog::DeclarationRef},
    retained,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeaderModifierPolicy {
    NoWrittenDefaultOrFinalOrCallerTracking,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HeaderModifierRecord {
    pub declaration: DeclarationRef,
    pub site: u32,
    pub policy: HeaderModifierPolicy,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HeaderModifierColumns {
    pub functions: Vec<HeaderModifierRecord>,
    pub traits: Vec<HeaderModifierRecord>,
    pub implementations: Vec<HeaderModifierRecord>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProvidedContractRows {
    pub namespace: NamespaceColumns,
    pub modifiers: HeaderModifierColumns,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) struct ProvidedContractColumns {
    rows: ProvidedContractRows,
}
#[derive(Clone, Copy)]
pub(crate) struct ProvidedContractView<'a> {
    columns: &'a ProvidedContractColumns,
}
impl ProvidedContractRows {
    pub(super) fn storage_bytes(&self, work: &mut Work) -> Result<usize, &'static str> {
        work.charge(1)?;
        let count = self
            .modifiers
            .functions
            .len()
            .saturating_add(self.modifiers.traits.len())
            .saturating_add(self.modifiers.implementations.len());
        work.charge(count)?;
        if count > MAX_COMPONENTS {
            return Err("provided modifier column count exceeds its profile budget");
        }
        Ok(self
            .namespace
            .storage_bytes(work)?
            .saturating_add(size_of::<HeaderModifierColumns>())
            .saturating_add(retained::vector(&self.modifiers.functions))
            .saturating_add(retained::vector(&self.modifiers.traits))
            .saturating_add(retained::vector(&self.modifiers.implementations)))
    }
    pub(super) fn validate(
        mut self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        attributes: &super::super::HeaderAttributeColumns,
        base: usize,
        work: &mut Work,
    ) -> Result<(ProvidedContractColumns, usize), &'static str> {
        work.charge(1)?;
        let bytes = self.storage_bytes(work)?;
        let scratch = self.namespace.validation_scratch(rows.declarations.len());
        if base.saturating_add(bytes).saturating_add(scratch) > MAX_BYTES {
            return Err("provided namespace/modifiers exceed combined storage budget");
        }
        #[cfg(test)]
        let before = work.steps;
        let namespace = self.namespace.validate_rows(rows, work)?;
        #[cfg(test)]
        eprintln!(
            "provided namespace steps={} observations={namespace:?}",
            work.steps.saturating_sub(before)
        );
        work.charge(1)?;
        if namespace.peak_bytes > bytes.saturating_add(scratch) {
            return Err("provided namespace exceeded its actual prepaid storage envelope");
        }
        self.modifiers.validate(rows, headers, attributes, work)?;
        Ok((
            ProvidedContractColumns { rows: self },
            bytes.saturating_add(scratch),
        ))
    }
}
impl HeaderModifierColumns {
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(retained::vector(&self.functions))
            .saturating_add(retained::vector(&self.traits))
            .saturating_add(retained::vector(&self.implementations))
    }
    fn validate(
        &self,
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        attributes: &super::super::HeaderAttributeColumns,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        check(
            &self.functions,
            rows.functions
                .iter()
                .zip(&attributes.functions)
                .map(|(row, contract)| (row.declaration, Some(contract.declaration_site))),
            rows,
            headers,
            work,
        )?;
        check(
            &self.traits,
            rows.traits
                .iter()
                .zip(&attributes.traits)
                .map(|(row, contract)| (row.declaration, Some(contract.declaration_site))),
            rows,
            headers,
            work,
        )?;
        check(
            &self.implementations,
            rows.implementations
                .iter()
                .map(|row| (row.self_definition, None)),
            rows,
            headers,
            work,
        )
    }
}
fn check(
    records: &[HeaderModifierRecord],
    expected: impl ExactSizeIterator<Item = (DeclarationRef, Option<u32>)>,
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(expected.len().saturating_mul(2).saturating_add(1))?;
    if records.len() != expected.len() {
        return Err("provided written-modifier columns are incomplete");
    }
    for (record, (declaration, site)) in records.iter().zip(expected) {
        let row = rows
            .declarations
            .get(declaration.0 as usize)
            .ok_or("provided modifier has an unknown declaration")?;
        if record.declaration != declaration
            || record.site >= row.sites
            || site.is_some_and(|site| record.site != site)
            || headers.header(declaration, work)?.is_none()
        {
            return Err("provided modifier changed its actual header owner/order/site");
        }
    }
    Ok(())
}
impl ProvidedContractColumns {
    pub(super) const fn view(&self) -> ProvidedContractView<'_> {
        ProvidedContractView { columns: self }
    }
}
impl<'a> ProvidedContractView<'a> {
    pub(crate) const fn rows(self) -> &'a ProvidedContractRows {
        &self.columns.rows
    }
}
