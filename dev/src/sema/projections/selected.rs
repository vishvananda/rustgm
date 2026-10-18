//! A selected member keeps its owner and complete arguments while its definition finishes.
use super::{Declarations, DefId, ItemId, Source, TypeId};
use crate::types::GenericArgs;

pub(super) struct Definition {
    pub id: DefId,
    owner: DefId,
    arguments: GenericArgs,
}
impl Definition {
    pub fn select(
        facts: &Declarations,
        item: ItemId,
        source: Source,
    ) -> Result<Self, &'static str> {
        let Source::Implementation { owner, arguments } = source else {
            return Err("associated type requires an unambiguous validated implementation");
        };
        Ok(Self {
            id: super::definition(facts, owner, item)?,
            owner,
            arguments,
        })
    }

    pub fn result(&self, facts: &mut Declarations) -> Result<Option<TypeId>, &'static str> {
        facts.definitions[self.id.0 as usize]
            .map(|template| {
                facts
                    .generics
                    .substitute(&mut facts.types, self.owner, self.arguments, template)
            })
            .transpose()
    }
}
