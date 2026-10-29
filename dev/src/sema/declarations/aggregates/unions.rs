//! Union field eligibility follows exact language identities, not destruction guesses.
use super::{Checker, Error, ItemId, NodeKey};
use crate::support::hash::HashSet;
use crate::{
    sema::{members::Field, nominals::Proof},
    support::library_types::LibraryType,
    types::{
        GenericArgs, Kind, TypeId,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};

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
        let source = field
            .origin
            .require_diagnostic()
            .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        let manually_drop = self
            .report
            .resolution
            .library_types
            .get(LibraryType::ManuallyDrop)
            .map(|nominal| nominal.definition);
        self.facts.union_work.fields += 1;
        let mut copy_bound = None;
        scratch.work.push(field.ty);
        while let Some(ty) = scratch.work.pop() {
            self.facts.union_work.steps = self
                .facts
                .union_work
                .steps
                .checked_add(1)
                .filter(|steps| *steps <= 8_000_000)
                .ok_or_else(|| {
                    self.error(source, "union field eligibility work budget exceeded")
                })?;
            if scratch.seen.len() >= 1_000_000 {
                return Err(self.error(source, "union field eligibility storage budget exceeded"));
            }
            let ty = proof
                .normalize(&mut self.facts, ty)
                .map_err(|e| self.error(source, e))?;
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
                            .ok_or_else(|| self.error(source, "missing union tuple fields"))?,
                    );
                }
                Some(Kind::Array { element, .. } | Kind::ArrayTerm { element, .. }) => {
                    scratch.work.push(element);
                }
                _ => {
                    let bound = if let Some(bound) = copy_bound {
                        bound
                    } else {
                        let bound = self.union_copy_bound(source)?;
                        copy_bound = Some(bound);
                        bound
                    };
                    if !proof
                        .prove(&mut self.facts, Predicate { ty, bound })
                        .map_err(|e| self.error(source, e))?
                    {
                        return Err(self.error(source, "union fields require Copy, references, ManuallyDrop, or allowed tuples and arrays"));
                    }
                }
            }
        }
        Ok(())
    }
    fn union_copy_bound(&mut self, source: NodeKey) -> Result<Bound, Error> {
        if !self.facts.uses_source_copy() {
            return Ok(Bound::Copy);
        }
        let declaration = crate::sema::compiler_items::copy_declaration(&mut self.facts)
            .map_err(|error| self.error(source, error))?
            .ok_or_else(|| self.error(source, "union field requires actual source Copy"))?;
        self.facts
            .types
            .intern_trait_application(Application {
                item: ItemId(declaration.0),
                arguments: GenericArgs::EMPTY,
            })
            .map(Bound::Trait)
            .map_err(|error| self.error(source, error))
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
