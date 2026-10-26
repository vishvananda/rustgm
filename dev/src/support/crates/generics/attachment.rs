//! One cold attachment dispatches legacy/source data without fabricated row sets.
use super::{ApplicationRef, PayloadView, Relation, Rows, Work, legacy_payload::Legacy, source};
use crate::support::crates::{
    CrateIdentity,
    owner_catalog::{BodyCatalog, BodyRef, DeclarationRef},
};

#[derive(Debug)]
#[expect(
    clippy::large_enum_variant,
    reason = "the existing optional Payload box owns this cold variant; another legacy box would add an allocation to every existing import"
)]
enum Data {
    Legacy(Legacy),
    Source(source::ValidatedRows),
}
/// Validated metadata data only. Import still authenticates actual declarations,
/// reconstructs complete contracts and proves every original and demanded use.
#[derive(Debug)]
pub struct Payload {
    data: Data,
}
impl PartialEq for Payload {
    fn eq(&self, other: &Self) -> bool {
        match (&self.data, &other.data) {
            (Data::Legacy(a), Data::Legacy(b)) => a == b,
            (Data::Source(a), Data::Source(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Payload {}
impl From<source::ValidatedRows> for Payload {
    fn from(source: source::ValidatedRows) -> Self {
        Self {
            data: Data::Source(source),
        }
    }
}
impl Payload {
    /// # Errors
    /// Rejects malformed legacy tables and combined resource limits.
    pub fn new(owner: &CrateIdentity, rows: Rows, work: &mut Work) -> Result<Self, &'static str> {
        Ok(Self {
            data: Data::Legacy(Legacy::new(owner, rows, work)?),
        })
    }
    pub(crate) fn with_base(
        owner: &CrateIdentity,
        rows: Rows,
        work: &mut Work,
        base: usize,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            data: Data::Legacy(Legacy::with_base(owner, rows, work, base)?),
        })
    }
    /// Validate the complete ordinary source function domain as unpaid data.
    /// Actual source-client consumption still authenticates and proves originals.
    /// # Errors
    /// Rejects unsupported forms, malformed inputs and combined resource limits.
    pub fn from_source(
        owner: &CrateIdentity,
        rows: source::SourceRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let overhead = size_of::<Self>().saturating_sub(size_of::<source::ValidatedRows>());
        let staged =
            source::ValidatedRows::stage(owner, rows, base.saturating_add(overhead), work)?;
        Ok(Self::from(staged))
    }
    /// Validate complete provided-body data without granting canonical import,
    /// defining-environment or executable permission.
    /// # Errors
    /// Rejects malformed/unsupported domains and combined resource limits.
    pub fn from_source_provided(
        owner: &CrateIdentity,
        rows: source::SourceRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let overhead = size_of::<Self>().saturating_sub(size_of::<source::ValidatedRows>());
        let staged = source::ValidatedRows::stage_provided(
            owner,
            rows,
            base.saturating_add(overhead),
            work,
        )?;
        Ok(Self::from(staged))
    }
    pub(crate) fn from_source_provided_observed(
        owner: &CrateIdentity,
        rows: source::SourceRows,
        base: usize,
        work: &mut Work,
        observe: &mut impl FnMut(usize, usize),
    ) -> Result<Self, &'static str> {
        let overhead = size_of::<Self>().saturating_sub(size_of::<source::ValidatedRows>());
        let staged = source::ValidatedRows::stage_provided_observed(
            owner,
            rows,
            base.saturating_add(overhead),
            work,
            observe,
        )?;
        Ok(Self::from(staged))
    }
    /// Validate complete ordinary header contracts without granting body proof.
    /// # Errors
    /// Rejects malformed/incomplete facts and combined storage/work limits.
    pub fn from_source_contracts(
        owner: &CrateIdentity,
        rows: source::SourceRows,
        attributes: source::HeaderAttributeColumns,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let overhead = size_of::<Self>().saturating_sub(size_of::<source::ValidatedRows>());
        Ok(Self::from(source::ValidatedRows::stage_contracts(
            owner,
            rows,
            attributes,
            base.saturating_add(overhead),
            work,
        )?))
    }
    /// Validate complete module/domain/advisory rows, without transporting paid proof.
    /// # Errors
    /// Rejects incomplete policy, unsupported forms and combined budgets.
    pub fn from_source_module_contracts(
        owner: &CrateIdentity,
        rows: source::SourceRows,
        attributes: source::HeaderAttributeColumns,
        modules: source::ModuleContractRows,
        base: usize,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let overhead = size_of::<Self>().saturating_sub(size_of::<source::ValidatedRows>());
        Ok(Self::from(source::ValidatedRows::stage_module_contracts(
            owner,
            rows,
            attributes,
            modules,
            base.saturating_add(overhead),
            work,
        )?))
    }
    #[must_use]
    pub const fn view(&self) -> PayloadView<'_> {
        match &self.data {
            Data::Legacy(data) => PayloadView::Legacy(data.rows()),
            Data::Source(data) => PayloadView::Source(data),
        }
    }
    /// # Errors
    /// A source packet cannot be consumed as an invented empty legacy table.
    pub fn legacy_rows(&self) -> Result<&Rows, &'static str> {
        match &self.data {
            Data::Legacy(data) => Ok(data.rows()),
            Data::Source(_) => Err("source packet requires its actual source row consumer"),
        }
    }
    pub(crate) fn application_owner(
        &self,
        application: ApplicationRef,
    ) -> Result<Option<DeclarationRef>, &'static str> {
        match &self.data {
            Data::Legacy(data) => data.application_owner(application),
            Data::Source(_) => Err("source application requires its actual source owner mapping"),
        }
    }
    #[must_use]
    pub fn owners(&self) -> &BodyCatalog {
        match &self.data {
            Data::Legacy(data) => data.owners(),
            Data::Source(data) => data.owners(),
        }
    }
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        match &self.data {
            Data::Legacy(data) => data.retained_bytes(),
            Data::Source(data) => data.budget().bytes.saturating_add(source_overhead()),
        }
    }
    /// # Errors
    /// Rejects a foreign legacy body or a source body in a legacy relation query.
    pub fn relations(
        &self,
        body: BodyRef,
    ) -> Result<impl Iterator<Item = Relation> + '_, &'static str> {
        let Data::Legacy(data) = &self.data else {
            return Err("source body requires its actual source relation consumer");
        };
        data.relations(body)
    }

    pub(crate) fn peak_bytes(&self) -> usize {
        match &self.data {
            Data::Legacy(data) => data.peak_bytes(),
            Data::Source(data) => data.budget().peak.saturating_add(source_overhead()),
        }
    }
    pub(crate) fn budget(&self) -> (usize, usize) {
        match &self.data {
            Data::Legacy(data) => data.budget(),
            Data::Source(data) => (data.budget().atoms, data.budget().names),
        }
    }
}
fn source_overhead() -> usize {
    size_of::<Payload>().saturating_sub(size_of::<source::ValidatedRows>())
}
