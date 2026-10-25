//! A descriptor borrows completed source data; it is never callable permission.
#[path = "column_storage.rs"]
mod columns;
#[path = "ordinary_profile.rs"]
mod functions;
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
pub use columns::{Columns, ValidatedView};

/// Existing source-mode facade; no source descriptor has been published yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Source2024,
    /// Ordinary function/move bodies; no const trait/default execution profile.
    Source2024Functions,
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
    budget: Budget,
}
impl PartialEq for ValidatedRows {
    fn eq(&self, other: &Self) -> bool {
        self.owners == other.owners
            && self.profile == other.profile
            && self.columns == other.columns
    }
}
impl Eq for ValidatedRows {}
impl ValidatedRows {
    /// Structural staging only. Public payload activation remains gated until
    /// source serialization, canonical remapping and fresh proof all exist.
    pub(crate) fn stage(
        owner: &CrateIdentity,
        rows: SourceRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let start = work.steps;
        let indexed = HeaderIndex::new(&rows, base, work)?;
        functions::validate(&rows, &indexed, work)?;
        let domains = indexed.domains(work)?;
        let plan = domains.original_inputs(work)?;
        let previous_peak = plan.peak_bytes();
        drop(plan);
        drop(domains);
        let (headers, raw_bytes) = indexed.into_header_rows()?;
        let mut budget = freeze_budget(&rows, &headers, raw_bytes, base, previous_peak, work)?;
        let mut kinds = Vec::with_capacity(rows.declarations.len());
        let mut names = 0_usize;
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
        budget.atoms = work.steps.saturating_sub(start);
        budget.names = names;
        Ok(Self {
            owners,
            profile: Profile::Source2024Functions,
            columns: Box::new(columns),
            headers: headers.into_boxed_slice(),
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
fn freeze_budget(
    rows: &SourceRows,
    headers: &Vec<Option<super::HeaderRef>>,
    raw_bytes: usize,
    base: usize,
    previous_peak: usize,
    work: &mut Work,
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
    #[must_use]
    pub const fn profile(&self) -> Profile {
        self.source.profile
    }
}
