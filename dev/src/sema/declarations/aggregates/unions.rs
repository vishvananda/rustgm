//! Union field eligibility follows exact language identities, not destruction guesses.
use super::{Checker, Error, ItemId, NodeKey};
use crate::{
    sema::{members::Field, nominals::Proof},
    support::library_types::LibraryType,
    types::{
        Kind, TypeId,
        parameters::predicates::{Bound, Predicate},
    },
};
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct Fields {
    work: Vec<TypeId>,
    seen: HashSet<TypeId>,
}

impl Checker<'_> {
    pub(super) fn validate_union_field(
        &mut self,
        field: Field,
        proof: &mut Proof,
        scratch: &mut Fields,
    ) -> Result<(), Error> {
        let manually_drop = self
            .report
            .resolution
            .library_types
            .get(LibraryType::ManuallyDrop)
            .map(|nominal| nominal.definition);
        self.facts.union_work.fields += 1;
        scratch.work.push(field.ty);
        while let Some(ty) = scratch.work.pop() {
            self.facts.union_work.steps = self
                .facts
                .union_work
                .steps
                .checked_add(1)
                .filter(|steps| *steps <= 8_000_000)
                .ok_or_else(|| {
                    self.error(field.node, "union field eligibility work budget exceeded")
                })?;
            if scratch.seen.len() >= 1_000_000 {
                return Err(self.error(
                    field.node,
                    "union field eligibility storage budget exceeded",
                ));
            }
            let ty = proof
                .normalize(&mut self.facts, ty)
                .map_err(|e| self.error(field.node, e))?;
            if !scratch.seen.insert(ty) {
                continue;
            }
            self.facts.union_work.unique_types += 1;
            match self.facts.types.kind(ty) {
                Some(Kind::Reference { .. }) => {}
                Some(Kind::Adt(item, _)) if manually_drop.is_some_and(|id| id.0 == item.0) => {}
                Some(Kind::Tuple(fields)) => {
                    scratch.work.extend(
                        self.facts
                            .types
                            .list(fields)
                            .ok_or_else(|| self.error(field.node, "missing union tuple fields"))?,
                    );
                }
                Some(Kind::Array { element, .. } | Kind::ArrayTerm { element, .. }) => {
                    scratch.work.push(element);
                }
                _ => {
                    if !proof
                        .prove(
                            &mut self.facts,
                            Predicate {
                                ty,
                                bound: Bound::Copy,
                            },
                        )
                        .map_err(|e| self.error(field.node, e))?
                    {
                        return Err(self.error(field.node, "union fields require Copy, references, ManuallyDrop, or allowed tuples and arrays"));
                    }
                }
            }
        }
        Ok(())
    }
    pub(super) fn register_union(&mut self, item: ItemId, key: NodeKey) -> Result<(), Error> {
        let definition = self
            .facts
            .aggregates
            .get(item)
            .ok_or_else(|| self.error(key, "missing union shape"))?;
        let empty = self.facts.aggregates.variants(definition).iter().all(|v| {
            self.facts
                .types
                .list(v.fields)
                .is_none_or(<[TypeId]>::is_empty)
        });
        if empty {
            return Err(self.error(key, "union requires at least one field"));
        }
        self.facts
            .aggregates
            .suppress_field_destruction(item)
            .map_err(|e| self.error(key, e))
    }
}
