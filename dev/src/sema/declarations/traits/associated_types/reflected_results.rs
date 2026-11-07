//! On-demand Field members reuse the genuine associated-result worklist.
use super::{Checker, DefId, Error, early_metadata::Phase};
use crate::{sema::compiler_items::field_protocol, support::bindings::LangItem};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn reflected_result_work(
        &mut self,
        id: DefId,
        phase: Phase,
    ) -> Result<(), Error> {
        if field_protocol::prepared_member(self.report, &self.facts, id)? {
            return Ok(());
        }
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| Error::unlocated("Field result work lost its actual catalog"))?;
        let trait_ = catalog
            .language(LangItem::Field)
            .ok_or_else(|| Error::unlocated("Field result work lost its actual trait"))?;
        let base = catalog
            .language(LangItem::FieldBase)
            .ok_or_else(|| Error::unlocated("Field result work requires Base"))?;
        let field = catalog
            .language(LangItem::FieldType)
            .ok_or_else(|| Error::unlocated("Field result work requires Type"))?;
        let nominal = catalog
            .language(LangItem::FieldRepresentingType)
            .ok_or_else(|| {
                Error::unlocated("Field result work requires its representing nominal")
            })?;
        let member = if base.definition == id {
            base
        } else if field.definition == id {
            field
        } else {
            return Err(Error::unlocated("Field result work names another member"));
        };
        let growth = crate::support::retained::Growth::vector::<crate::sema::declarations::Work>(
            self.work.len(),
            self.work.capacity(),
            2,
        );
        crate::sema::header_attributes::charge_source_retention(
            &self.facts,
            growth.work.saturating_add(2),
            growth.bytes,
        )?;
        self.work
            .try_reserve(2)
            .map_err(|_| Error::unlocated("Field result continuation allocation failed"))?;
        match phase {
            Phase::Begin => {
                if self.facts.associated.require_projection_header(id).is_ok() {
                    self.work
                        .push(crate::sema::declarations::Work::SourceResultContracts(
                            id,
                            Phase::Seal,
                        ));
                    return Ok(());
                }
                let owner = self.source_metadata_owner(member, trait_)?;
                self.prepay_source_result(id, member.origin)?;
                self.declare_associated_type(id, member.origin, owner, trait_.definition)?;
                self.facts.generics.parents.insert(id, trait_.definition);
                self.work
                    .push(crate::sema::declarations::Work::SourceResultContracts(
                        id,
                        Phase::Seal,
                    ));
                let pending = self.bound_syntax.contains_key(&id);
                self.work
                    .push(crate::sema::declarations::Work::SourceResultContracts(
                        id,
                        Phase::Results(pending),
                    ));
                if pending {
                    self.begin_associated_result_bounds(id, member.origin)?;
                }
            }
            Phase::Results(pending) => {
                if pending {
                    self.prepay_result_bound_completion(id, member.origin)?;
                    self.finish_associated_result_bounds(id, member.origin)?;
                }
                crate::sema::traits::results::prepare_source_outlives(&mut self.facts, id)
                    .map_err(|e| self.error(member.origin, e))?;
            }
            Phase::Seal => {
                let other = if id == base.definition {
                    field.definition
                } else {
                    base.definition
                };
                self.seal_field_results(id, other, nominal.definition)?;
            }
        }
        Ok(())
    }
    fn seal_field_results(&mut self, id: DefId, other: DefId, nominal: DefId) -> Result<(), Error> {
        if self
            .facts
            .associated
            .require_projection_header(other)
            .is_err()
        {
            self.work
                .push(crate::sema::declarations::Work::SourceResultContracts(
                    id,
                    Phase::Seal,
                ));
            self.work
                .push(crate::sema::declarations::Work::SourceResultContracts(
                    other,
                    Phase::Begin,
                ));
        } else if !self.facts.generics.nominal_contracts.contains(&nominal) {
            self.work
                .push(crate::sema::declarations::Work::SourceResultContracts(
                    id,
                    Phase::Seal,
                ));
            self.work
                .push(crate::sema::declarations::Work::NominalContract(
                    crate::types::ItemId(nominal.0),
                    false,
                ));
        } else {
            field_protocol::prepare(self.report, &mut self.facts)?;
        }
        Ok(())
    }
}
