//! Demand genuine source Metadata result facts without completing all results.
use super::{
    Checker, DefId, Error, ItemId, ItemKind, NodeKey, OwnerKind, TypeContract, Visibility,
};
use super::{Clause, Syntax};
#[derive(Clone, Copy)]
pub(in crate::sema::declarations) enum Phase {
    Begin,
    Results(bool),
    Seal,
}
impl Checker<'_> {
    /// Complete only the genuine source role's required member before callers
    /// can demand metadata. The final package pass reuses its authenticated seal.
    pub(in crate::sema::declarations) fn prepare_pointee_member(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let member =
            self.facts.source_bindings.as_deref().and_then(|catalog| {
                catalog.language(crate::support::bindings::LangItem::MetadataType)
            });
        if let Some(member) = member {
            self.work
                .push(crate::sema::declarations::Work::PointeeContracts(
                    member.definition,
                    Phase::Begin,
                ));
            self.drain(steps, limit)?;
        }
        Ok(())
    }
    pub(in crate::sema::declarations) fn pointee_member_work(
        &mut self,
        id: DefId,
        phase: Phase,
    ) -> Result<(), Error> {
        use crate::{
            sema::{compiler_items::pointee, header_attributes},
            support::bindings::LangItem,
        };
        let catalog = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| Error::unlocated("source Metadata work lost its catalog"))?;
        let member = catalog
            .language(LangItem::MetadataType)
            .filter(|member| member.definition == id)
            .ok_or_else(|| {
                Error::unlocated("source Metadata work has a different actual member")
            })?;
        if pointee::prepared_member(self.report, &self.facts, id)? {
            return Ok(());
        }
        match phase {
            Phase::Begin => {
                let trait_ = catalog.language(LangItem::PointeeTrait).ok_or_else(|| {
                    self.error(member.origin, "source Metadata requires actual Pointee")
                })?;
                let nominal = catalog.language(LangItem::DynMetadata);
                header_attributes::charge_source_retention(&self.facts, 3, 0)?;
                let owner = self.source_metadata_owner(member, trait_)?;
                self.prepay_source_result(id, member.origin)?;
                self.declare_associated_type(id, member.origin, owner, trait_.definition)?;
                self.facts.generics.parents.insert(id, trait_.definition);
                let growth = crate::support::retained::Growth::vector::<
                    crate::sema::declarations::Work,
                >(self.work.len(), self.work.capacity(), 3);
                header_attributes::charge_source_retention(
                    &self.facts,
                    3 + growth.work,
                    growth.bytes,
                )?;
                self.work
                    .push(crate::sema::declarations::Work::PointeeContracts(
                        id,
                        Phase::Seal,
                    ));
                if let Some(nominal) = nominal {
                    self.work
                        .push(crate::sema::declarations::Work::NominalContract(
                            ItemId(nominal.definition.0),
                            false,
                        ));
                }
                let pending = self.bound_syntax.contains_key(&id);
                self.work
                    .push(crate::sema::declarations::Work::PointeeContracts(
                        id,
                        Phase::Results(pending),
                    ));
                if pending {
                    self.begin_associated_result_bounds(id, member.origin)?;
                }
                Ok(())
            }
            Phase::Results(pending) => {
                if pending {
                    self.prepay_result_bound_completion(id, member.origin)?;
                    self.finish_associated_result_bounds(id, member.origin)?;
                }
                crate::sema::traits::results::prepare_source_outlives(&mut self.facts, id)
                    .map_err(|message| self.error(member.origin, message))
            }
            Phase::Seal => pointee::prepare(self.report, &mut self.facts),
        }
    }

    fn source_metadata_owner(
        &self,
        member: crate::resolve::bindings::Declaration,
        trait_: crate::resolve::bindings::Declaration,
    ) -> Result<super::Owner, Error> {
        let owner_id = self
            .facts
            .associated
            .owner(member.definition)
            .ok_or_else(|| self.error(member.origin, "source Metadata has no associated owner"))?;
        let owner = self
            .facts
            .associated
            .source_at(owner_id.0 as usize)
            .map_err(Error::unlocated)?
            .ok_or_else(|| {
                self.error(
                    member.origin,
                    "source Metadata requires local declaration ownership",
                )
            })?;
        if !matches!(owner.kind, OwnerKind::Trait { definition, .. } if definition == trait_.definition)
            || member.parent != Some(trait_.definition)
            || self.report.resolution.definitions[member.definition.0 as usize].node
                != Some(member.origin)
        {
            return Err(self.error(
                member.origin,
                "source Metadata changed its genuine trait owner",
            ));
        }
        let item = self.item(member.origin)?;
        if item.visibility != Visibility::Private
            || !matches!(item.kind, ItemKind::TypeAlias { ty: None, .. })
            || !self
                .report
                .package
                .unit(member.origin.file)
                .configured
                .ast
                .list(item.generics)
                .is_empty()
        {
            return Err(self.error(
                member.origin,
                "source Metadata requires an inherited-visibility required type",
            ));
        }
        Ok(owner)
    }

    fn prepay_source_result(&self, id: DefId, key: NodeKey) -> Result<(), Error> {
        use crate::support::retained;
        let reserved = self
            .associated_type_bounds
            .get(&id)
            .ok_or_else(|| self.error(key, "source Metadata lost its reserved result syntax"))?;
        let inherited = self
            .facts
            .generics
            .traits
            .as_deref()
            .map_or(0, |traits| traits.associated_bounds(ItemId(id.0)).len());
        let clauses = reserved.builtin.len().saturating_add(inherited);
        let bytes = retained::new_map::<DefId, TypeContract>(1)
            .saturating_add(retained::new_map::<DefId, DefId>(1))
            .saturating_add(retained::new_map::<DefId, Vec<Syntax>>(1))
            .saturating_add(
                4_usize
                    .saturating_mul(clauses)
                    .saturating_mul(size_of::<Clause>()),
            )
            .saturating_add(
                4_usize
                    .saturating_mul(reserved.syntax.len())
                    .saturating_mul(size_of::<Syntax>()),
            );
        crate::sema::header_attributes::charge_source_retention(
            &self.facts,
            clauses
                .saturating_add(reserved.syntax.len())
                .saturating_add(1),
            bytes,
        )
    }

    fn prepay_result_bound_completion(&self, id: DefId, key: NodeKey) -> Result<(), Error> {
        let inputs = self
            .facts
            .associated
            .families
            .get(&id)
            .and_then(|family| family.requirements.as_ref())
            .map_or(0, Vec::len);
        let predicates = self
            .facts
            .generics
            .predicates
            .get(&id)
            .map_or(&[][..], Vec::as_slice);
        let results = predicates
            .get(inputs..)
            .ok_or_else(|| self.error(key, "missing associated input predicates"))?;
        let contract = self
            .facts
            .associated
            .type_contract(id)
            .ok_or_else(|| self.error(key, "missing associated type contract"))?;
        let growth = crate::support::retained::Growth::vector::<Clause>(
            contract.bounds.len(),
            contract.bounds.capacity(),
            results.len(),
        );
        crate::sema::header_attributes::charge_source_retention(
            &self.facts,
            results.len().saturating_add(growth.work),
            growth.bytes,
        )
    }
}
