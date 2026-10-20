//! Publish completed input lifetime contracts before shared outlives inference.
use crate::{
    sema::model::Declarations,
    types::{
        ItemId,
        arguments::Argument,
        outlives::families::{Families, Inputs},
        parameters::argument_id,
    },
};

impl Declarations {
    pub(in crate::sema) fn family_outlives(&mut self) -> Result<Families, &'static str> {
        let mut result = Families::default();
        if self.associated.families.is_empty() {
            return Ok(result);
        }
        // Declaration order determines any canonical types created by lifting
        // unused quantifiers. Impl conditions never become a trait's input facts.
        for member in self.associated.members() {
            let id = member.definition;
            let Some(family) = self.associated.families.get(&id) else {
                continue;
            };
            let Some(owner) = self.associated.trait_owner(id) else {
                continue;
            };
            let schema = self
                .associated
                .schema(owner)
                .ok_or("missing family parent schema")?;
            let clauses = family
                .requirements
                .as_deref()
                .ok_or("associated family input contract is not complete")?;
            let inputs = Inputs {
                owner: ItemId(owner.0),
                subject: argument_id(&self.types, Argument::Type(schema.self_type))?,
                captures: schema.captures,
                parent: self
                    .generics
                    .parameters
                    .get(&owner)
                    .into_iter()
                    .flatten()
                    .map(|p| p.argument)
                    .collect(),
                own: self
                    .generics
                    .parameters
                    .get(&id)
                    .into_iter()
                    .flatten()
                    .map(|p| p.argument)
                    .collect(),
            };
            result.insert(
                &mut self.types,
                ItemId(id.0),
                inputs,
                clauses,
                &family.regions,
            )?;
        }
        Ok(result)
    }
}
