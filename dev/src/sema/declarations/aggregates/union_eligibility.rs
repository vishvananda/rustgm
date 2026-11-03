//! Source union fields use the pinned trivial-copy shortcut, then an actual marker goal.
use super::{Checker, Error, ItemId, NodeKey};
use crate::{
    sema::nominals::Proof,
    support::{bindings::LangItem, hash::HashSet, retained::Growth},
    types::{
        GenericArgs, TypeId,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};
impl Checker<'_> {
    pub(super) fn validate_source_union_field(
        &mut self,
        ty: TypeId,
        source: NodeKey,
        proof: &mut Proof,
        proved: &mut HashSet<TypeId>,
    ) -> Result<(), Error> {
        let ty = proof
            .normalize(&mut self.facts, ty)
            .map_err(|e| self.error(source, e))?;
        crate::sema::header_attributes::charge_source_contract_query(&self.facts, 1)?;
        self.facts.union_work.steps += 1;
        if proved.contains(&ty) {
            return Ok(());
        }
        let trivial = self
            .facts
            .types
            .trivial_copy(ty)
            .ok_or_else(|| self.error(source, "missing completed union shape fact"))?;
        self.facts.union_work.unique_types += 1;
        if !trivial {
            let role = self
                .facts
                .source_bindings
                .as_deref()
                .ok_or_else(|| self.error(source, "missing source union catalog"))?
                .validated_language(LangItem::BikeshedGuaranteedNoDrop)
                .map_err(|e| self.error(source, e))?;
            let definition = if let Some(role) = role {
                role.definition
            } else {
                crate::sema::compiler_items::copy_declaration(&mut self.facts)
                    .map_err(|e| self.error(source, e))?
                    .ok_or_else(|| {
                        self.error(source, "source union field requires actual Copy or NoDrop")
                    })?
            };
            let application = self
                .facts
                .types
                .intern_trait_application(Application {
                    item: ItemId(definition.0),
                    arguments: GenericArgs::EMPTY,
                })
                .map_err(|e| self.error(source, e))?;
            if !proof
                .prove(
                    &mut self.facts,
                    Predicate {
                        ty,
                        bound: Bound::Trait(application),
                    },
                )
                .map_err(|e| self.error(source, e))?
            {
                return Err(self.error(
                    source,
                    "source union field requires its actual NoDrop or Copy proof",
                ));
            }
        }
        let growth = Growth::map::<TypeId, ()>(proved.len(), proved.capacity(), 1);
        crate::sema::header_attributes::charge_source_retention(
            &self.facts,
            1 + growth.work,
            growth.bytes,
        )?;
        proved
            .try_reserve(1)
            .map_err(|_| self.error(source, "union field proof allocation failed"))?;
        proved.insert(ty);
        Ok(())
    }
}
